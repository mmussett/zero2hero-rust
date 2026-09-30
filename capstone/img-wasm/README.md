# img-wasm — WASM-Powered Image Editor

A browser-based image editor implemented in Rust, compiled to WebAssembly.
All image processing runs locally in the browser — no server required.

## Prerequisites

```bash
cargo install wasm-pack
```

## Build

```bash
# From the img-wasm/ directory
wasm-pack build --target web
```

This generates a `pkg/` directory containing the JavaScript bindings and the
`.wasm` file.

## Serve

Any HTTP server works.  The page **cannot** be opened as a `file://` URL
because ES modules require HTTP.

```bash
# Python (built-in)
python -m http.server 8080

# Node.js (npx, no install required)
npx serve .

# Rust (cargo install miniserve)
miniserve . --port 8080
```

Then open `http://localhost:8080` in your browser.

## UI features

| Feature | Description |
|---------|-------------|
| **File open** | Click the drop zone or drag-and-drop a PNG or JPEG |
| **Greyscale** | Convert to monochrome |
| **Invert** | Complement all pixel values |
| **Flip H / Flip V** | Mirror horizontally or vertically |
| **Rotate 90°** | Clockwise rotation |
| **Brightness** | Adjust by ±255 |
| **Blur** | Gaussian blur with configurable sigma |
| **Contrast** | Non-linear contrast adjustment |
| **Sharpen** | Unsharp mask with configurable strength |
| **Apply** | Apply all non-zero slider adjustments in sequence |
| **Undo / Redo** | Up to 20 history steps; Ctrl+Z / Ctrl+Y keyboard shortcuts |
| **Download** | Save the current image as a PNG |

## Exported WASM functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `grayscale` | `(Uint8Array) → Uint8Array` | Convert to greyscale |
| `brightness` | `(Uint8Array, i32) → Uint8Array` | Adjust brightness by delta |
| `blur` | `(Uint8Array, f32) → Uint8Array` | Gaussian blur |
| `flip_horizontal` | `(Uint8Array) → Uint8Array` | Mirror left-right |
| `flip_vertical` | `(Uint8Array) → Uint8Array` | Mirror top-bottom |
| `rotate90` | `(Uint8Array) → Uint8Array` | Rotate 90° clockwise |
| `invert` | `(Uint8Array) → Uint8Array` | Invert colours |
| `sharpen` | `(Uint8Array, f32) → Uint8Array` | Unsharp mask |
| `contrast` | `(Uint8Array, f32) → Uint8Array` | Adjust contrast |
| `dimensions` | `(Uint8Array) → Uint32Array` | Returns `[width, height]` |

All functions accept raw PNG or JPEG bytes and return PNG bytes (except
`dimensions`).  On error they throw a JavaScript `Error`.

## Running Rust tests

```bash
# Native tests (no WASM target required)
cargo test -p img-wasm

# WASM tests (requires wasm-pack and a browser/Node runner)
wasm-pack test --headless --firefox
```
