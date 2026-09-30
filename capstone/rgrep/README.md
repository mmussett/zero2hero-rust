# rgrep

A recursive grep replacement with coloured output, context lines, and .gitignore support.

## Build

```bash
cargo build -p rgrep --release
```

## Usage

```bash
rgrep "fn main" src/           # search for pattern in src/
rgrep -i "error" . -C 2       # case-insensitive, 2 context lines
rgrep --count "TODO" .         # count matches only
```
