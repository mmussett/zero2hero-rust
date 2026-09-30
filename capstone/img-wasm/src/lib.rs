//! Browser-based image processing library compiled to WebAssembly.
//!
//! Each exported function accepts raw image bytes (PNG or JPEG) and returns
//! the processed image re-encoded as PNG bytes.  Error conditions are surfaced
//! as JavaScript `Error` objects via [`JsError`].
//!
//! # Building
//! ```text
//! wasm-pack build --target web
//! ```
//!
//! # JavaScript usage
//! ```js
//! import init, { grayscale, blur } from './pkg/img_wasm.js';
//! await init();
//! const result = grayscale(inputBytes);   // Uint8Array → Uint8Array
//! ```

use std::io::Cursor;

use image::{DynamicImage, ImageFormat};
use wasm_bindgen::prelude::*;

// ── Private helpers ───────────────────────────────────────────────────────────

/// Decode raw image bytes (auto-detected PNG or JPEG) into a [`DynamicImage`].
fn decode(bytes: &[u8]) -> Result<DynamicImage, JsError> {
    image::load_from_memory(bytes).map_err(|e| JsError::new(&e.to_string()))
}

/// Encode a [`DynamicImage`] to PNG bytes.
///
/// Uses `DynamicImage::write_to` with an in-memory `Cursor` so no filesystem
/// access is needed — safe to call from a WASM context.
fn encode(img: DynamicImage) -> Result<Vec<u8>, JsError> {
    let mut buf: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    img.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(buf)
}

// ── Exported image operations ─────────────────────────────────────────────────

/// Convert the image to greyscale (luma-only, single channel lifted to RGB).
///
/// Returns the greyscale image re-encoded as PNG bytes.
#[wasm_bindgen]
pub fn grayscale(input: &[u8]) -> Result<Vec<u8>, JsError> {
    encode(decode(input)?.grayscale())
}

/// Adjust the brightness of the image by `delta` in the range `[-255, 255]`.
///
/// Positive values lighten; negative values darken.  Pixel values are clamped
/// to `[0, 255]` — no wrapping occurs.
#[wasm_bindgen]
pub fn brightness(input: &[u8], delta: i32) -> Result<Vec<u8>, JsError> {
    encode(decode(input)?.brighten(delta))
}

/// Apply a Gaussian blur with the given `sigma` value (e.g. `1.0`–`5.0`).
///
/// Larger sigma values produce a stronger blur effect.
#[wasm_bindgen]
pub fn blur(input: &[u8], sigma: f32) -> Result<Vec<u8>, JsError> {
    encode(decode(input)?.blur(sigma))
}

/// Flip the image horizontally (mirror left ↔ right).
#[wasm_bindgen]
pub fn flip_horizontal(input: &[u8]) -> Result<Vec<u8>, JsError> {
    encode(decode(input)?.fliph())
}

/// Flip the image vertically (mirror top ↔ bottom).
#[wasm_bindgen]
pub fn flip_vertical(input: &[u8]) -> Result<Vec<u8>, JsError> {
    encode(decode(input)?.flipv())
}

/// Rotate the image 90° clockwise.
#[wasm_bindgen]
pub fn rotate90(input: &[u8]) -> Result<Vec<u8>, JsError> {
    encode(decode(input)?.rotate90())
}

/// Invert all pixel colours (bitwise NOT per channel).
///
/// Black becomes white, white becomes black, and all intermediate colours are
/// complemented.
#[wasm_bindgen]
pub fn invert(input: &[u8]) -> Result<Vec<u8>, JsError> {
    let mut img = decode(input)?;
    img.invert(); // mutates in place
    encode(img)
}

/// Sharpen the image using an unsharp mask.
///
/// `strength` controls the blur radius used to compute the mask (try `1.0`–`3.0`).
/// A threshold of `0` means all edges are sharpened regardless of contrast.
#[wasm_bindgen]
pub fn sharpen(input: &[u8], strength: f32) -> Result<Vec<u8>, JsError> {
    // unsharpen(sigma, threshold): sigma is the blur radius; threshold is the
    // minimum pixel-value difference required before sharpening is applied.
    encode(decode(input)?.unsharpen(strength, 0))
}

/// Adjust the contrast of the image.
///
/// `factor` is passed directly to the image crate's `adjust_contrast` method,
/// which applies a non-linear S-curve: positive values increase contrast and
/// negative values decrease it (0 gives a flat grey image).
#[wasm_bindgen]
pub fn contrast(input: &[u8], factor: f32) -> Result<Vec<u8>, JsError> {
    encode(decode(input)?.adjust_contrast(factor))
}

/// Return the image dimensions as a two-element `Uint32Array` `[width, height]`.
///
/// Does not re-encode the image — suitable as a cheap metadata query.
#[wasm_bindgen]
pub fn dimensions(input: &[u8]) -> Result<Vec<u32>, JsError> {
    let img = decode(input)?;
    Ok(vec![img.width(), img.height()])
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Produce a minimal 1×1 white RGBA PNG in memory for use in tests.
    ///
    /// This avoids any filesystem dependency and keeps tests self-contained.
    fn white_1x1_png() -> Vec<u8> {
        // A 1×1 white pixel RGBA image.
        let img = DynamicImage::new_rgba8(1, 1);
        encode(img).expect("encode test image")
    }

    #[test]
    fn test_grayscale_roundtrip() {
        let png = white_1x1_png();
        let result = grayscale(&png).expect("grayscale");
        // The result must be non-empty valid PNG bytes.
        assert!(!result.is_empty());
        // PNG magic bytes: 0x89 P N G \r \n 0x1a \n
        assert_eq!(&result[..4], &[0x89, 0x50, 0x4e, 0x47]);
    }

    #[test]
    fn test_dimensions() {
        let img = encode(DynamicImage::new_rgb8(64, 32)).expect("encode");
        let dims = dimensions(&img).expect("dimensions");
        assert_eq!(dims, vec![64, 32]);
    }

    #[test]
    fn test_flip_horizontal_dimensions_preserved() {
        let img = encode(DynamicImage::new_rgb8(20, 10)).expect("encode");
        let flipped = flip_horizontal(&img).expect("flip_horizontal");
        let dims = dimensions(&flipped).expect("dimensions after flip");
        assert_eq!(dims, vec![20, 10], "flip_horizontal must preserve dimensions");
    }

    #[test]
    fn test_rotate90_swaps_dimensions() {
        let img = encode(DynamicImage::new_rgb8(30, 10)).expect("encode");
        let rotated = rotate90(&img).expect("rotate90");
        let dims = dimensions(&rotated).expect("dimensions after rotate90");
        assert_eq!(dims, vec![10, 30], "rotate90 must swap width and height");
    }

    #[test]
    fn test_invalid_input_returns_error() {
        let garbage = b"not an image";
        let result = grayscale(garbage);
        assert!(result.is_err(), "garbage input should return Err");
    }
}
