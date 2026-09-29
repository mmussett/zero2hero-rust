# Day 28: Macros

> **Project:** `map!` declarative macro, `vec_of_strings!` declarative macro, and a `#[derive(Describe)]` procedural macro

## Learning Objectives

By the end of today you will be able to:
- Write declarative macros with `macro_rules!` using pattern matching and repetition
- Explain when macros offer advantages over functions
- Understand macro hygiene and variable scoping
- Describe the three kinds of procedural macros
- Write a minimal custom derive macro using `syn` and `quote`

---

## Concepts

### 1. Declarative Macros with `macro_rules!`

> **Docs:** [macro_rules! Reference](https://doc.rust-lang.org/reference/macros-by-example.html) · [Rust by Example — Macros](https://doc.rust-lang.org/rust-by-example/macros.html) · [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)

Declarative macros match on token trees — the raw syntactic structure of Rust code before it is parsed into an AST. Each arm of a `macro_rules!` block is a pattern → transcription pair. The patterns use special fragment specifiers to classify what kind of token they match:

- `$name:expr` — any expression
- `$name:ident` — an identifier (variable name, function name, etc.)
- `$name:ty` — a type
- `$name:literal` — a literal value
- `$name:tt` — any single token tree (most flexible)
- `$name:stmt` — a statement

Repetition is expressed with `$(...),*` (zero or more, comma-separated) or `$(...),+` (one or more):

```rust
macro_rules! print_all {
    // Match zero or more comma-separated expressions
    ($($x:expr),*) => {
        $(
            println!("{:?}", $x);
        )*
    };
}

// Usage:
print_all!(1, "hello", true, 3.14);
// Expands to:
// println!("{:?}", 1);
// println!("{:?}", "hello");
// println!("{:?}", true);
// println!("{:?}", 3.14);
```

Multiple arms work like `match` — the first matching pattern wins:

```rust
macro_rules! describe {
    () => { "nothing" };
    ($x:expr) => { format!("one thing: {:?}", $x) };
    ($x:expr, $($rest:expr),+) => { format!("many things, starting with: {:?}", $x) };
}
```

#### Exercise 1.1 — Simple Print Macro

**Goal:** Write a macro that accepts a variable number of arguments.

Write a `my_println!` macro that accepts any number of expressions and prints each one on a separate line with a line number prefix: `1: value`, `2: value`, etc. Test it with `my_println!(42, "hello", true)`.

**Expected output:**
```
1: 42
2: hello
3: true
```

> **Hint:** You cannot easily generate sequential numbers inside `macro_rules!` repetition. Instead, make the macro expand into a block that uses a mutable counter variable. Or use a `let mut n = 0usize; $( n += 1; println!("{}: {:?}", n, $x); )*` pattern.

---

### 2. When Macros Beat Functions

> **Docs:** [Macros Book chapter](https://doc.rust-lang.org/book/ch19-06-macros.html)

Functions are generally preferable to macros — they're easier to read, test, and compose. But macros have capabilities that functions fundamentally cannot match:

**Variable number of arguments:** Rust functions have fixed arity. `println!`, `vec!`, and `format!` all accept different numbers of arguments. A function cannot.

**Syntactic flexibility:** `assert_eq!(a, b)` generates a helpful error message that includes the source expressions `a` and `b`. A function receives only the values, not the expressions they came from.

**Lazy evaluation:** `log::debug!("expensive {}", compute())` only calls `compute()` if debug logging is enabled. A function would evaluate all arguments eagerly. The macro can wrap the body in `if condition { ... }`.

**Code generation:** `#[derive(Debug)]` generates entire `impl` blocks. This is only possible with macros.

**DSL syntax:** `html! { <div class="foo"> <p>"Hello"</p> </div> }` (from the `yew` crate) uses a macro to embed HTML-like syntax in Rust source. A function cannot change the language's grammar.

```rust
// This cannot be a function — it captures the expression text:
macro_rules! check {
    ($cond:expr) => {
        if !($cond) {
            panic!("Condition failed: {}", stringify!($cond));
        }
    };
}

check!(2 + 2 == 4);   // ok
check!(1 + 1 == 3);   // panics: "Condition failed: 1 + 1 == 3"
```

`stringify!($cond)` converts the token tree for `$cond` into a string literal — a capability with no function equivalent.

#### Exercise 2.1 — Lazy Evaluation Macro

**Goal:** Write a macro that conditionally evaluates an expensive computation.

Write a `log_if_debug!(condition, value_expr)` macro. It should evaluate `value_expr` and print it only when `condition` is `true`. Call it with `log_if_debug!(true, { println!("side effect"); 42 })` and `log_if_debug!(false, { println!("side effect"); 42 })`. Verify the side effect only occurs when condition is true.

**Expected output:**
```
side effect
debug value: 42
```
(When condition is false, no output — side effect not evaluated.)

> **Hint:** `if $condition { let v = $value_expr; println!("debug value: {}", v); }` prevents evaluation when condition is false.

---

### 3. Macro Hygiene

> **Docs:** [macro_rules! Reference](https://doc.rust-lang.org/reference/macros-by-example.html) · [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)

Hygiene prevents macros from accidentally capturing variables from the call site. In Rust's `macro_rules!`, each macro expansion creates its own scope for identifiers introduced by the macro. Variables defined inside the macro don't leak out, and variables from the call site don't leak in (unless explicitly passed as arguments).

```rust
macro_rules! make_x {
    () => {
        let x = 42;  // This x is hygienic — it won't conflict with caller's x
    };
}

fn main() {
    let x = 1;
    make_x!();
    println!("{}", x);  // Still prints 1, not 42
}
```

The compiler gives each `x` a unique internal identity. The caller's `x` and the macro's `x` are different variables, even though they have the same textual name. This makes macros safe to use without fear of name collision.

Hygiene applies to identifiers introduced by the macro. It does not apply to identifiers passed in as arguments — `$name:ident` captures the caller's identifier verbatim, which is exactly what you want when writing macros that create new names.

```rust
macro_rules! create_getter {
    ($field:ident, $type:ty) => {
        // $field is the caller's identifier — hygienic for new names inside macro
        pub fn $field(&self) -> &$type {
            &self.$field
        }
    };
}

struct Person { name: String }
impl Person {
    create_getter!(name, String);
}
```

#### Exercise 3.1 — Observe Hygiene

**Goal:** Prove that macro-internal variables don't pollute the call site.

Write a macro `let_temp!($val:expr)` that creates a variable `temp` inside the macro body, assigns `$val` to it, and returns `temp * 2` from the block. In `main`, define `let temp = 99;`, then call `let result = let_temp!(5);`. Verify that `temp` in `main` is still `99` and `result` is `10`.

**Expected output:**
```
temp: 99
result: 10
```

> **Hint:** Wrap the macro body in a block `{ let temp = $val; temp * 2 }`. The inner `temp` is hygienic.

---

### 4. Procedural Macros Overview

> **Docs:** [Procedural macros Reference](https://doc.rust-lang.org/reference/procedural-macros.html) · [proc-macro2](https://docs.rs/proc-macro2/latest/proc_macro2/)

Procedural macros operate on the Rust token stream at compile time. Unlike `macro_rules!` which does pattern matching on tokens, procedural macros are actual Rust programs that receive a `TokenStream` and produce a `TokenStream`. This gives them full programmatic power.

The three kinds of procedural macros:

**Custom derive** (`#[derive(MyTrait)]`): Generates `impl MyTrait for SomeType`. The input token stream is the struct/enum definition. The output is appended after it. Used for: `Debug`, `Serialize`, `Deserialize`, `FromRow`, `Error`, etc.

**Attribute macros** (`#[my_attr]` on a function or item): Replaces or augments the annotated item. The macro receives both the attribute's arguments and the item it annotates. Used for: `#[instrument]`, `#[tokio::main]`, `#[test]`.

**Function-like macros** (`my_macro!(tokens)`): Look like `macro_rules!` macros at the call site but are implemented as proc macros. Used for: `sql!("SELECT ...")`, `html! { ... }`.

Procedural macros must live in a separate crate with `proc-macro = true` in `Cargo.toml`. This is a compiler requirement — proc macros are compiled as shared libraries loaded by the compiler, not as part of your regular binary.

```toml
# In day-28-derive/Cargo.toml:
[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

#### Exercise 4.1 — Identify the Kind

**Goal:** Classify real-world macros by their kind.

For each of the following, identify whether it is a `macro_rules!` macro, a custom derive, an attribute macro, or a function-like macro: `vec![]`, `#[derive(Clone)]`, `#[tokio::main]`, `println!()`, `sqlx::query!()`, `#[instrument]`, `#[derive(serde::Serialize)]`. Write your answers with one sentence of justification each.

**Expected output:** Written answers in comments — no code output required.

> **Hint:** `macro_rules!` macros look identical to function-like proc macros at the call site. The difference is in their implementation, not their usage.

---

### 5. Writing a Simple Derive Macro

> **Docs:** [syn crate](https://docs.rs/syn/latest/syn/) · [quote crate](https://docs.rs/quote/latest/quote/) · [Procedural macros Reference](https://doc.rust-lang.org/reference/procedural-macros.html)

A custom derive macro needs a crate with `proc-macro = true`. The macro function is annotated with `#[proc_macro_derive(TraitName)]`. It receives a `TokenStream` representing the struct/enum definition, and returns a `TokenStream` of new code to append.

`syn` parses the input `TokenStream` into a typed AST (`DeriveInput`). `quote!` generates the output `TokenStream` using quasi-quoting — write Rust code inside `quote! { ... }` and interpolate computed values with `#variable`.

```rust
// day-28-derive/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Describe)]
pub fn describe_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let field_count = match &ast.data {
        syn::Data::Struct(s) => s.fields.len(),
        _ => 0,
    };
    let description = format!("Struct {} with {} fields", name, field_count);

    quote! {
        impl #name {
            pub fn describe(&self) -> String {
                #description.to_string()
            }
        }
    }
    .into()
}
```

In the main `day-28` crate, declare the dependency on the derive crate:

```toml
# day-28/Cargo.toml
[dependencies]
day-28-derive = { path = "../day-28-derive" }
```

```rust
// day-28/src/main.rs
use day_28_derive::Describe;

#[derive(Describe)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let p = Point { x: 1.0, y: 2.0 };
    println!("{}", p.describe());  // "Struct Point with 2 fields"
}
```

#### Exercise 5.1 — Derive with a Named Method

**Goal:** Extend the Describe derive to include the field names.

Modify the `describe_derive` function to iterate over `s.fields` and collect field names. Generate the description as `"Struct TypeName with fields: field1, field2"`. Use `quote!` interpolation with a `Vec<String>` or `Vec<Ident>` to include the field names.

**Expected output:**
```
Struct Point with fields: x, y
```

> **Hint:** `s.fields.iter().map(|f| f.ident.as_ref().map(|i| i.to_string()).unwrap_or("unnamed".to_string())).collect::<Vec<_>>().join(", ")` produces the comma-separated field list.

---

## Day Project: Three Macros

### What You're Building

Three distinct macro implementations demonstrating the range of Rust's macro system:
- `map!` — a `macro_rules!` macro that creates a `HashMap` from key-value pairs
- `vec_of_strings!` — a `macro_rules!` macro that creates a `Vec<String>` from string literals
- `#[derive(Describe)]` — a procedural custom derive that generates a `describe()` method

The project spans two crates: `day-28` (the binary) and `day-28-derive` (the proc-macro crate).

### Requirements

**Part 1: `map!` macro**
- `map!{ key => value, key => value }` syntax
- Returns `std::collections::HashMap`
- Works with any key and value types that are valid HashMap keys/values
- Works with zero pairs (returns empty HashMap)

**Part 2: `vec_of_strings!` macro**
- `vec_of_strings!["hello", "world"]` syntax
- Returns `Vec<String>` (each `&str` converted with `.to_string()`)
- Works with zero arguments (returns empty Vec)

**Part 3: `#[derive(Describe)]`**
- Works on any struct
- Generates `pub fn describe(&self) -> String`
- Returns `"Struct TypeName with N fields"` (or `"with fields: a, b, c"` for named fields)
- Lives in the `day-28-derive` crate

**Main binary:** Demonstrates all three in `main`. Add both crates to the workspace `Cargo.toml`.

### Getting Started

Create two crates (add both to your workspace's `Cargo.toml` `members` list):

```
zero2hero-rust/
  day-28/
    Cargo.toml
    src/main.rs
  day-28-derive/
    Cargo.toml
    src/lib.rs
```

```toml
# day-28-derive/Cargo.toml
[package]
name = "day-28-derive"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

```toml
# day-28/Cargo.toml
[package]
name = "day-28"
version = "0.1.0"
edition = "2021"

[dependencies]
day-28-derive = { path = "../day-28-derive" }
```

In `day-28/src/main.rs`:

```rust
use std::collections::HashMap;
use day_28_derive::Describe;

// Part 1: map! macro
macro_rules! map {
    // Empty case
    () => { HashMap::new() };
    // One or more key => value pairs
    ($($key:expr => $val:expr),+ $(,)?) => {
        {
            let mut m = HashMap::new();
            $(m.insert($key, $val);)+
            m
        }
    };
}

// Part 2: vec_of_strings! macro
macro_rules! vec_of_strings {
    () => { Vec::<String>::new() };
    ($($s:expr),+ $(,)?) => {
        vec![$($s.to_string()),+]
    };
}

// Part 3: custom derive
#[derive(Debug, Describe)]
struct Config {
    host: String,
    port: u16,
    debug: bool,
}

#[derive(Debug, Describe)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    // map!
    let scores: HashMap<&str, u32> = map! {
        "Alice" => 95,
        "Bob"   => 87,
        "Carol" => 92,
    };
    println!("Scores: {:?}", scores);

    let empty: HashMap<&str, i32> = map!();
    println!("Empty map len: {}", empty.len());

    // vec_of_strings!
    let words = vec_of_strings!["hello", "world", "from", "macros"];
    println!("Words: {:?}", words);
    println!("Type check — first word in uppercase: {}", words[0].to_uppercase());

    let empty_vec = vec_of_strings!();
    println!("Empty vec len: {}", empty_vec.len());

    // #[derive(Describe)]
    let cfg = Config {
        host: "localhost".to_string(),
        port: 8080,
        debug: true,
    };
    println!("{}", cfg.describe());

    let p = Point { x: 3.0, y: 4.0 };
    println!("{}", p.describe());
}
```

In `day-28-derive/src/lib.rs`:

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Describe)]
pub fn describe_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let description = match &ast.data {
        Data::Struct(s) => {
            match &s.fields {
                Fields::Named(named) => {
                    let field_names: Vec<String> = named
                        .named
                        .iter()
                        .filter_map(|f| f.ident.as_ref())
                        .map(|i| i.to_string())
                        .collect();
                    let count = field_names.len();
                    let joined = field_names.join(", ");
                    format!("Struct {} with {} fields: {}", name, count, joined)
                }
                Fields::Unnamed(unnamed) => {
                    format!("Struct {} with {} unnamed fields", name, unnamed.unnamed.len())
                }
                Fields::Unit => {
                    format!("Struct {} with no fields", name)
                }
            }
        }
        _ => format!("{} (not a struct)", name),
    };

    quote! {
        impl #name {
            pub fn describe(&self) -> String {
                #description.to_string()
            }
        }
    }
    .into()
}
```

### Running Your Solution

```bash
cargo run -p day-28
```

Expected output:
```
Scores: {"Alice": 95, "Bob": 87, "Carol": 92}
Empty map len: 0
Words: ["hello", "world", "from", "macros"]
Type check — first word in uppercase: HELLO
Empty vec len: 0
Struct Config with 3 fields: host, port, debug
Struct Point with 2 fields: x, y
```

To see the macro expansion (requires nightly):
```bash
cargo +nightly expand -p day-28
```

### Extension Challenges

- **Easy:** Add a trailing comma variant to `map!` so `map!{ "a" => 1, }` (with trailing comma) also works. The `$(,)?` pattern handles optional trailing commas — add it if not already present.
- **Medium:** Write a `#[derive(Builder)]` procedural macro that generates a builder pattern for any struct. For `struct Foo { x: u32, y: String }`, it generates `FooBuilder` with methods `.x(u32)`, `.y(String)`, and `.build() -> Foo`. This requires more `syn` API exploration.
- **Hard:** Write an attribute macro `#[memoize]` that wraps a function to cache its return value keyed on the arguments (using a `HashMap` in a `Mutex`). This requires generating both a new function body and a static variable to hold the cache.
