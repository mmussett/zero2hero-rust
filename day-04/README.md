# Day 04: Ownership and Move Semantics

> **Project:** Memory Visualizer — a series of short functions that demonstrate moves, copies, clones, drops in nested scopes, and returning ownership; a custom `Drop` implementation makes every drop visible

## Learning Objectives

By the end of today you will be able to:
- State Rust's three ownership rules and explain why they exist
- Predict which types are moved vs. copied when assigned or passed to a function
- Use `clone()` deliberately to make an explicit deep copy of heap data
- Explain when a value is dropped and trace drop order in nested scopes
- Implement the `Drop` trait to observe cleanup behaviour

---

## Concepts

### 1. The Three Ownership Rules

> **Docs:** [The Rust Book — What is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) · [Rust by Example — Ownership and moves](https://doc.rust-lang.org/rust-by-example/scope/move.html) · [Rustonomicon — Ownership](https://doc.rust-lang.org/nomicon/ownership.html)

Every programming language must answer the question: "who is responsible for cleaning up memory?" Garbage-collected languages (Python, Java, Go) use a runtime that periodically scans memory and frees unused values. C and C++ require you to manually call `free`/`delete`, which is error-prone. Rust takes a third path: the compiler enforces a set of ownership rules at compile time, with no runtime cost and no manual memory management.

The three rules are:

1. **Each value in Rust has exactly one owner.** A value cannot belong to two variables simultaneously.
2. **When the owner goes out of scope, the value is dropped.** "Dropped" means the memory is freed and any cleanup code runs. This is automatic and deterministic.
3. **There can only be one owner at a time.** Ownership can be transferred (moved), but not duplicated (without an explicit clone).

These rules are checked at compile time. If your code violates them, it does not compile. This is the fundamental trade-off Rust makes: a steeper learning curve upfront in exchange for memory safety guarantees without a garbage collector.

Think of ownership like a room key. Only one person can hold the key at a time. When they leave the building (go out of scope), they return the key (the value is dropped). You can hand the key to someone else (move ownership), but then you no longer have it.

```rust
fn main() {
    // Rule 1: s1 is the sole owner of the String data
    let s1 = String::from("hello");

    // Rule 2: when this block ends, the String is dropped
    {
        let s2 = String::from("world");
        println!("{}", s2);
    } // s2 goes out of scope here — String "world" is dropped

    // Rule 3: s1 is still valid here because this block is still alive
    println!("{}", s1);
} // s1 goes out of scope here — String "hello" is dropped
```

#### Exercise 1.1 — Scope and Drop

**Goal:** Observe the exact moment values are dropped by tracing scope boundaries.

Read the code below and, before running it, write down on paper the order you expect the two `println!` calls to appear relative to each other. Then run the code and verify.

```rust
fn main() {
    let outer = String::from("outer");
    {
        let inner = String::from("inner");
        println!("Inside inner scope: {}", inner);
    } // What happens here?
    println!("Back in outer scope: {}", outer);
} // What happens here?
```

**Expected output:**
```
Inside inner scope: inner
Back in outer scope: outer
```

> **Hint:** The `inner` `String` is freed when the inner `{}` block ends — before the second `println!` runs. The `outer` `String` is freed when `main` returns.

---

### 2. Move Semantics

> **Docs:** [The Rust Book — What is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) · [Rust by Example — Ownership and moves](https://doc.rust-lang.org/rust-by-example/scope/move.html)

When you assign a `String` (or any heap-allocated type) to another variable, or pass it to a function, ownership is **moved**. The original variable becomes invalid — the compiler will refuse to compile any code that tries to use it after the move. This prevents the classic double-free bug: if two variables both owned the same heap memory, both would try to free it when they go out of scope, corrupting the heap.

This is the single most jarring difference between Rust and other languages. In Python or Java, `a = b` just makes another reference to the same object. In Rust, for heap types, it transfers ownership. The old variable is gone.

```rust
fn main() {
    let s1 = String::from("hello");

    // Move: s1's ownership transfers to s2
    let s2 = s1;

    // This line would be a compile error:
    // println!("{}", s1); // error[E0382]: borrow of moved value: `s1`

    println!("{}", s2); // s2 is the owner now, this is fine

    // Same thing happens when passing to a function
    let s3 = String::from("world");
    takes_ownership(s3); // s3 is moved into the function

    // This would also be a compile error:
    // println!("{}", s3); // s3 was moved
}

fn takes_ownership(some_string: String) {
    println!("Got: {}", some_string);
} // some_string is dropped here — memory freed
```

#### Exercise 2.1 — Find the Move Error

**Goal:** Trigger a move compile error, read the error message carefully, and fix it by returning ownership.

Write the code below, run `cargo run`, and read the full error message. Then fix it: change `print_string` to return the `String` back to the caller, and update `main` to capture the returned value.

```rust
fn print_string(s: String) {
    println!("{}", s);
    // After this function returns, s is dropped
}

fn main() {
    let my_string = String::from("moved!");
    print_string(my_string);
    println!("Still mine: {}", my_string); // Error: my_string was moved
}
```

**Expected output (after the fix):**
```
moved!
Still mine: moved!
```

> **Hint:** Change the return type of `print_string` to `-> String` and add `s` as the last line (implicit return). In `main`, write `let my_string = print_string(my_string);`.

---

### 3. The `Copy` Trait — When Moves Are Actually Copies

> **Docs:** [`Copy` trait](https://doc.rust-lang.org/std/marker/trait.Copy.html) · [The Rust Book — What is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)

Not every type is moved when assigned. Types that implement the `Copy` trait are silently **copied** instead. A copy means the bits are duplicated and both the original and the new binding are valid owners of independent values.

Types that implement `Copy` are always small and live entirely on the stack (no heap allocation):
- All integer types: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`, `isize`, `usize`
- Floating-point types: `f32`, `f64`
- `bool`
- `char`
- Tuples of `Copy` types: `(i32, f64)` is `Copy`, but `(i32, String)` is not

`String` is **not** `Copy` because it owns heap-allocated memory. Copying a `String` would require allocating new heap memory and copying the bytes — that is non-trivial work the compiler does not do silently. You must call `.clone()` explicitly (see next section).

```rust
fn main() {
    // i32 is Copy — both x and y are valid after this
    let x = 5;
    let y = x; // copy, not move
    println!("x = {}, y = {}", x, y); // both still valid

    // bool is Copy
    let flag = true;
    let also_flag = flag; // copy
    println!("{} and {}", flag, also_flag);

    // Passing Copy types to functions does not invalidate the original
    let n = 42;
    square(n);
    println!("n is still {}", n); // still valid because i32 is Copy
}

fn square(num: i32) -> i32 {
    num * num
}
```

#### Exercise 3.1 — Copy vs. Move

**Goal:** Identify which types are `Copy` and which are moved by writing a small test for each.

Write a program that assigns each of the following to a new variable, then uses **both** the original and the new variable after the assignment. The types that are `Copy` will compile; the moved types will not. For moved types, comment out the second use so the program compiles.

Types to test: `i32`, `f64`, `bool`, `char`, `String`.

**Expected output:**
```
i32: 10 and 10
f64: 3.14 and 3.14
bool: true and true
char: Z and Z
String (only one owner): hello
```

> **Hint:** `String` is not `Copy`, so you cannot print both `s` and `s2` after `let s2 = s;`. Comment out one of the two `println!` calls for the `String` test.

---

### 4. `clone()` — Explicit Deep Copy

> **Docs:** [`Clone` trait](https://doc.rust-lang.org/std/clone/trait.Clone.html) · [The Rust Book — What is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)

When you genuinely need two independent copies of heap data, use `.clone()`. This performs a deep copy: it allocates new heap memory and copies the content. Both the original and the clone are independent owners.

Use `clone()` sparingly. It is not wrong, but it has a performance cost proportional to the size of the data. In most real Rust code, borrowing (Day 5) is the right answer — you pass a reference to the data instead of cloning it. But for learning, for small data, or when you truly need two independent copies, `clone()` is the tool.

```rust
fn main() {
    let s1 = String::from("hello");

    // clone() creates a completely independent copy
    let s2 = s1.clone();

    // Both s1 and s2 are valid — they own separate heap allocations
    println!("s1 = {}, s2 = {}", s1, s2);

    // Modifying s2 does not affect s1
    let mut s3 = s1.clone();
    s3.push_str(", world");
    println!("s1 = {}, s3 = {}", s1, s3);
}
```

#### Exercise 4.1 — Clone to Pass and Keep

**Goal:** Use `.clone()` to pass a `String` to a function while keeping a usable copy.

Write a function `loudify(s: String) -> String` that takes ownership of a string and returns a new `String` with all characters uppercased (use `s.to_uppercase()`). In `main`, create a `String`, clone it before passing to `loudify`, and print both the original (lowercase) and the return value (uppercase).

```rust
fn loudify(s: String) -> String {
    s.to_uppercase()
}

fn main() {
    let original = String::from("hello, rust");
    let loud = loudify(original.clone());
    println!("original: {}", original);
    println!("loud: {}", loud);
}
```

**Expected output:**
```
original: hello, rust
loud: HELLO, RUST
```

> **Hint:** Without the `.clone()`, `original` would be moved into `loudify` and the final `println!` would fail to compile. Borrowing (tomorrow's topic) is the better solution — but for today, `clone()` is the lesson.

---

### 5. `Drop` — Observing Cleanup

> **Docs:** [`Drop` trait](https://doc.rust-lang.org/std/ops/trait.Drop.html) · [The Rust Book — What is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)

Rust calls a value's `drop` method automatically when it goes out of scope. Most of the time this is invisible — it just frees memory. But you can implement the `Drop` trait yourself to run custom cleanup code (close a file, release a lock, print a log message).

Implementing `Drop` is straightforward: add `impl Drop for YourType` and define `fn drop(&mut self)`. You cannot call `.drop()` manually — if you need to drop something early, use `std::mem::drop(value)` (the free function, not the trait method).

Understanding `Drop` is valuable because it makes scope-based cleanup tangible. Nested scopes drop in reverse order of construction (last in, first out — like a stack). This deterministic, predictable cleanup is one of Rust's most elegant features.

```rust
struct Tracked {
    name: String,
}

impl Drop for Tracked {
    fn drop(&mut self) {
        println!("Dropping: {}", self.name);
    }
}

fn main() {
    let _a = Tracked { name: String::from("A") };
    {
        let _b = Tracked { name: String::from("B") };
        let _c = Tracked { name: String::from("C") };
        println!("End of inner scope");
    } // B and C are dropped here, in reverse order: C then B
    println!("End of main");
} // A is dropped here
```

**Expected output:**
```
End of inner scope
Dropping: C
Dropping: B
End of main
Dropping: A
```

#### Exercise 5.1 — Drop Order

**Goal:** Predict drop order across nested scopes, then verify.

Write a program that creates `Tracked` values (using the struct above) named `"outer1"`, `"outer2"`, and inside a nested block, `"inner1"` and `"inner2"`. Add `println!` calls to mark the start and end of each scope. Before running, write down the exact output you expect. Then run and compare.

**Expected output:**
```
Start of main
Start of inner block
End of inner block
Dropping: inner2
Dropping: inner1
End of main
Dropping: outer2
Dropping: outer1
```

> **Hint:** Variables are dropped in reverse order of declaration within the same scope. Inner scopes are fully dropped before the outer scope ends.

---

## Day Project: Memory Visualizer

### What You're Building

A program that makes Rust's ownership system visible. Instead of abstract rules, you will write runnable code that shows exactly when values are moved, what happens when you try to use a moved value (the compile error, then the fix), how copies silently work, what clone costs you, and when drops occur. Each scenario is its own commented section in `main`.

This project is deliberately annotation-heavy. The goal is not just to run the code but to understand *why* each line does what it does. By the end, you will have a personal reference you can come back to whenever ownership confuses you.

### Requirements

1. Define a `Resource` struct with a `name: String` field. Implement `Drop` to print `"Dropping: {name}"` when dropped.
2. **Section A — Move and compile error:** Create a `Resource`, move it to a second variable, and show (as a commented-out line) what the compile error would look like. Continue using the second variable.
3. **Section B — Copy types:** Demonstrate that `i32`, `f64`, and `bool` are still usable after assignment to another variable.
4. **Section C — Clone:** Create a `Resource`, clone it, pass the clone to a function that takes ownership, and show that the original is still usable.
5. **Section D — Nested drops:** Create two `Resource` values in `main` and two more inside a nested block. Print `"--- entering inner scope ---"` before the block and `"--- leaving inner scope ---"` (put this *inside* the block, as the last statement). Observe the drop order in the output.
6. **Section E — Return ownership:** Write a function `make_resource(name: &str) -> Resource` that constructs and returns a `Resource`. Call it in `main` and show the returned value is usable.

### Getting Started

```rust
struct Resource {
    name: String,
}

impl Resource {
    fn new(name: &str) -> Resource {
        Resource { name: name.to_string() }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        println!("Dropping: {}", self.name);
    }
}

fn consume(r: Resource) {
    println!("Consuming: {}", r.name);
    // r is dropped at the end of this function
}

fn make_resource(name: &str) -> Resource {
    Resource::new(name) // ownership returned to caller
}

fn main() {
    // ---- Section A: Move ----
    println!("=== A: Move ===");
    let r1 = Resource::new("alpha");
    let r2 = r1; // r1 is moved into r2
    // println!("{}", r1.name); // COMPILE ERROR: r1 was moved
    println!("r2 is: {}", r2.name);

    // ---- Section B: Copy types ----
    println!("\n=== B: Copy ===");
    let x: i32 = 42;
    let y = x; // copied, not moved
    println!("x={}, y={}", x, y); // both valid

    // ---- Section C: Clone ----
    println!("\n=== C: Clone ===");
    let original = Resource::new("beta");
    let cloned = Resource { name: original.name.clone() };
    consume(cloned); // cloned is moved and dropped inside consume
    println!("original still valid: {}", original.name);

    // ---- Section D: Nested drops ----
    println!("\n=== D: Nested Drops ===");
    let _outer1 = Resource::new("outer1");
    let _outer2 = Resource::new("outer2");
    {
        let _inner1 = Resource::new("inner1");
        let _inner2 = Resource::new("inner2");
        println!("--- leaving inner scope ---");
    }
    println!("Back in main after inner scope");

    // ---- Section E: Return ownership ----
    println!("\n=== E: Return Ownership ===");
    let returned = make_resource("gamma");
    println!("Got back: {}", returned.name);

    println!("\n=== main ending ===");
}
```

### Running Your Solution

```bash
cargo run -p day-04
```

**Expected output:**
```
=== A: Move ===
r2 is: alpha
Dropping: alpha

=== B: Copy ===
x=42, y=42

=== C: Clone ===
Consuming: beta
Dropping: beta
original still valid: beta
Dropping: beta

=== D: Nested Drops ===
--- leaving inner scope ---
Dropping: inner2
Dropping: inner1
Back in main after inner scope

=== E: Return Ownership ===
Got back: gamma

=== main ending ===
Dropping: gamma
Dropping: outer2
Dropping: outer1
```

### Extension Challenges

- **Easy:** Add a `Section F` that calls `std::mem::drop(r)` on a `Resource` explicitly, before `main` ends. Observe that the drop message appears earlier than it would have naturally.
- **Medium:** Write a function `transfer_chain(r: Resource) -> Resource` that takes a `Resource`, prints it, modifies its name by appending `" (transferred)"`, and returns it. Call it three times in a chain: `let r = transfer_chain(transfer_chain(transfer_chain(Resource::new("chain"))));`. Trace the drops.
- **Hard:** Create a struct `Container` that holds a `Vec<Resource>` (a growable list, covered in Week 2). Implement `Drop` for `Container` to print `"Dropping container"`. Observe that when `Container` drops, each `Resource` inside it also drops — and which message appears first. This reveals that Rust drops the struct's `drop` implementation first, then drops its fields.
