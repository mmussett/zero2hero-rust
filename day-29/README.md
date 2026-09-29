# Day 29: Publishing and Packaging

> **Project:** Polished Shape Library — take the Day 10 geometry crate from working code to publish-ready library with full documentation, doc tests, and correct metadata

## Learning Objectives

By the end of today you will be able to:
- Fill in all required `Cargo.toml` metadata fields for publishing to crates.io
- Write `///` doc comments and `//!` crate-level docs using Markdown
- Write doc tests in `# Examples` sections that run with `cargo test --doc`
- Use `cargo publish --dry-run` to verify a crate is ready without uploading
- Define feature flags with `[features]` and `#[cfg(feature = "...")]`

---

## Concepts

### 1. `Cargo.toml` Metadata for Publishing

> **Docs:** [Publishing — Cargo Book](https://doc.rust-lang.org/cargo/reference/publishing.html) · [Cargo.toml manifest](https://doc.rust-lang.org/cargo/reference/manifest.html) · [crates.io](https://crates.io/)

When you run `cargo publish`, Cargo uploads your crate to crates.io. Before it does, it validates that certain metadata fields are present. These fields also appear on the crates.io listing page, in search results, and in documentation links, so they directly affect discoverability and first impressions.

The essential fields beyond `[package]` defaults:

```toml
[package]
name = "my-shapes"
version = "0.1.0"
edition = "2021"
description = "2D geometric shapes with area and perimeter calculations"
license = "MIT OR Apache-2.0"   # SPDX identifier; dual-license is conventional in Rust
repository = "https://github.com/yourname/my-shapes"
homepage = "https://github.com/yourname/my-shapes"
documentation = "https://docs.rs/my-shapes"
readme = "README.md"
keywords = ["geometry", "shapes", "math", "2d"]   # max 5, single words
categories = ["science", "mathematics"]            # from the crates.io category list
```

The `license` field uses SPDX identifiers. The Rust ecosystem convention is `"MIT OR Apache-2.0"` (dual-license) so users can choose the more permissive option for their context. A `LICENSE-MIT` and `LICENSE-APACHE` file should exist in the crate root.

`keywords` are used for crates.io search. Choose words your users would actually search for. Maximum five, all lowercase, no punctuation. `categories` come from a fixed list at `crates.io/categories` — browse it and pick the closest match.

`readme = "README.md"` tells Cargo to display the README on the crates.io page. The README lives in the crate root (next to `Cargo.toml`).

#### Exercise 1.1 — Validate Metadata

**Goal:** Catch missing metadata before a real publish.

Add all required fields to your `Cargo.toml`. Then run `cargo publish --dry-run -p day-29`. Read the output carefully — Cargo will report any missing required fields or other issues without uploading anything.

**Expected output:**
```
    Packaging day-29 v0.1.0
    Verifying day-29 v0.1.0
    Compiling day-29 v0.1.0
    Finished `release` profile [optimized] target(s) in X.XXs
```

> **Hint:** If you see "warning: manifest has no description" or similar, the field is missing. `cargo publish --dry-run` exits with code 0 even with warnings, so read the output carefully.

---

### 2. Documentation with `cargo doc`

> **Docs:** [cargo doc](https://doc.rust-lang.org/cargo/commands/cargo-doc.html) · [rustdoc](https://doc.rust-lang.org/rustdoc/) · [rustdoc — doc comments](https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html)

Rust's built-in documentation system generates HTML from doc comments directly in your source code. This means documentation is always co-located with the code it describes and can never become stale in a way that doesn't compile.

Two comment styles:

- `///` — documents the item immediately following it (function, struct, trait, field)
- `//!` — documents the enclosing item (the file or module it appears in)

Use `//!` at the top of `src/lib.rs` to write the crate's top-level documentation page.

Doc comments fully support Markdown: headings, bold, code blocks, bullet lists, and links. The `# Panics`, `# Errors`, `# Safety`, and `# Examples` subsections are conventional and rendered with special formatting on docs.rs:

```rust
//! # my-shapes
//!
//! A library for working with 2D geometric shapes.
//!
//! ## Quick Start
//!
//! ```rust
//! use my_shapes::Circle;
//!
//! let c = Circle::new(5.0);
//! println!("Area: {:.2}", c.area());
//! ```

/// A circle defined by its radius.
///
/// # Examples
///
/// ```
/// use day_29::Circle;
///
/// let c = Circle::new(3.0);
/// assert!((c.area() - 28.274).abs() < 0.001);
/// ```
///
/// # Panics
///
/// Panics if `radius` is negative.
pub struct Circle {
    radius: f64,
}
```

`cargo doc --open -p day-29` builds the documentation and opens it in your browser. `cargo doc --no-deps` skips building docs for dependencies (much faster).

`#[doc(hidden)]` hides an item from the generated documentation. Use it for internal helper types or re-exports you don't want to surface:

```rust
#[doc(hidden)]
pub fn internal_helper() {}  // not shown in cargo doc output
```

#### Exercise 2.1 — Write a Doc Comment

**Goal:** Document a function with all conventional sections.

Write a public function `fn rectangle_area(width: f64, height: f64) -> f64`. Add a `///` doc comment with: a one-line summary, a blank line, a paragraph of explanation, an `# Examples` section with a code block, and a `# Panics` section noting it panics if either dimension is negative. Run `cargo doc --open -p day-29` and verify the output looks correct.

**Expected output (in browser):** A documentation page showing the function with formatted sections.

> **Hint:** `cargo doc` output for your crate appears at `target/doc/day_29/fn.rectangle_area.html`. Note underscores in the path — crate names use underscores internally even if the package name uses hyphens.

---

### 3. Doc Tests

> **Docs:** [rustdoc — doc tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)

Code blocks inside `///` and `//!` comments are executed as tests by `cargo test --doc`. This is one of Rust's most distinctive features: your documentation examples are also your tests. They prove examples stay correct as the code evolves.

A doc test is just a code block with no annotation (or ```` ```rust ````):

```rust
/// Computes the area of a circle with the given radius.
///
/// # Examples
///
/// ```
/// use day_29::circle_area;
///
/// let area = circle_area(1.0);
/// assert!((area - std::f64::consts::PI).abs() < 1e-10);
/// ```
pub fn circle_area(radius: f64) -> f64 {
    std::f64::consts::PI * radius * radius
}
```

Running `cargo test --doc -p day-29` executes every code block in every doc comment.

Special annotations control doc test behaviour:

- ```` ```rust,ignore ```` — the block is shown in docs but not compiled or run (for illustrative snippets)
- ```` ```rust,should_panic ```` — the block must panic; used to demonstrate panic conditions
- ```` ```rust,no_run ```` — the block compiles but is not executed (for examples that need external resources)

Lines prefixed with `# ` in a doc test are compiled but hidden in the rendered output. This lets you write setup code that would distract from the point of the example:

```rust
/// ```
/// # use day_29::Circle;  // hidden line — compiles but not shown
/// let c = Circle::new(2.0);
/// assert_eq!(c.radius(), 2.0);
/// ```
```

#### Exercise 3.1 — Write a Doc Test That Fails

**Goal:** Understand how doc test failures are reported.

Write a doc test in a `///` comment that has an incorrect assertion (e.g., `assert_eq!(2 + 2, 5)`). Run `cargo test --doc`. Observe the failure output including the file, line number, and assertion error. Then fix the assertion and verify the test passes.

**Expected output (failing):**
```
---- src/lib.rs - circle_area (line 8) FAILED ----
thread 'main' panicked at 'assertion failed: `(left == right)`
  left: `4`,
 right: `5`'
```

> **Hint:** The failure output shows the crate path and line number of the doc comment, making it easy to find the broken example.

---

### 4. `cargo publish` and Semver

> **Docs:** [cargo publish](https://doc.rust-lang.org/cargo/commands/cargo-publish.html) · [Cargo — Semantic Versioning](https://doc.rust-lang.org/cargo/reference/semver.html) · [semver.org](https://semver.org/)

Publishing a crate is permanent. Crates.io does not allow deleting versions (only yanking, which prevents new projects from selecting a yanked version but doesn't break existing users who already depend on it). This makes following semver essential.

The semver conventions for Rust crates:
- `0.x.y` — pre-stability. Minor version bumps (`0.2` → `0.3`) are allowed to break APIs.
- `1.0.0` — stable public API. From here, the contract is firm.
- Patch `1.0.x` — bug fixes only, no API changes
- Minor `1.x.0` — backward-compatible additions
- Major `x.0.0` — breaking changes

The publish workflow:

```bash
# 1. Log in (one-time setup with your crates.io API token)
cargo login <your-token>

# 2. Verify without uploading
cargo publish --dry-run -p day-29

# 3. Publish for real
cargo publish -p day-29
```

After publishing, your crate is immediately available at `crates.io/crates/your-crate` and `docs.rs/your-crate` (docs.rs automatically builds and hosts documentation for every published crate).

To prevent a broken version from being selected by new projects without removing it:
```bash
cargo yank --vers 0.1.0 -p day-29
```

#### Exercise 4.1 — Dry Run

**Goal:** Run a full pre-publish check.

With all metadata in place, run `cargo publish --dry-run -p day-29`. If it succeeds, you're ready to publish (though you don't have to for this exercise). Note what files are included in the package — Cargo prints the list. Verify your `README.md` and `LICENSE` files appear.

**Expected output:**
```
    Packaging day-29 v0.1.0
    Packaged X files, Y.Z KiB
```

> **Hint:** Run `cargo package --list -p day-29` to see exactly which files would be included in the upload, without running the full verification build.

---

### 5. Feature Flags

> **Docs:** [Cargo — Features](https://doc.rust-lang.org/cargo/reference/features.html)

Feature flags let you make parts of your crate optional. Users opt in to features they need; others don't pay the compile-time or dependency cost.

```toml
[features]
default = []                    # no features enabled by default
serde = ["dep:serde"]          # "serde" feature enables the serde dependency
svg = ["dep:svg-builder"]

[dependencies]
serde = { version = "1", features = ["derive"], optional = true }
svg-builder = { version = "0.2", optional = true }
```

In source code, gate optional functionality with `#[cfg(feature = "...")]`:

```rust
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Circle {
    radius: f64,
}

#[cfg(feature = "svg")]
pub fn to_svg(&self) -> String {
    // SVG generation, only available with the svg feature
    todo!()
}
```

Users enable features in their `Cargo.toml`:
```toml
[dependencies]
day-29 = { version = "0.1", features = ["serde"] }
```

Feature flags are additive — you can enable multiple features. They are never subtractive. The `default` feature list controls what's enabled when the user doesn't specify features explicitly.

#### Exercise 5.1 — Add a Feature Gate

**Goal:** Make serialization an optional feature.

Add a `serde` feature to `Cargo.toml`. Gate `#[derive(Serialize, Deserialize)]` on your `Circle` struct behind `#[cfg_attr(feature = "serde", derive(...))]`. Build without the feature (`cargo build -p day-29`) and with it (`cargo build -p day-29 --features serde`). Verify both compile.

**Expected output:**
```
# Without feature:
cargo build -p day-29
   Compiling day-29 v0.1.0 -- ok

# With feature:
cargo build -p day-29 --features serde
   Compiling serde v1.x.x
   Compiling day-29 v0.1.0 -- ok
```

> **Hint:** When `serde` is an optional dependency, refer to it in `[features]` as `"dep:serde"` (with the `dep:` prefix) to avoid ambiguity with a feature of the same name.

---

## Day Project: Polished Shape Library

### What You're Building

The Day 10 shape library gets the treatment it deserves: complete documentation, doc tests that double as examples, correct Cargo.toml metadata, feature-gated serde support, and a runnable `examples/basic.rs`. By the end, running `cargo publish --dry-run` will report a clean bill of health.

This is a **library crate** (`src/lib.rs`), not a binary.

### Requirements

1. Implement (or re-implement) a `Circle`, `Rectangle`, and `Triangle` struct, each with:
   - `fn new(...)` constructor (panic if dimensions are negative)
   - `fn area(&self) -> f64`
   - `fn perimeter(&self) -> f64`
   - `fn name(&self) -> &'static str`
2. A `pub trait Shape { fn area(&self) -> f64; fn perimeter(&self) -> f64; fn name(&self) -> &'static str; }` implemented for all three
3. `//!` crate-level documentation at the top of `src/lib.rs` with a usage example
4. `///` doc comments on every public struct, field, and method with `# Examples` sections
5. At least one `# Panics` doc section (on the constructors that panic for negative input)
6. Doc tests that actually compile and pass (`cargo test --doc -p day-29`)
7. `Cargo.toml` with all publish metadata: description, license, keywords, categories, repository
8. A `serde` feature flag that gates `#[derive(Serialize, Deserialize)]` on all three structs
9. `examples/basic.rs` — a runnable demo using all three shapes
10. `cargo publish --dry-run -p day-29` exits cleanly

### Getting Started

```toml
[package]
name = "day-29"
version = "0.1.0"
edition = "2021"
description = "2D geometric shapes with area and perimeter calculations"
license = "MIT"
repository = "https://github.com/yourname/zero2hero-rust"
documentation = "https://docs.rs/day-29"
readme = "README.md"
keywords = ["geometry", "shapes", "math", "2d", "area"]
categories = ["science"]

[features]
default = []
serde = ["dep:serde"]

[dependencies]
serde = { version = "1", features = ["derive"], optional = true }
```

Full `src/lib.rs` scaffold:

```rust
//! # day-29: Shape Library
//!
//! A library providing 2D geometric shapes with area and perimeter calculations.
//!
//! ## Supported Shapes
//!
//! - [`Circle`] — defined by radius
//! - [`Rectangle`] — defined by width and height
//! - [`Triangle`] — defined by three sides
//!
//! ## Quick Start
//!
//! ```rust
//! use day_29::{Circle, Rectangle, Shape};
//!
//! let c = Circle::new(5.0);
//! let r = Rectangle::new(4.0, 6.0);
//!
//! println!("{} area: {:.2}", c.name(), c.area());
//! println!("{} area: {:.2}", r.name(), r.area());
//! ```

/// A trait for 2D shapes that can report their area and perimeter.
pub trait Shape {
    /// The area enclosed by the shape.
    fn area(&self) -> f64;

    /// The total length of the shape's boundary.
    fn perimeter(&self) -> f64;

    /// The human-readable name of the shape.
    fn name(&self) -> &'static str;
}

/// A circle defined by its radius.
///
/// # Examples
///
/// ```
/// use day_29::{Circle, Shape};
///
/// let c = Circle::new(1.0);
/// assert!((c.area() - std::f64::consts::PI).abs() < 1e-10);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    /// The radius of the circle. Must be non-negative.
    pub radius: f64,
}

impl Circle {
    /// Creates a new circle with the given radius.
    ///
    /// # Examples
    ///
    /// ```
    /// use day_29::Circle;
    ///
    /// let c = Circle::new(3.0);
    /// assert_eq!(c.radius, 3.0);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `radius` is negative.
    ///
    /// ```should_panic
    /// use day_29::Circle;
    ///
    /// let _ = Circle::new(-1.0); // panics!
    /// ```
    pub fn new(radius: f64) -> Self {
        assert!(radius >= 0.0, "radius must be non-negative, got {}", radius);
        Circle { radius }
    }
}

impl Shape for Circle {
    /// ```
    /// use day_29::{Circle, Shape};
    ///
    /// let c = Circle::new(2.0);
    /// assert!((c.area() - 12.566).abs() < 0.001);
    /// ```
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    /// ```
    /// use day_29::{Circle, Shape};
    ///
    /// let c = Circle::new(1.0);
    /// assert!((c.perimeter() - 6.283).abs() < 0.001);
    /// ```
    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    fn name(&self) -> &'static str {
        "Circle"
    }
}

/// A rectangle defined by its width and height.
///
/// # Examples
///
/// ```
/// use day_29::{Rectangle, Shape};
///
/// let r = Rectangle::new(4.0, 5.0);
/// assert_eq!(r.area(), 20.0);
/// assert_eq!(r.perimeter(), 18.0);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Rectangle {
    /// The width of the rectangle. Must be non-negative.
    pub width: f64,
    /// The height of the rectangle. Must be non-negative.
    pub height: f64,
}

impl Rectangle {
    /// Creates a new rectangle with the given width and height.
    ///
    /// # Panics
    ///
    /// Panics if either `width` or `height` is negative.
    pub fn new(width: f64, height: f64) -> Self {
        assert!(width >= 0.0, "width must be non-negative");
        assert!(height >= 0.0, "height must be non-negative");
        Rectangle { width, height }
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }

    fn name(&self) -> &'static str {
        "Rectangle"
    }
}

/// A triangle defined by its three side lengths.
///
/// The triangle inequality is enforced: the sum of any two sides
/// must be greater than the third side.
///
/// Area is computed using Heron's formula.
///
/// # Examples
///
/// ```
/// use day_29::{Triangle, Shape};
///
/// // Right triangle with legs 3 and 4
/// let t = Triangle::new(3.0, 4.0, 5.0);
/// assert_eq!(t.area(), 6.0);
/// assert_eq!(t.perimeter(), 12.0);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    /// First side length.
    pub a: f64,
    /// Second side length.
    pub b: f64,
    /// Third side length.
    pub c: f64,
}

impl Triangle {
    /// Creates a new triangle with the given side lengths.
    ///
    /// # Panics
    ///
    /// Panics if any side is non-positive or if the triangle inequality is violated.
    pub fn new(a: f64, b: f64, c: f64) -> Self {
        assert!(a > 0.0 && b > 0.0 && c > 0.0, "all sides must be positive");
        assert!(
            a + b > c && a + c > b && b + c > a,
            "triangle inequality violated: sides {}, {}, {}",
            a, b, c
        );
        Triangle { a, b, c }
    }
}

impl Shape for Triangle {
    fn area(&self) -> f64 {
        let s = self.perimeter() / 2.0;
        (s * (s - self.a) * (s - self.b) * (s - self.c)).sqrt()
    }

    fn perimeter(&self) -> f64 {
        self.a + self.b + self.c
    }

    fn name(&self) -> &'static str {
        "Triangle"
    }
}

/// Returns the shape with the largest area from a slice.
///
/// Returns `None` if the slice is empty.
///
/// # Examples
///
/// ```
/// use day_29::{Circle, Rectangle, largest_shape, Shape};
///
/// let c = Circle::new(1.0);
/// let r = Rectangle::new(10.0, 10.0);
/// let shapes: Vec<&dyn Shape> = vec![&c, &r];
/// let biggest = largest_shape(&shapes).unwrap();
/// assert_eq!(biggest.name(), "Rectangle");
/// ```
pub fn largest_shape<'a>(shapes: &[&'a dyn Shape]) -> Option<&'a dyn Shape> {
    shapes.iter().copied().max_by(|a, b| {
        a.area().partial_cmp(&b.area()).unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_area() {
        let c = Circle::new(1.0);
        assert!((c.area() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn rectangle_area() {
        let r = Rectangle::new(3.0, 4.0);
        assert_eq!(r.area(), 12.0);
    }

    #[test]
    fn triangle_heron() {
        let t = Triangle::new(3.0, 4.0, 5.0);
        assert!((t.area() - 6.0).abs() < 1e-10);
    }

    #[test]
    #[should_panic]
    fn circle_negative_radius() {
        let _ = Circle::new(-1.0);
    }

    #[test]
    fn largest() {
        let c = Circle::new(1.0);
        let r = Rectangle::new(5.0, 5.0);
        let shapes: Vec<&dyn Shape> = vec![&c, &r];
        assert_eq!(largest_shape(&shapes).unwrap().name(), "Rectangle");
    }
}
```

Create `examples/basic.rs`:

```rust
use day_29::{Circle, Rectangle, Shape, Triangle, largest_shape};

fn print_shape(s: &dyn Shape) {
    println!(
        "{}: area = {:.4}, perimeter = {:.4}",
        s.name(),
        s.area(),
        s.perimeter()
    );
}

fn main() {
    let c = Circle::new(5.0);
    let r = Rectangle::new(4.0, 6.0);
    let t = Triangle::new(3.0, 4.0, 5.0);

    let shapes: Vec<&dyn Shape> = vec![&c, &r, &t];

    println!("All shapes:");
    for s in &shapes {
        print_shape(*s);
    }

    if let Some(biggest) = largest_shape(&shapes) {
        println!("\nLargest shape: {} (area = {:.4})", biggest.name(), biggest.area());
    }
}
```

### Running Your Solution

Run the library tests:
```bash
cargo test -p day-29
```

Run the doc tests:
```bash
cargo test --doc -p day-29
```

Run the example:
```bash
cargo run --example basic -p day-29
```

Expected output:
```
All shapes:
Circle: area = 78.5398, perimeter = 31.4159
Rectangle: area = 24.0000, perimeter = 20.0000
Triangle: area = 6.0000, perimeter = 12.0000

Largest shape: Circle (area = 78.5398)
```

Build and open documentation:
```bash
cargo doc --open -p day-29
```

Verify publish readiness:
```bash
cargo publish --dry-run -p day-29
```

Test with serde feature enabled:
```bash
cargo test -p day-29 --features serde
```

### Extension Challenges

- **Easy:** Add a `Square` struct that is a thin wrapper around `Rectangle::new(side, side)`. Document it fully and write doc tests. Re-export it alongside the other shapes.
- **Medium:** Add a `svg` feature flag that adds a `fn to_svg(&self) -> String` method to the `Shape` trait (with a default implementation that returns a generic SVG comment). Implement shape-specific SVG generation for Circle (using `<circle>`) and Rectangle (using `<rect>`).
- **Hard:** Publish a real 0.1.0 version to crates.io (you'll need a free account and API token). Then make a breaking change (rename a method), bump to 0.2.0, and publish again. Observe how the two versions coexist on crates.io and how users can pin to either.
