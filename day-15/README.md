# Day 15: Trait Objects and Dynamic Dispatch

> **Project:** Plugin Renderer — a factory that returns different renderers as `Box<dyn Renderer>` to transform the same content into plain, uppercase, Markdown, and JSON formats.

## Learning Objectives

By the end of today you will be able to:
- Explain the difference between static dispatch (`impl Trait`) and dynamic dispatch (`dyn Trait`) and choose between them deliberately.
- Allocate a trait object on the heap with `Box<dyn Trait>` and store mixed types in a `Vec<Box<dyn Trait>>`.
- Identify whether a trait is object-safe and fix traits that are not.
- Write factory functions that return `Box<dyn Trait>` based on runtime information.
- Articulate the performance trade-offs between monomorphization and vtable dispatch.

---

## Concepts

### 1. `impl Trait` vs `dyn Trait`

> **Docs:** [Trait objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html) · [dyn keyword](https://doc.rust-lang.org/std/keyword.dyn.html) · [Rust by Example — Trait objects](https://doc.rust-lang.org/rust-by-example/trait/dyn.html)

When you write a function that accepts or returns a trait, Rust gives you two mechanisms. With `impl Trait` the compiler knows the concrete type at compile time and generates a dedicated copy of the function for each type — this is called *monomorphization* and produces zero-overhead dispatch. With `dyn Trait` the concrete type is erased at compile time; the program discovers which function to call at runtime through a *vtable* — a table of function pointers stored alongside the data.

Think of `impl Trait` as printing a separate recipe card for each dish — the instructions are perfectly specific. `dyn Trait` is more like a laminated card that says "call the chef and ask them what to do" — flexible, but requires an extra lookup.

A `dyn Trait` value is *unsized*: the compiler does not know how many bytes it occupies because it could be any type implementing the trait. This means you can never store a bare `dyn Trait` on the stack — you must put it behind a pointer. The most common choice is `Box<dyn Trait>`.

```rust
trait Greet {
    fn hello(&self) -> String;
}

struct English;
struct Spanish;

impl Greet for English {
    fn hello(&self) -> String { "Hello!".to_string() }
}

impl Greet for Spanish {
    fn hello(&self) -> String { "¡Hola!".to_string() }
}

// Static dispatch — the compiler generates two copies of this function.
fn greet_static(greeter: &impl Greet) {
    println!("{}", greeter.hello());
}

// Dynamic dispatch — one copy of this function, vtable decides at runtime.
fn greet_dynamic(greeter: &dyn Greet) {
    println!("{}", greeter.hello());
}

fn main() {
    let en = English;
    let es = Spanish;

    greet_static(&en);   // compiler-specialised to English
    greet_static(&es);   // compiler-specialised to Spanish

    greet_dynamic(&en);  // single function, vtable call at runtime
    greet_dynamic(&es);  // same function body, different vtable entry
}
```

#### Exercise 1.1 — Dispatch Detector

**Goal:** Confirm that static and dynamic dispatch produce identical results while building intuition about the syntax.

Create a trait `Shout` with a method `fn shout(&self) -> String`. Implement it for `Cat` (returns `"MEOW!"`) and `Dog` (returns `"WOOF!"`). Write two functions: `shout_static(animal: &impl Shout)` and `shout_dynamic(animal: &dyn Shout)`. Call both with each animal in `main`.

```rust
trait Shout {
    fn shout(&self) -> String;
}

struct Cat;
struct Dog;

// Your implementations here
```

**Expected output:**
```
MEOW!
MEOW!
WOOF!
WOOF!
```

> **Hint:** The output is the same — that is the point. The difference is entirely inside the compiler.

---

### 2. `Box<dyn Trait>` and Heterogeneous Collections

> **Docs:** [Trait objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html) · [Box&lt;T&gt;](https://doc.rust-lang.org/std/boxed/struct.Box.html) · [Reference — Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html)

The killer use-case for `dyn Trait` is a collection of values that share a trait but have different concrete types. A `Vec<Box<dyn Trait>>` can hold a `Cat`, a `Dog`, and a `Penguin` simultaneously — something `Vec<T>` cannot express with a single `T`.

`Box<dyn Trait>` is a fat pointer: two words on the stack. The first word points to the heap-allocated data; the second points to the vtable for that concrete type. Every method call on the trait object follows the vtable pointer, finds the function pointer for the method being called, and jumps to it. This indirection costs a few nanoseconds — negligible unless you are calling millions of methods per second.

Cloning a `Box<dyn Trait>` requires the trait to include a `clone`-equivalent method, because `Clone` itself is not object-safe (more on that in the next section). For most application code — plugin systems, UI widget trees, event handlers — the vtable overhead is invisible.

```rust
trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> &str;
}

struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }
struct Triangle { base: f64, height: f64 }

impl Shape for Circle {
    fn area(&self) -> f64 { std::f64::consts::PI * self.radius * self.radius }
    fn name(&self) -> &str { "circle" }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 { self.width * self.height }
    fn name(&self) -> &str { "rectangle" }
}

impl Shape for Triangle {
    fn area(&self) -> f64 { 0.5 * self.base * self.height }
    fn name(&self) -> &str { "triangle" }
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 3.0 }),
        Box::new(Rectangle { width: 4.0, height: 5.0 }),
        Box::new(Triangle { base: 6.0, height: 7.0 }),
    ];

    let total_area: f64 = shapes.iter().map(|s| s.area()).sum();

    for shape in &shapes {
        println!("{}: area = {:.2}", shape.name(), shape.area());
    }
    println!("Total area: {:.2}", total_area);
}
```

**Expected output:**
```
circle: area = 28.27
rectangle: area = 20.00
triangle: area = 21.00
Total area: 69.27
```

#### Exercise 2.1 — Shape Sorter

**Goal:** Practice building and iterating over a heterogeneous `Vec<Box<dyn Trait>>`.

Add a `perimeter(&self) -> f64` method to the `Shape` trait. Implement it for each shape. Then write a function `largest_perimeter(shapes: &[Box<dyn Shape>]) -> &dyn Shape` that returns a reference to the shape with the largest perimeter. Call it from `main` and print the winner's name.

```rust
// Add to the Shape trait:
fn perimeter(&self) -> f64;

// Implement for each type, then write:
fn largest_perimeter<'a>(shapes: &'a [Box<dyn Shape>]) -> &'a dyn Shape {
    // your code here
}
```

**Expected output:**
```
Largest perimeter: circle (18.85)
```

> **Hint:** Use `.iter().max_by(|a, b| a.perimeter().partial_cmp(&b.perimeter()).unwrap())` and `.unwrap()` to get the reference.

---

### 3. When to Choose Dynamic Dispatch

> **Docs:** [Trait objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html) · [dyn keyword](https://doc.rust-lang.org/std/keyword.dyn.html)

Choose `dyn Trait` when: (1) you need a collection of mixed types, (2) the type is not known until runtime (e.g., chosen from config or user input), or (3) you want to keep binary size small — monomorphization can bloat binaries in large codebases.

Choose `impl Trait` when: (1) you always know the type at the call site, (2) the function is called in a tight loop where vtable overhead matters, or (3) you need to call methods from multiple traits on the same value (`impl TraitA + TraitB` is fine statically but `dyn TraitA + TraitB` has limited support).

A useful rule of thumb: start with `impl Trait` for functions and switch to `Box<dyn Trait>` when you need to store values of mixed types together.

```rust
trait Plugin {
    fn run(&self, input: &str) -> String;
}

struct UpperPlugin;
struct ReversePlugin;

impl Plugin for UpperPlugin {
    fn run(&self, input: &str) -> String { input.to_uppercase() }
}

impl Plugin for ReversePlugin {
    fn run(&self, input: &str) -> String { input.chars().rev().collect() }
}

// Runtime selection — must use dyn Trait
fn load_plugin(name: &str) -> Box<dyn Plugin> {
    match name {
        "upper" => Box::new(UpperPlugin),
        "reverse" => Box::new(ReversePlugin),
        _ => panic!("unknown plugin: {name}"),
    }
}

fn main() {
    let plugin_names = ["upper", "reverse", "upper"];
    let text = "hello world";

    for name in &plugin_names {
        let plugin = load_plugin(name);
        println!("{name}: {}", plugin.run(text));
    }
}
```

**Expected output:**
```
upper: HELLO WORLD
reverse: dlrow olleh
upper: HELLO WORLD
```

#### Exercise 3.1 — Conditional Dispatch

**Goal:** Practice the factory pattern with `Box<dyn Trait>`.

Write a function `make_formatter(style: &str) -> Box<dyn Fn(&str) -> String>` that returns a boxed closure. For `"shout"` return a closure that uppercases and appends `"!"`. For `"whisper"` return a closure that lowercases and prepends `"..."`. For anything else, return the identity closure. Closures implement `Fn` so they can be used as trait objects.

**Expected output:**
```
shout: HELLO!
whisper: ...hello
default: hello
```

> **Hint:** `Box<dyn Fn(&str) -> String>` is a trait object for a closure. You can return `Box::new(|s: &str| ...)` from a function.

---

### 4. Object Safety

> **Docs:** [Object safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety) · [Reference — Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html)

A trait is *object-safe* when the compiler can build a vtable for it. The two most common rules that break object safety are:

1. **Methods with generic type parameters** — the vtable would need infinitely many entries (one per concrete type).
2. **Methods that return `Self`** — the vtable stores raw function pointers; the concrete type of `Self` is erased, so the caller would not know how large the returned value is.

When you write `dyn NotObjectSafe`, the compiler tells you exactly which method violates the rules and why.

```rust
// This trait IS object-safe
trait Drawable {
    fn draw(&self);
    fn bounding_box(&self) -> (f64, f64, f64, f64);
}

// This trait is NOT object-safe — generic method
trait Converter {
    fn convert<T: std::fmt::Display>(&self, val: T) -> String; // ERROR with dyn
}

// This trait is NOT object-safe — returns Self
trait Duplicate {
    fn duplicate(&self) -> Self; // ERROR with dyn
}

// Fix for Duplicate: use a helper trait with a boxed return
trait DuplicateBoxed {
    fn duplicate_boxed(&self) -> Box<dyn DuplicateBoxed>;
}

struct Point { x: f64, y: f64 }

impl DuplicateBoxed for Point {
    fn duplicate_boxed(&self) -> Box<dyn DuplicateBoxed> {
        Box::new(Point { x: self.x, y: self.y })
    }
}

fn main() {
    // Box<dyn Drawable> is fine
    // Box<dyn Converter> would NOT compile
    // Box<dyn DuplicateBoxed> is fine because we fixed the signature
    let p = Point { x: 1.0, y: 2.0 };
    let _dup: Box<dyn DuplicateBoxed> = p.duplicate_boxed();
    println!("DuplicateBoxed works with dyn");
}
```

#### Exercise 4.1 — Spot the Violation

**Goal:** Practice reading object-safety errors and fixing them.

The following trait is not object-safe. Identify why, then rewrite it so it can be used as `Box<dyn Summarize>`.

```rust
trait Summarize {
    fn summary(&self) -> Self; // Problem here
    fn label() -> &'static str; // Problem here too
}
```

**Expected output:**
```
Object-safe version compiles successfully.
```

> **Hint:** Change `summary` to return `String` instead of `Self`, and either remove `label` or mark it `where Self: Sized` so it's excluded from the vtable.

---

### 5. Returning `Box<dyn Trait>` from Functions

> **Docs:** [Trait objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html) · [Box&lt;T&gt;](https://doc.rust-lang.org/std/boxed/struct.Box.html)

The factory pattern is one of the clearest wins for `dyn Trait`. The caller does not need to know or care which concrete type was created — it just uses the trait interface. This lets you add new implementations without changing any call sites.

```rust
trait Encoder {
    fn encode(&self, data: &str) -> String;
    fn decoder_hint(&self) -> &str;
}

struct Base64Encoder;
struct HexEncoder;
struct NopEncoder;

impl Encoder for Base64Encoder {
    fn encode(&self, data: &str) -> String {
        // Simplified demo — not real base64
        data.bytes().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join("")
    }
    fn decoder_hint(&self) -> &str { "base64" }
}

impl Encoder for HexEncoder {
    fn encode(&self, data: &str) -> String {
        data.bytes().map(|b| format!("{:02x}", b)).collect()
    }
    fn decoder_hint(&self) -> &str { "hex" }
}

impl Encoder for NopEncoder {
    fn encode(&self, data: &str) -> String { data.to_string() }
    fn decoder_hint(&self) -> &str { "none" }
}

fn make_encoder(format: &str) -> Box<dyn Encoder> {
    match format {
        "base64" => Box::new(Base64Encoder),
        "hex" => Box::new(HexEncoder),
        _ => Box::new(NopEncoder),
    }
}

fn main() {
    for format in ["base64", "hex", "raw"] {
        let enc = make_encoder(format);
        let encoded = enc.encode("hi");
        println!("[{}] hint={} encoded={}", format, enc.decoder_hint(), encoded);
    }
}
```

**Expected output:**
```
[base64] hint=base64 encoded=6869
[hex] hint=hex encoded=6869
[raw] hint=none encoded=hi
```

#### Exercise 5.1 — Logger Factory

**Goal:** Build a factory returning `Box<dyn Logger>` based on a config string.

Define trait `Logger` with `fn log(&self, message: &str)`. Implement `ConsoleLogger` (prints `[CONSOLE] message`), `FileLogger` (holds a `filename: String`, prints `[FILE:filename] message`), and `NullLogger` (does nothing). Write `make_logger(kind: &str, target: &str) -> Box<dyn Logger>`. For `"console"` ignore `target`. For `"file"` use `target` as the filename. For anything else return `NullLogger`.

**Expected output:**
```
[CONSOLE] Server started
[FILE:app.log] Server started
```

> **Hint:** `FileLogger` stores a `String` field. In the factory, construct it as `Box::new(FileLogger { filename: target.to_string() })`.

---

## Day Project: Plugin Renderer

### What You're Building

You will build a small rendering pipeline where different `Renderer` implementations transform a piece of content into different output formats. A factory function selects the right renderer at runtime based on a format string, demonstrating the full factory + `dyn Trait` pattern in a realistic context.

### Requirements

1. Define trait `Renderer` with methods `fn render(&self, content: &str) -> String` and `fn name(&self) -> &str`.
2. Implement `PlainRenderer`: returns content unchanged. Name: `"plain"`.
3. Implement `UpperCaseRenderer`: returns `content.to_uppercase()`. Name: `"uppercase"`.
4. Implement `MarkdownRenderer`: wraps content in a triple-backtick code block. Name: `"markdown"`.
5. Implement `JsonRenderer`: returns `{"content": "<escaped>"}` (use `serde_json` or simple string formatting). Name: `"json"`.
6. Write `fn get_renderer(format: &str) -> Box<dyn Renderer>` using a `match`.
7. In `main`, iterate over `["plain", "upper", "markdown", "json"]`, retrieve each renderer, render the same content string, and print the renderer name and output.
8. Build a `Vec<Box<dyn Renderer>>` holding all four renderers and call `render` on each to demonstrate heterogeneous storage.

### Getting Started

```bash
cargo new day-15
cd day-15
```

No external dependencies needed — standard library only.

### Running Your Solution

```bash
cargo run -p day-15
```

Successful output looks like:

```
=== Factory Pattern ===
[plain] Hello, Rust World!
[uppercase] HELLO, RUST WORLD!
[markdown]
```rust
Hello, Rust World!
` ``
[json] {"content": "Hello, Rust World!"}

=== Heterogeneous Vec ===
Stored 4 renderers. Rendering each:
plain        -> Hello, Rust World!
uppercase    -> HELLO, RUST WORLD!
markdown     -> ```\nHello, Rust World!\n```
json         -> {"content": "Hello, Rust World!"}
```

### Extension Challenges

- **Easy:** Add a `HtmlRenderer` that wraps content in `<pre>` tags and registers it in the factory.
- **Medium:** Add a `Pipeline` struct that holds a `Vec<Box<dyn Renderer>>` and applies them in sequence (each renderer's output becomes the next renderer's input). Test with `[UpperCase, Markdown]`.
- **Hard:** Make `Renderer` require `fn supported_extensions(&self) -> &[&str]` and write a function that picks the right renderer by inspecting a filename's extension. Ensure the trait stays object-safe.
