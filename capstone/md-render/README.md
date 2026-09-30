# md-render

Render Markdown to styled terminal output with ANSI colours and formatting.

## Build

```bash
cargo build -p md-render --release
```

## Usage

```bash
# Render a Markdown file
md-render README.md

# Render from stdin
cat README.md | md-render
echo "# Hello World" | md-render
```

## Output Styles

- **H1**: Bold + yellow underline
- **H2**: Bold + cyan
- **H3**: Bold + green
- **H4**: Bold
- **Bold text**: Bold attribute
- *Italic text*: Italic attribute
- `Inline code`: Bright yellow
- Code blocks: Bright cyan with `━━━━` divider
- > Blockquotes: Dimmed with `│ ` prefix
- Unordered lists: `  • item`
- Ordered lists: `  1. item`
- Horizontal rules: `──────────────────────────────`
- Links: `text [url]` in dim style
