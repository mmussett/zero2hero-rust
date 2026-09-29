# Day 02: Variables, Types, and Control Flow

> **Project:** CLI Calculator — hardcode two `f64` values and a `char` operator, use control flow to perform the right operation, and print the result with division-by-zero protection

## Learning Objectives

By the end of today you will be able to:
- Explain why Rust variables are immutable by default and when to use `let mut`
- Use shadowing to transform a variable without mutation
- Distinguish between Copy types (scalars) and Move types (heap-allocated), and predict what happens when you assign one variable to another
- Identify Rust's scalar types (`i32`, `i64`, `u32`, `f64`, `bool`, `char`) and annotate types explicitly
- Write `if` expressions (not just statements) that produce values
- Use `loop`, `while`, and `for` to repeat code, including `for` with integer ranges
- Explain the difference between a panic and a recoverable error, and use `unwrap()` and `expect()` appropriately

---

## Concepts

### 1. `let` and Immutability

> **Docs:** [The Rust Book — Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html) · [Rust by Example — Variable Bindings](https://doc.rust-lang.org/rust-by-example/variable_bindings.html)

In Rust, every variable you declare with `let` is **immutable by default** — you cannot change its value after you assign it. This is the opposite of most languages, where variables are mutable by default. If you try to reassign an immutable variable, Rust refuses to compile your code.

Why does Rust make this choice? Immutability makes code easier to reason about. When you see a variable name, you know its value never changes unless you explicitly opted into mutability. This also helps the compiler catch bugs: if you accidentally overwrite a value you meant to keep, Rust tells you at compile time rather than letting the bug hide until runtime.

**Shadowing** is different from mutation. You can use `let` again with the same name to create a brand-new binding that shadows the old one. The old value is not changed — a new variable is created that happens to share the name. Shadowing lets you reuse a name when you are transforming a value through a series of steps. Importantly, shadowing can also change the type, which `let mut` cannot do.

```rust
fn main() {
    // Immutable by default — this is fine
    let x = 5;
    println!("x is {}", x);

    // This would be a compile error:
    // x = 6; // error[E0384]: cannot assign twice to immutable variable `x`

    // Shadowing: a new `let` with the same name
    let x = x + 1; // x is now 6, but the old binding was not mutated
    println!("x is now {}", x);

    // Shadowing can even change the type
    let spaces = "   "; // &str
    let spaces = spaces.len(); // now usize — the old `spaces` is gone
    println!("Number of spaces: {}", spaces);
}
```

#### Exercise 1.1 — Immutability Compile Error

**Goal:** See the Rust compiler error for reassigning an immutable variable, then fix it with shadowing.

Write the program below exactly as shown (including the broken line), run `cargo run`, and read the error message carefully. Then fix it by changing `x = x * 2;` to `let x = x * 2;` (shadowing) and run again.

```rust
fn main() {
    let x = 10;
    x = x * 2; // This line should fail to compile
    println!("x is {}", x);
}
```

**Expected output (after the fix):**
```
x is 20
```

> **Hint:** The compiler error will say something like `cannot assign twice to immutable variable`. Shadowing with a new `let` is always the answer when you want to transform a value and reuse the name.

---

### 2. Ownership Preview — Why Assignment Might Move

> **Docs:** [The Rust Book — What is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) · [`Copy` trait](https://doc.rust-lang.org/std/marker/trait.Copy.html) · [`Clone` trait](https://doc.rust-lang.org/std/clone/trait.Clone.html)

You will explore ownership fully on Day 04, but you need a mental model *right now* because it affects the most basic thing you do: assign a variable to another variable.

In Rust, there are two kinds of values:

- **Copy types** — small, fixed-size values that live on the stack (`i32`, `f64`, `bool`, `char`, all the scalar types you learn today). When you assign one to another, Rust copies the bits. Both variables remain valid.
- **Move types** — heap-allocated values like `String`. When you assign one to another, Rust *moves* ownership. The original variable becomes invalid. This prevents double-free.

```rust
fn main() {
    // --- Copy types (all scalars) ---
    let x: i32 = 5;
    let y = x;          // x is COPIED — both x and y are valid
    println!("x={}, y={}", x, y);

    // --- Move types (heap-allocated) ---
    let s1 = String::from("hello");
    let s2 = s1;        // s1 is MOVED into s2 — s1 is no longer valid
    // println!("{}", s1); // compile error: value borrowed after move
    println!("{}", s2);

    // --- Clone if you want a real copy of a String ---
    let s3 = String::from("world");
    let s4 = s3.clone(); // deep copy — both are valid
    println!("{} and {}", s3, s4);
}
```

**Why does this matter today?** When you write `let` bindings for `String` values, remember that assignment moves. You will see this rule enforced by the compiler throughout the week. Day 04 explains the full ownership model and why it makes Rust memory-safe without a garbage collector.

> **Forward reference:** Day 05 introduces *borrowing* (`&`) — the mechanism for temporarily lending a value without moving it. Day 05 also introduces *lifetime annotations*, which the compiler uses to prove that borrowed references are always valid. Day 12 covers lifetimes in depth.

#### Exercise 2.0 — Copy vs Move

**Goal:** Confirm empirically that scalar types copy and `String` moves.

```rust
fn main() {
    let a: i32 = 100;
    let b = a;
    println!("a={}, b={}", a, b); // both valid — Copy

    let hello = String::from("hello");
    let world = hello; // hello is moved
    // Try uncommenting the next line and observe the compile error:
    // println!("{}", hello);
    println!("{}", world);
}
```

**Expected output:**
```
a=100, b=100
hello
```

> **Hint:** Uncomment `println!("{}", hello)` to see the compiler error. Read the error message — it will say "value borrowed here after move" and point to the line where the move happened.

---

### 3. `let mut` — Explicit Mutability

> **Docs:** [The Rust Book — Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)

When you genuinely need to change a variable's value over time — for example, a counter you increment in a loop — you declare it with `let mut`. The `mut` keyword is a signal to everyone reading the code (and to the compiler) that this binding will change.

Prefer immutability unless you have a specific reason for mutability. When you reach for `let mut`, ask yourself: do I really need to change this, or could I restructure the code to transform values instead? Keeping mutation local and intentional is a Rust best practice that pays dividends in larger programs.

```rust
fn main() {
    // Mutable variable — value can change after initial assignment
    let mut count = 0;
    println!("count starts at {}", count);

    count = count + 1;
    println!("count is now {}", count);

    count += 1; // Rust supports compound assignment operators
    println!("count is now {}", count);

    // Contrast: immutable by default is fine when the value never changes
    let max = 100;
    println!("max is always {}", max);
}
```

#### Exercise 2.1 — Temperature Converter

**Goal:** Practice `let mut` by accumulating a result through several steps.

Write a program that starts with a temperature of `100.0` degrees Celsius as a mutable `f64`, converts it to Fahrenheit by first multiplying by `9.0 / 5.0` and then adding `32.0` (two separate mutation steps), and prints the result.

```rust
fn main() {
    let mut temp_c = 100.0_f64;
    // Step 1: multiply by 9.0 / 5.0
    // Step 2: add 32.0
    // Print the result
}
```

**Expected output:**
```
100°C is 212°F
```

> **Hint:** In Rust, dividing `9 / 5` (both integers) gives `1`, not `1.8`. Use `9.0 / 5.0` or use `f64` literals like `9.0_f64 / 5.0`.

#### `const` — Compile-Time Constants

`const` declares a **compile-time constant**: a value fixed for the lifetime of the program and baked directly into the compiled binary. Unlike `let`, a `const` requires an explicit type annotation and its value must be computable at compile time (no runtime function calls, no heap allocation). Constants can be declared at any scope — including module level, outside `main` — and are conventionally named in `SCREAMING_SNAKE_CASE`.

```rust
const MAX_CONNECTIONS: u32 = 100;
const PI: f64 = 3.14159265358979;
const APP_NAME: &str = "Zero to Hero";

fn main() {
    println!("App: {}", APP_NAME);
    println!("Max connections: {}", MAX_CONNECTIONS);

    // const can be used in array lengths — let cannot
    let buffer = [0u8; MAX_CONNECTIONS as usize];
    println!("Buffer length: {}", buffer.len());
}
```

**`const` vs `let`:** `let` creates a runtime binding; `const` is substituted at compile time. Use `const` for fundamental, named facts about your program: maximum sizes, mathematical constants, fixed configuration values.

---

### 4. Scalar Types

> **Docs:** [The Rust Book — Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html) · [`i32`](https://doc.rust-lang.org/std/primitive.i32.html) · [`f64`](https://doc.rust-lang.org/std/primitive.f64.html) · [`bool`](https://doc.rust-lang.org/std/primitive.bool.html) · [`char`](https://doc.rust-lang.org/std/primitive.char.html)

Rust is **statically typed** — every variable has a type that is known at compile time. The compiler is usually smart enough to infer the type from context, but you can always annotate it explicitly with `: Type` after the variable name. Explicit annotations are especially useful for clarity and when inference cannot determine the type.

Rust's primitive scalar types are:

| Type | Description | Example |
|------|-------------|---------|
| `i32` | 32-bit signed integer (default for whole numbers) | `let x: i32 = -42;` |
| `i64` | 64-bit signed integer | `let big: i64 = 9_000_000_000;` |
| `u32` | 32-bit unsigned integer (no negatives) | `let count: u32 = 100;` |
| `u64` | 64-bit unsigned integer | `let size: u64 = 1_000_000;` |
| `f64` | 64-bit floating-point (default for decimals) | `let pi: f64 = 3.14159;` |
| `f32` | 32-bit floating-point | `let x: f32 = 1.5;` |
| `bool` | Boolean | `let flag: bool = true;` |
| `char` | A single Unicode character (4 bytes, single quotes) | `let c: char = 'Z';` |

**Integer overflow** behaves differently in debug and release builds. In debug mode (the default with `cargo run`), integer overflow causes a **panic** (a controlled crash) so bugs surface immediately. In release mode (`cargo build --release`), Rust wraps around using two's complement arithmetic — `255u8 + 1` becomes `0`. Never rely on wrapping in production code; use the explicit `wrapping_add`, `checked_add`, or `saturating_add` methods instead.

```rust
fn main() {
    // Type inference — Rust figures out the type
    let score = 100; // i32
    let price = 9.99; // f64
    let passed = true; // bool
    let grade = 'A'; // char

    // Explicit type annotations
    let population: u64 = 8_000_000_000; // underscores improve readability
    let temperature: f64 = -12.5;
    let flag: bool = false;

    println!("Score: {}", score);
    println!("Price: ${}", price);
    println!("Passed: {}", passed);
    println!("Grade: {}", grade);
    println!("World population: {}", population);
    println!("Temperature: {}°C", temperature);
    println!("Flag: {}", flag);
}
```

#### Exercise 3.1 — Type Annotations

**Goal:** Practice writing explicit type annotations and understanding type inference.

Declare five variables using explicit `: Type` annotations: an `i32` with a negative value, a `u32` with a value over 1000, an `f64` with a decimal, a `bool`, and a `char`. Print all five on separate lines using `println!`.

**Expected output:**
```
i32: -42
u32: 1234
f64: 3.14
bool: true
char: R
```

> **Hint:** Characters in Rust use single quotes (`'R'`), not double quotes. Double quotes are for string literals (`"hello"`), which have type `&str`.

#### Type Casting with `as`

Rust never implicitly converts between numeric types — adding an `i32` to an `f64` is a compile error. When you need to convert, use the `as` keyword for an **explicit cast**. `as` is defined for all numeric-to-numeric and `char`↔`u32` conversions.

```rust
fn main() {
    let x: i32 = 42;
    let y: f64 = x as f64;         // i32 → f64: lossless
    println!("{} as f64 = {}", x, y);

    let pi: f64 = 3.99;
    let truncated: i32 = pi as i32; // truncates toward zero → 3 (not rounded)
    println!("{} as i32 = {}", pi, truncated);

    let big: i32 = 300;
    let small: u8 = big as u8;      // wraps: 300 % 256 = 44
    println!("{} as u8 = {}", big, small);

    // usize is the type used for indexing — you'll cast to/from it constantly
    let len: usize = 10;
    let offset: i32 = len as i32 - 3;
    println!("offset: {}", offset);
}
```

**When to use `as`:** Numeric conversions, casting between `usize` and integer types for indexing, and `char`↔`u32`. For conversions that might overflow and should return an error instead of wrapping, use `.try_into()` from the `TryInto` trait.

---

### 5. `if` Expressions

> **Docs:** [The Rust Book — Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html) · [Reference — if expressions](https://doc.rust-lang.org/reference/expressions/if-expr.html)

Rust's `if` works like you expect from other languages, with one important difference: **`if` is an expression**, not just a statement. This means an `if` block can produce a value, and you can use that value to initialise a `let` binding. Both branches must produce the same type.

This is powerful because it eliminates the need for a mutable variable and a later assignment — you can express "set x to one of two values based on a condition" in a single, readable line. The condition in `if` must be a `bool`; unlike C, Rust does not treat `0` as false or non-zero as true.

```rust
fn main() {
    let temperature = 23;

    // Classic if/else if/else — works like any language
    if temperature > 30 {
        println!("It's hot outside!");
    } else if temperature > 15 {
        println!("It's pleasant outside.");
    } else {
        println!("Bring a jacket.");
    }

    // if as an expression — both arms return a &str value
    let description = if temperature > 20 { "warm" } else { "cool" };
    println!("The weather is {}.", description);

    // Note: no semicolons on the last expression inside a block
    // when using if as an expression that produces a value.

    // This would be a compile error — mismatched types:
    // let x = if true { 5 } else { "hello" };
}
```

#### Exercise 4.1 — Grade Classifier

**Goal:** Use `if` as an expression to assign a letter grade.

Write a program with an `i32` variable called `score` set to any value between 0 and 100. Use `if`/`else if`/`else` as an **expression** to assign a `&str` letter grade to a `let` variable (`"A"` for 90–100, `"B"` for 80–89, `"C"` for 70–79, `"D"` for 60–69, `"F"` for below 60). Print the score and grade together.

**Expected output (if score is 85):**
```
Score: 85 → Grade: B
```

> **Hint:** To use `if` as an expression with more than two branches, chain `else if` and make sure the final `else` arm is present — the compiler requires all possible cases to be covered.

---

### 6. `loop` — Infinite Loops with `break`

> **Docs:** [The Rust Book — Repetition with Loops](https://doc.rust-lang.org/book/ch03-05-control-flow.html#repetition-with-loops) · [Reference — Loop expressions](https://doc.rust-lang.org/reference/expressions/loop-expr.html)

`loop` creates an infinite loop that runs until a `break` statement exits it. Like `if`, `loop` is an expression: a `break` can carry a value out of the loop, which the loop expression evaluates to. This is handy for retry logic and event loops.

Think of `loop` as "keep doing this until I explicitly say stop." It is most useful when the exit condition is in the middle of the loop body, or when the loop itself computes the value you need.

```rust
fn main() {
    // Simple loop with break
    let mut counter = 0;
    loop {
        counter += 1;
        if counter == 3 {
            break;
        }
    }
    println!("Broke out after {} iterations", counter);

    // loop as an expression — break returns a value
    let mut attempts = 0;
    let result = loop {
        attempts += 1;
        if attempts == 5 {
            break attempts * 10; // this value becomes the result of the loop
        }
    };
    println!("Loop result: {} after {} attempts", result, attempts);
}
```

#### Exercise 5.1 — Countdown

**Goal:** Use `loop` and `break` to count down from 10 to 1.

Write a program with a mutable `i32` variable starting at `10`. Use `loop` to print the current value, decrement it by 1, and break when it reaches `0`. After the loop, print `"Liftoff!"`.

**Expected output:**
```
10
9
8
7
6
5
4
3
2
1
Liftoff!
```

> **Hint:** Make sure you print the value before decrementing, otherwise your countdown will start at 9.

#### `continue` — Skip to the Next Iteration

`break` exits a loop entirely; `continue` skips the rest of the *current* iteration and jumps straight to the next one. Use `continue` to filter out unwanted cases without deeply nesting your logic.

```rust
fn main() {
    // Print only odd numbers from 1 to 10
    for i in 1..=10 {
        if i % 2 == 0 {
            continue; // skip even — go straight to i+1
        }
        println!("{}", i); // only reached for odd i
    }

    // continue in a loop: skip multiples of 3
    let mut n = 0;
    loop {
        n += 1;
        if n > 9 { break; }
        if n % 3 == 0 { continue; }
        print!("{} ", n);
    }
    println!(); // 1 2 4 5 7 8
}
```

`continue` works in `loop`, `while`, and `for` alike. In nested loops it targets the *innermost* loop. To skip an outer loop's iteration from inside an inner one, use a **loop label**: `'outer: for ... { for ... { continue 'outer; } }`.

---

### 7. `while` — Condition-Based Loops

> **Docs:** [The Rust Book — Conditional Loops with while](https://doc.rust-lang.org/book/ch03-05-control-flow.html#conditional-loops-with-while) · [Reference — Predicate loops](https://doc.rust-lang.org/reference/expressions/loop-expr.html#predicate-loops)

`while` is the straightforward condition loop: it checks a `bool` expression before each iteration and stops when the condition is false. It is the right tool when you know the loop should run as long as some condition holds.

```rust
fn main() {
    let mut n = 1;

    // Double n as long as it is less than 100
    while n < 100 {
        n *= 2;
        println!("n is now {}", n);
    }

    println!("Final value: {}", n);
}
```

#### Exercise 6.1 — Fizz Counter

**Goal:** Use `while` with `if`/`else` to print FizzBuzz from 1 to 15.

Write a program with a mutable counter starting at `1`. Use a `while` loop that runs while the counter is `<= 15`. Inside the loop: print `"Fizz"` if the counter is divisible by 3, `"Buzz"` if divisible by 5, `"FizzBuzz"` if divisible by both, and the number itself otherwise. Increment the counter at the end of each iteration.

**Expected output:**
```
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
```

> **Hint:** The modulo operator in Rust is `%`. Check for divisibility by both 3 and 5 first (before checking either individually), or the individual checks will trigger for numbers like 15.

---

### 8. `for` with Ranges

> **Docs:** [The Rust Book — Looping with for](https://doc.rust-lang.org/book/ch03-05-control-flow.html#looping-through-a-collection-with-for) · [`std::ops::Range`](https://doc.rust-lang.org/std/ops/struct.Range.html)

`for` loops in Rust iterate over anything that implements the `Iterator` trait — but the most common case for beginners is iterating over a numeric range. Ranges come in two flavours: `1..5` is exclusive (1, 2, 3, 4) and `1..=5` is inclusive (1, 2, 3, 4, 5). Prefer ranges to `while` with a counter when the range is what you care about.

You can also iterate over collections using `for item in &collection`. The `&` borrows the collection so you do not take ownership — we will explain exactly why on Days 4 and 5. For now, just remember the `&`.

```rust
fn main() {
    // Exclusive range: 0, 1, 2, 3, 4
    for i in 0..5 {
        println!("i = {}", i);
    }

    // Inclusive range: 1, 2, 3, 4, 5
    println!("--- inclusive ---");
    for i in 1..=5 {
        println!("i = {}", i);
    }

    // Iterate over an array
    let fruits = ["apple", "banana", "cherry"];
    for fruit in &fruits {
        println!("I like {}", fruit);
    }

    // rev() reverses a range
    for i in (1..=3).rev() {
        println!("Countdown: {}", i);
    }
}
```

#### Exercise 7.1 — Multiplication Table

**Goal:** Use nested `for` loops with ranges to print a multiplication table.

Print a 5x5 multiplication table. Each row should show the products for that row number multiplied by 1 through 5. Use two nested `for` loops with inclusive ranges.

**Expected output:**
```
1  2  3  4  5  
2  4  6  8  10 
3  6  9  12 15 
4  8  12 16 20 
5  10 15 20 25 
```

> **Hint:** Use `print!` (without the newline) inside the inner loop, and `println!()` (empty call, just a newline) after the inner loop finishes each row.

---

### 9. Panics and Basic Error Handling

> **Docs:** [The Rust Book — Unrecoverable Errors with panic!](https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html) · [`panic!`](https://doc.rust-lang.org/std/macro.panic.html) · [`unwrap`](https://doc.rust-lang.org/std/result/enum.Result.html#method.unwrap) · [`expect`](https://doc.rust-lang.org/std/result/enum.Result.html#method.expect)

Rust programs can fail in two ways: **panics** (unrecoverable, crash the thread) and **recoverable errors** (represented as values). Understanding the difference from day one shapes how you write Rust.

**`panic!`** immediately terminates the current thread with a message and a backtrace. It is for *bugs* — situations that should never happen in correct code. Indexing a `Vec` out of bounds, unwrapping a `None`, dividing an integer by zero at runtime: these all panic.

```rust
fn main() {
    // Integer division by zero is a panic (not a compile error)
    let x: i32 = 10;
    let y: i32 = 0;
    // let result = x / y; // uncomment to see: thread panicked 'attempt to divide by zero'

    // Explicit panic with a message
    // panic!("something went very wrong");

    // Indexing out of bounds
    let v = vec![1, 2, 3];
    // let _ = v[5]; // panics: index out of bounds
    println!("v has {} elements", v.len());
}
```

**`unwrap()` and `expect()`** extract the value from an `Option` or `Result` — or panic if there is none. `expect("message")` panics with your custom message, making the failure easier to diagnose.

```rust
fn main() {
    // unwrap on Some — fine
    let name: Option<&str> = Some("Alice");
    println!("{}", name.unwrap()); // "Alice"

    // unwrap on None — panics
    // let empty: Option<&str> = None;
    // println!("{}", empty.unwrap()); // thread panicked 'called `Option::unwrap()` on a `None` value'

    // expect gives a better panic message
    let score: Option<u32> = None;
    // score.expect("score must be present before calling this function"); // panics with your message

    // Parsing a string into a number: returns Result, unwrap extracts the value
    let n: i32 = "42".parse().unwrap();
    println!("Parsed: {}", n);

    // This would panic — "abc" is not a valid i32
    // let bad: i32 = "abc".parse().unwrap();
}
```

**When is `unwrap()` acceptable?** In short scripts, tests, and examples where you know the value exists and a panic is an acceptable failure mode. In production code, prefer proper error handling (Day 09 covers this fully). Think of `unwrap()` as a placeholder: "I know this won't fail *right now*, but I'll come back and handle it properly."

#### Exercise 8.1 — Safe Division

**Goal:** Write a `divide` function that returns `0.0` instead of panicking when the divisor is zero.

```rust
fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        println!("Warning: division by zero, returning 0.0");
        0.0
    } else {
        a / b
    }
}

fn main() {
    println!("{}", divide(10.0, 2.0));
    println!("{}", divide(10.0, 0.0));
}
```

**Expected output:**
```
5
Warning: division by zero, returning 0.0
0
```

> **Hint:** This is a preview of what `Result` formalises — Day 03 introduces `Result<T, E>` as the proper way to express "this function can fail."

---

> **Deep dive:** The scalar types covered today (`i32`, `f64`, `bool`, `char`) are just a few of Rust's primitives. See [`PRIMITIVES.md`](../PRIMITIVES.md) at the repo root for full coverage of all integer widths, float constants, string types, arrays, tuples, and unit — including bit representations, ranges, associated functions, and methods with examples.

---

## Day Project: CLI Calculator

### What You're Building

A calculator that operates on two hardcoded `f64` values using a hardcoded `char` operator. It does not (yet) read user input — that comes in a later day when you learn about standard input. The point of today's project is to exercise control flow: using `if`/`else` (or a sneak-peek at `match`) to dispatch to the right arithmetic operation, and handling the edge case of division by zero. By the end you will have a program that can add, subtract, multiply, and divide any two numbers you hardcode into it.

Division by zero is the interesting edge case. In floating-point arithmetic, `x / 0.0` in Rust produces `f64::INFINITY` rather than panicking, but we want to detect it explicitly and print an error message instead. This is your first encounter with a conditional that guards against an invalid operation.

### Requirements

1. Declare two `f64` variables: `a` and `b` (set them to any values you like).
2. Declare a `char` variable `op` set to one of `'+'`, `'-'`, `'*'`, or `'/'`.
3. Use `if`/`else if`/`else` to check the operator and compute the result.
4. For division, check whether `b == 0.0` before dividing. If it is zero, print an error message instead of a result.
5. Print the full expression and result in the format `a op b = result`.
6. Use explicit `f64` type annotations on `a` and `b`.

### Getting Started

```rust
fn main() {
    // Change these values to test different operations
    let a: f64 = 12.0;
    let b: f64 = 4.0;
    let op: char = '/';

    // Compute the result based on the operator
    if op == '+' {
        println!("{} + {} = {}", a, b, a + b);
    } else if op == '-' {
        println!("{} - {} = {}", a, b, a - b);
    } else if op == '*' {
        println!("{} * {} = {}", a, b, a * b);
    } else if op == '/' {
        if b == 0.0 {
            println!("Error: division by zero");
        } else {
            println!("{} / {} = {}", a, b, a / b);
        }
    } else {
        println!("Unknown operator: {}", op);
    }
}
```

Try changing `a`, `b`, and `op` to verify each branch works correctly.

### Running Your Solution

```bash
cargo run -p day-02
```

With `a = 12.0`, `b = 4.0`, `op = '/'`:
```
12 / 4 = 3
```

With `a = 7.0`, `b = 3.0`, `op = '+'`:
```
7 + 3 = 10
```

With `b = 0.0`, `op = '/'`:
```
Error: division by zero
```

### Extension Challenges

- **Easy:** Add support for `'%'` (modulo/remainder) using the `%` operator. Handle the case where `b == 0.0` for modulo too.
- **Medium:** Use a `for` loop over an array of `(f64, f64, char)` tuples to run five calculations in sequence and print each result, so you can test all operators without manually changing variables.
- **Hard:** Refactor your operator dispatch using a `match` expression on `op` instead of `if`/`else if`. Look up how `match` works on `char` values — a sneak peek at what you will learn fully on Day 6. Combine the result into a single `let result: f64` using `match` as an expression, then print it once after the `match`.
