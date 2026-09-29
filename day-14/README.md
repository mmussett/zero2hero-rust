# Day 14: Testing

> **Project:** Comprehensive test suite for the Shape library (Day 10) and Generic Stack (Day 11) — unit tests, `#[should_panic]`, `Result`-returning tests, integration tests, and doc tests.

## Learning Objectives

By the end of today you will be able to:
- Write unit tests with `#[test]`, `assert!`, `assert_eq!`, and `assert_ne!`
- Use `#[should_panic]` and `#[ignore]` attributes appropriately
- Return `Result` from tests to use `?` inside them
- Write integration tests in a `tests/` directory that test only the public API
- Write doc tests in `///` comments that double as living documentation
- Run targeted subsets of tests with `cargo test <filter>`

> **Note:** Today's project is a **library crate** (`src/lib.rs`) that re-exports the Day 10 shape types and Day 11 `Stack`. The real work is the tests themselves, not new application logic.

---

## Concepts

### 1. Unit Tests

> **Docs:** [Writing tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html) · [assert!](https://doc.rust-lang.org/std/macro.assert.html) · [assert_eq!](https://doc.rust-lang.org/std/macro.assert_eq.html) · [assert_ne!](https://doc.rust-lang.org/std/macro.assert_ne.html)

Unit tests live in the same file as the code they test, inside a `#[cfg(test)]` module. The `cfg(test)` annotation means this module is compiled only when running `cargo test` — it does not appear in your release binary.

Each test is a function marked `#[test]`. If the function returns normally, the test passes. If it panics (including via `assert!` macros), it fails. The three core assertion macros are:

- `assert!(expr)` — panics if `expr` is false
- `assert_eq!(left, right)` — panics if `left != right`, printing both values
- `assert_ne!(left, right)` — panics if `left == right`

Always prefer `assert_eq!` over `assert!(a == b)` — the former prints the actual vs. expected values on failure, which makes debugging much easier.

```rust
pub fn add(a: i32, b: i32) -> i32 { a + b }
pub fn is_even(n: i32) -> bool { n % 2 == 0 }
pub fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { None } else { Some(a / b) }
}

#[cfg(test)]
mod tests {
    use super::*; // bring the code under test into scope

    #[test]
    fn add_positive_numbers() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn add_negative_numbers() {
        assert_eq!(add(-1, -1), -2);
        assert_ne!(add(-1, -1), 0);
    }

    #[test]
    fn even_detection() {
        assert!(is_even(4));
        assert!(!is_even(7));
    }

    #[test]
    fn divide_by_zero_returns_none() {
        assert_eq!(divide(10.0, 0.0), None);
    }

    #[test]
    fn valid_division() {
        let result = divide(10.0, 4.0).unwrap();
        // Use f64 tolerance for floating point comparisons
        assert!((result - 2.5).abs() < f64::EPSILON);
    }
}
```

Run with `cargo test` (all tests) or `cargo test add` (only tests whose name contains "add").

#### Exercise 1.1 — Write Your First Tests

**Goal:** Write at least three tests for a simple function.

Write `pub fn celsius_to_fahrenheit(c: f64) -> f64` that converts Celsius to Fahrenheit using `c * 9.0 / 5.0 + 32.0`. Then write tests for: freezing point (0°C = 32°F), boiling point (100°C = 212°F), and body temperature (37°C ≈ 98.6°F, use a tolerance of `0.01`).

**Expected test output:**
```
running 3 tests
test tests::freezing_point ... ok
test tests::boiling_point ... ok
test tests::body_temperature ... ok

test result: ok. 3 passed; 0 failed
```

> **Hint:** For floating point comparisons, `(actual - expected).abs() < 0.01` is more reliable than `assert_eq!`.

---

### 2. `#[should_panic]` — Testing for Panics

> **Docs:** [#[should_panic]](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#checking-for-panics-with-should_panic)

Some functions are supposed to panic under certain conditions (out-of-bounds access, invalid arguments). `#[should_panic]` marks a test that is expected to panic — if the function does not panic, the test fails.

Use `#[should_panic(expected = "substring")]` to verify the panic message contains a specific string. This prevents false positives where a different panic passes the test.

```rust
pub fn nth_element(v: &[i32], n: usize) -> i32 {
    if n >= v.len() {
        panic!("index {} out of bounds for slice of length {}", n, v.len());
    }
    v[n]
}

pub fn square_root(x: f64) -> f64 {
    if x < 0.0 {
        panic!("cannot take square root of negative number: {x}");
    }
    x.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nth_element_valid() {
        assert_eq!(nth_element(&[10, 20, 30], 1), 20);
    }

    #[test]
    #[should_panic]
    fn nth_element_out_of_bounds() {
        nth_element(&[1, 2, 3], 10); // should panic
    }

    #[test]
    #[should_panic(expected = "negative number")]
    fn sqrt_of_negative() {
        square_root(-4.0); // should panic with message containing "negative number"
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn nth_element_message_matches() {
        nth_element(&[], 0);
    }
}
```

#### Exercise 2.1 — Should Panic Test

**Goal:** Write a function that validates its input and panics on invalid input, then test the panic.

Write `pub fn new_rectangle(width: f64, height: f64) -> (f64, f64)` that panics with message `"dimensions must be positive"` if either dimension is zero or negative. Write a `#[should_panic(expected = "...")]` test for the invalid case, and a normal test for the valid case.

**Expected test output:**
```
test tests::valid_rectangle ... ok
test tests::zero_dimension_panics ... ok
```

> **Hint:** Use `assert!(width > 0.0 && height > 0.0, "dimensions must be positive")`.

---

### 3. Returning `Result` from Tests

> **Docs:** [Writing tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html) · [Running tests](https://doc.rust-lang.org/book/ch11-02-running-tests.html)

Instead of panicking, tests can return `Result<(), E>`. This allows you to use `?` inside the test — any `Err` return fails the test and prints the error, which is often more informative than a panic.

This style is especially useful when you are testing code that naturally produces `Result` values, because you can chain fallible operations without wrapping every call in `unwrap()`.

```rust
use std::num::ParseIntError;

pub fn parse_and_double(s: &str) -> Result<i32, ParseIntError> {
    let n: i32 = s.parse()?;
    Ok(n * 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Returns Result — uses ? instead of unwrap()
    #[test]
    fn parse_valid_number() -> Result<(), String> {
        let result = parse_and_double("21").map_err(|e| e.to_string())?;
        assert_eq!(result, 42);
        Ok(())
    }

    #[test]
    fn parse_and_double_five() -> Result<(), String> {
        let result = parse_and_double("5").map_err(|e| e.to_string())?;
        assert_eq!(result, 10);
        Ok(())
    }

    // For the error case, still use a normal test (Result tests pass on Ok)
    #[test]
    fn parse_invalid_returns_err() {
        assert!(parse_and_double("abc").is_err());
    }
}
```

---

### 4. Integration Tests

> **Docs:** [Test organisation](https://doc.rust-lang.org/book/ch11-03-test-organization.html) · [cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html)

Integration tests live in a `tests/` directory alongside `src/`. Each file in `tests/` is compiled as a separate binary that links against your library's public API. They cannot see private items or `#[cfg(test)]` modules — they are genuine external users of your crate.

Create integration test files with `use your_crate_name::...` at the top. No `#[cfg(test)]` wrapper needed because the entire file is only compiled for testing.

```
my-crate/
├── Cargo.toml
├── src/
│   └── lib.rs
└── tests/
    ├── basic_integration.rs
    └── advanced_integration.rs
```

```rust
// tests/basic_integration.rs
use my_crate::{add, is_even};

#[test]
fn add_works_from_outside() {
    assert_eq!(add(5, 5), 10);
}

#[test]
fn even_detection_public_api() {
    assert!(is_even(100));
    assert!(!is_even(99));
}
```

Run only integration tests: `cargo test --test basic_integration`.

#### Exercise 4.1 — Write an Integration Test

**Goal:** Create a `tests/` directory and write one integration test.

After building your library with a public function (e.g. `pub fn greet(name: &str) -> String`), create `tests/greeting.rs` and verify the function returns the expected string from outside the crate.

**Expected test output:**
```
running 1 test
test greet_returns_correct_string ... ok
```

> **Hint:** The crate name for `use` comes from the `name` field in `Cargo.toml` under `[package]`, with hyphens replaced by underscores.

---

### 5. Doc Tests

> **Docs:** [Doc tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)

Doc tests are code examples inside `///` doc comments that are compiled and run as tests. They serve double duty: they document how to use your API and they verify the documentation is correct. Outdated docs that no longer compile are caught immediately.

A doc test is any ` ```rust ` block in a `///` comment. The code runs in its own scope, so you need `use` statements if required. Lines starting with `# ` are compiled but not shown in rendered documentation — useful for setup code.

```rust
/// Computes the factorial of `n`.
///
/// # Panics
/// Panics if `n > 20` because the result would overflow `u64`.
///
/// # Examples
///
/// ```
/// let result = day_14::factorial(5);
/// assert_eq!(result, 120);
/// ```
///
/// ```
/// let result = day_14::factorial(0);
/// assert_eq!(result, 1);
/// ```
pub fn factorial(n: u64) -> u64 {
    if n > 20 { panic!("overflow: n={n} > 20"); }
    (1..=n).product()
}

/// Returns the nth Fibonacci number (0-indexed).
///
/// # Examples
///
/// ```
/// # use day_14::fibonacci;
/// assert_eq!(fibonacci(0), 0);
/// assert_eq!(fibonacci(1), 1);
/// assert_eq!(fibonacci(7), 13);
/// ```
pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let (mut a, mut b) = (0u64, 1u64);
            for _ in 2..=n {
                (a, b) = (b, a + b);
            }
            b
        }
    }
}
```

Run doc tests specifically: `cargo test --doc`.

---

### 6. Test Organisation

> **Docs:** [Test organisation](https://doc.rust-lang.org/book/ch11-03-test-organization.html) · [#[ignore]](https://doc.rust-lang.org/book/ch11-02-running-tests.html#ignoring-some-tests-unless-specifically-requested) · [cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html) · [Rust by Example — Testing](https://doc.rust-lang.org/rust-by-example/testing.html)

As your test suite grows, organisation keeps it maintainable:

- **Name tests descriptively:** `test_name_describes_behaviour_being_tested` over `test1`.
- **Group related tests** in nested `mod` blocks inside `#[cfg(test)]`.
- **`#[ignore]`** marks tests that are slow, require external resources, or are temporarily broken. They are skipped by default; run with `cargo test -- --ignored`.
- **Test helper functions** do not need `#[test]` — they are regular functions inside the test module, called by multiple tests.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function — shared setup, no #[test]
    fn sample_data() -> Vec<i32> {
        vec![5, 3, 8, 1, 9, 2, 7]
    }

    mod sorting {
        use super::*;

        #[test]
        fn sorted_ascending() {
            let mut data = sample_data();
            data.sort();
            assert_eq!(data[0], 1);
            assert_eq!(data[data.len() - 1], 9);
        }
    }

    mod searching {
        use super::*;

        #[test]
        fn find_existing() {
            let data = sample_data();
            assert!(data.contains(&8));
        }

        #[test]
        #[ignore] // slow — only run with --ignored
        fn expensive_search() {
            // imagine this is very slow
            let data: Vec<i32> = (0..1_000_000).collect();
            assert!(data.contains(&999_999));
        }
    }
}
```

---

## Day Project: Comprehensive Test Suite

### What You're Building

A library crate that re-exports the `Shape` types from Day 10 and the `Stack<T>` from Day 11, then provides an exhaustive test suite covering unit tests, panic tests, `Result` tests, integration tests, and doc tests. This is the Week 2 "milestone" project — the suite validates everything you built this week.

### Requirements

**In `src/lib.rs`:**

1. Re-export (or re-implement) `Circle`, `Rectangle`, `Triangle`, `Shape` trait, and `Stack<T>`.
2. Add doc tests to at least two `Stack` methods (`push`/`pop` and `peek`) showing correct usage.
3. Add a `pub fn describe_shape(shape: &impl Shape) -> String` helper that returns `shape.describe()`.

**Unit tests (`#[cfg(test)]` in `src/lib.rs`):**

4. At least 2 tests for `Circle`: area and perimeter with known values.
5. At least 2 tests for `Rectangle`: area and perimeter.
6. At least 1 test for `Triangle` describe output using `starts_with`.
7. A `#[should_panic]` test for an invalid `Triangle` (negative side — add a `Triangle::new` constructor that panics on negative or zero sides).
8. All `Stack<i32>` methods: `new`, `push`, `pop`, `peek`, `is_empty`, `len`, `Default`.
9. A `Stack<String>` test showing heterogeneous type usage.
10. One test returning `Result<(), String>` using `?`.

**Integration tests (`tests/shapes_integration.rs`):**

11. Import `describe_shape` from the crate and verify the `describe()` output format for `Circle` and `Rectangle`.

**Doc tests:**

12. On `Stack::push` — show push and then peek to verify.
13. On `Stack::pop` — show push then pop returning `Some`, then pop on empty returning `None`.

### Getting Started

```rust
// src/lib.rs
use std::f64::consts::PI;
use std::fmt;

pub trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn name(&self) -> &str;
    fn describe(&self) -> String {
        format!("[{}]: area={:.2}, perimeter={:.2}", self.name(), self.area(), self.perimeter())
    }
}

pub struct Circle { pub radius: f64 }
impl Shape for Circle {
    fn area(&self) -> f64 { PI * self.radius * self.radius }
    fn perimeter(&self) -> f64 { 2.0 * PI * self.radius }
    fn name(&self) -> &str { "circle" }
}

pub struct Rectangle { pub width: f64, pub height: f64 }
impl Shape for Rectangle {
    fn area(&self) -> f64 { self.width * self.height }
    fn perimeter(&self) -> f64 { 2.0 * (self.width + self.height) }
    fn name(&self) -> &str { "rectangle" }
}

pub struct Triangle { pub a: f64, pub b: f64, pub c: f64 }
impl Triangle {
    /// Creates a Triangle, panicking if any side is not positive.
    ///
    /// # Panics
    /// Panics if any side is zero or negative.
    pub fn new(a: f64, b: f64, c: f64) -> Self {
        assert!(a > 0.0 && b > 0.0 && c > 0.0, "triangle sides must be positive");
        Triangle { a, b, c }
    }
}
impl Shape for Triangle {
    fn area(&self) -> f64 {
        let s = self.perimeter() / 2.0;
        (s * (s - self.a) * (s - self.b) * (s - self.c)).sqrt()
    }
    fn perimeter(&self) -> f64 { self.a + self.b + self.c }
    fn name(&self) -> &str { "triangle" }
}

pub fn describe_shape(shape: &impl Shape) -> String {
    shape.describe()
}

pub struct Stack<T> { elements: Vec<T> }

impl<T> Stack<T> {
    /// Creates a new, empty `Stack`.
    ///
    /// # Examples
    /// ```
    /// use day_14::Stack;
    /// let s: Stack<i32> = Stack::new();
    /// assert!(s.is_empty());
    /// ```
    pub fn new() -> Self { Stack { elements: Vec::new() } }

    /// Pushes an item onto the top of the stack.
    ///
    /// # Examples
    /// ```
    /// use day_14::Stack;
    /// let mut s = Stack::new();
    /// s.push(42);
    /// assert_eq!(s.peek(), Some(&42));
    /// ```
    pub fn push(&mut self, item: T) { self.elements.push(item); }

    /// Removes and returns the top item, or `None` if the stack is empty.
    ///
    /// # Examples
    /// ```
    /// use day_14::Stack;
    /// let mut s = Stack::new();
    /// s.push(1);
    /// assert_eq!(s.pop(), Some(1));
    /// assert_eq!(s.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> { self.elements.pop() }

    pub fn peek(&self) -> Option<&T> { self.elements.last() }
    pub fn is_empty(&self) -> bool { self.elements.is_empty() }
    pub fn len(&self) -> usize { self.elements.len() }
}

impl<T> Default for Stack<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Shape tests ---

    #[test]
    fn circle_area_known_value() {
        let c = Circle { radius: 1.0 };
        assert!((c.area() - PI).abs() < 1e-10);
    }

    #[test]
    fn circle_perimeter_known_value() {
        let c = Circle { radius: 2.0 };
        assert!((c.perimeter() - 4.0 * PI).abs() < 1e-10);
    }

    #[test]
    fn rectangle_area() {
        let r = Rectangle { width: 5.0, height: 3.0 };
        assert_eq!(r.area(), 15.0);
    }

    #[test]
    fn rectangle_perimeter() {
        let r = Rectangle { width: 5.0, height: 3.0 };
        assert_eq!(r.perimeter(), 16.0);
    }

    #[test]
    fn triangle_describe_format() {
        let t = Triangle::new(3.0, 4.0, 5.0);
        assert!(t.describe().starts_with("[triangle]"));
        assert!(t.describe().contains("area="));
    }

    #[test]
    #[should_panic(expected = "triangle sides must be positive")]
    fn triangle_negative_side_panics() {
        let _ = Triangle::new(-1.0, 4.0, 5.0);
    }

    // --- Stack tests ---

    #[test]
    fn stack_new_is_empty() {
        let s: Stack<i32> = Stack::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn stack_push_pop_i32() {
        let mut s: Stack<i32> = Stack::new();
        s.push(1);
        s.push(2);
        s.push(3);
        assert_eq!(s.len(), 3);
        assert_eq!(s.pop(), Some(3));
        assert_eq!(s.pop(), Some(2));
        assert_eq!(s.pop(), Some(1));
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn stack_peek_does_not_consume() {
        let mut s: Stack<i32> = Stack::new();
        s.push(99);
        assert_eq!(s.peek(), Some(&99));
        assert_eq!(s.len(), 1); // still there
    }

    #[test]
    fn stack_default_is_empty() {
        let s: Stack<f64> = Stack::default();
        assert!(s.is_empty());
    }

    #[test]
    fn stack_with_strings() {
        let mut s: Stack<String> = Stack::new();
        s.push("hello".to_string());
        s.push("world".to_string());
        assert_eq!(s.pop(), Some("world".to_string()));
        assert_eq!(s.peek(), Some(&"hello".to_string()));
    }

    #[test]
    fn stack_result_test() -> Result<(), String> {
        let mut s: Stack<i32> = Stack::new();
        s.push(42);
        let top = s.pop().ok_or("stack was empty")?;
        assert_eq!(top, 42);
        Ok(())
    }
}
```

```rust
// tests/shapes_integration.rs
use day_14::{Circle, Rectangle, describe_shape};

#[test]
fn circle_describe_contains_name() {
    let c = Circle { radius: 3.0 };
    let description = describe_shape(&c);
    assert!(description.contains("circle"), "expected 'circle' in: {description}");
    assert!(description.contains("area="), "expected 'area=' in: {description}");
}

#[test]
fn rectangle_describe_format() {
    let r = Rectangle { width: 4.0, height: 5.0 };
    let description = describe_shape(&r);
    assert!(description.starts_with("[rectangle]"),
        "expected '[rectangle]' prefix in: {description}");
    assert!(description.contains("perimeter=30.00"),
        "expected perimeter=30.00 in: {description}");
}
```

### Running Your Solution

```bash
# All tests
cargo test -p day-14

# Only tests whose name contains "shape"
cargo test -p day-14 shapes

# Only tests whose name contains "stack"
cargo test -p day-14 stack

# Only doc tests
cargo test -p day-14 --doc

# Integration tests only
cargo test -p day-14 --test shapes_integration
```

Full passing run:
```
running 12 tests
test tests::circle_area_known_value ... ok
test tests::circle_perimeter_known_value ... ok
test tests::rectangle_area ... ok
test tests::rectangle_perimeter ... ok
test tests::triangle_describe_format ... ok
test tests::triangle_negative_side_panics ... ok
test tests::stack_new_is_empty ... ok
test tests::stack_push_pop_i32 ... ok
test tests::stack_peek_does_not_consume ... ok
test tests::stack_default_is_empty ... ok
test tests::stack_with_strings ... ok
test tests::stack_result_test ... ok

test result: ok. 12 passed; 0 failed

running 2 tests
test circle_describe_contains_name ... ok
test rectangle_describe_format ... ok

test result: ok. 2 passed; 0 failed

Doc-tests day-14

running 3 tests
test src/lib.rs - Stack::new (line N) ... ok
test src/lib.rs - Stack::push (line N) ... ok
test src/lib.rs - Stack::pop (line N) ... ok

test result: ok. 3 passed; 0 failed
```

### Extension Challenges

- **Easy:** Add `#[ignore]` to a placeholder test called `stress_test_large_stack` with a comment explaining what it would eventually test. Run `cargo test -- --ignored` to confirm it is skipped then run.
- **Medium:** Write a property-based style test (without an external crate) that pushes N random numbers onto a `Stack<i32>`, then pops all of them and verifies they come out in reverse order. Use a hardcoded seed array of at least 20 values.
- **Hard:** Add a second integration test file `tests/stack_integration.rs` that demonstrates `Stack<T>` implements `Default`, and tests that a `Stack<Box<dyn std::fmt::Display>>` (a stack of trait objects) works correctly — pushing both integer and string values, then popping and displaying each.
