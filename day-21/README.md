# Day 21: WebAssembly with wasm-pack and wasm-bindgen

> **Project:** Image Processing Library — expose grayscale, invert, and brighten filters as `#[wasm_bindgen]` functions and use them from a browser demo that draws on a `<canvas>`.

## Learning Objectives

By the end of today you will be able to:
- Explain what WebAssembly is and why Rust is well-suited to compile to it.
- Annotate Rust functions with `#[wasm_bindgen]` to expose them to JavaScript.
- Understand which types cross the Wasm–JS boundary natively and which require conversion.
- Build a Wasm module with `wasm-pack build --target web` and inspect the generated `pkg/` directory.
- Import and call Wasm functions from a plain HTML page without a bundler.
- Use `console_error_panic_hook` to surface Rust panics in the browser console.

---

## Concepts

### 1. What WebAssembly Is

> **Docs:** [Rust and WebAssembly](https://rustwasm.github.io/docs/book/) · [wasm-pack](https://rustwasm.github.io/docs/wasm-pack/) · [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/)

WebAssembly (Wasm) is a binary instruction format designed as a compilation target. It runs in all modern browsers (and Node.js) inside a sandboxed virtual machine at near-native speed. Unlike JavaScript, Wasm is typed, deterministic, and does not include a garbage collector — which is exactly why Rust, with its ownership-based memory model, compiles to Wasm cleanly: no GC needed, no runtime overhead.

Wasm is not a replacement for JavaScript. It excels at compute-intensive tasks: image and audio processing, cryptography, codecs, physics simulations, game engines. JavaScript handles the DOM, event handling, and orchestration. The two communicate across a well-defined boundary.

Rust's toolchain supports Wasm as a first-class target via `wasm32-unknown-unknown`. The `wasm-bindgen` crate provides the glue layer that makes calling Rust from JS (and vice versa) ergonomic. `wasm-pack` is the build tool that combines compilation, `wasm-bindgen` post-processing, and npm packaging into a single command.

#### Exercise 1.1 — Mental Model

**Goal:** Solidify your mental model before writing code. Answer these in comments:

```rust
// 1. Why does Rust compile to Wasm more cleanly than C++?
// Answer: ___

// 2. What is the job of wasm-bindgen vs wasm-pack?
// Answer: ___

// 3. Name two workloads where Wasm outperforms equivalent JavaScript:
// Answer: ___
```

**Expected output:** (written exercise — no runtime output)
```
No runtime output — answers in comments.
```

> **Hint:** Rust has no GC and its ownership model maps directly to Wasm's linear memory model.

---

### 2. `wasm-bindgen` — The Rust–JavaScript Bridge

> **Docs:** [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/) · [`js-sys`](https://docs.rs/js-sys/latest/js_sys/) · [`web-sys`](https://docs.rs/web-sys/latest/web_sys/)

`wasm-bindgen` is a Rust library and a build-time code generator. Annotating a Rust function with `#[wasm_bindgen]` does two things: it marks the function for export in the Wasm binary, and it instructs the `wasm-bindgen` CLI (invoked automatically by `wasm-pack`) to generate JavaScript and TypeScript wrapper code that calls the Wasm function correctly.

Without `wasm-bindgen`, you can only pass integers and floats across the Wasm boundary — everything else requires manual memory management. `wasm-bindgen` automates the serialization of strings, `Vec<u8>`, and JavaScript objects so you can use idiomatic Rust types.

```rust
use wasm_bindgen::prelude::*;

// This function is callable from JavaScript as: module.add(3, 4)
#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

// This function accepts and returns a String (wasm-bindgen handles conversion)
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

JavaScript side (after `wasm-pack build --target web`):

```javascript
import init, { add, greet } from './pkg/day_21.js';

async function main() {
    await init(); // loads the .wasm binary
    console.log(add(3, 4));      // 7
    console.log(greet("Rust"));  // "Hello, Rust!"
}
main();
```

#### Exercise 2.1 — First Export

**Goal:** Add a `#[wasm_bindgen]` function `fn factorial(n: u32) -> u32` that computes `n!`. Verify it compiles with `wasm-pack build --target web`.

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn factorial(n: u32) -> u32 {
    (1..=n).product()
}
```

**Expected output** (from browser console after calling `factorial(5)`):
```
120
```

> **Hint:** `(1..=0).product::<u32>()` correctly returns `1` (the identity for multiplication), so factorial(0) = 1.

---

### 3. Types Across the Wasm–JS Boundary

> **Docs:** [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/) · [`js-sys`](https://docs.rs/js-sys/latest/js_sys/) · [`web-sys`](https://docs.rs/web-sys/latest/web_sys/)

Not all Rust types map directly to JavaScript. Here is the practical breakdown:

- **Numbers** (`u8`, `u16`, `u32`, `u64`, `i32`, `i64`, `f32`, `f64`): pass directly as JS numbers (note: `u64`/`i64` require `BigInt` in JS).
- **`bool`**: maps to JS `boolean`.
- **`String` and `&str`**: `wasm-bindgen` allocates a copy in Wasm memory and copies to/from JS. This involves a small allocation cost per call.
- **`Vec<u8>` and `&[u8]`**: the primary way to pass binary data (pixel buffers, file bytes). `Vec<u8>` is passed as a `Uint8Array` in JS.
- **`JsValue`**: an escape hatch for arbitrary JS values. Use when the type is dynamic or cannot be expressed in Rust's type system.

For image data, `Vec<u8>` is the natural type: browsers represent pixel data as `Uint8Array` from `ImageData`, which converts automatically.

```rust
use wasm_bindgen::prelude::*;

// Demonstrate type mappings
#[wasm_bindgen]
pub fn sum_bytes(data: Vec<u8>) -> u32 {
    data.iter().map(|&b| b as u32).sum()
}

#[wasm_bindgen]
pub fn repeat_string(s: &str, n: u32) -> String {
    s.repeat(n as usize)
}

#[wasm_bindgen]
pub fn is_even(n: u32) -> bool {
    n % 2 == 0
}
```

#### Exercise 3.1 — Type Tour

**Goal:** Add three exported functions: `fn double_all(data: Vec<u8>) -> Vec<u8>` (doubles each byte, clamping to 255), `fn to_upper(s: &str) -> String`, and `fn count_ones(n: u32) -> u32` (number of set bits). Verify they compile.

```rust
#[wasm_bindgen]
pub fn double_all(data: Vec<u8>) -> Vec<u8> {
    data.iter().map(|&b| b.saturating_mul(2)).collect()
}

#[wasm_bindgen]
pub fn to_upper(s: &str) -> String {
    s.to_uppercase()
}

#[wasm_bindgen]
pub fn count_ones(n: u32) -> u32 {
    n.count_ones()
}
```

**Expected output** (in browser console):
```
double_all([1, 100, 200]): [2, 200, 255]
to_upper("rust"): "RUST"
count_ones(7): 3
```

> **Hint:** `saturating_mul` clamps to `u8::MAX` (255) on overflow — no wrapping or panics.

---

### 4. `wasm-pack build` — Building and the `pkg/` Directory

> **Docs:** [wasm-pack](https://rustwasm.github.io/docs/wasm-pack/) · [Cargo — cdylib](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#library)

`wasm-pack build --target web` performs these steps:

1. Compiles your Rust library crate to `target/wasm32-unknown-unknown/release/*.wasm`.
2. Runs the `wasm-bindgen` CLI to post-process the `.wasm` binary and generate JavaScript glue code.
3. Creates a `pkg/` directory containing:
   - `day_21_bg.wasm` — the compiled Wasm binary.
   - `day_21.js` — JavaScript/ESM glue with type conversions.
   - `day_21.d.ts` — TypeScript type declarations.
   - `package.json` — npm metadata.

The `Cargo.toml` must declare the crate type as `cdylib` (for Wasm) and `rlib` (for Rust unit tests):

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

**Important:** `wasm-pack build` requires the `wasm-pack` tool. Install it once with:

```bash
cargo install wasm-pack
```

Building:

```bash
wasm-pack build --target web
```

After building, your `pkg/` directory will look like:

```
pkg/
  day_21_bg.wasm
  day_21.js
  day_21.d.ts
  package.json
  README.md
```

#### Exercise 4.1 — Build Inspection

**Goal:** After running `wasm-pack build --target web`, open `pkg/day_21.js` in a text editor and find the `greet` function wrapper. Note: (1) how string arguments are passed to Wasm, (2) how the return string is read back from Wasm memory.

**Expected output:** (investigation exercise — no runtime output)
```
Found wasm-bindgen JS wrapper for greet.
String arguments use TextEncoder to write into Wasm memory.
Return strings use TextDecoder to read from Wasm memory.
```

> **Hint:** Look for `wasm.greet(` in the generated `.js` file and trace the memory operations around it.

---

### 5. Using the Wasm Module from JavaScript

> **Docs:** [Rust and WebAssembly](https://rustwasm.github.io/docs/book/) · [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/)

Once you have a `pkg/` directory, you can import it from any HTML file served over HTTP (Wasm cannot be loaded from `file://` URLs due to browser security restrictions — you need a local HTTP server).

The import pattern for `--target web`:

```javascript
import init, { my_function } from './pkg/day_21.js';

async function main() {
    await init(); // fetches and compiles the .wasm file
    // now call exported functions directly
    const result = my_function(arg1, arg2);
    console.log(result);
}

main().catch(console.error);
```

`init()` is the initialiser generated by `wasm-bindgen`. It fetches `day_21_bg.wasm`, compiles it in the browser's Wasm engine, and sets up the memory and function table. After `await init()`, all exported functions are ready.

For pixel data, the browser's `ImageData` API gives you a `Uint8Array` of RGBA bytes (4 bytes per pixel: R, G, B, A). Pass it directly to your Rust function — `wasm-bindgen` converts `Uint8Array` to `Vec<u8>` automatically when your Rust parameter is `Vec<u8>`. The returned `Vec<u8>` becomes a `Uint8Array` you can put back into `ImageData`.

#### Exercise 5.1 — Console Hello

**Goal:** Write a minimal HTML file that imports the Wasm module, calls `greet("World")`, and logs the result to the console. No canvas needed.

```html
<!DOCTYPE html>
<html>
<head><title>Wasm Hello</title></head>
<body>
<script type="module">
    import init, { greet } from './pkg/day_21.js';
    async function main() {
        await init();
        console.log(greet("World"));
    }
    main();
</script>
</body>
</html>
```

**Expected output** (in browser console):
```
Hello, World!
```

> **Hint:** Serve from a local HTTP server — `python3 -m http.server 8000` or `npx serve .` both work.

---

### 6. `console_error_panic_hook`

> **Docs:** [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/) · [`wasm-bindgen-futures`](https://docs.rs/wasm-bindgen-futures/latest/wasm_bindgen_futures/)

When Rust panics inside Wasm, the error is opaque by default — you see something like `RuntimeError: unreachable executed` in the browser console with no Rust stack trace. `console_error_panic_hook` replaces the panic handler with one that formats the panic message and passes it to `console.error`, giving you readable Rust backtraces in the browser devtools.

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
```

`#[wasm_bindgen(start)]` marks a function to be called automatically when the Wasm module initialises — before any other code runs. Always add this in your Wasm projects.

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        panic!("division by zero"); // Now appears clearly in browser console
    }
    a / b
}
```

#### Exercise 6.1 — Panic Visibility

**Goal:** Add `console_error_panic_hook::set_once()` in a `#[wasm_bindgen(start)]` function. Export a function `fn assert_positive(n: i32)` that panics with a helpful message if `n <= 0`. Call it from JavaScript with `assert_positive(-1)` and observe the panic message in the browser console.

**Expected output** (browser console, after calling `assert_positive(-1)`):
```
panicked at 'Expected positive number, got -1', src/lib.rs:15:9
```

> **Hint:** `panic!("Expected positive number, got {n}")` — after setting the hook, this message appears in the browser console via `console.error`.

---

## Day Project: Image Processing Library

### What You're Building

A WebAssembly library that exposes three image filter functions callable from JavaScript: `grayscale`, `invert`, and `brighten`. The library operates on raw RGBA pixel buffers — the same format used by the browser's `ImageData` API, so filters can be applied directly to canvas pixels. A minimal `demo.html` file draws an image on a `<canvas>`, applies each filter, and displays the result.

### Requirements

**`src/lib.rs`:**

1. `#[wasm_bindgen(start)]` function that calls `console_error_panic_hook::set_once()`.
2. `pub fn grayscale(pixels: Vec<u8>) -> Vec<u8>` — converts RGBA pixels to grayscale. For each 4-byte pixel, compute luminance as `0.299 * R + 0.587 * G + 0.114 * B`, set R=G=B=luminance, keep A unchanged.
3. `pub fn invert(pixels: Vec<u8>) -> Vec<u8>` — inverts each RGB channel (`255 - value`), keeps A unchanged.
4. `pub fn brighten(pixels: Vec<u8>, factor: f32) -> Vec<u8>` — multiplies each RGB channel by `factor`, clamps to 0–255 with `min(val * factor, 255.0) as u8`, keeps A unchanged.
5. All functions are annotated with `#[wasm_bindgen]`.

**`demo.html`:**
1. A `<canvas>` element.
2. Four buttons: `Original`, `Grayscale`, `Invert`, `Brighten (1.5x)`.
3. JavaScript that draws a procedurally generated test image (coloured rectangles, or a gradient), then on each button click: reads canvas `ImageData`, passes the pixel array to the corresponding Rust/Wasm function, puts the result back into the canvas.

### Getting Started

```bash
# Install wasm-pack (once)
cargo install wasm-pack

# Create the crate
cargo new day-21 --lib
cd day-21
```

`Cargo.toml`:

```toml
[package]
name = "day-21"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
```

Starter `src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn grayscale(pixels: Vec<u8>) -> Vec<u8> {
    let mut result = pixels.clone();
    for chunk in result.chunks_mut(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        let lum = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
        chunk[0] = lum;
        chunk[1] = lum;
        chunk[2] = lum;
        // chunk[3] is alpha — leave unchanged
    }
    result
}

#[wasm_bindgen]
pub fn invert(pixels: Vec<u8>) -> Vec<u8> {
    let mut result = pixels.clone();
    for chunk in result.chunks_mut(4) {
        chunk[0] = 255 - chunk[0];
        chunk[1] = 255 - chunk[1];
        chunk[2] = 255 - chunk[2];
        // alpha unchanged
    }
    result
}

#[wasm_bindgen]
pub fn brighten(pixels: Vec<u8>, factor: f32) -> Vec<u8> {
    let mut result = pixels.clone();
    for chunk in result.chunks_mut(4) {
        chunk[0] = ((chunk[0] as f32 * factor).min(255.0)) as u8;
        chunk[1] = ((chunk[1] as f32 * factor).min(255.0)) as u8;
        chunk[2] = ((chunk[2] as f32 * factor).min(255.0)) as u8;
    }
    result
}
```

### Building

```bash
wasm-pack build --target web
```

This creates `pkg/` in the `day-21` directory. Both `demo.html` and `pkg/` must be served from the same origin.

### Running Your Solution

Serve the `day-21` directory with a local HTTP server:

```bash
# Option A — Python
python3 -m http.server 8000

# Option B — Node.js
npx serve .

# Option C — Rust (if you have miniserve installed)
miniserve . --port 8000
```

Open `http://localhost:8000/demo.html` in a browser. You should see:

```
[Canvas with a colourful test image]
[Original] [Grayscale] [Invert] [Brighten 1.5x]
```

Clicking each button transforms the image in-place on the canvas using the Rust/Wasm function. Open browser devtools (F12) and switch to the Console tab — if anything panics, the `console_error_panic_hook` will print a readable Rust panic message.

**Successful console output on load:**
```
Wasm module loaded. Functions: grayscale, invert, brighten
```

### Testing with Rust Unit Tests

Because the crate type includes `rlib`, you can unit-test the filter logic with `cargo test` (without Wasm):

```bash
cargo test
```

Add these tests to the bottom of `src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grayscale_pure_red() {
        // Pure red RGBA pixel
        let input = vec![255u8, 0, 0, 255];
        let output = grayscale(input);
        // 0.299 * 255 = 76.245 -> 76
        assert_eq!(output[0], 76);
        assert_eq!(output[1], 76);
        assert_eq!(output[2], 76);
        assert_eq!(output[3], 255); // alpha preserved
    }

    #[test]
    fn invert_white() {
        let input = vec![255u8, 255, 255, 128];
        let output = invert(input);
        assert_eq!(output[0], 0);
        assert_eq!(output[1], 0);
        assert_eq!(output[2], 0);
        assert_eq!(output[3], 128); // alpha preserved
    }

    #[test]
    fn brighten_clamps_to_255() {
        let input = vec![200u8, 100, 50, 255];
        let output = brighten(input, 2.0);
        assert_eq!(output[0], 255); // 200 * 2.0 = 400 -> clamped to 255
        assert_eq!(output[1], 200); // 100 * 2.0 = 200
        assert_eq!(output[2], 100); // 50 * 2.0 = 100
        assert_eq!(output[3], 255); // alpha preserved
    }
}
```

**Expected test output:**
```
running 3 tests
test tests::grayscale_pure_red ... ok
test tests::invert_white ... ok
test tests::brighten_clamps_to_255 ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

### Extension Challenges

- **Easy:** Add a `pub fn pixelate(pixels: Vec<u8>, width: u32, block_size: u32) -> Vec<u8>` function that averages each `block_size x block_size` block of pixels. Expose it via `#[wasm_bindgen]` and add a button in `demo.html`.
- **Medium:** Add a `pub fn blur(pixels: Vec<u8>, width: u32, height: u32) -> Vec<u8>` box blur function. For each pixel, average it with its 8 neighbors (handle edges by clamping). Add it to the demo.
- **Hard:** Use `wasm_bindgen_futures` to make the Wasm functions async-friendly, and implement a `process_pipeline(pixels: Vec<u8>, steps: JsValue) -> Vec<u8>` function that accepts a JavaScript array of step names (`["grayscale", "brighten_1.5", "invert"]`) and applies them in sequence, returning the final pixel buffer.
