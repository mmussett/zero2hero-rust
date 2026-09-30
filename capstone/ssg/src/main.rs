//! Static Site Generator — converts Markdown + Jinja2 templates to a deployable HTML site.
//!
//! # Subcommands
//! - `ssg init` — scaffold a new site with sample content and default templates
//! - `ssg build [options]` — render all Markdown files in `content/` to `output/`
//! - `ssg new <title>` — create a new Markdown post with pre-filled front matter
//!
//! # Directory Layout
//! ```text
//! content/      Markdown source files (may be nested)
//! templates/    Jinja2 HTML templates (base.html, post.html, index.html)
//! static/       Assets copied unchanged to output/static/
//! output/       Generated site ready to serve
//! ```

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use pulldown_cmark::{html as md_html, Options, Parser as CmarkParser};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

// ── Error type ────────────────────────────────────────────────────────────────

/// Domain errors specific to the static site generator.
#[derive(Debug, Error)]
pub enum SsgError {
    /// An I/O error occurred while reading or writing a file or directory.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// A Jinja2 template error from minijinja (syntax or rendering failure).
    #[error("Template error: {0}")]
    Template(#[from] minijinja::Error),
    /// A Markdown file that requires front matter is missing it.
    #[error("Front matter missing in '{0}'")]
    FrontMatterMissing(String),
    /// A date string in front matter could not be parsed as YYYY-MM-DD.
    #[error("Invalid date '{0}': expected YYYY-MM-DD format")]
    InvalidDate(String),
}

// ── Data types ────────────────────────────────────────────────────────────────

/// Metadata extracted from a Markdown file's front-matter block.
///
/// Front matter is delimited by `---` lines at the top of the file and
/// contains `key: value` pairs (no external YAML library required).
#[derive(Debug, Clone)]
pub struct FrontMatter {
    /// Post or page title; defaults to `"Untitled"` when absent.
    pub title: String,
    /// Optional publication date in `YYYY-MM-DD` format.
    pub date: Option<String>,
    /// Comma-separated tags, split and trimmed during parsing.
    pub tags: Vec<String>,
    /// When `true`, this post is excluded from all index and tag pages.
    pub draft: bool,
}

/// A fully processed page ready to be written to `output/`.
#[derive(Debug, Clone)]
pub struct Page {
    /// Parsed front-matter metadata.
    pub meta: FrontMatter,
    /// Markdown body converted to an HTML string.
    pub html_body: String,
    /// Source path relative to the content root (`posts/hello.md`).
    pub rel_path: PathBuf,
    /// Destination path relative to the output root (`posts/hello.html`).
    pub out_path: PathBuf,
    /// Slash-delimited URL suitable for `<a href>` (`/posts/hello.html`).
    pub url: String,
}

/// A compact view of a `Page` used in index and tag-listing templates.
#[derive(Debug, Serialize)]
pub struct PostSummary {
    /// Post title.
    pub title: String,
    /// Absolute URL of the post.
    pub url: String,
    /// Publication date, or empty string when absent.
    pub date: String,
    /// List of tag names.
    pub tags: Vec<String>,
}

// ── Front matter ──────────────────────────────────────────────────────────────

/// Parse the front matter from the beginning of `content`.
///
/// Returns `(Some(FrontMatter), body)` if a `---`-delimited block is found,
/// otherwise `(None, original_content)`.
pub fn parse_front_matter(content: &str) -> (Option<FrontMatter>, &str) {
    // Front matter must begin with --- immediately at the start of the file.
    let rest = match content.strip_prefix("---") {
        Some(r) => r,
        None => return (None, content),
    };
    // Require a newline immediately after the opening ---
    let rest = match rest.strip_prefix('\n').or_else(|| rest.strip_prefix("\r\n")) {
        Some(r) => r,
        None => return (None, content),
    };

    // Locate the closing --- on its own line
    let close = "\n---";
    let close_pos = match rest.find(close) {
        Some(p) => p,
        None => return (None, content),
    };

    let fm_text = &rest[..close_pos];
    let after_close = &rest[close_pos + close.len()..];
    // Skip the newline that follows the closing ---
    let body = after_close
        .strip_prefix('\n')
        .or_else(|| after_close.strip_prefix("\r\n"))
        .unwrap_or(after_close);

    let fm = parse_kv_pairs(fm_text);
    (Some(fm), body)
}

/// Parse `key: value` lines into a `FrontMatter`.
///
/// Unknown keys are silently ignored; missing keys receive sensible defaults.
fn parse_kv_pairs(text: &str) -> FrontMatter {
    let mut title = String::new();
    let mut date: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut draft = false;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Split on the first colon only — values may themselves contain colons
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "title" => title = value.to_string(),
                "date" => date = Some(value.to_string()),
                "tags" => {
                    tags = value
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect();
                }
                "draft" => draft = value.eq_ignore_ascii_case("true"),
                _ => {} // Silently ignore unrecognised keys
            }
        }
    }

    if title.is_empty() {
        title = "Untitled".to_string();
    }

    FrontMatter { title, date, tags, draft }
}

// ── Markdown conversion ───────────────────────────────────────────────────────

/// Convert a Markdown string to an HTML string using pulldown-cmark.
///
/// Enables tables, strikethrough, footnotes, and task-list checkboxes.
pub fn markdown_to_html(markdown: &str) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS;
    let parser = CmarkParser::new_ext(markdown, opts);
    let mut html_out = String::new();
    md_html::push_html(&mut html_out, parser);
    html_out
}

// ── Template system ───────────────────────────────────────────────────────────

/// Source for the default `base.html` template (full HTML5 skeleton).
const DEFAULT_BASE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{% block title %}My Site{% endblock %}</title>
  <link rel="stylesheet" href="/static/style.css">
</head>
<body>
  <header>
    <nav><a href="/">Home</a></nav>
  </header>
  <main>
    {% block content %}{% endblock %}
  </main>
  <footer><p>Built with <strong>ssg</strong></p></footer>
</body>
</html>
"#;

/// Source for the default `post.html` template (extends base, renders an article).
const DEFAULT_POST: &str = r#"{% extends "base.html" %}
{% block title %}{{ title }} – My Site{% endblock %}
{% block content %}
<article>
  <h1>{{ title }}</h1>
  {% if date %}<time datetime="{{ date }}">{{ date }}</time>{% endif %}
  {% if tags %}
  <ul class="tags">
    {% for tag in tags %}
    <li><a href="/tags/{{ tag }}.html">{{ tag }}</a></li>
    {% endfor %}
  </ul>
  {% endif %}
  <div class="content">{{ content | safe }}</div>
</article>
{% endblock %}
"#;

/// Source for the default `index.html` template (extends base, renders a post list).
const DEFAULT_INDEX: &str = r#"{% extends "base.html" %}
{% block title %}{% if tag %}Posts tagged "{{ tag }}"{% else %}All Posts{% endif %} – My Site{% endblock %}
{% block content %}
{% if tag %}
<h1>Posts tagged "{{ tag }}"</h1>
{% else %}
<h1>All Posts</h1>
{% endif %}
{% if posts %}
<ul class="posts">
  {% for post in posts %}
  <li>
    <a href="{{ post.url }}">{{ post.title }}</a>
    {% if post.date %}<span class="date">{{ post.date }}</span>{% endif %}
    {% if post.tags %}
    <span class="tags">
      {% for t in post.tags %}
      <a href="/tags/{{ t }}.html">{{ t }}</a>{% if not loop.last %},{% endif %}
      {% endfor %}
    </span>
    {% endif %}
  </li>
  {% endfor %}
</ul>
{% else %}
<p>No posts yet. Run <code>ssg new "My First Post"</code> to get started.</p>
{% endif %}
{% endblock %}
"#;

/// Write the three default templates into `dir`, skipping files that already exist.
pub fn write_default_templates(dir: &Path) -> Result<(), SsgError> {
    fs::create_dir_all(dir)?;
    let write_if_new = |name: &str, src: &str| -> Result<(), SsgError> {
        let p = dir.join(name);
        if !p.exists() {
            fs::write(&p, src)?;
        }
        Ok(())
    };
    write_if_new("base.html", DEFAULT_BASE)?;
    write_if_new("post.html", DEFAULT_POST)?;
    write_if_new("index.html", DEFAULT_INDEX)?;
    Ok(())
}

/// Build a `minijinja::Environment` that lazy-loads templates from `templates_dir`.
///
/// Uses a closure-based loader so the environment carries no borrowed template
/// sources and has a `'static` lifetime, making it easy to pass around.
pub fn make_env(templates_dir: PathBuf) -> minijinja::Environment<'static> {
    let mut env = minijinja::Environment::new();
    env.set_loader(move |name: &str| {
        let path = templates_dir.join(name);
        match fs::read_to_string(&path) {
            Ok(s) => Ok(Some(s)),
            // A missing template is not found — return None so minijinja tries the next loader.
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                e.to_string(),
            )),
        }
    });
    env
}

// ── URL helpers ───────────────────────────────────────────────────────────────

/// Convert an output-relative `PathBuf` to a slash-delimited URL string.
///
/// Example: `posts\hello.html` → `/posts/hello.html`
fn path_to_url(p: &Path) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    format!("/{}", s)
}

/// Slugify a title string for use as a filename.
///
/// Converts to lowercase and replaces any non-alphanumeric character with a
/// hyphen, collapsing consecutive hyphens.
fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut prev_hyphen = false;
    for ch in title.chars() {
        if ch.is_alphanumeric() {
            slug.push(ch.to_lowercase().next().unwrap_or(ch));
            prev_hyphen = false;
        } else if !prev_hyphen && !slug.is_empty() {
            slug.push('-');
            prev_hyphen = true;
        }
    }
    // Trim trailing hyphen
    slug.trim_end_matches('-').to_string()
}

// ── Asset pipeline ─────────────────────────────────────────────────────────

/// Copy all files from `static_dir` to `output_dir/static/` preserving structure.
///
/// Non-existent `static_dir` is silently skipped (the site may have no static assets).
pub fn copy_assets(static_dir: &Path, output_dir: &Path) -> Result<(), SsgError> {
    if !static_dir.exists() {
        return Ok(());
    }
    let out_static = output_dir.join("static");
    for entry in WalkDir::new(static_dir) {
        let entry = entry.map_err(|e| SsgError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
        let src = entry.path();
        // Compute destination: replace static_dir prefix with output/static/
        let rel = src
            .strip_prefix(static_dir)
            .map_err(|e| SsgError::Io(io::Error::new(io::ErrorKind::InvalidData, e)))?;
        let dst = out_static.join(rel);
        if src.is_dir() {
            fs::create_dir_all(&dst)?;
        } else {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(src, &dst)?;
        }
    }
    Ok(())
}

// ── Default CSS ───────────────────────────────────────────────────────────────

/// Minimal CSS written to `static/style.css` during `ssg init`.
const DEFAULT_CSS: &str = r#"*, *::before, *::after { box-sizing: border-box; }
body { font-family: system-ui, sans-serif; max-width: 800px; margin: 2rem auto; padding: 0 1rem; line-height: 1.6; }
header, footer { border-top: 1px solid #eee; padding: 0.5rem 0; }
header { border-top: none; border-bottom: 1px solid #eee; margin-bottom: 2rem; }
h1, h2, h3 { line-height: 1.2; }
a { color: #0066cc; }
code { background: #f4f4f4; padding: 0.1em 0.3em; border-radius: 3px; font-size: 0.9em; }
pre code { background: none; }
pre { background: #f4f4f4; padding: 1rem; border-radius: 4px; overflow-x: auto; }
ul.posts { list-style: none; padding: 0; }
ul.posts li { margin: 0.8rem 0; display: flex; align-items: baseline; gap: 1rem; }
ul.tags, ul.tags li { display: inline; list-style: none; padding: 0; }
span.date { color: #666; font-size: 0.9em; }
span.tags a { font-size: 0.85em; color: #555; background: #eee; padding: 0.1em 0.4em; border-radius: 3px; text-decoration: none; }
"#;

// ── Site builder ──────────────────────────────────────────────────────────────

/// Walk `content_dir`, parse every `.md` file, convert to HTML, and return the list.
///
/// Draft posts (front matter `draft: true`) are included in the returned vec so
/// callers can decide whether to publish or skip them.
pub fn collect_pages(content_dir: &Path) -> Result<Vec<Page>, SsgError> {
    let mut pages = Vec::new();

    for entry in WalkDir::new(content_dir).sort_by_file_name() {
        let entry = entry.map_err(|e| SsgError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
        let src = entry.path();

        // Only process .md files
        if !src.is_file() || src.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let raw = fs::read_to_string(src)?;
        let (fm_opt, body) = parse_front_matter(&raw);

        // Use front matter if present; fall back to filename-derived defaults
        let meta = fm_opt.unwrap_or_else(|| FrontMatter {
            title: src
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .replace('-', " ")
                .replace('_', " "),
            date: None,
            tags: Vec::new(),
            draft: false,
        });

        let html_body = markdown_to_html(body);

        // Build relative paths: strip the content_dir prefix
        let rel = src
            .strip_prefix(content_dir)
            .map_err(|e| SsgError::Io(io::Error::new(io::ErrorKind::InvalidData, e)))?
            .to_path_buf();

        // Change extension .md → .html for the output path
        let out_path = rel.with_extension("html");
        let url = path_to_url(&out_path);

        pages.push(Page { meta, html_body, rel_path: rel, out_path, url });
    }

    Ok(pages)
}

/// Render a single `page` using `post.html` and write it to `output_dir`.
fn render_page(page: &Page, env: &minijinja::Environment<'_>, output_dir: &Path) -> Result<(), SsgError> {
    let tmpl = env.get_template("post.html")?;
    let ctx = serde_json::json!({
        "title": page.meta.title,
        "content": page.html_body,
        "date": page.meta.date.as_deref().unwrap_or(""),
        "tags": page.meta.tags,
    });
    let rendered = tmpl.render(ctx)?;

    let dst = output_dir.join(&page.out_path);
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&dst, rendered)?;
    Ok(())
}

/// Render `output/index.html` listing all published, non-draft posts by date.
fn render_index(pages: &[Page], env: &minijinja::Environment<'_>, output_dir: &Path) -> Result<(), SsgError> {
    // Sort by date descending (lexicographic comparison works for YYYY-MM-DD)
    let mut published: Vec<&Page> = pages.iter().filter(|p| !p.meta.draft).collect();
    published.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));

    let summaries: Vec<PostSummary> = published
        .iter()
        .map(|p| PostSummary {
            title: p.meta.title.clone(),
            url: p.url.clone(),
            date: p.meta.date.clone().unwrap_or_default(),
            tags: p.meta.tags.clone(),
        })
        .collect();

    let tmpl = env.get_template("index.html")?;
    let ctx = serde_json::json!({ "posts": summaries, "tag": null });
    let rendered = tmpl.render(ctx)?;
    fs::write(output_dir.join("index.html"), rendered)?;
    Ok(())
}

/// Generate one `output/tags/<TAG>.html` page for every unique tag in `pages`.
fn render_tag_pages(pages: &[Page], env: &minijinja::Environment<'_>, output_dir: &Path) -> Result<(), SsgError> {
    // Group non-draft pages by tag
    let mut by_tag: HashMap<String, Vec<&Page>> = HashMap::new();
    for page in pages.iter().filter(|p| !p.meta.draft) {
        for tag in &page.meta.tags {
            by_tag.entry(tag.clone()).or_default().push(page);
        }
    }

    let tags_dir = output_dir.join("tags");
    fs::create_dir_all(&tags_dir)?;

    for (tag, mut tag_pages) in by_tag {
        // Sort posts within a tag by date descending
        tag_pages.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));
        let summaries: Vec<PostSummary> = tag_pages
            .iter()
            .map(|p| PostSummary {
                title: p.meta.title.clone(),
                url: p.url.clone(),
                date: p.meta.date.clone().unwrap_or_default(),
                tags: p.meta.tags.clone(),
            })
            .collect();

        let tmpl = env.get_template("index.html")?;
        let ctx = serde_json::json!({ "posts": summaries, "tag": tag });
        let rendered = tmpl.render(ctx)?;
        fs::write(tags_dir.join(format!("{}.html", tag)), rendered)?;
    }
    Ok(())
}

/// Full site build: collect pages → render all pages + index + tags → copy assets.
pub fn build_site(content_dir: &Path, output_dir: &Path, templates_dir: &Path) -> Result<(), SsgError> {
    fs::create_dir_all(output_dir)?;
    let env = make_env(templates_dir.to_path_buf());
    let pages = collect_pages(content_dir)?;

    for page in &pages {
        render_page(page, &env, output_dir)?;
    }
    render_index(&pages, &env, output_dir)?;
    render_tag_pages(&pages, &env, output_dir)?;

    // Copy static assets (if the static/ directory exists)
    let static_dir = content_dir
        .parent()
        .map(|p| p.join("static"))
        .unwrap_or_else(|| PathBuf::from("static"));
    copy_assets(&static_dir, output_dir)?;

    Ok(())
}

// ── CLI ───────────────────────────────────────────────────────────────────────

/// A command-line static site generator: Markdown + templates → HTML.
#[derive(Debug, Parser)]
#[command(name = "ssg", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Build the site from Markdown sources.
    Build(BuildArgs),
    /// Scaffold a new site in the current directory.
    Init,
    /// Create a new post in content/ with pre-filled front matter.
    New {
        /// Post title (will be slugified for the filename).
        title: String,
    },
}

/// Arguments for the `build` subcommand.
#[derive(Debug, clap::Args)]
struct BuildArgs {
    /// Directory containing Markdown source files.
    #[arg(long, default_value = "content")]
    content: PathBuf,
    /// Directory to write generated HTML into.
    #[arg(long, default_value = "output")]
    output: PathBuf,
    /// Directory containing Jinja2 templates.
    #[arg(long, default_value = "templates")]
    templates: PathBuf,
}

// ── Subcommand handlers ────────────────────────────────────────────────────────

/// Handle `ssg init`: scaffold directories, default templates, sample post, and CSS.
fn cmd_init() -> Result<()> {
    for dir in &["content", "templates", "static", "output"] {
        fs::create_dir_all(dir).with_context(|| format!("creating {dir}/"))?;
    }
    write_default_templates(Path::new("templates"))
        .context("writing default templates")?;

    // Write a minimal default stylesheet
    let css_path = Path::new("static/style.css");
    if !css_path.exists() {
        fs::write(css_path, DEFAULT_CSS).context("writing static/style.css")?;
    }

    // Create a sample post
    let sample = Path::new("content/hello-world.md");
    if !sample.exists() {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let content = format!(
            "---\ntitle: Hello World\ndate: {}\ntags: intro, rust\ndraft: false\n---\n\n# Hello, World!\n\nWelcome to your new site built with **ssg**.\n\nEdit `content/hello-world.md`, then run `ssg build` to regenerate.\n"
        , today);
        fs::write(sample, content).context("writing content/hello-world.md")?;
    }

    println!("Site scaffolded. Run `ssg build` to generate output/.");
    Ok(())
}

/// Handle `ssg build`: read sources, render templates, write output.
fn cmd_build(args: &BuildArgs) -> Result<()> {
    if !args.content.exists() {
        anyhow::bail!("content directory '{}' not found — run `ssg init` first", args.content.display());
    }
    if !args.templates.exists() {
        anyhow::bail!("templates directory '{}' not found — run `ssg init` first", args.templates.display());
    }

    build_site(&args.content, &args.output, &args.templates)
        .with_context(|| format!("building site from '{}'", args.content.display()))?;

    println!("Site built → '{}'", args.output.display());
    Ok(())
}

/// Handle `ssg new <title>`: create a new Markdown file in `content/`.
fn cmd_new(title: &str) -> Result<()> {
    let slug = slugify(title);
    if slug.is_empty() {
        anyhow::bail!("could not derive a valid filename slug from title: {:?}", title);
    }
    let filename = format!("content/{}.md", slug);
    let path = Path::new(&filename);
    if path.exists() {
        anyhow::bail!("file already exists: {}", filename);
    }
    fs::create_dir_all("content").context("creating content/")?;
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let content = format!(
        "---\ntitle: {}\ndate: {}\ntags: \ndraft: false\n---\n\n# {}\n\nWrite your post here.\n",
        title, today, title
    );
    fs::write(path, content).with_context(|| format!("writing {}", filename))?;
    println!("Created: {}", filename);
    Ok(())
}

// ── Entry point ────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Init => cmd_init(),
        Commands::Build(args) => cmd_build(args),
        Commands::New { title } => cmd_new(title),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that well-formed front matter is parsed correctly.
    #[test]
    fn test_front_matter_full() {
        let src = "---\ntitle: My Post\ndate: 2024-06-01\ntags: rust, wasm\ndraft: false\n---\n\n# Body";
        let (fm_opt, body) = parse_front_matter(src);
        let fm = fm_opt.expect("should have front matter");
        assert_eq!(fm.title, "My Post");
        assert_eq!(fm.date.as_deref(), Some("2024-06-01"));
        assert_eq!(fm.tags, vec!["rust", "wasm"]);
        assert!(!fm.draft);
        assert!(body.contains("# Body"));
    }

    /// Verify that a file without front matter returns `None` and the original content.
    #[test]
    fn test_front_matter_absent() {
        let src = "# Just a heading\n\nNo front matter here.";
        let (fm_opt, body) = parse_front_matter(src);
        assert!(fm_opt.is_none());
        assert_eq!(body, src);
    }

    /// Verify that `draft: true` is parsed correctly.
    #[test]
    fn test_front_matter_draft_true() {
        let src = "---\ntitle: Draft Post\ndraft: true\n---\n\nContent";
        let (fm_opt, _) = parse_front_matter(src);
        let fm = fm_opt.unwrap();
        assert!(fm.draft);
        assert_eq!(fm.title, "Draft Post");
        // Tags should default to empty when absent
        assert!(fm.tags.is_empty());
    }

    /// Verify that Markdown bold, headings, and links convert to expected HTML.
    #[test]
    fn test_markdown_to_html_basic() {
        let md = "# Hello\n\nThis is **bold** and *italic*.";
        let html = markdown_to_html(md);
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    /// Verify that Markdown tables (an extension) are rendered to `<table>` elements.
    #[test]
    fn test_markdown_tables() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = markdown_to_html(md);
        assert!(html.contains("<table>"), "expected <table> in: {html}");
    }

    /// Verify that `slugify` converts titles to URL-safe slugs.
    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("Rust & WebAssembly"), "rust-webassembly");
        assert_eq!(slugify("  spaces  "), "spaces");
    }

    /// Verify that minijinja renders a post template with a context.
    #[test]
    fn test_template_render() {
        // Build an in-memory environment with the default templates
        let mut env = minijinja::Environment::new();
        env.add_template("base.html", DEFAULT_BASE).expect("base template invalid");
        env.add_template("post.html", DEFAULT_POST).expect("post template invalid");
        let tmpl = env.get_template("post.html").expect("post.html not found");
        let ctx = serde_json::json!({
            "title": "Test Post",
            "content": "<p>Hello</p>",
            "date": "2024-01-01",
            "tags": ["rust"],
        });
        let out = tmpl.render(ctx).expect("render failed");
        assert!(out.contains("Test Post"), "title missing");
        assert!(out.contains("<p>Hello</p>"), "content missing");
        assert!(out.contains("2024-01-01"), "date missing");
    }
}
