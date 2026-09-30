# ssg — Static Site Generator

A command-line static site generator that converts Markdown files and Jinja2 templates into a deployable HTML site.

## Build

```bash
cargo build -p ssg
# or run directly:
cargo run -p ssg -- --help
```

## Quick start

```bash
# 1. Scaffold a new site in the current directory
cargo run -p ssg -- init

# 2. Build the site
cargo run -p ssg -- build

# 3. Serve the output (any HTTP server works)
python -m http.server 8080 --directory output/
```

## Subcommands

### `ssg init`

Creates the following structure in the current directory:

```
content/
  hello-world.md       Sample post with front matter
templates/
  base.html            Full HTML5 skeleton (block content)
  post.html            Extends base — renders an article
  index.html           Extends base — renders a post list
static/
  style.css            Minimal default stylesheet
output/                Generated site (created by build)
```

### `ssg build [options]`

| Flag | Default | Description |
|------|---------|-------------|
| `--content <DIR>` | `content/` | Directory containing Markdown sources |
| `--output <DIR>` | `output/` | Directory to write generated HTML |
| `--templates <DIR>` | `templates/` | Directory containing Jinja2 templates |

What the build does:

1. Walks `content/` recursively, processing every `.md` file.
2. Strips the front matter block and converts the Markdown body to HTML with **pulldown-cmark** (tables, strikethrough, footnotes, and task lists enabled).
3. Renders the HTML through `templates/post.html` (which extends `base.html`).
4. Writes the output to `output/`, preserving subdirectory structure and changing `.md` to `.html`.
5. Generates `output/index.html` listing all non-draft posts, sorted newest-first.
6. Generates `output/tags/<TAG>.html` for every unique tag found in front matter.
7. Copies everything from `static/` to `output/static/` unchanged.

### `ssg new <title>`

Creates a new Markdown file in `content/` with the filename derived from the title:

```bash
cargo run -p ssg -- new "My Rust Journey"
# → content/my-rust-journey.md
```

The file is pre-filled with front matter:

```markdown
---
title: My Rust Journey
date: 2024-06-01
tags:
draft: false
---

# My Rust Journey

Write your post here.
```

## Front matter

Each Markdown file may begin with a `---`-delimited block containing `key: value` pairs:

| Key | Type | Description |
|-----|------|-------------|
| `title` | string | Page title (required for listings) |
| `date` | YYYY-MM-DD | Publication date; used for sorting |
| `tags` | comma-separated | Tag labels |
| `draft` | true/false | When `true`, excluded from all index and tag pages |

Example:

```markdown
---
title: Hello World
date: 2024-06-01
tags: rust, intro
draft: false
---

# Hello World

Welcome to my first post!
```

## Template variables

### `post.html`

| Variable | Type | Description |
|----------|------|-------------|
| `title` | string | Front matter title |
| `content` | HTML string (safe) | Rendered Markdown body |
| `date` | string | Date string or empty |
| `tags` | list of strings | Tag names |

### `index.html` / tag pages

| Variable | Type | Description |
|----------|------|-------------|
| `posts` | list of objects | Each has `title`, `url`, `date`, `tags` |
| `tag` | string or null | Set to the tag name on tag pages |

## Running tests

```bash
cargo test -p ssg
```
