# Day 07: Modules and Packages

> **Project:** Contact Card Library — refactor the Day 03 contact card into a library crate with `src/contact.rs`, clean re-exports in `src/lib.rs`, and a runnable `examples/demo.rs`

## Learning Objectives

By the end of today you will be able to:
- Declare inline modules and file-based modules using `mod`
- Control visibility with `pub` at the struct, field, function, and module level
- Bring items into scope with `use` and combine imports with nested `use`
- Re-export internal items with `pub use` to create a clean public API
- Explain the difference between a library crate and a binary crate
- Navigate `crate::`, `super::`, and `self::` paths

---

## Concepts

### 1. `mod` — Declaring Modules

> **Docs:** [Book — Modules overview](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html) · [Book — Defining modules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html) · [Reference — Modules](https://doc.rust-lang.org/reference/items/modules.html) · [Rust by Example — Modules](https://doc.rust-lang.org/rust-by-example/mod.html)

As programs grow, cramming everything into one file becomes unmanageable. Rust's module system lets you split code into logical namespaces. A **module** is a named scope — it groups related items (functions, structs, enums, constants) and controls their visibility.

There are two ways to declare a module:

**Inline modules** — the module body lives directly in the current file, surrounded by curly braces:

```rust
mod greetings {
    pub fn hello(name: &str) {
        println!("Hello, {}!", name);
    }
}
```

**File-based modules** — you write `mod greetings;` (no body, just a semicolon) and Rust looks for the module's code in either `src/greetings.rs` or `src/greetings/mod.rs`. This is how you split large programs across multiple files.

Modules can be nested: a module can contain other modules, forming a tree. Every Rust crate has an implicit root module — for a binary crate that is `src/main.rs`, for a library crate it is `src/lib.rs`.

```rust
// src/main.rs — demonstrates both inline and file-based module styles

// Inline module — body is here in this file
mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn square(n: i32) -> i32 {
        n * n
    }

    // A nested module inside math
    pub mod trig {
        pub fn sin_approx(x: f64) -> f64 {
            // Very rough approximation for small x
            x - (x * x * x) / 6.0
        }
    }
}

// File-based module — code lives in src/utils.rs
// mod utils;  ← you would write this to load src/utils.rs

fn main() {
    // Access items using the full path
    println!("{}", math::add(3, 4));
    println!("{}", math::square(5));
    println!("{:.4}", math::trig::sin_approx(0.5));
}
```

#### Exercise 1.1 — Inline Modules

**Goal:** Define two inline modules and call functions from each.

Create a program with two inline modules: `mod strings` containing `pub fn reverse(s: &str) -> String` (hint: `s.chars().rev().collect()`), and `mod numbers` containing `pub fn factorial(n: u64) -> u64` (compute n!). Call both functions from `main`.

```rust
mod strings {
    pub fn reverse(s: &str) -> String {
        todo!()
    }
}

mod numbers {
    pub fn factorial(n: u64) -> u64 {
        todo!()
    }
}

fn main() {
    println!("{}", strings::reverse("hello"));
    println!("{}", numbers::factorial(5));
}
```

**Expected output:**
```
olleh
120
```

> **Hint:** `factorial(0)` and `factorial(1)` should both return 1. Use a `match` or `if` for the base case, and multiply `n * factorial(n - 1)` for the recursive case.

---

### 2. `pub` — Controlling Visibility

> **Docs:** [Book — pub keyword](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html) · [Reference — Visibility](https://doc.rust-lang.org/reference/visibility-and-privacy.html)

Every item in Rust is **private by default**. Nothing inside a module is accessible from outside that module unless explicitly marked `pub`. This is the opposite of most languages, where things are public by default and you add keywords to make them private.

This default-private philosophy is deliberate: it forces you to think about your API surface. Code you do not mark `pub` can be changed freely without breaking anything outside the module. Code you mark `pub` is a commitment to your users.

Visibility rules to know:
- `pub fn`, `pub struct`, `pub enum`, `pub mod` — makes the item accessible from outside the module.
- `pub struct` with private fields — the struct is public, but its fields are not accessible unless also marked `pub`. Callers must use your constructor functions.
- `pub(crate)` — accessible anywhere in the same crate, but not to external users.
- `pub(super)` — accessible in the parent module only.

```rust
mod account {
    // Struct is public, but fields are private
    pub struct BankAccount {
        owner: String,      // private — outsiders cannot access this
        pub balance: f64,   // public — outsiders can read this
    }

    impl BankAccount {
        // Public constructor — the only way to create an account
        pub fn new(owner: &str, opening_balance: f64) -> BankAccount {
            BankAccount {
                owner: owner.to_string(),
                balance: opening_balance,
            }
        }

        // Public method to read the private field
        pub fn owner(&self) -> &str {
            &self.owner
        }

        pub fn deposit(&mut self, amount: f64) {
            self.balance += amount;
        }

        // Private helper — not part of the public API
        fn log(&self, action: &str) {
            println!("[{}] {} — balance: {}", self.owner, action, self.balance);
        }

        pub fn withdraw(&mut self, amount: f64) -> bool {
            if amount <= self.balance {
                self.balance -= amount;
                self.log("withdrawal"); // private fn, callable within the module
                true
            } else {
                false
            }
        }
    }
}

fn main() {
    let mut acc = account::BankAccount::new("Alice", 1000.0);
    println!("Owner: {}", acc.owner());
    println!("Balance: {}", acc.balance); // balance is pub, direct access is fine

    acc.deposit(500.0);
    let ok = acc.withdraw(200.0);
    println!("Withdrew: {}, new balance: {}", ok, acc.balance);

    // This would be a compile error — owner is private:
    // println!("{}", acc.owner); // error[E0616]: field `owner` of struct is private
}
```

#### Exercise 2.1 — Private Fields, Public API

**Goal:** Practice the pattern of private fields accessed through public methods.

Create a module `mod counter` with a `pub struct Counter` that has a private `value: u32` and a private `step: u32`. Implement `pub fn new(step: u32) -> Counter`, `pub fn increment(&mut self)`, `pub fn reset(&mut self)`, and `pub fn value(&self) -> u32`. In `main`, create a counter with step `3`, increment it four times, print the value, reset, print again.

**Expected output:**
```
After 4 increments (step 3): 12
After reset: 0
```

> **Hint:** `increment` should add `self.step` to `self.value`. The private fields mean no one outside the module can create a `Counter` with arbitrary field values — only `new()` can.

---

### 3. `use` — Bringing Paths into Scope

> **Docs:** [Book — use keyword](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html)

Typing `math::trig::sin_approx(x)` every time is verbose. `use` brings a path into the current scope so you can use the short name. Best practice in Rust is to bring the **parent module** into scope for functions (so calls look like `module::function()`) but bring the **type itself** into scope for structs and enums (so you can write `ContactCard` instead of `contact::ContactCard`).

`use` statements can be nested using curly braces to reduce repetition. You can also create an alias with `as` to avoid name conflicts.

```rust
// Without use:
// std::collections::HashMap::new()
// std::io::BufReader::new()

// With use — bring the type into scope
use std::collections::HashMap;

// Nested use — import multiple items from the same path
use std::{
    fmt,           // the fmt module
    io::{self, Write},  // io itself (as `io`) plus io::Write
};

// Alias with as
use std::collections::HashMap as Map;

fn main() {
    // Now we can write HashMap directly
    let mut scores: HashMap<&str, i32> = HashMap::new();
    scores.insert("Alice", 95);
    scores.insert("Bob", 87);

    for (name, score) in &scores {
        println!("{}: {}", name, score);
    }

    // Alias version
    let mut map: Map<&str, bool> = Map::new();
    map.insert("active", true);
    println!("active: {}", map["active"]);
}
```

#### Exercise 3.1 — Import Practice

**Goal:** Rewrite a program to use `use` statements instead of full paths everywhere.

Take the code from Exercise 1.1 and add `use strings::reverse;` and `use numbers::factorial;` at the top of `main` (or at module level). Rewrite the `main` body to call `reverse` and `factorial` directly without the module prefix.

**Expected output:**
```
olleh
120
```

> **Hint:** `use` inside a function is valid and scopes to that function. `use` at module level (outside any function) is visible to the whole file.

---

### 4. `pub use` Re-exports

> **Docs:** [Book — use keyword](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html)

Sometimes you want users of your crate to use items without knowing the internal module structure. `pub use` re-exports an item — it makes it available at the current module level as if it were defined there.

This is how Rust library authors create clean, stable public APIs: they organise code internally however they like, then re-export the important types at the crate root with `pub use`. Users see only the re-exported names, not the internal layout. The internal layout can change without breaking the public API.

```rust
// Imagine this is src/lib.rs

mod internal {
    pub mod types {
        pub struct Config {
            pub debug: bool,
            pub timeout: u64,
        }

        impl Config {
            pub fn default() -> Config {
                Config { debug: false, timeout: 30 }
            }
        }
    }
}

// Re-export Config at the crate root — users see `my_crate::Config`
// not `my_crate::internal::types::Config`
pub use internal::types::Config;

// Without pub use, a user would have to write:
// use my_crate::internal::types::Config; // messy and exposes internals
```

#### Exercise 4.1 — Re-export Clean API

**Goal:** Organise code into a nested module and re-export the important type at the outer level.

Create a module `mod geometry` containing a sub-module `mod shapes` with `pub struct Circle { pub radius: f64 }` and `pub fn area(c: &Circle) -> f64`. At the `geometry` module level, add `pub use shapes::{Circle, area};`. In `main`, use `geometry::Circle` and `geometry::area` (not `geometry::shapes::Circle`).

**Expected output:**
```
Circle with radius 5 has area 78.54
```

> **Hint:** After `pub use shapes::{Circle, area};` in the `geometry` module, you can write `use geometry::Circle;` in `main` and access it without the `shapes::` prefix.

---

### 5. Library Crates and Binary Crates

> **Docs:** [Book — Packages and crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html) · [Book — Separate files](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html) · [rustdoc](https://doc.rust-lang.org/rustdoc/)

Every Cargo package is either a **binary crate**, a **library crate**, or both.

- **Binary crate** — has `src/main.rs`. Compiles to an executable. This is what you have built on Days 1–6.
- **Library crate** — has `src/lib.rs`. Compiles to a `.rlib` file that other crates can depend on. There is no `main` — you cannot run a library directly.

A single Cargo package can have both `src/lib.rs` (the library) and `src/main.rs` (a binary that uses the library). You can also add additional binaries under `src/bin/` and runnable examples under `examples/`.

**`examples/`** is particularly useful during development: files in the `examples/` directory are small standalone programs that demonstrate the library's API. They are not tests, not benchmarks — just runnable usage examples. Run them with:

```bash
cargo run --example demo -p day-07
```

Inside an example file, you use the library the same way any external user would — with `use crate_name::SomeType;` where `crate_name` matches the `name` field in `Cargo.toml`.

```
day-07/
├── Cargo.toml          ← name = "day-07", no [[bin]] needed
├── src/
│   ├── lib.rs          ← crate root for the library
│   └── contact.rs      ← ContactCard module (loaded via `mod contact;` in lib.rs)
└── examples/
    └── demo.rs         ← runnable demo (run with `cargo run --example demo -p day-07`)
```

#### Exercise 5.1 — Identify the Crate Type

**Goal:** Understand how Cargo determines crate type from file layout.

Answer the following questions in comments at the top of a scratch file (or just think through them):

1. A package has only `src/main.rs`. What kind of crate is it?
2. A package has only `src/lib.rs`. What kind of crate is it?
3. A package has both `src/main.rs` and `src/lib.rs`. What can you run?
4. A package has `src/lib.rs` and `examples/demo.rs`. What command runs the example?

No code output for this exercise — it is a comprehension check. Write the answers in a comment block in your `src/lib.rs`.

> **Hint:** `cargo run` runs `src/main.rs`. `cargo run --example demo` runs `examples/demo.rs`. Library-only crates can only be tested or demonstrated via examples and tests.

---

### 6. `crate::`, `super::`, and `self::` Paths

> **Docs:** [Book — Paths](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html) · [Book — Modules overview](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)

Rust has three special path prefixes for navigating the module tree without relying on absolute paths from the crate root.

- **`crate::`** — starts from the crate root (`src/lib.rs` or `src/main.rs`). Useful when you need to reference something that is far away in the module tree.
- **`super::`** — refers to the parent module. Useful in nested modules when you want to call something in the enclosing module.
- **`self::`** — refers to the current module. Usually redundant (you can write names directly), but occasionally needed for disambiguation.

```rust
// src/lib.rs

pub mod outer {
    pub fn outer_function() {
        println!("Called outer_function");
    }

    pub mod inner {
        pub fn inner_function() {
            println!("Called inner_function");

            // super:: reaches up to the `outer` module
            super::outer_function();

            // crate:: reaches the crate root (src/lib.rs)
            crate::top_level_function();
        }
    }
}

pub fn top_level_function() {
    println!("Called top_level_function");
}
```

#### Exercise 6.1 — Navigate the Module Tree

**Goal:** Write a nested module that calls a function in its parent module using `super::`.

Create an inline module `mod parent` with `pub fn greet() { println!("Hello from parent"); }`. Inside it, create `pub mod child` with `pub fn call_parent()` that calls `super::greet()`. In `main`, call `parent::child::call_parent()`.

**Expected output:**
```
Hello from parent
```

> **Hint:** `super::greet()` inside `child` refers to `parent::greet()` — one level up in the module tree.

---

## Day Project: Contact Card Library

### What You're Building

You will refactor the Day 03 `ContactCard` into a proper library crate. The code splits across two source files: `src/contact.rs` holds the struct and its implementation, and `src/lib.rs` declares the module and re-exports the type at the crate root. A runnable example in `examples/demo.rs` demonstrates the full API exactly as an external user would see it.

This is the real Rust workflow for library authors: keep implementation details in modules, expose a clean public API via re-exports, and write examples that double as documentation and integration tests.

Because this is a **library crate**, there is no `src/main.rs`. The example file is the only runnable entry point.

### Requirements

1. `src/contact.rs`:
   - `pub struct ContactCard` with `pub name: String`, `pub email: String`, `pub age: u32`.
   - `#[derive(Debug)]` on the struct.
   - `impl ContactCard` with:
     - `pub fn new(name: &str, email: &str, age: u32) -> ContactCard`
     - `pub fn greet(&self) -> String`
     - `pub fn is_adult(&self) -> bool`
     - `pub fn summary(&self) -> String` — e.g., `"Alice (30) <alice@example.com>"`

2. `src/lib.rs`:
   - `pub mod contact;` — loads `src/contact.rs`
   - `pub use contact::ContactCard;` — re-exports `ContactCard` at the crate root

3. `examples/demo.rs`:
   - `use day_07::ContactCard;` — import from the library crate (name matches `Cargo.toml`)
   - Create three `ContactCard` instances using `ContactCard::new(...)`
   - Print each with `{:#?}`
   - Print each summary
   - Call `greet()` and `is_adult()` on each
   - Iterate over them with a `for` loop

### Getting Started

**`src/contact.rs`:**

```rust
#[derive(Debug)]
pub struct ContactCard {
    pub name: String,
    pub email: String,
    pub age: u32,
}

impl ContactCard {
    pub fn new(name: &str, email: &str, age: u32) -> ContactCard {
        ContactCard {
            name: name.to_string(),
            email: email.to_string(),
            age,
        }
    }

    pub fn greet(&self) -> String {
        format!("Hi, I'm {} — reach me at {}", self.name, self.email)
    }

    pub fn is_adult(&self) -> bool {
        self.age >= 18
    }

    pub fn summary(&self) -> String {
        format!("{} ({}) <{}>", self.name, self.age, self.email)
    }
}
```

**`src/lib.rs`:**

```rust
pub mod contact;

// Re-export ContactCard at the crate root for a clean API
pub use contact::ContactCard;
```

**`examples/demo.rs`:**

```rust
// Use the library crate — name must match Cargo.toml [package] name
use day_07::ContactCard;

fn main() {
    let contacts = vec![
        ContactCard::new("Alice", "alice@example.com", 30),
        ContactCard::new("Bob", "bob@example.com", 16),
        ContactCard::new("Carol", "carol@example.com", 42),
    ];

    for contact in &contacts {
        println!("{:#?}", contact);
        println!("Summary:  {}", contact.summary());
        println!("Greeting: {}", contact.greet());
        println!("Adult:    {}", contact.is_adult());
        println!("---");
    }
}
```

### Running Your Solution

```bash
cargo run --example demo -p day-07
```

**Expected output:**
```
ContactCard {
    name: "Alice",
    email: "alice@example.com",
    age: 30,
}
Summary:  Alice (30) <alice@example.com>
Greeting: Hi, I'm Alice — reach me at alice@example.com
Adult:    true
---
ContactCard {
    name: "Bob",
    email: "bob@example.com",
    age: 16,
}
Summary:  Bob (16) <bob@example.com>
Greeting: Hi, I'm Bob — reach me at bob@example.com
Adult:    false
---
ContactCard {
    name: "Carol",
    email: "carol@example.com",
    age: 42,
}
Summary:  Carol (42) <carol@example.com>
Greeting: Hi, I'm Carol — reach me at carol@example.com
Adult:    true
---
```

### Extension Challenges

- **Easy:** Add a `pub mod utils;` to `src/lib.rs` with a `src/utils.rs` file containing `pub fn format_email(email: &str) -> String` that wraps the email in angle brackets: `"<alice@example.com>"`. Use it inside `ContactCard::summary()`. Import it in the example with `use day_07::utils::format_email;`.
- **Medium:** Add a second example file `examples/search.rs` that creates a `Vec<ContactCard>`, then writes `fn find_by_name<'a>(contacts: &'a [ContactCard], name: &str) -> Option<&'a ContactCard>` and demonstrates searching for an existing and a non-existing name using `match`.
- **Hard:** Add a `pub mod contact_book` module to the library with a `pub struct ContactBook` that holds a `Vec<ContactCard>` internally (private field). Implement `pub fn add(&mut self, c: ContactCard)`, `pub fn find(&self, name: &str) -> Option<&ContactCard>`, `pub fn all(&self) -> &[ContactCard]`, and `pub fn remove(&mut self, name: &str) -> bool`. Re-export `ContactBook` with `pub use contact_book::ContactBook;` in `src/lib.rs`. Write an example that exercises all four methods.
