# Day 09: Option & Result and the ? Operator

> **Project:** CSV Row Parser — parse a hardcoded CSV string into typed structs, handling missing fields and bad data gracefully.

## Learning Objectives

By the end of today you will be able to:
- Use `Option<T>` combinators (`map`, `and_then`, `unwrap_or`, `unwrap_or_else`) instead of manual `match` everywhere
- Return and propagate `Result<T, E>` from functions that can fail
- Use the `?` operator to eliminate boilerplate error-propagation code
- Chain fallible function calls up a call stack with early returns
- Write a `main` that returns `Result<(), Box<dyn std::error::Error>>`

---

## Concepts

### 1. `Option<T>` in Depth

> **Docs:** [Option](https://doc.rust-lang.org/std/option/enum.Option.html) · [Rust by Example — Option](https://doc.rust-lang.org/rust-by-example/error/option_unwrap.html)

You met `Option` on Day 6 as a pattern-matching exercise. Today you learn to work with it fluently. `Option<T>` is the Rust way of saying "this value might not exist" — a zero-cost alternative to null pointers.

**When is `unwrap()` acceptable?** In tests, in short scripts where a panic is appropriate, or when you have already proved (by prior logic) that the value is `Some`. In production library code, avoid it. Use `expect("descriptive message")` over `unwrap()` when you do reach for a panic — the message surfaces in crash reports.

The combinators let you transform `Option` values without unwrapping them. `map` applies a function to the inner value and re-wraps the result — it does nothing on `None`. `and_then` (also called `flatMap` in other languages) applies a function that itself returns an `Option`, flattening the result so you do not end up with `Option<Option<T>>`. `unwrap_or` and `unwrap_or_else` provide fallback values for the `None` case.

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 42 {
        Some("Alice".to_string())
    } else {
        None
    }
}

fn greet(id: u32) -> Option<String> {
    // and_then chains two Option-returning operations
    find_user(id).map(|name| format!("Hello, {name}!"))
}

fn main() {
    // unwrap_or — default if None
    let user = find_user(99).unwrap_or("unknown".to_string());
    println!("{user}"); // unknown

    // unwrap_or_else — compute the default lazily
    let user = find_user(99).unwrap_or_else(|| "guest_user".to_string());
    println!("{user}"); // guest_user

    // map — transform the inner value
    let length: Option<usize> = find_user(42).map(|name| name.len());
    println!("{:?}", length); // Some(5)

    // and_then — chain Option-returning functions
    println!("{:?}", greet(42));  // Some("Hello, Alice!")
    println!("{:?}", greet(99));  // None

    // ? operator inside a function returning Option
    let greeting = greet(42)?;   // would early-return None if greet returned None
    // (this line is inside a function returning Option — see below)
    println!("{greeting}");
}
```

The `?` operator works on `Option` too when the surrounding function also returns `Option`. If the value is `None`, the function returns `None` immediately. If it is `Some(v)`, execution continues with `v`.

```rust
fn first_char_of_user_name(id: u32) -> Option<char> {
    let name = find_user(id)?;   // returns None if find_user returns None
    name.chars().next()          // Option<char>
}

fn main() {
    println!("{:?}", first_char_of_user_name(42)); // Some('A')
    println!("{:?}", first_char_of_user_name(99)); // None
}

fn find_user(id: u32) -> Option<String> {
    if id == 42 { Some("Alice".to_string()) } else { None }
}
```

#### Exercise 1.1 — Option Combinator Chain

**Goal:** Replace nested `match` with combinator calls.

The function below uses nested `match`. Rewrite `describe_length` using only `map` and `unwrap_or` — no `match`, no `if let`.

```rust
fn get_word(index: usize) -> Option<&'static str> {
    let words = ["hello", "world", "rust"];
    words.get(index).copied()
}

// Rewrite this without match
fn describe_length(index: usize) -> String {
    match get_word(index) {
        Some(w) => format!("'{}' has {} letters", w, w.len()),
        None    => "no word at that index".to_string(),
    }
}

fn main() {
    println!("{}", describe_length(1)); // 'world' has 5 letters
    println!("{}", describe_length(9)); // no word at that index
}
```

**Expected output:**
```
'world' has 5 letters
no word at that index
```

> **Hint:** `get_word(index).map(|w| format!(...)).unwrap_or(...)` is the pattern you want.

---

### 2. `Result<T, E>` — Typed, Recoverable Errors

> **Docs:** [Result](https://doc.rust-lang.org/std/result/enum.Result.html) · [Book — Result](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html) · [Rust by Example — Result](https://doc.rust-lang.org/rust-by-example/error/result.html)

`Result<T, E>` models operations that can fail with a specific error type. `Ok(value)` carries the success value; `Err(error)` carries information about what went wrong. Unlike exceptions in other languages, errors in Rust are values — they are always visible in the function signature, and the compiler forces you to handle them.

The same combinators you learned for `Option` work on `Result`: `map` transforms the `Ok` value, `map_err` transforms the `Err` value (useful for converting error types), and `and_then` chains fallible operations. Converting between the two types: `.ok()` turns `Result<T, E>` into `Option<T>` (discarding the error), and `.err()` turns it into `Option<E>` (discarding the success value).

```rust
use std::num::ParseIntError;

fn parse_positive(s: &str) -> Result<u32, String> {
    // str::parse returns Result<T, ParseIntError>
    let n: i32 = s.parse().map_err(|e: ParseIntError| e.to_string())?;
    if n < 0 {
        Err(format!("{n} is negative"))
    } else {
        Ok(n as u32)
    }
}

fn double_positive(s: &str) -> Result<u32, String> {
    // and_then chains another fallible step
    parse_positive(s).and_then(|n| {
        n.checked_mul(2).ok_or_else(|| "overflow".to_string())
    })
}

fn main() {
    println!("{:?}", parse_positive("42"));   // Ok(42)
    println!("{:?}", parse_positive("-5"));   // Err("-5 is negative")
    println!("{:?}", parse_positive("abc"));  // Err("invalid digit found in string")

    // map — transform the Ok value
    let doubled: Result<String, String> = parse_positive("7")
        .map(|n| format!("Result: {n}"));
    println!("{:?}", doubled); // Ok("Result: 7")

    // Converting to Option
    let opt: Option<u32> = parse_positive("10").ok();
    println!("{:?}", opt); // Some(10)
}
```

#### Exercise 2.1 — Safe Division

**Goal:** Return a `Result` from a function that can fail, then handle both cases in `main`.

Write `fn divide(a: f64, b: f64) -> Result<f64, String>` that returns `Err("division by zero".to_string())` when `b` is `0.0`, and `Ok(a / b)` otherwise. Call it from `main` for several inputs.

**Expected output:**
```
10 / 2 = 5
0 / 0: Error: division by zero
```

> **Hint:** Use `if b == 0.0` to guard. `f64` comparison with `==` is fine here because we are checking for literal zero.

---

### 3. The `?` Operator

> **Docs:** [Book — Error handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) · [? operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator)

The `?` operator is syntactic sugar that expands into a `match` with an early `return`. Written manually, returning a `Result` from a fallible call looks like:

```rust
// Desugared — what ? actually does
let value = match some_result {
    Ok(v)  => v,
    Err(e) => return Err(e.into()),
};

// The same thing with ?
let value = some_result?;
```

The `.into()` call is important: `?` automatically converts the error type using `From`, so you can mix functions that return different error types as long as your function's error type implements `From<TheOtherError>`. The common escape hatch is `Box<dyn std::error::Error>`, which any standard error can be converted into.

`?` can only be used in a function whose return type is `Result` (or `Option`). Using it in `main` requires `fn main() -> Result<(), Box<dyn std::error::Error>>`.

```rust
use std::num::ParseIntError;

fn parse_and_add(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x: i32 = a.parse()?;  // early return Err if parse fails
    let y: i32 = b.parse()?;  // same for b
    Ok(x + y)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sum = parse_and_add("10", "32")?;
    println!("Sum: {sum}"); // Sum: 42

    let bad = parse_and_add("10", "abc");
    println!("Bad parse: {bad:?}"); // Err(...)

    Ok(())
}
```

#### Exercise 3.1 — Chained Parsing

**Goal:** Practice `?` in a chain of fallible operations.

Write a function `fn age_in_months(s: &str) -> Result<u32, String>` that parses a string as a `u32` age in years and returns the equivalent in months. Use `?` to propagate the parse error, mapping it to `String` with `.map_err(|e| e.to_string())`.

**Expected output:**
```
25 years = 300 months
Error: invalid digit found in string
```

> **Hint:** Call `s.parse::<u32>().map_err(|e| e.to_string())?` to get the parsed number, then multiply by 12.

---

### 4. Propagating Errors up the Call Stack

> **Docs:** [Error trait](https://doc.rust-lang.org/std/error/trait.Error.html) · [From trait](https://doc.rust-lang.org/std/convert/trait.From.html)

Real programs have layers: a low-level function parses bytes, a mid-level function builds a struct, a high-level function orchestrates the workflow. With `?`, each layer simply propagates errors upward without needing to know the details — the outermost caller decides what to do.

```rust
use std::error::Error;

fn read_temperature(raw: &str) -> Result<f64, String> {
    raw.trim()
       .parse::<f64>()
       .map_err(|e| format!("bad temperature '{}': {}", raw.trim(), e))
}

fn validate_temperature(temp: f64) -> Result<f64, String> {
    if temp < -273.15 {
        Err(format!("{temp} is below absolute zero"))
    } else {
        Ok(temp)
    }
}

fn process_reading(raw: &str) -> Result<f64, String> {
    let temp = read_temperature(raw)?;      // propagate parse error
    let valid = validate_temperature(temp)?; // propagate range error
    Ok(valid)
}

fn main() -> Result<(), Box<dyn Error>> {
    let readings = ["22.5", "-300.0", "not_a_number", "36.6"];
    for raw in readings {
        match process_reading(raw) {
            Ok(t)  => println!("Valid reading: {t:.1}°C"),
            Err(e) => println!("Error — {e}"),
        }
    }
    Ok(())
}
```

**Expected output:**
```
Valid reading: 22.5°C
Error — -300 is below absolute zero
Error — bad temperature 'not_a_number': invalid float literal
Valid reading: 36.6°C
```

---

### 5. `type` — Type Aliases

> **Docs:** [type aliases](https://doc.rust-lang.org/reference/items/type-aliases.html)

`type` creates an **alias** for an existing type — a new name that refers to the same underlying type. Aliases don't create new types; they are purely a readability tool.

The most important use in error-handling code is shortening `Result<T, MyError>` to just `Result<T>`:

```rust
use std::num::ParseIntError;

// Without alias — repetitive
fn parse_a(s: &str) -> Result<i32, ParseIntError> { s.parse() }
fn parse_b(s: &str) -> Result<i32, ParseIntError> { s.parse() }

// With alias — DRY and readable
type ParseResult<T> = Result<T, ParseIntError>;

fn parse_c(s: &str) -> ParseResult<i32> { s.parse() }
fn parse_d(s: &str) -> ParseResult<i32> { s.parse() }
```

The stdlib uses this heavily: `std::io::Result<T>` is an alias for `Result<T, std::io::Error>`, and `std::fmt::Result` is an alias for `Result<(), std::fmt::Error>`.

A second use is simplifying long generic types you repeat throughout a module:

```rust
use std::collections::HashMap;

type Registry = HashMap<String, Vec<u64>>;

fn new_registry() -> Registry {
    HashMap::new()
}
```

`type` aliases are also used with traits to name associated types (you'll see this in the `Iterator` trait: `type Item`). The alias does not enforce anything — it is purely cosmetic. If you want a truly distinct type that the compiler treats as separate, use a **newtype** (a one-field struct) instead.

#### Exercise 5.1 — Simplify a Function Signature

**Goal:** Use `type` to remove repetition from a module that has many functions returning the same `Result` variant.

Define `type AppResult<T> = Result<T, String>`. Rewrite these three function signatures to use it: `fn load(path: &str) -> Result<String, String>`, `fn validate(s: &str) -> Result<bool, String>`, `fn process(n: u32) -> Result<u32, String>`. Each function can just return `Ok(...)` with a placeholder value. Confirm the code compiles.

**Expected output:**
```
Loaded: placeholder
Valid: true
Processed: 42
```

> **Hint:** After defining the alias, replace `Result<_, String>` with `AppResult<_>` in all three signatures. The implementations don't change at all.

---

## Day Project: CSV Row Parser

### What You're Building

You will parse a hardcoded multi-line CSV string into `Student` structs. Each row has three fields: name, age, and score. Some rows have intentional problems (missing score, bad age). You will use `Result` and `?` to parse fields, and return `Vec<Result<Student, String>>` so callers can process successes and errors separately.

### Requirements

1. Define `struct Student { name: String, age: u32, score: f64 }`.
2. Write `fn parse_row(line: &str) -> Result<Student, String>` that splits on `,`, trims whitespace, and uses `?` to propagate parse errors, building a descriptive `Err` message with `map_err`.
3. Handle an empty score field as `score: 0.0` (use `Option` internally — if the field is empty or missing, default to `0.0` using `.unwrap_or(0.0)` on the parsed result).
4. Write `fn parse_csv(csv: &str) -> Vec<Result<Student, String>>` that skips the header row and maps each subsequent line through `parse_row`.
5. In `main`, collect results, print successful rows, and print all errors separately.

### Getting Started

```rust
struct Student {
    name: String,
    age: u32,
    score: f64,
}

fn parse_row(line: &str) -> Result<Student, String> {
    let parts: Vec<&str> = line.splitn(3, ',').collect();
    if parts.len() < 2 {
        return Err(format!("too few columns: '{line}'"));
    }
    let name = parts[0].trim().to_string();
    if name.is_empty() {
        return Err("name is empty".to_string());
    }
    let age: u32 = parts[1]
        .trim()
        .parse()
        .map_err(|e| format!("bad age in '{line}': {e}"))?;
    let score: f64 = if parts.len() == 3 && !parts[2].trim().is_empty() {
        parts[2]
            .trim()
            .parse()
            .map_err(|e| format!("bad score in '{line}': {e}"))?
    } else {
        0.0
    };
    Ok(Student { name, age, score })
}

fn parse_csv(csv: &str) -> Vec<Result<Student, String>> {
    csv.lines()
        .skip(1) // skip header
        .filter(|l| !l.trim().is_empty())
        .map(|line| parse_row(line.trim()))
        .collect()
}

fn main() {
    let csv = "
name,age,score
Alice,22,91.5
Bob,nineteen,88.0
Carol,21,
Dave,23,76.3
,25,55.0
Eve,20,bad_score
";

    let results = parse_csv(csv);
    let (successes, errors): (Vec<_>, Vec<_>) = results
        .into_iter()
        .partition(Result::is_ok);

    println!("=== Successfully Parsed Students ===");
    for s in successes.into_iter().flatten() {
        println!("  {} | age {} | score {:.1}", s.name, s.age, s.score);
    }

    println!("\n=== Parse Errors ===");
    for e in errors.into_iter().map(Result::unwrap_err) {
        println!("  ERROR: {e}");
    }
}
```

### Running Your Solution

```bash
cargo run -p day-09
```

Successful output:
```
=== Successfully Parsed Students ===
  Alice | age 22 | score 91.5
  Carol | age 21 | score 0.0
  Dave  | age 23 | score 76.3

=== Parse Errors ===
  ERROR: bad age in 'Bob,nineteen,88.0': invalid digit found in string
  ERROR: name is empty
  ERROR: bad score in 'Eve,20,bad_score': invalid float literal
```

### Extension Challenges

- **Easy:** Add a `grade` method to `Student` that returns `"A"`, `"B"`, `"C"`, or `"F"` based on score ranges.
- **Medium:** Define a custom `ParseError` enum with variants for each kind of failure (EmptyName, BadAge, BadScore) instead of using `String` as the error type.
- **Hard:** Read the CSV from a real file on disk using `std::fs::read_to_string`, return errors with `Box<dyn Error>` from main, and handle file-not-found gracefully.

---

## Quick Reference — `Option<T>` and `Result<T, E>` Cheat Sheet

### `Option<T>` Methods

| Method | Signature | What it does |
|--------|-----------|--------------|
| `unwrap()` | `Option<T> -> T` | panics on None |
| `expect("msg")` | `Option<T> -> T` | panics with message on None |
| `unwrap_or(default)` | `Option<T> -> T` | returns default on None |
| `unwrap_or_else(\|\| f())` | `Option<T> -> T` | computes default lazily |
| `map(\|v\| f(v))` | `Option<T> -> Option<U>` | transforms Some, passes None through |
| `and_then(\|v\| g(v))` | `Option<T> -> Option<U>` | chains Option-returning functions |
| `filter(\|v\| pred(v))` | `Option<T> -> Option<T>` | None if predicate false |
| `or(other)` | `Option<T> -> Option<T>` | returns `other` if None |
| `ok_or(err)` | `Option<T> -> Result<T,E>` | converts to Result |
| `is_some()` | `-> bool` | |
| `is_none()` | `-> bool` | |

### `Result<T, E>` Methods

| Method | Signature | What it does |
|--------|-----------|--------------|
| `unwrap()` | `Result<T,E> -> T` | panics on Err |
| `expect("msg")` | `Result<T,E> -> T` | panics with message on Err |
| `unwrap_or(default)` | `Result<T,E> -> T` | default on Err |
| `map(\|v\| f(v))` | `Result<T,E> -> Result<U,E>` | transforms Ok value |
| `map_err(\|e\| g(e))` | `Result<T,E> -> Result<T,F>` | transforms Err value |
| `and_then(\|v\| f(v))` | `Result<T,E> -> Result<U,E>` | chains fallible operations |
| `ok()` | `Result<T,E> -> Option<T>` | converts to Option (discards Err) |
| `is_ok()` | `-> bool` | |
| `is_err()` | `-> bool` | |

### The `?` Operator — Desugared

```rust
// These two are equivalent:
let value = some_result?;

let value = match some_result {
    Ok(v)  => v,
    Err(e) => return Err(e.into()),
};
```

The `.into()` call converts the error type automatically using `From`. To use `?` in `main`, declare it as:
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // now you can use ? here
    Ok(())
}
```
