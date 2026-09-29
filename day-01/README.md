# Day 01: Toolchain, Cargo, and Hello World

> **Project:** Hello, Cargo — a binary that prints your name, the day number, and a motivational message using `println!` with string interpolation

## Learning Objectives

By the end of today you will be able to:
- Explain what Rust's safety guarantees are and why they matter
- Distinguish between safe Rust and `unsafe` Rust and explain when each applies
- Explain what `rustup`, `rustc`, and `cargo` each do and verify they are installed
- Create a new Rust project with `cargo new` and understand the generated file structure
- Write and run a basic `fn main()` entry point
- Use `println!` to print text, interpolate variables with `{}`, and print multiple values
- Write single-line and block comments in Rust

---

## Concepts

### 1. Safe and Unsafe Rust — The Foundation

> **Docs:** [The Rust Book — Unsafe Rust](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html) · [Rustonomicon](https://doc.rust-lang.org/nomicon/) · [Rust by Example — Unsafe](https://doc.rust-lang.org/rust-by-example/unsafe.html)

Before you write a single line of Rust, you need to understand the idea that everything else in this course is built on: **Rust is a memory-safe language by default, with a deliberate and explicit mechanism for stepping outside those guarantees when absolutely necessary.**

#### What does "memory safe" mean?

In languages like C and C++, the programmer manages memory manually — allocating it, using it, and freeing it. Getting this wrong causes some of the most severe and common software vulnerabilities in existence:

- **Use-after-free** — reading memory after it has been freed (crash or data corruption)
- **Buffer overflow** — writing past the end of an array (exploited in almost every major system vulnerability)
- **Double-free** — freeing the same memory twice (heap corruption)
- **Data races** — two threads reading and writing the same memory simultaneously (undefined behaviour)
- **Null pointer dereference** — following a pointer that points to nothing (crash)

These are not theoretical. Buffer overflows and use-after-free account for approximately **70% of all critical security vulnerabilities** in large C/C++ codebases (Microsoft, Google, and Mozilla have all reported this figure). Rust eliminates all of them — not by adding a garbage collector, but by checking your code at **compile time**.

#### How Rust achieves safety

Rust's compiler enforces a set of rules — the **ownership system** and **borrow checker** — that make it impossible to write the bugs listed above in safe Rust. You will learn these rules in detail over the coming days. For now, the key insight is:

- **The compiler rejects your program** if it cannot prove the code is memory-safe. You do not get a warning — you get a compile error. There is no "trust me, it's fine" in safe Rust.
- Because these checks happen at compile time, they have **zero runtime cost**. The resulting binary is as fast as equivalent C code.
- Thread safety is also enforced at compile time. If you try to share data between threads unsafely, the compiler rejects it.

#### Safe Rust vs `unsafe` Rust

The vast majority of Rust code — and all of the code in this course until Day 27 — is **safe Rust**. You cannot cause undefined behaviour in safe Rust. The compiler guarantees it.

`unsafe` is an explicit keyword that creates a block where four additional operations become available:

1. Dereference a raw pointer
2. Call an unsafe function
3. Implement an unsafe trait
4. Access or modify a mutable static variable
5. Access fields of a `union`

Everything else — the borrow checker, type system, lifetime rules — still applies inside `unsafe`. The keyword does not turn off Rust. It is a narrow escape hatch for the rare cases where you need to do something the compiler cannot verify: calling C libraries, implementing low-level data structures, or writing operating system code.

```rust
// This is SAFE Rust — the compiler proves this is correct
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// This is UNSAFE Rust — the programmer takes responsibility
// for ensuring the pointer is valid
unsafe fn read_raw(ptr: *const i32) -> i32 {
    *ptr // dereferencing a raw pointer requires unsafe
}
```

**The rule:** write safe Rust unless you have a specific, documented reason not to. When you use `unsafe`, write a comment explaining exactly why it is sound. You will write your first `unsafe` code on Day 27 — until then, every program you write is fully memory-safe.

#### Ownership — the mechanism behind memory safety

You just saw *that* Rust is memory safe. The question is *how*. The answer is **ownership** — a set of compile-time rules that govern who is responsible for each piece of memory:

1. Every value has exactly one **owner** — the variable that holds it.
2. When the owner goes out of scope, the value is **dropped** (memory freed). No garbage collector needed.
3. Ownership can be **moved** (transferred to a new variable) or **borrowed** (lent temporarily via a reference).

```rust
fn main() {
    let s = String::from("hello"); // s owns the string
    let t = s;                     // ownership MOVES to t — s is no longer valid
    // println!("{}", s);          // compile error: s was moved
    println!("{}", t);             // t owns it now — this is fine
}
```

Because only one variable can own a value at a time, Rust can guarantee:
- **No double-free**: only the owner frees the memory, and only once.
- **No use-after-free**: after the move, `s` cannot be accessed.
- **No dangling references**: the borrow checker ensures references cannot outlive the data they point to.

You will explore ownership in full detail on **Day 04** and borrowing on **Day 05**. You will encounter **lifetimes** — the compiler's way of naming how long a borrow is valid — on **Day 05** (introduction) and **Day 12** (mastery). For now, hold onto the core idea: *every value has one owner, and ownership is how Rust knows when to free memory safely.*

#### Why this matters for you

Most languages ask you to choose between safety and performance. Rust says you do not have to choose. This is why Rust is used in:
- The Linux kernel (first language other than C permitted for kernel code)
- Firefox's layout engine (replacing C++ for safety)
- AWS, Cloudflare, and Microsoft (rewriting critical infrastructure)
- Embedded systems and WebAssembly (no runtime overhead)

You are not just learning a language — you are learning a new way of thinking about correctness that will make you a better programmer in any language.

#### Exercise 1.0 — Reflect on Safety

**Goal:** Understand the safe/unsafe boundary conceptually before writing code.

Answer these questions in a comment block at the top of a new file:

1. What is one type of memory bug that safe Rust prevents at compile time?
2. What keyword do you use when you need to step outside Rust's safety guarantees?
3. Is `unsafe` code automatically incorrect or dangerous? Why or why not?

```rust
// Your answers here:
// 1.
// 2.
// 3.
fn main() {}
```

**Expected:** The file compiles. There is no "expected output" — this exercise builds mental model, not program output.

> **Hint:** Revisit the section above. For question 3, think about what `unsafe` actually does and does not disable.

---

### 2. The Rust Toolchain

> **Docs:** [The Rust Book — Installation](https://doc.rust-lang.org/book/ch01-01-installation.html) · [Cargo Book](https://doc.rust-lang.org/cargo/) · [rustup.rs](https://rustup.rs)

Rust has three main command-line tools that work together. Understanding what each one does prevents a lot of early confusion.

**`rustup`** is the toolchain installer and version manager. Think of it like `nvm` for Node.js or `pyenv` for Python. It downloads and manages Rust compiler versions, lets you switch between `stable`, `beta`, and `nightly` releases, and adds target platforms for cross-compilation. Most of the time you install it once and forget it exists — until you need to update.

**`rustc`** is the actual Rust compiler. It takes a `.rs` source file and produces a native binary. You could use it directly (`rustc main.rs`), but in practice almost nobody does — that is what Cargo is for. Knowing `rustc` exists helps you understand what is actually happening under the hood when you run `cargo build`.

**`cargo`** is Rust's build system and package manager combined. It handles creating projects, compiling code, managing dependencies ("crates"), running tests, and publishing libraries. One tool does everything that would take `make` + `npm` + a custom build script in other ecosystems. This is where you will spend virtually all your time.

```rust
// You can verify your installation by running these commands in your terminal.
// The output should look similar to:
//
// $ rustc --version
// rustc 1.78.0 (9b00956e5 2024-04-29)
//
// $ cargo --version
// cargo 1.78.0 (54d8815d0 2024-03-26)
//
// $ rustup --version
// rustup 1.27.1 (54dd3d00f 2024-04-24)
//
// If any of these commands fail, visit https://rustup.rs to install the toolchain.

fn main() {
    println!("Toolchain verified!");
}
```

#### Exercise 1.1 — Verify Your Installation

**Goal:** Confirm all three toolchain components are installed and working.

Open a terminal and run `rustc --version`, `cargo --version`, and `rustup --version` one by one. Each command should print a version number without errors. If you see a "command not found" error, visit `https://rustup.rs` and follow the installation instructions for your operating system.

**Expected output:**
```
rustc 1.XX.X (...)
cargo 1.XX.X (...)
rustup 1.XX.X (...)
```

> **Hint:** On Windows you may need to restart your terminal after installation so the PATH changes take effect.

---

### 3. Cargo and Project Structure

> **Docs:** [The Rust Book — Hello, Cargo!](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html) · [Cargo Book — Creating a Package](https://doc.rust-lang.org/cargo/guide/creating-a-new-project.html)

Running `cargo new hello-cargo` creates a fully set-up project in a new directory. Understanding what it generates saves you from being confused by mysterious files.

The generated layout looks like this:

```
hello-cargo/
├── Cargo.toml       ← project manifest (name, version, dependencies)
└── src/
    └── main.rs      ← your source code lives here
```

**`Cargo.toml`** is the project manifest. It is written in TOML (Tom's Obvious Minimal Language) — a simple key-value config format. The `[package]` section names your project, sets its version, and records which edition of Rust you are targeting. The `[dependencies]` section is where you will eventually list external libraries (crates). Right now it is empty.

**`src/main.rs`** is where your Rust source code lives. Cargo expects source files to be inside `src/`. The name `main.rs` is special: it tells Cargo this is a binary crate (a program you can run), not a library.

The **Rust Edition** listed in `Cargo.toml` (`edition = "2021"`) is worth a quick mention. Rust releases new editions every few years to make backwards-incompatible language improvements while keeping old code working. Edition 2021 is the current standard and the one we use throughout this curriculum.

```rust
// This is what a freshly generated Cargo.toml looks like:
//
// [package]
// name = "hello-cargo"
// version = "0.1.0"
// edition = "2021"
//
// [dependencies]
//
// And this is the generated src/main.rs:

fn main() {
    println!("Hello, world!");
}
```

#### Exercise 1.2 — Create and Explore a Project

**Goal:** Practice creating a Cargo project and reading its generated files.

Run `cargo new my-first-project` in your terminal. Open `Cargo.toml` in a text editor and read each line. Now open `src/main.rs`. Run `cargo run` from inside the `my-first-project` directory to compile and execute the program. Note that Cargo creates a `target/` directory for build artifacts — you never edit anything inside `target/`.

```bash
cargo new my-first-project
cd my-first-project
cargo run
```

**Expected output:**
```
   Compiling my-first-project v0.1.0 (...)
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
     Running `target/debug/my-first-project`
Hello, world!
```

> **Hint:** `cargo run` does `cargo build` followed by running the resulting binary. You can run just `cargo build` if you only want to compile without executing.

---

### 4. The `fn main()` Entry Point

> **Docs:** [The Rust Book — Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html) · [Reference — Functions](https://doc.rust-lang.org/reference/items/functions.html)

Every Rust binary must have a `main` function. This is the first function the operating system calls when your program starts. If there is no `main`, the program cannot run.

The syntax `fn main()` breaks down as: `fn` is the keyword that introduces a function definition, `main` is its name, `()` means it takes no parameters, and the absence of `-> SomeType` means it returns nothing (the "unit" type, written `()`). We will cover functions in depth on Day 3 — for now just know that `main` is special and mandatory.

The curly braces `{ }` define the function body. Every statement and expression in your program lives inside some set of curly braces. Rust is strict about this — unlike Python, indentation is not meaningful to the compiler (though you should still indent for readability).

```rust
// A minimal, complete Rust program.
// fn   = "function" keyword
// main = the name — required for binary crates
// ()   = no parameters
// { }  = function body
fn main() {
    // Your program's logic goes here.
    // When main returns, the program exits.
}
```

#### Exercise 1.3 — Empty Main

**Goal:** Understand that a Rust program with just `fn main() {}` is valid and compiles cleanly.

In the `src/main.rs` you created in Exercise 1.2, delete everything inside the curly braces so only `fn main() {}` remains. Run `cargo run`. The program should compile and exit immediately with no output and no error.

**Expected output:**
```
   Compiling my-first-project v0.1.0 (...)
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
     Running `target/debug/my-first-project`
```

> **Hint:** No output after "Running" is correct — an empty `main` does nothing before it exits.

---

### 5. `println!` — Printing to the Screen

> **Docs:** [`println!`](https://doc.rust-lang.org/std/macro.println.html) · [`print!`](https://doc.rust-lang.org/std/macro.print.html) · [Rust by Example — Formatted Print](https://doc.rust-lang.org/rust-by-example/hello/print.html)

`println!` is how you print a line of text to the terminal. Notice the exclamation mark: `println!` is a **macro**, not a regular function. In Rust, macros look like function calls but end with `!`. They are expanded by the compiler before your code is compiled. You do not need to understand how macros work internally right now — just remember the `!` means "this is a macro."

The most important feature of `println!` is **format strings**. The first argument is always a string literal that can contain `{}` placeholders. Each `{}` is replaced, in order, by the subsequent arguments. This is how you print variable values alongside fixed text.

You can have as many `{}` placeholders as you like, as long as you provide a matching number of values after the format string. If the counts do not match, you get a **compile-time** error — Rust catches this before your program ever runs.

```rust
fn main() {
    // Print a plain string — no placeholders needed
    println!("Hello, world!");

    // Print with one placeholder
    let name = "Alice";
    println!("Hello, {}!", name);

    // Print with multiple placeholders — values fill in left to right
    let day = 1;
    let language = "Rust";
    println!("Day {}: Learning {}", day, language);

    // You can also use the value directly without a variable
    println!("2 + 2 = {}", 2 + 2);

    // {:#?} is a "debug pretty-print" format — useful for complex types (Day 3)
    // {:?} is a compact debug format
}
```

#### Exercise 1.4 — Interpolation Practice

**Goal:** Practice using `{}` placeholders to combine variables and text.

Write a program that stores your name in a variable, stores your favourite number in another variable, and prints: `My name is NAME and my favourite number is NUMBER.` — with your actual name and number substituted in.

```rust
fn main() {
    let name = "your name here";
    let favourite_number = 0; // replace with your favourite number
    // println! call goes here
}
```

**Expected output:**
```
My name is Alice and my favourite number is 7.
```

> **Hint:** You need exactly two `{}` placeholders in your format string, and two values after it separated by commas.

---

### 6. Comments

> **Docs:** [The Rust Book — Comments](https://doc.rust-lang.org/book/ch03-04-comments.html) · [rustdoc — Documentation Comments](https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html)

Comments are notes for human readers — the compiler ignores them entirely. Rust supports two comment styles.

**Single-line comments** start with `//` and continue to the end of the line. This is by far the most common style. Everything from the `//` to the end of the line is ignored.

**Block comments** start with `/*` and end with `*/`. They can span multiple lines. Block comments can be nested, which is unusual among programming languages — `/* outer /* inner */ still outer */` is valid Rust. In practice, most Rust code uses `//` for everything, even multi-line comments (one `//` per line).

There is also a third style — **documentation comments** (`///` and `//!`) — which generate HTML documentation when you run `cargo doc`. We will use these properly on Day 7 when we build a library crate. For now, just know they exist.

```rust
fn main() {
    // This is a single-line comment.
    // Use these for most explanatory notes.

    let x = 5; // Comments can go at the end of a line too

    /*
        This is a block comment.
        It spans multiple lines.
        The compiler ignores everything between the delimiters.
    */

    println!("x is {}", x);

    // Documentation comments look like this (triple slash):
    // /// Computes the sum of two numbers.
    // They appear in cargo-generated docs for public items.
}
```

#### Exercise 1.5 — Comment Your Code

**Goal:** Practice writing informative comments that explain the "why" behind code.

Take your solution from Exercise 1.4 and add at least one single-line comment above each `let` statement explaining what the variable represents. Add a block comment at the top of the file describing what the program does.

**Expected output:**
```
My name is Alice and my favourite number is 7.
```

> **Hint:** The output does not change — comments are invisible to the running program. This exercise is about the habit of explaining your code to future readers (including future you).

---

## Day Project: Hello, Cargo

### What You're Building

A small binary that introduces you — it prints your name, announces that today is Day 1, and displays an encouraging motivational message. The twist is that none of these are hardcoded directly in `println!` — they are stored in `let` bindings first, then interpolated into the output. This forces you to use everything you learned today: Cargo project structure, `fn main()`, `println!` with placeholders, `let` bindings (previewed here, covered fully on Day 2), and comments.

It is a tiny program, but it is genuinely yours, and getting a clean `cargo run` with the right output is a satisfying milestone on Day 1.

### Requirements

1. Create a new Cargo project called `day-01` (or use the existing `day-01` directory in this workspace).
2. Declare a `let` binding for your name (a string value in double quotes).
3. Declare a `let` binding for the day number (`1`).
4. Declare a `let` binding for a motivational message of your choice.
5. Use `println!` to print all three values in a single formatted sentence.
6. Use `println!` a second time to print just the motivational message on its own line.
7. Add a block comment at the top of the file with a one-sentence description of the program.
8. Add single-line comments explaining each `let` binding.

### Getting Started

If you are working inside the `zero2hero-rust` workspace, your `day-01` directory already has a `Cargo.toml`. Open `day-01/src/main.rs` and replace its contents with:

```rust
/*
 * Day 01 — Hello, Cargo
 * Prints a personal greeting and motivational message.
 */

fn main() {
    // Store your name as a string literal
    let name = "Your Name";

    // The current day of the curriculum
    let day_number = 1;

    // Something encouraging to keep you going
    let message = "Every expert was once a beginner!";

    // Print the greeting using all three variables
    println!("Hi! I'm {}, and today is Day {} of Zero to Hero: Rust.", name, day_number);

    // Print just the motivational message
    println!("Thought for the day: {}", message);
}
```

Replace `"Your Name"` with your actual name and `"Every expert was once a beginner!"` with your own motivational message.

### Running Your Solution

```bash
cargo run -p day-01
```

Successful output looks like:

```
   Compiling day-01 v0.1.0 (...)
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
     Running `target/debug/day-01`
Hi! I'm Alice, and today is Day 1 of Zero to Hero: Rust.
Thought for the day: Every expert was once a beginner!
```

The exact wording of your message and name will differ — what matters is that both `println!` calls produce output and the variables are correctly interpolated (not printed literally as `{}`).

### Extension Challenges

- **Easy:** Add a third `println!` that prints your city and country using two separate variables: `"I'm coding from CITY, COUNTRY today."`.
- **Medium:** Print the same greeting five times by writing five `println!` calls, each with a slightly different message (for example, increasing enthusiasm). Notice how repetitive this is — you'll fix this elegantly with loops on Day 2.
- **Hard:** Look up the `print!` macro (no newline at the end) and rewrite the output so everything appears on a single line, then add a `\n` newline escape sequence to end it cleanly. Run `cargo doc --open` on a Rust standard library reference to find out what other format specifiers like `{:>10}` do.
