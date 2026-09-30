//! `md-render` — A terminal Markdown renderer.
//!
//! Parses a Markdown document using `pulldown-cmark` and walks its event stream,
//! translating each structural and inline element into `crossterm` ANSI escape
//! sequences for rich terminal output.
//!
//! Supported features:
//! - Six heading levels with distinct colour schemes
//! - Bold and italic inline text
//! - Inline code (bright yellow) and fenced/indented code blocks (bright cyan)
//! - Unordered and ordered lists
//! - Blockquotes with a `│ ` left-hand margin
//! - Hyperlinks displayed as `text [url]`
//! - Horizontal rules
//!
//! # Examples
//!
//! ```text
//! md-render README.md
//! echo "# Hello, world!" | md-render
//! ```

use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use crossterm::{
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser as MdParser, Tag, TagEnd};
use thiserror::Error;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors that can occur while rendering Markdown to the terminal.
#[derive(Debug, Error)]
pub enum RenderError {
    /// Wraps any [`std::io::Error`] from reading input or writing terminal output.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// A terminal styling operation failed for an unrecoverable reason.
    #[error("Terminal error: {0}")]
    Terminal(String),
}

// ── CLI ───────────────────────────────────────────────────────────────────────

/// Command-line arguments accepted by `md-render`.
#[derive(Parser)]
#[command(
    name = "md-render",
    about = "Render Markdown to styled terminal output with ANSI colours",
    long_about = "Reads a Markdown document and writes a richly styled representation \
                  to the terminal, using ANSI escape codes for colours and text attributes.\n\n\
                  With no FILE argument, reads from standard input."
)]
struct Args {
    /// Path to the Markdown file to render.
    ///
    /// Omit to read from standard input, enabling pipeline usage:
    /// `echo "# Hello" | md-render`
    file: Option<PathBuf>,
}

// ── Renderer state machine ────────────────────────────────────────────────────

/// State tracked as we walk the `pulldown-cmark` event stream.
///
/// Each field corresponds to a Markdown construct we may be nested inside.
/// Because Markdown is a tree, multiple flags can be `true` simultaneously —
/// for example, bold text inside a blockquote inside an ordered list.
///
/// The renderer is deliberately kept *stateless with respect to terminal style*:
/// [`styled_print`] resets all ANSI attributes after every text segment, so the
/// Rust flags here are the single source of truth for "what style applies next."
struct Renderer {
    /// The current heading depth (1 = H1 … 6 = H6), or `None` outside headings.
    heading_level: Option<u32>,
    /// `true` while we are processing a `> blockquote` block.
    in_blockquote: bool,
    /// Counter for the next ordered-list item number; set from `Tag::List(Some(n))`.
    ordered_list_counter: u32,
    /// `true` while inside an unordered (bullet) list.
    in_unordered_list: bool,
    /// `true` while inside a numbered (ordered) list.
    in_ordered_list: bool,
    /// `true` while inside a fenced or indented code block.
    in_code_block: bool,
    /// `true` while inside `**strong**` / `__strong__` text.
    in_strong: bool,
    /// `true` while inside `*emphasis*` / `_emphasis_` text.
    in_emphasis: bool,
    /// The destination URL of the hyperlink currently being processed, or `None`.
    ///
    /// Stored here so we can append it after the link's inner text is rendered.
    link_url: Option<String>,
    /// `true` when a blank line should be emitted before the next block element.
    pending_newline: bool,
    /// `true` after the first block has been emitted; suppresses a leading blank line.
    started: bool,
}

impl Renderer {
    /// Creates a new renderer with all state flags at their initial values.
    fn new() -> Self {
        Self {
            heading_level: None,
            in_blockquote: false,
            ordered_list_counter: 1,
            in_unordered_list: false,
            in_ordered_list: false,
            in_code_block: false,
            in_strong: false,
            in_emphasis: false,
            link_url: None,
            pending_newline: false,
            started: false,
        }
    }

    /// Dispatch one event from the pulldown-cmark parser to the correct handler.
    ///
    /// This is the central method of the renderer. It pattern-matches on every
    /// event variant that `md-render` cares about and either updates state flags
    /// or emits styled text to `stdout`. Unknown variants are silently skipped —
    /// this makes the renderer forward-compatible with new pulldown-cmark events.
    ///
    /// # Parameters
    /// - `event`  — the next event from the Markdown parser.
    /// - `stdout` — any writer that accepts terminal escape sequences.
    ///
    /// # Errors
    /// Returns [`RenderError::Io`] if a write to `stdout` fails.
    fn handle_event<W: Write>(&mut self, event: Event, stdout: &mut W) -> Result<(), RenderError> {
        match event {
            // ── Headings ──────────────────────────────────────────────────────
            Event::Start(Tag::Heading { level, .. }) => {
                // Insert a blank separator line before every heading except the first
                // element in the document (when `started` is still false).
                if self.started {
                    execute!(stdout, Print("\n"))?;
                }
                self.started = true;
                self.pending_newline = false;
                self.heading_level = Some(heading_level_to_u32(level));
            }
            Event::End(TagEnd::Heading(_)) => {
                self.heading_level = None;
                // Full attribute reset ensures no colour or weight bleeds past the heading.
                execute!(stdout, SetAttribute(Attribute::Reset), Print("\n"))?;
                self.pending_newline = true;
            }

            // ── Paragraphs ────────────────────────────────────────────────────
            Event::Start(Tag::Paragraph) => {
                if self.in_blockquote {
                    // Each blockquote paragraph starts with the margin marker.
                    execute!(
                        stdout,
                        SetAttribute(Attribute::Dim),
                        Print("│ "),
                        SetAttribute(Attribute::Reset)
                    )?;
                } else if self.pending_newline && self.started {
                    // Separate consecutive non-blockquote paragraphs with a blank line.
                    execute!(stdout, Print("\n"))?;
                }
                self.started = true;
                self.pending_newline = false;
            }
            Event::End(TagEnd::Paragraph) => {
                execute!(stdout, SetAttribute(Attribute::Reset), Print("\n"))?;
                self.pending_newline = true;
            }

            // ── Blockquotes ───────────────────────────────────────────────────
            Event::Start(Tag::BlockQuote(_)) => {
                // Tag::BlockQuote(Option<BlockQuoteKind>) in pulldown-cmark 0.12;
                // the kind encodes GitHub-style alert types (Note, Warning, …).
                self.in_blockquote = true;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                self.in_blockquote = false;
                execute!(stdout, SetAttribute(Attribute::Reset), Print("\n"))?;
                self.pending_newline = true;
            }

            // ── Fenced / indented code blocks ────────────────────────────────
            Event::Start(Tag::CodeBlock(_)) => {
                if self.pending_newline && self.started {
                    execute!(stdout, Print("\n"))?;
                }
                self.started = true;
                self.in_code_block = true;
                // Visual divider above the code block, printed in dim grey.
                execute!(
                    stdout,
                    SetAttribute(Attribute::Dim),
                    Print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"),
                    SetAttribute(Attribute::Reset)
                )?;
                self.pending_newline = false;
            }
            Event::End(TagEnd::CodeBlock) => {
                self.in_code_block = false;
                // Matching visual divider below the code block.
                execute!(
                    stdout,
                    SetAttribute(Attribute::Dim),
                    Print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"),
                    SetAttribute(Attribute::Reset)
                )?;
                self.pending_newline = true;
            }

            // ── Lists ─────────────────────────────────────────────────────────
            Event::Start(Tag::List(None)) => {
                // `None` discriminant signals an unordered (bullet) list.
                self.in_unordered_list = true;
                self.started = true;
            }
            Event::End(TagEnd::List(false)) => {
                // `false` = the unordered list has ended.
                self.in_unordered_list = false;
                self.pending_newline = true;
            }
            Event::Start(Tag::List(Some(start))) => {
                // `Some(n)` signals an ordered list beginning at item `n`.
                self.in_ordered_list = true;
                // Cast from u64 to u32 is safe for any realistic document length.
                self.ordered_list_counter = start as u32;
                self.started = true;
            }
            Event::End(TagEnd::List(true)) => {
                // `true` = the ordered list has ended.
                self.in_ordered_list = false;
                self.pending_newline = true;
            }
            Event::Start(Tag::Item) => {
                if self.in_unordered_list {
                    // Unicode bullet keeps the output clean on modern terminals.
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Cyan),
                        Print("  • "),
                        SetAttribute(Attribute::Reset)
                    )?;
                } else {
                    // Ordered item: print the current counter, then advance it.
                    let n = self.ordered_list_counter;
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Cyan),
                        Print(format!("  {n}. ")),
                        SetAttribute(Attribute::Reset)
                    )?;
                    self.ordered_list_counter += 1;
                }
            }
            Event::End(TagEnd::Item) => {
                execute!(stdout, SetAttribute(Attribute::Reset), Print("\n"))?;
            }

            // ── Inline formatting ─────────────────────────────────────────────
            //
            // We only flip our state flags here. The actual ANSI attributes are
            // applied in `render_inline_text` via `styled_print`, which resets
            // styles after every text segment — so there is nothing to undo here.
            Event::Start(Tag::Strong) => {
                self.in_strong = true;
            }
            Event::End(TagEnd::Strong) => {
                self.in_strong = false;
            }
            Event::Start(Tag::Emphasis) => {
                self.in_emphasis = true;
            }
            Event::End(TagEnd::Emphasis) => {
                self.in_emphasis = false;
            }

            // ── Hyperlinks ────────────────────────────────────────────────────
            Event::Start(Tag::Link { dest_url, .. }) => {
                // Stash the URL; we append it in dim style after the link text.
                self.link_url = Some(dest_url.into_string());
            }
            Event::End(TagEnd::Link) => {
                if let Some(url) = self.link_url.take() {
                    execute!(
                        stdout,
                        SetAttribute(Attribute::Dim),
                        Print(format!(" [{url}]")),
                        SetAttribute(Attribute::Reset)
                    )?;
                }
            }

            // ── Inline code ───────────────────────────────────────────────────
            Event::Code(text) => {
                // Inline code is a *leaf* event in pulldown-cmark — it is not
                // wrapped in Start/End tag pairs like strong or emphasis.
                styled_print(
                    &text,
                    stdout,
                    Some(Color::Yellow),
                    &[Attribute::Bold],
                    None,
                )?;
            }

            // ── Text (the bulk of rendered content) ───────────────────────────
            Event::Text(text) => {
                self.render_text(&text, stdout)?;
            }

            // ── Thematic break (horizontal rule) ──────────────────────────────
            Event::Rule => {
                execute!(
                    stdout,
                    SetAttribute(Attribute::Dim),
                    Print("──────────────────────────────\n"),
                    SetAttribute(Attribute::Reset)
                )?;
            }

            // ── Line breaks ───────────────────────────────────────────────────
            Event::SoftBreak => {
                if self.in_blockquote {
                    // Continuation lines inside a blockquote need the `│ ` prefix
                    // on the next line so the margin is visually consistent.
                    execute!(
                        stdout,
                        Print("\n"),
                        SetAttribute(Attribute::Dim),
                        Print("│ "),
                        SetAttribute(Attribute::Reset)
                    )?;
                } else {
                    execute!(stdout, Print("\n"))?;
                }
            }
            Event::HardBreak => {
                execute!(stdout, Print("\n"))?;
            }

            // All other events (raw HTML, footnote definitions, metadata blocks,
            // table structure, task-list checkboxes …) are intentionally ignored.
            // Expanding their handling is left as an exercise.
            _ => {}
        }
        Ok(())
    }

    /// Route a `Text` event to the appropriate sub-renderer based on current state.
    ///
    /// Text can appear inside headings, code blocks, paragraphs, and blockquotes,
    /// each requiring different styling. This method checks the state flags in
    /// priority order: code blocks take precedence over headings, which take
    /// precedence over inline formatting.
    ///
    /// # Parameters
    /// - `text`   — the raw text string from the parser event.
    /// - `stdout` — the terminal writer.
    ///
    /// # Errors
    /// Returns [`RenderError::Io`] on write failure.
    fn render_text<W: Write>(&self, text: &str, stdout: &mut W) -> Result<(), RenderError> {
        if self.in_code_block {
            // Code-block content: bright cyan to distinguish from prose.
            styled_print(text, stdout, Some(Color::Cyan), &[], None)?;
        } else if let Some(level) = self.heading_level {
            // Heading text: colour and weight depend on heading depth.
            self.render_heading_text(text, level, stdout)?;
        } else {
            // Normal paragraph, list item, or blockquote inline text.
            self.render_inline_text(text, stdout)?;
        }
        Ok(())
    }

    /// Render text inside a heading with level-appropriate colour and weight.
    ///
    /// | Level | Foreground | Attributes               |
    /// |-------|-----------|--------------------------|
    /// | H1    | Yellow    | Bold + Underlined         |
    /// | H2    | Cyan      | Bold                     |
    /// | H3    | Green     | Bold                     |
    /// | H4–H6 | (default) | Bold                     |
    ///
    /// # Parameters
    /// - `text`   — heading text content.
    /// - `level`  — heading depth as a `u32` (1–6).
    /// - `stdout` — terminal writer.
    ///
    /// # Errors
    /// Propagates [`RenderError::Io`] from [`styled_print`].
    fn render_heading_text<W: Write>(
        &self,
        text: &str,
        level: u32,
        stdout: &mut W,
    ) -> Result<(), RenderError> {
        match level {
            1 => styled_print(
                text,
                stdout,
                Some(Color::Yellow),
                &[Attribute::Bold, Attribute::Underlined],
                None,
            ),
            2 => styled_print(text, stdout, Some(Color::Cyan), &[Attribute::Bold], None),
            3 => styled_print(text, stdout, Some(Color::Green), &[Attribute::Bold], None),
            // H4, H5, H6 — bold in the terminal's default foreground colour.
            _ => styled_print(text, stdout, None, &[Attribute::Bold], None),
        }
    }

    /// Render paragraph or list-item text, applying any active inline formatting.
    ///
    /// Reads `in_strong`, `in_emphasis`, and `in_blockquote` flags to build
    /// the attribute list, then delegates to [`styled_print`].
    ///
    /// # Parameters
    /// - `text`   — the text content to render.
    /// - `stdout` — terminal writer.
    ///
    /// # Errors
    /// Propagates [`RenderError::Io`] from [`styled_print`].
    fn render_inline_text<W: Write>(&self, text: &str, stdout: &mut W) -> Result<(), RenderError> {
        let mut attrs: Vec<Attribute> = Vec::new();
        if self.in_strong {
            attrs.push(Attribute::Bold);
        }
        if self.in_emphasis {
            attrs.push(Attribute::Italic);
        }
        if self.in_blockquote {
            // Blockquote prose is rendered in a muted (dim) style to visually
            // separate it from surrounding document text.
            attrs.push(Attribute::Dim);
        }
        styled_print(text, stdout, None, &attrs, None)
    }
}

// ── Free functions ────────────────────────────────────────────────────────────

/// Convert a [`HeadingLevel`] enum variant to its integer depth (1–6).
///
/// `pulldown-cmark` models heading levels as an opaque enum rather than a plain
/// integer, so we convert them here to avoid depending on any unstable numeric
/// representation.
fn heading_level_to_u32(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Print `text` to `stdout` with optional foreground colour, background colour,
/// and a list of terminal attributes; unconditionally reset all styles afterwards.
///
/// Resetting after every call means callers never need to manually undo a style.
/// This "apply → print → reset" pattern keeps the renderer correct even when
/// styled regions are not perfectly nested.
///
/// # Parameters
/// - `text`   — the string to emit.
/// - `stdout` — terminal writer (typically `std::io::stdout()`).
/// - `fg`     — optional foreground [`Color`] to apply before printing.
/// - `attrs`  — zero or more [`Attribute`]s (bold, italic, underline, dim, …).
/// - `bg`     — optional background [`Color`]; included for completeness and
///              extension, but not used by current style rules.
///
/// # Errors
/// Returns [`RenderError::Io`] on any write failure.
fn styled_print<W: Write>(
    text: &str,
    stdout: &mut W,
    fg: Option<Color>,
    attrs: &[Attribute],
    bg: Option<Color>,
) -> Result<(), RenderError> {
    // Apply each requested terminal attribute in order.
    for &attr in attrs {
        execute!(stdout, SetAttribute(attr))?;
    }
    if let Some(color) = fg {
        execute!(stdout, SetForegroundColor(color))?;
    }
    if let Some(color) = bg {
        use crossterm::style::SetBackgroundColor;
        execute!(stdout, SetBackgroundColor(color))?;
    }
    // Print the content, then perform a full attribute + colour reset.
    // `Attribute::Reset` emits ESC[0m which clears every active attribute.
    execute!(stdout, Print(text), SetAttribute(Attribute::Reset), ResetColor)?;
    Ok(())
}

/// Parse `markdown` and render it to stdout with ANSI terminal styles.
///
/// Creates a [`pulldown_cmark::Parser`] with all extension options enabled
/// (tables, footnotes, strikethrough, task lists, …), instantiates a fresh
/// [`Renderer`] state machine, then drives the event loop until the input is
/// exhausted.
///
/// Unrecognised or unimplemented events are silently ignored so the renderer
/// degrades gracefully when encountering advanced Markdown features.
///
/// # Errors
/// Propagates [`RenderError`] from any failed terminal write.
pub fn render(markdown: &str) -> Result<(), RenderError> {
    // Enable all pulldown-cmark extension options so GFM tables, footnotes,
    // strikethrough, and task-list checkboxes parse without errors.
    let options = Options::all();
    let parser = MdParser::new_ext(markdown, options);

    let mut renderer = Renderer::new();
    let mut stdout = io::stdout();

    for event in parser {
        renderer.handle_event(event, &mut stdout)?;
    }

    // Ensure the shell prompt appears on a fresh line after the rendered document.
    execute!(stdout, SetAttribute(Attribute::Reset), ResetColor, Print("\n"))?;
    Ok(())
}

/// Application entry point — parses arguments and drives the render pipeline.
///
/// Reads source Markdown from a file (if provided) or from standard input,
/// then calls [`render`]. Any error is propagated as an [`anyhow::Error`],
/// causing `main` to print a human-friendly message and exit with status 1.
fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let markdown = match args.file {
        Some(ref path) => {
            // Read the entire file into memory. For a terminal renderer this is
            // acceptable — the file must fit in memory to be formatted anyway.
            std::fs::read_to_string(path)
                .with_context(|| format!("failed to read '{}'", path.display()))?
        }
        None => {
            // No FILE argument — drain stdin until EOF so we can work with the
            // full document (pulldown-cmark is not a streaming parser).
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .context("failed to read from stdin")?;
            buf
        }
    };

    render(&markdown).map_err(|e| anyhow::anyhow!(e))
}
