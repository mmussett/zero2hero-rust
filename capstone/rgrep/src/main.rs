//! # rgrep — Recursive Grep Replacement
//!
//! `rgrep` is a production-quality recursive grep tool that walks a directory
//! tree (respecting `.gitignore` files via the `ignore` crate), matches lines
//! against a regular expression, and renders results with coloured output and
//! optional context lines.
//!
//! ## Features
//! - Regex pattern matching (optionally case-insensitive)
//! - `.gitignore`-aware recursive directory walk
//! - Coloured output: green filenames, yellow line numbers, red match highlights
//! - Context lines (`-C N`) with `--` separators between non-adjacent groups
//! - Count-only mode (`--count`) that suppresses per-line output
//! - `NO_COLOR` / `--no-color` support for piped output
//!
//! ## Example
//! ```bash
//! rgrep "fn main" src/
//! rgrep -i "error" . -C 2
//! rgrep --count "TODO" .
//! ```

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use clap::Parser;
use colored::Colorize;
use ignore::Walk;
use regex::Regex;
use thiserror::Error;

// ─── Error Type ─────────────────────────────────────────────────────────────

/// All errors that can originate from rgrep's core logic.
///
/// Uses `thiserror` so that each variant carries its source error transparently
/// and the `?` operator converts automatically via `#[from]`.
#[derive(Debug, Error)]
pub enum RgrepError {
    /// The user-supplied regex pattern is syntactically invalid.
    #[error("invalid regex pattern: {0}")]
    Regex(#[from] regex::Error),

    /// A file could not be read (permissions, binary content converted to UTF-8
    /// loss, etc.).
    #[error("I/O error reading `{path}`: {source}")]
    Io {
        /// The file path that caused the error.
        path: PathBuf,
        /// The underlying OS I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The `ignore` crate encountered an error while walking the directory tree.
    #[error("directory walk error: {0}")]
    Walk(#[from] ignore::Error),
}

// ─── CLI Arguments ───────────────────────────────────────────────────────────

/// Command-line arguments parsed by `clap`.
///
/// All fields map directly to CLI flags/arguments; see `--help` for details.
#[derive(Debug, Parser)]
#[command(
    name = "rgrep",
    about = "A recursive grep replacement with coloured output and context lines",
    long_about = None,
    version
)]
pub struct Args {
    /// The regular expression pattern to search for.
    ///
    /// Rust `regex` crate syntax — e.g. `"fn\s+\w+"`.
    #[arg(value_name = "PATTERN")]
    pub pattern: String,

    /// Root path to search (file or directory).
    ///
    /// Defaults to the current directory (`.`) when omitted.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Print N lines of context before and after each matching line.
    ///
    /// Adjacent or overlapping context groups are merged; non-adjacent groups
    /// are separated by a `--` divider line.
    #[arg(short = 'C', long = "context", value_name = "N", default_value_t = 0)]
    pub context: usize,

    /// Perform a case-insensitive search.
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Print only a count of matching lines per file rather than the lines
    /// themselves. A grand total is still printed at the end.
    #[arg(long = "count")]
    pub count: bool,

    /// Disable all ANSI colour output.
    ///
    /// Colour is also disabled automatically when the `NO_COLOR` environment
    /// variable is set (per <https://no-color.org>).
    #[arg(long = "no-color")]
    pub no_color: bool,
}

// ─── Core Data Structures ────────────────────────────────────────────────────

/// A single matched (or context) line from a file.
///
/// Produced by [`search_file`] and consumed by [`print_match`] /
/// [`print_context_line`].
#[derive(Debug, Clone)]
pub struct Match {
    /// Absolute or relative path to the file containing this line.
    pub file: PathBuf,

    /// 1-based line number within the file.
    pub line_number: usize,

    /// The full text of the line (without the trailing newline).
    pub line: String,

    /// `true` when this line is an actual regex match; `false` when it is a
    /// context-only line included because of `-C N`.
    pub is_match: bool,
}

// ─── File Search ─────────────────────────────────────────────────────────────

/// Search `path` for lines matching `regex`, returning a flat list of
/// [`Match`] values that includes the matched lines and any requested context
/// lines.
///
/// # Arguments
/// * `path`    — path to the file to search (must be a regular file).
/// * `regex`   — compiled regular expression to test each line against.
/// * `context` — number of lines of context to include before/after each match.
///
/// # Returns
/// A `Vec<Match>` in file order.  Context lines that belong to two overlapping
/// ranges appear only once (deduplication happens before the vec is built).
///
/// # Errors
/// Returns [`RgrepError::Io`] if the file cannot be read.
pub fn search_file(
    path: &Path,
    regex: &Regex,
    context: usize,
) -> Result<Vec<Match>, RgrepError> {
    // Read the entire file upfront so we can random-access lines for context.
    // This trades memory for simplicity; for very large files a streaming
    // approach would be preferable.
    let content = std::fs::read_to_string(path).map_err(|source| RgrepError::Io {
        path: path.to_owned(),
        source,
    })?;

    // Collect all lines into an indexed Vec for O(1) slice access later.
    let all_lines: Vec<&str> = content.lines().collect();
    let total = all_lines.len();

    // Find the 0-based indices of all lines that contain a match.
    let match_indices: Vec<usize> = all_lines
        .iter()
        .enumerate()
        .filter(|(_, line)| regex.is_match(line))
        .map(|(i, _)| i)
        .collect();

    if match_indices.is_empty() {
        return Ok(Vec::new());
    }

    // Expand each match index into a range [start, end] that includes context.
    // Then merge overlapping/adjacent ranges so that shared lines are emitted
    // only once.
    let ranges = merge_ranges(
        match_indices.iter().map(|&i| {
            let start = i.saturating_sub(context);
            let end = (i + context).min(total.saturating_sub(1));
            (start, end, i) // (range_start, range_end, match_line)
        }),
        total,
    );

    // Build the final Vec<Match> from the merged ranges.
    let mut results = Vec::new();
    for (range_start, range_end, match_set) in ranges {
        for idx in range_start..=range_end {
            let is_match = match_set.contains(&idx);
            results.push(Match {
                file: path.to_owned(),
                line_number: idx + 1, // convert to 1-based
                line: all_lines[idx].to_owned(),
                is_match,
            });
        }
    }

    Ok(results)
}

/// A merged range record: `(range_start, range_end, set_of_match_indices)`.
type MergedRange = (usize, usize, Vec<usize>);

/// Merge a sequence of potentially overlapping `(start, end, match_idx)` ranges
/// into a minimal set of non-overlapping ranges, preserving which indices are
/// true matches.
///
/// Adjacent ranges (end + 1 == next_start) are also merged to avoid a spurious
/// `--` separator.
fn merge_ranges(
    ranges: impl Iterator<Item = (usize, usize, usize)>,
    _total: usize,
) -> Vec<MergedRange> {
    let mut merged: Vec<MergedRange> = Vec::new();

    for (start, end, match_idx) in ranges {
        if let Some(last) = merged.last_mut() {
            // Overlap or adjacency: extend the current range.
            if start <= last.1 + 1 {
                if end > last.1 {
                    last.1 = end;
                }
                if !last.2.contains(&match_idx) {
                    last.2.push(match_idx);
                }
                continue;
            }
        }
        // No overlap: start a new range.
        merged.push((start, end, vec![match_idx]));
    }

    merged
}

// ─── Output Rendering ────────────────────────────────────────────────────────

/// Print a single [`Match`] line to stdout, highlighting the matched portion
/// in red when colour is enabled.
///
/// Output format:
/// ```text
/// filename:line_number: line_content
/// ```
/// where the filename is printed in green and the line number in yellow when
/// `no_color` is `false`.
///
/// # Arguments
/// * `m`        — the match to render.
/// * `regex`    — used to locate the matched span within the line for
///                highlighting.
/// * `no_color` — when `true`, suppresses all ANSI escape codes.
pub fn print_match(m: &Match, regex: &Regex, no_color: bool) {
    let file_str = m.file.display().to_string();
    let line_num = m.line_number.to_string();

    if no_color {
        println!("{}:{}: {}", file_str, line_num, m.line);
    } else {
        // Colour the filename and line number.
        let colored_file = file_str.green();
        let colored_num = line_num.yellow();

        // Highlight every match occurrence in the line with a red background.
        let highlighted_line = highlight_matches(&m.line, regex);

        println!("{}:{}: {}", colored_file, colored_num, highlighted_line);
    }
}

/// Print a context (non-matching) line to stdout.
///
/// Context lines use the same `filename:line_number: content` format but
/// without match highlighting.
pub fn print_context_line(m: &Match, no_color: bool) {
    let file_str = m.file.display().to_string();
    let line_num = m.line_number.to_string();

    if no_color {
        println!("{}:{}: {}", file_str, line_num, m.line);
    } else {
        println!("{}:{}: {}", file_str.green(), line_num.yellow(), m.line);
    }
}

/// Replace every occurrence of `regex` in `line` with its red-coloured
/// equivalent, returning the fully annotated string.
///
/// Matches are replaced left-to-right; non-matching spans are copied verbatim.
fn highlight_matches(line: &str, regex: &Regex) -> String {
    let mut result = String::with_capacity(line.len() * 2);
    let mut last_end = 0;

    for mat in regex.find_iter(line) {
        // Copy the non-matching prefix as-is.
        result.push_str(&line[last_end..mat.start()]);
        // Append the matching span highlighted in bold red.
        result.push_str(&line[mat.start()..mat.end()].red().bold().to_string());
        last_end = mat.end();
    }
    // Append any trailing non-matching suffix.
    result.push_str(&line[last_end..]);
    result
}

// ─── Application Logic ───────────────────────────────────────────────────────

/// Run the full rgrep workflow for the given parsed [`Args`].
///
/// This is the main entry point for the application logic, separated from
/// `main()` so that it returns a typed `Result` rather than calling `process::exit`.
///
/// # Steps
/// 1. Configure colour output.
/// 2. Compile the regular expression.
/// 3. Walk the target path with `ignore::Walk`.
/// 4. For each regular file, call [`search_file`].
/// 5. Print results (or counts) and a final summary.
///
/// # Errors
/// Propagates [`RgrepError`] variants wrapped in [`anyhow::Error`].
pub fn run(args: Args) -> Result<(), anyhow::Error> {
    // ── Colour configuration ───────────────────────────────────────────────
    // Honour both --no-color and the NO_COLOR env var (https://no-color.org).
    let no_color = args.no_color || std::env::var("NO_COLOR").is_ok();
    if no_color {
        colored::control::set_override(false);
    }

    // ── Regex compilation ──────────────────────────────────────────────────
    // Prepend `(?i)` for case-insensitive matching; the regex crate supports
    // inline flags so this is the cleanest approach without branching later.
    let pattern = if args.ignore_case {
        format!("(?i){}", args.pattern)
    } else {
        args.pattern.clone()
    };
    let regex = Regex::new(&pattern)
        .map_err(RgrepError::Regex)
        .with_context(|| format!("compiling pattern `{}`", args.pattern))?;

    // ── Walk target path ───────────────────────────────────────────────────
    let root = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let walker = Walk::new(&root);

    let mut total_matches: usize = 0;
    let mut total_files: usize = 0;

    for entry in walker {
        let entry = entry.map_err(RgrepError::Walk)?;

        // Skip directories — we only process regular files.
        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let path = entry.path();

        // Attempt to skip binary files heuristically: if the file contains a
        // null byte in its first 8 KiB, treat it as binary and move on.
        if is_likely_binary(path) {
            continue;
        }

        let matches = search_file(path, &regex, args.context)
            .with_context(|| format!("searching file `{}`", path.display()))?;

        if matches.is_empty() {
            continue;
        }

        // Count only the real matches (not context lines).
        let match_count = matches.iter().filter(|m| m.is_match).count();

        if args.count {
            // In count mode, print one summary line per file.
            if match_count > 0 {
                println!("{}: {} matches", path.display(), match_count);
                total_matches += match_count;
                total_files += 1;
            }
        } else {
            // Full output mode: render each line, inserting `--` separators
            // between non-adjacent context groups.
            print_file_results(&matches, &regex, no_color);
            total_matches += match_count;
            total_files += 1;
        }
    }

    // ── Final summary ──────────────────────────────────────────────────────
    println!("\nTotal: {} matches in {} files", total_matches, total_files);

    Ok(())
}

/// Print all matches (and their context lines) for a single file, inserting
/// `--` separators between non-adjacent output groups.
///
/// # Arguments
/// * `matches`  — the flat list of match/context lines for this file,
///                in file order.
/// * `regex`    — used for highlighting in [`print_match`].
/// * `no_color` — forwarded to the print helpers.
fn print_file_results(matches: &[Match], regex: &Regex, no_color: bool) {
    // Detect gaps between consecutive lines to know when to print `--`.
    // A gap exists when two adjacent entries have non-consecutive line numbers.
    let mut prev_line_number: Option<usize> = None;

    for m in matches {
        if let Some(prev) = prev_line_number {
            // Non-consecutive line numbers indicate a gap between context groups.
            if m.line_number > prev + 1 {
                println!("--");
            }
        }

        if m.is_match {
            print_match(m, regex, no_color);
        } else {
            print_context_line(m, no_color);
        }

        prev_line_number = Some(m.line_number);
    }
}

/// Heuristically determine whether a file is binary by checking for null bytes
/// in the first 8 KiB.
///
/// Returns `true` if the file appears to be binary, `false` otherwise (including
/// if the file cannot be read, in which case we let `search_file` report the
/// error).
fn is_likely_binary(path: &Path) -> bool {
    use std::io::Read;

    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };

    let mut buf = [0u8; 8192];
    let Ok(n) = file.read(&mut buf) else {
        return false;
    };

    // The presence of a null byte is a reliable indicator of binary data.
    buf[..n].contains(&0)
}

// ─── Entry Point ─────────────────────────────────────────────────────────────

/// Application entry point.
///
/// Parses CLI arguments with `clap` and delegates to [`run`].  Any error is
/// printed to `stderr` and the process exits with code `1`.
fn main() {
    let args = Args::parse();

    if let Err(err) = run(args) {
        eprintln!("rgrep: error: {:#}", err);
        std::process::exit(1);
    }
}
