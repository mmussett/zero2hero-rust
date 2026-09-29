# Rust Primitive Types — Complete Reference

> 30-Day Rust Curriculum — Day Reference Card
> Every primitive type, with memory layout, invariants, associated functions, methods, and runnable examples.

---

## Table of Contents

1. [Integer Types — Signed](#integer-types--signed)
2. [Integer Types — Unsigned](#integer-types--unsigned)
3. [Floating-Point Types](#floating-point-types)
4. [Boolean — `bool`](#boolean--bool)
5. [Character — `char`](#character--char)
6. [String Slice — `&str`](#string-slice--str)
7. [Owned String — `String`](#owned-string--string)
8. [Unit Type — `()`](#unit-type--)
9. [Arrays — `[T; N]`](#arrays--t-n)
10. [Tuples — `(T1, T2, ...)`](#tuples--t1-t2-)

---

## Integer Types — Signed

Rust's signed integers use **two's complement** representation. The bit pattern with the leading 1 is negative.

| Type    | Bits | Min                                      | Max                                     |
|---------|------|------------------------------------------|-----------------------------------------|
| `i8`    | 8    | -128                                     | 127                                     |
| `i16`   | 16   | -32 768                                  | 32 767                                  |
| `i32`   | 32   | -2 147 483 648                           | 2 147 483 647                           |
| `i64`   | 64   | -9 223 372 036 854 775 808               | 9 223 372 036 854 775 807               |
| `i128`  | 128  | -(2^127)                                 | 2^127 − 1                               |
| `isize` | arch | platform-dependent (32 or 64 bits)       | platform-dependent                      |

**Invariants:** The value is always a valid two's complement integer in the stated range. Overflow in debug builds panics; in release builds it wraps (unless you opt into explicit checked/saturating/wrapping arithmetic).

---

## `i32` — Default Signed Integer

**Representation:** 32-bit two's complement. Range: −2 147 483 648 to 2 147 483 647.

`i32` is the default integer type inferred by Rust when no suffix is given (e.g., `let x = 5;`).

### Constants and Associated Functions

| Item              | Value / Signature             | Description                            |
|-------------------|-------------------------------|----------------------------------------|
| `i32::MIN`        | `-2_147_483_648`              | Smallest representable value           |
| `i32::MAX`        | `2_147_483_647`               | Largest representable value            |
| `i32::BITS`       | `32`                          | Number of bits in this type            |
| `i32::from(v)`    | `i32::from(i16_val)`          | Infallible widening conversion         |

### Key Methods

| Method                            | Returns        | Description                                               |
|-----------------------------------|----------------|-----------------------------------------------------------|
| `.abs()`                          | `i32`          | Absolute value (panics on `i32::MIN` in debug)            |
| `.pow(exp: u32)`                  | `i32`          | Integer exponentiation                                    |
| `.checked_add(rhs)`               | `Option<i32>`  | Returns `None` on overflow                                |
| `.saturating_add(rhs)`            | `i32`          | Clamps to `MIN`/`MAX` on overflow                         |
| `.wrapping_add(rhs)`              | `i32`          | Wraps on overflow (defined, no panic)                     |
| `.overflowing_add(rhs)`           | `(i32, bool)`  | Returns (result, did_overflow)                            |
| `.count_ones()`                   | `u32`          | Number of set bits                                        |
| `.leading_zeros()`                | `u32`          | Number of leading zero bits                               |
| `.trailing_zeros()`               | `u32`          | Number of trailing zero bits                              |
| `.signum()`                       | `i32`          | Returns -1, 0, or 1                                       |
| `.min(other)`                     | `i32`          | Returns the smaller of two values                         |
| `.max(other)`                     | `i32`          | Returns the larger of two values                          |
| `.clamp(min, max)`                | `i32`          | Clamps value into [min, max]                              |

### Examples

```rust
fn main() {
    // Constants
    println!("i32::MIN = {}", i32::MIN); // -2147483648
    println!("i32::MAX = {}", i32::MAX); // 2147483647
    println!("i32::BITS = {}", i32::BITS); // 32

    // Overflow protection — checked arithmetic
    let big: i32 = i32::MAX;
    match big.checked_add(1) {
        Some(v) => println!("sum = {v}"),
        None    => println!("overflow!"), // prints this
    }

    // Saturating — clamps instead of wrapping
    let sat = big.saturating_add(100);
    println!("saturated = {sat}"); // 2147483647 (stays at MAX)

    // Wrapping — defined two's complement wrap
    let wrapped = big.wrapping_add(1);
    println!("wrapped = {wrapped}"); // -2147483648 (MIN)

    // Parsing from string
    let parsed: i32 = "42".parse().expect("not a number");
    println!("parsed = {parsed}");

    // Bit inspection
    let n: i32 = 0b0000_1111;
    println!("count_ones  = {}", n.count_ones());   // 4
    println!("leading_zeros = {}", n.leading_zeros()); // 28

    // abs / signum
    println!("{}", (-7_i32).abs());    // 7
    println!("{}", (-7_i32).signum()); // -1
}
```

---

## Integer Types — Unsigned

| Type    | Bits | Min | Max                                      |
|---------|------|-----|------------------------------------------|
| `u8`    | 8    | 0   | 255                                      |
| `u16`   | 16   | 0   | 65 535                                   |
| `u32`   | 32   | 0   | 4 294 967 295                            |
| `u64`   | 64   | 0   | 18 446 744 073 709 551 615               |
| `u128`  | 128  | 0   | 2^128 − 1                                |
| `usize` | arch | 0   | platform-dependent (typically 2^64 − 1)  |

**Invariants:** Always non-negative. No negative numbers are representable. Overflow wraps in release mode.

---

## `u8` — Byte

**Representation:** 8-bit unsigned. Range: 0 to 255. Commonly used to represent raw bytes.

### Constants and Associated Functions

| Item         | Value / Signature | Description              |
|--------------|-------------------|--------------------------|
| `u8::MIN`    | `0`               | Always zero              |
| `u8::MAX`    | `255`             | Largest byte value       |
| `u8::BITS`   | `8`               | Bit width                |

### Key Methods

Same arithmetic methods as `i32` — `checked_*`, `saturating_*`, `wrapping_*`, `overflowing_*`, `pow`, `count_ones`, `leading_zeros`, `trailing_zeros` — plus:

| Method              | Returns  | Description                             |
|---------------------|----------|-----------------------------------------|
| `.is_ascii()`       | `bool`   | True if value is in the ASCII range     |
| `.to_ascii_uppercase()` | `u8` | ASCII letter uppercased (no-op if non-alpha) |
| `.to_ascii_lowercase()` | `u8` | ASCII letter lowercased                 |

### Examples

```rust
fn main() {
    let byte: u8 = 200;
    println!("saturated add: {}", byte.saturating_add(100)); // 255

    // u8 as ASCII character
    let ch = b'A'; // u8 literal
    println!("'A' as u8 = {ch}"); // 65
    println!("uppercase: {}", ch.to_ascii_uppercase()); // 65

    // Byte array processing
    let data: &[u8] = b"hello";
    for b in data {
        print!("{b} "); // 104 101 108 108 111
    }
    println!();
}
```

---

## `usize` — Platform-Sized Index

**Representation:** Same bit width as a pointer — 32 bits on 32-bit targets, 64 bits on 64-bit targets. Range: 0 to `usize::MAX`.

`usize` is the canonical index and length type. All slice indexing and collection lengths use `usize`.

### Key Points

- `Vec::len()`, `slice.len()`, `HashMap::len()` all return `usize`
- Array and slice indexing (`arr[i]`) requires `usize`
- `.enumerate()` on iterators yields `(usize, &T)` pairs
- Casting: `let i: usize = 5_i32 as usize;` (use `as` with care; negative i32 → large usize)

### Examples

```rust
fn main() {
    let numbers = vec![10, 20, 30, 40, 50];

    // len() returns usize
    let n: usize = numbers.len();
    println!("length = {n}"); // 5

    // enumerate gives (usize, &T)
    for (idx, val) in numbers.iter().enumerate() {
        println!("[{idx}] = {val}");
    }

    // Safe vs panicking indexing
    let idx: usize = 2;
    println!("arr[2] = {}", numbers[idx]); // 30

    match numbers.get(10) {
        Some(v) => println!("got {v}"),
        None    => println!("index out of bounds — no panic"), // prints this
    }
}
```

---

## Floating-Point Types

Rust's floats follow **IEEE 754** binary floating-point arithmetic.

| Type  | Bits | Mantissa bits | Significant decimal digits | Exponent range   |
|-------|------|---------------|---------------------------|------------------|
| `f32` | 32   | 23 + 1 (implicit) | ~7                    | ±3.4 × 10^38     |
| `f64` | 64   | 52 + 1 (implicit) | ~15-16                | ±1.8 × 10^308    |

`f64` is the default. Use `f32` when memory or SIMD/GPU throughput matters.

---

## `f64` — Double-Precision Float

**Representation:** 1 sign bit, 11 exponent bits, 52 mantissa bits (IEEE 754 binary64).

### Constants and Associated Functions

| Item                 | Value / Signature        | Description                              |
|----------------------|--------------------------|------------------------------------------|
| `f64::MIN`           | `-1.7976931...e308`      | Largest negative finite value            |
| `f64::MAX`           | `1.7976931...e308`       | Largest positive finite value            |
| `f64::MIN_POSITIVE`  | `2.2250738...e-308`      | Smallest positive normal value           |
| `f64::EPSILON`       | `2.220446...e-16`        | Smallest x where `1.0 + x != 1.0`       |
| `f64::INFINITY`      | `+∞`                     | Positive infinity                        |
| `f64::NEG_INFINITY`  | `-∞`                     | Negative infinity                        |
| `f64::NAN`           | `NaN`                    | Not a Number (not equal to itself)       |
| `f64::BITS`          | `64`                     | Bit width                                |
| `f64::consts::PI`    | `3.14159265358979...`    | π (in `std::f64::consts`)               |
| `f64::consts::E`     | `2.71828182845904...`    | Euler's number                           |

### Key Methods

| Method              | Returns | Description                                  |
|---------------------|---------|----------------------------------------------|
| `.sqrt()`           | `f64`   | Square root (NaN if negative)                |
| `.abs()`            | `f64`   | Absolute value                               |
| `.powi(n: i32)`     | `f64`   | Integer exponent                             |
| `.powf(n: f64)`     | `f64`   | Float exponent                               |
| `.floor()`          | `f64`   | Round down to nearest integer                |
| `.ceil()`           | `f64`   | Round up to nearest integer                  |
| `.round()`          | `f64`   | Round to nearest, ties away from zero        |
| `.ln()`             | `f64`   | Natural logarithm                            |
| `.log2()`           | `f64`   | Base-2 logarithm                             |
| `.log10()`          | `f64`   | Base-10 logarithm                            |
| `.sin()`            | `f64`   | Sine (radians)                               |
| `.cos()`            | `f64`   | Cosine (radians)                             |
| `.tan()`            | `f64`   | Tangent (radians)                            |
| `.is_nan()`         | `bool`  | True if NaN                                  |
| `.is_infinite()`    | `bool`  | True if ±infinity                            |
| `.is_finite()`      | `bool`  | True if neither NaN nor infinite             |
| `.is_sign_positive()` | `bool`| True if sign bit is positive (includes +0.0)|
| `.min(other)`       | `f64`   | Smaller of two floats (NaN-propagating)      |
| `.max(other)`       | `f64`   | Larger of two floats (NaN-propagating)       |
| `.clamp(min, max)`  | `f64`   | Clamps into [min, max]                       |

### Examples

```rust
fn main() {
    // NaN is not equal to itself — the fundamental IEEE 754 rule
    let nan = f64::NAN;
    println!("NAN == NAN: {}", nan == nan);     // false
    println!("NAN.is_nan(): {}", nan.is_nan()); // true

    // Distance formula: sqrt((x2-x1)^2 + (y2-y1)^2)
    let (x1, y1) = (0.0_f64, 0.0_f64);
    let (x2, y2) = (3.0_f64, 4.0_f64);
    let dist = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    println!("distance = {dist}"); // 5

    // Compound interest: A = P(1 + r/n)^(nt)
    let principal = 1000.0_f64;
    let rate      = 0.05;
    let n         = 12.0; // compounded monthly
    let t         = 10.0; // years
    let amount = principal * (1.0 + rate / n).powf(n * t);
    println!("compound interest result: {amount:.2}"); // 1647.01

    // Temperature conversion: Celsius -> Fahrenheit
    let celsius = 100.0_f64;
    let fahrenheit = celsius * 9.0 / 5.0 + 32.0;
    println!("{celsius}°C = {fahrenheit:.1}°F"); // 212.0°F

    // Formatting with decimal places
    let pi = std::f64::consts::PI;
    println!("{pi:.5}"); // 3.14159

    // Infinity arithmetic
    let inf = f64::INFINITY;
    println!("{}", inf + 1.0);   // inf
    println!("{}", inf - inf);   // NaN
    println!("{}", 1.0 / 0.0_f64); // inf
}
```

---

## `f32` vs `f64`

- **Use `f32`** when: working with GPU shaders (GLSL/WGSL expect f32), large arrays where memory footprint matters, SIMD operations that process more lanes at 32-bit width.
- **Use `f64`** (default) when: precision matters (scientific computing, financial approximations, geometry), matching C `double`, general purpose computation.
- **Never compare floats with `==`**: use `(a - b).abs() < f64::EPSILON` instead.

---

## Boolean — `bool`

**Representation:** 1 byte in memory (not 1 bit). Valid bit patterns: `0x00` (`false`) and `0x01` (`true`). Any other bit pattern is undefined behaviour.

**Invariants:** Only two values — `true` and `false`. Rust does not allow implicit integer-to-bool conversion.

### Operators

| Operator | Name               | Behaviour                                 |
|----------|--------------------|-------------------------------------------|
| `&&`     | Short-circuit AND  | Skips right side if left is `false`       |
| `\|\|`   | Short-circuit OR   | Skips right side if left is `true`        |
| `!`      | NOT                | Inverts the value                         |
| `&`      | Bitwise AND        | Evaluates both sides always               |
| `\|`     | Bitwise OR         | Evaluates both sides always               |
| `^`      | Bitwise XOR        | Evaluates both sides always               |

### Key Methods

| Method / Conversion        | Returns      | Description                                      |
|----------------------------|--------------|--------------------------------------------------|
| `bool as i32`              | `0` or `1`   | `false` → 0, `true` → 1                         |
| `.then(|| value)`          | `Option<T>`  | `Some(value)` if true, `None` if false           |
| `.then_some(value)`        | `Option<T>`  | Same as `.then` but takes a value directly       |

### Examples

```rust
fn main() {
    let is_admin = true;
    let is_logged_in = false;

    // Short-circuit: is_admin check skipped because is_logged_in is false
    if is_logged_in && is_admin {
        println!("welcome, admin");
    } else {
        println!("access denied");
    }

    // Casting bool to integer
    let score_bonus = is_admin as i32 * 100;
    println!("bonus = {score_bonus}"); // 100

    // .then_some() for optional values
    let admin_panel: Option<&str> = is_admin.then_some("admin dashboard");
    println!("{:?}", admin_panel); // Some("admin dashboard")

    // Flag tracking
    let items = vec![1, 2, 3, 4, 5];
    let has_even = items.iter().any(|x| x % 2 == 0);
    let all_positive = items.iter().all(|x| *x > 0);
    println!("has_even={has_even}, all_positive={all_positive}");

    // Conditional iteration
    let verbose = true;
    for i in 0..3 {
        if verbose {
            println!("step {i}");
        }
    }
}
```

---

## Character — `char`

**Representation:** 4 bytes (u32 internally). Holds a **Unicode scalar value**: U+0000 to U+D7FF and U+E000 to U+10FFFF. Surrogate code points (U+D800..=U+DFFF) are excluded — they are not valid scalar values.

**Invariants:** Every `char` value is a valid Unicode scalar value. You cannot construct a `char` from an arbitrary `u32` without going through `char::from_u32` (which returns `Option<char>`).

### Constants and Associated Functions

| Item                        | Signature / Value       | Description                                         |
|-----------------------------|-------------------------|-----------------------------------------------------|
| `char::from_u32(n: u32)`    | `Option<char>`          | Returns `None` for surrogates or out-of-range       |
| `char::from(b: u8)`         | `char`                  | Infallible: every byte is a valid Unicode scalar    |
| `c as u32`                  | `u32`                   | Cast to the Unicode codepoint                       |
| `char::MAX`                 | `'\u{10FFFF}'`          | Largest possible char                               |

### Key Methods

| Method                  | Returns              | Description                                              |
|-------------------------|----------------------|----------------------------------------------------------|
| `.is_alphabetic()`      | `bool`               | True for letters in any Unicode script                   |
| `.is_alphanumeric()`    | `bool`               | Letters or digits                                        |
| `.is_numeric()`         | `bool`               | Unicode numeric characters                               |
| `.is_ascii_digit()`     | `bool`               | ASCII '0'–'9' only                                       |
| `.is_whitespace()`      | `bool`               | Spaces, tabs, newlines, Unicode whitespace               |
| `.is_uppercase()`       | `bool`               | Has uppercase form in Unicode                            |
| `.is_lowercase()`       | `bool`               | Has lowercase form in Unicode                            |
| `.is_ascii()`           | `bool`               | In the ASCII range (U+0000–U+007F)                       |
| `.to_uppercase()`       | `impl Iterator<char>` | Yields uppercase chars (may expand, e.g., 'ß' → "SS") |
| `.to_lowercase()`       | `impl Iterator<char>` | Yields lowercase chars                                   |
| `.len_utf8()`           | `usize`              | How many bytes this char uses in UTF-8 (1–4)            |
| `.len_utf16()`          | `usize`              | How many UTF-16 code units (1–2)                         |

### Why `char` is not `u8`

ASCII is a 7-bit subset of Unicode. A `char` can hold any Unicode scalar value — 'é' (U+00E9), '中' (U+4E2D), '😀' (U+1F600) — which require 2, 3, and 4 bytes in UTF-8 respectively. Using `u8` for text would be incorrect for non-ASCII content.

### Examples

```rust
fn main() {
    // Codepoint display
    let ch = '😀';
    println!("char: {ch}");
    println!("codepoint: U+{:04X}", ch as u32); // U+1F600
    println!("utf-8 bytes: {}", ch.len_utf8());  // 4

    // Safe construction from u32
    println!("{:?}", char::from_u32(65));       // Some('A')
    println!("{:?}", char::from_u32(0xD800));   // None (surrogate)

    // Counting vowels in a string
    let text = "Hello, World!";
    let vowels: usize = text
        .chars()
        .filter(|c| "aeiouAEIOU".contains(*c))
        .count();
    println!("vowels in \"{text}\": {vowels}"); // 3

    // Filtering punctuation
    let clean: String = text.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect();
    println!("cleaned: {clean}"); // Hello World

    // to_uppercase yields an iterator
    let upper: String = 'ß'.to_uppercase().collect();
    println!("'ß' uppercased: {upper}"); // SS  (two characters!)
}
```

---

## String Slice — `&str`

**Representation:** A **fat pointer** — `(ptr: *const u8, len: usize)`. Points into a UTF-8 byte sequence. No null terminator. No heap ownership.

**Invariants:** The bytes in the range `[ptr, ptr+len)` are always valid UTF-8. Rust enforces this at compile time and at conversion boundaries.

String literals (`"hello"`) have type `&'static str` — they live in the binary's read-only data segment.

### Key Methods

| Method                      | Returns            | Description                                     |
|-----------------------------|--------------------|-------------------------------------------------|
| `.len()`                    | `usize`            | Byte length (not character count)               |
| `.is_empty()`               | `bool`             | True if `len() == 0`                            |
| `.chars()`                  | `impl Iterator<char>` | Iterator over Unicode scalar values          |
| `.bytes()`                  | `impl Iterator<u8>` | Iterator over raw UTF-8 bytes                 |
| `.contains(pat)`            | `bool`             | True if slice contains the pattern              |
| `.starts_with(pat)`         | `bool`             | True if starts with pattern                     |
| `.ends_with(pat)`           | `bool`             | True if ends with pattern                       |
| `.find(pat)`                | `Option<usize>`    | Byte index of first match                       |
| `.split(pat)`               | `impl Iterator<&str>` | Split on a pattern                           |
| `.split_whitespace()`       | `impl Iterator<&str>` | Split on any whitespace, skipping empties    |
| `.lines()`                  | `impl Iterator<&str>` | Iterate over lines (strips `\n`/`\r\n`)      |
| `.trim()`                   | `&str`             | Strip leading and trailing whitespace           |
| `.trim_start()`             | `&str`             | Strip leading whitespace only                   |
| `.trim_end()`               | `&str`             | Strip trailing whitespace only                  |
| `.to_uppercase()`           | `String`           | Returns a new owned uppercase string            |
| `.to_lowercase()`           | `String`           | Returns a new owned lowercase string            |
| `.replace(from, to)`        | `String`           | Replace all occurrences                         |
| `.parse::<T>()`             | `Result<T, _>`     | Parse into any type that implements `FromStr`   |
| `.to_string()`              | `String`           | Convert to owned `String`                       |

### Examples

```rust
fn main() {
    let sentence = "  the quick brown fox  ";

    // trim and split
    let words: Vec<&str> = sentence.trim().split_whitespace().collect();
    println!("{words:?}"); // ["the", "quick", "brown", "fox"]
    println!("word count: {}", words.len()); // 4

    // Search
    let haystack = "Hello, world!";
    println!("contains 'world': {}", haystack.contains("world")); // true
    println!("starts with 'Hello': {}", haystack.starts_with("Hello")); // true
    println!("find 'world': {:?}", haystack.find("world")); // Some(7)

    // Lines iterator
    let multiline = "line one\nline two\nline three";
    for (i, line) in multiline.lines().enumerate() {
        println!("{i}: {line}");
    }

    // Parsing
    let n: i32 = "42".parse().unwrap();
    let f: f64 = "3.14".parse().unwrap();
    println!("parsed: {n}, {f}");

    // char count vs byte length for multi-byte content
    let emoji = "café";
    println!("bytes: {}", emoji.len());        // 5 (é is 2 bytes)
    println!("chars: {}", emoji.chars().count()); // 4
}
```

---

## Owned String — `String`

**Representation:** `(ptr: *mut u8, len: usize, capacity: usize)` — a heap-allocated, growable, UTF-8 encoded buffer. Owns its memory; dropping a `String` frees the heap allocation.

**Invariants:** Same UTF-8 guarantee as `&str`. Capacity is always ≥ length.

### Constants and Associated Functions

| Item                         | Signature               | Description                                     |
|------------------------------|-------------------------|-------------------------------------------------|
| `String::new()`              | `-> String`             | Empty string, no allocation                     |
| `String::from("lit")`        | `-> String`             | From a string literal                           |
| `String::with_capacity(n)`   | `-> String`             | Pre-allocate at least `n` bytes                 |

### Key Methods

| Method                        | Returns        | Description                                              |
|-------------------------------|----------------|----------------------------------------------------------|
| `.push_str(s: &str)`          | `()`           | Append a string slice in place                           |
| `.push(c: char)`              | `()`           | Append a single char                                     |
| `.pop()`                      | `Option<char>` | Remove and return the last char                          |
| `.insert(idx, char)`          | `()`           | Insert a char at byte index (must be char boundary)      |
| `.insert_str(idx, &str)`      | `()`           | Insert a string slice at byte index                      |
| `.remove(idx)`                | `char`         | Remove and return char at byte index                     |
| `.len()`                      | `usize`        | Current byte length                                      |
| `.capacity()`                 | `usize`        | Current allocated capacity                               |
| `.is_empty()`                 | `bool`         | True if length is zero                                   |
| `.clear()`                    | `()`           | Set length to zero (retains allocation)                  |
| `.as_str()`                   | `&str`         | Borrow as a string slice                                 |
| `.as_bytes()`                 | `&[u8]`        | Borrow as a byte slice                                   |
| `.split_at(mid)`              | `(&str, &str)` | Split at byte offset (panics if not char boundary)       |
| `.truncate(new_len)`          | `()`           | Shorten to `new_len` bytes (panics if not char boundary) |
| `.retain(f: FnMut(char)->bool)` | `()`         | Keep only chars for which `f` returns true               |

### Concatenation

```rust
// + operator: takes ownership of the left String
let s1 = String::from("Hello, ");
let s2 = String::from("world!");
let s3 = s1 + &s2; // s1 moved here; s2 still accessible

// format! macro: no ownership taken, more readable for multi-piece
let a = String::from("tic");
let b = String::from("tac");
let c = String::from("toe");
let joined = format!("{a}-{b}-{c}"); // "tic-tac-toe"
```

### Examples

```rust
fn main() {
    // Building a string incrementally
    let mut s = String::with_capacity(64);
    s.push_str("Hello");
    s.push(',');
    s.push(' ');
    s.push_str("Rust!");
    println!("{s}"); // Hello, Rust!
    println!("len={}, cap={}", s.len(), s.capacity());

    // pop removes the last char
    let mut editable = String::from("Hello!");
    let removed = editable.pop();
    println!("{editable} — removed: {removed:?}"); // Hello — removed: Some('!')

    // retain: keep only alphabetic chars
    let mut messy = String::from("h3ll0 w0rld");
    messy.retain(|c| c.is_alphabetic() || c == ' ');
    println!("{messy}"); // hll wrld

    // Converting &str to String and back
    let literal: &str = "static text";
    let owned: String = literal.to_string();
    let borrowed: &str = owned.as_str(); // or &owned
    println!("{borrowed}");

    // format! for complex assembly
    let name = "Alice";
    let score = 98;
    let report = format!("Student: {name:<10} Score: {score:>3}/100");
    println!("{report}"); // Student: Alice       Score:  98/100
}
```

---

## Unit Type — `()`

**Representation:** Zero bytes. A zero-sized type (ZST) — it carries no information and takes up no memory.

**Invariants:** Has exactly one value, also written `()`. It is both the type and its sole value.

### When `()` Appears

| Context                                      | Explanation                                             |
|----------------------------------------------|---------------------------------------------------------|
| `fn foo() { ... }`                           | Implicit return type `()` when no `->` annotation       |
| `fn foo() -> () { ... }`                     | Explicit form; identical to the above                   |
| `Result<(), E>`                              | Success carries no data; failure carries `E`            |
| `let _ = expr;` or `expr;`                   | Discarding a value yields `()`                          |
| Block ending with a `;` expression           | `{ 5 + 3; }` evaluates to `()`                         |

### Examples

```rust
fn greet(name: &str) {
    // No return type annotation — implicitly returns ()
    println!("Hi, {name}!");
}

fn save_to_disk(data: &str) -> Result<(), String> {
    if data.is_empty() {
        return Err(String::from("cannot save empty data"));
    }
    // ... write to disk ...
    println!("saved: {data}");
    Ok(()) // success with no meaningful return value
}

fn main() {
    let result: () = greet("Bob"); // () is the return value
    println!("greet returned: {result:?}"); // ()

    match save_to_disk("hello") {
        Ok(())   => println!("success"),
        Err(e)   => println!("error: {e}"),
    }

    // Block ending in ; is ()
    let x: () = { let _ = 5 + 3; };
    println!("{x:?}"); // ()
}
```

---

## Arrays — `[T; N]`

**Representation:** `N` contiguous values of type `T` laid out on the stack (or inline wherever the array lives). **The size `N` is part of the type.** `[i32; 3]` and `[i32; 5]` are different, incompatible types. Size is fixed at compile time.

**Invariants:** Always fully initialized. Length never changes. Every element is of the same type `T`.

### Initialisation Forms

```rust
let a: [i32; 3] = [1, 2, 3];        // explicit values
let b = [0u8; 256];                  // 256 zeros — T must be Copy
let c: [&str; 3] = ["a", "b", "c"]; // non-Copy (but literals are Copy)
```

### Key Operations

| Method / Operation       | Returns         | Description                                           |
|--------------------------|-----------------|-------------------------------------------------------|
| `arr[i]`                 | `T` (ref)       | Index access — panics on out-of-bounds                |
| `.get(i)`                | `Option<&T>`    | Safe index access                                     |
| `.len()`                 | `usize`         | Always returns `N` (the compile-time constant)        |
| `.iter()`                | `Iterator<&T>`  | Immutable element iterator                            |
| `.iter_mut()`            | `Iterator<&mut T>` | Mutable element iterator                           |
| `.map(|x| ...)`          | `[U; N]`        | Transform every element (returns a new array)         |
| `.contains(&val)`        | `bool`          | True if any element equals `val`                      |
| `arr.sort()`             | `()`            | Sorts in place (via deref to slice `&mut [T]`)        |
| `arr.sort_by(cmp)`       | `()`            | Sort with a custom comparator                         |
| `&arr` coerces to `&[T]` | —               | All slice methods become available via deref coercion |

### Examples

```rust
fn main() {
    // Fixed-size buffer
    let mut buf = [0u8; 8];
    buf[0] = 0xDE;
    buf[1] = 0xAD;
    println!("{buf:?}"); // [222, 173, 0, 0, 0, 0, 0, 0]

    // Lookup table — days of the week
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    println!("day 3: {}", days[2]); // Wed

    // Safe get
    let idx = 10_usize;
    match days.get(idx) {
        Some(d) => println!("day: {d}"),
        None    => println!("no such day"), // prints this
    }

    // map — square each element
    let nums = [1_i32, 2, 3, 4, 5];
    let squares = nums.map(|x| x * x);
    println!("{squares:?}"); // [1, 4, 9, 16, 25]

    // sort (via deref to slice)
    let mut scores = [88, 42, 95, 70, 55];
    scores.sort();
    println!("{scores:?}"); // [42, 55, 70, 88, 95]

    // contains
    println!("{}", days.contains(&"Sat")); // true

    // Passing to a function that takes a slice
    fn sum(data: &[i32]) -> i32 { data.iter().sum() }
    println!("sum = {}", sum(&nums)); // 15
}
```

---

## Tuples — `(T1, T2, ...)`

**Representation:** Fields laid out contiguously in memory (with alignment padding between fields as needed). Each field can have a different type. The number of fields is part of the type — `(i32, bool)` and `(i32, bool, f64)` are different types.

**Invariants:** Always fully initialized. Length and field types are fixed at compile time. Rust's standard traits (`Debug`, `Clone`, `PartialEq`, etc.) are implemented for tuples up to 12 elements.

### Access and Destructuring

```rust
let pair = (42_i32, true);
let first  = pair.0; // 42
let second = pair.1; // true

let triple = (1.0_f64, "hello", 99_u8);
let (x, label, count) = triple; // destructuring

// Ignore fields with _
let (a, _, c) = triple;
```

### Common Use Cases

| Pattern                        | Example                                      |
|--------------------------------|----------------------------------------------|
| Multiple return values         | `fn divmod(a: i32, b: i32) -> (i32, i32)`   |
| `.enumerate()` on iterators    | Yields `(usize, &T)`                         |
| `.zip()` on two iterators      | Yields `(A, B)` pairs                        |
| Key-value pairs                | `(key, value)` before `HashMap` involvement  |
| Swapping two values            | `(a, b) = (b, a);` (in nightly); or manually|

### Limitations

- No methods on arbitrary tuples — use a struct when you need named fields or `impl` blocks.
- Tuple indexing (`.0`, `.1`) is compile-time; you cannot index with a runtime variable.
- Traits are only auto-implemented up to 12-element tuples in the standard library.

### Examples

```rust
fn divmod(a: i32, b: i32) -> (i32, i32) {
    (a / b, a % b)
}

fn minmax(data: &[i32]) -> (i32, i32) {
    let mut lo = data[0];
    let mut hi = data[0];
    for &x in &data[1..] {
        if x < lo { lo = x; }
        if x > hi { hi = x; }
    }
    (lo, hi)
}

fn main() {
    // Multiple return values
    let (quotient, remainder) = divmod(17, 5);
    println!("{quotient} remainder {remainder}"); // 3 remainder 2

    // Destructuring in a loop with enumerate
    let fruits = ["apple", "banana", "cherry"];
    for (idx, fruit) in fruits.iter().enumerate() {
        println!("[{idx}] {fruit}");
    }

    // zip two iterators
    let names = ["Alice", "Bob", "Carol"];
    let scores = [95, 87, 91];
    for (name, score) in names.iter().zip(scores.iter()) {
        println!("{name}: {score}");
    }

    // Coordinates
    let origin: (f64, f64) = (0.0, 0.0);
    let point:  (f64, f64) = (3.0, 4.0);
    let dist = ((point.0 - origin.0).powi(2) + (point.1 - origin.1).powi(2)).sqrt();
    println!("distance from origin: {dist}"); // 5

    // minmax
    let data = [3, 1, 4, 1, 5, 9, 2, 6];
    let (lo, hi) = minmax(&data);
    println!("min={lo}, max={hi}"); // min=1, max=9

    // Tuple as function argument
    fn describe(point: (i32, i32)) {
        let (x, y) = point;
        println!("point at ({x}, {y})");
    }
    describe((10, -3));
}
```

---

## Quick Reference Card

| Type        | Size     | Key Trait | Default  | Notes                                     |
|-------------|----------|-----------|----------|-------------------------------------------|
| `i8`        | 1 byte   | Copy      | —        | -128 to 127                               |
| `i16`       | 2 bytes  | Copy      | —        | -32768 to 32767                           |
| `i32`       | 4 bytes  | Copy      | ✓ int    | Default integer type                      |
| `i64`       | 8 bytes  | Copy      | —        | Timestamps, large counts                  |
| `i128`      | 16 bytes | Copy      | —        | Cryptography, 128-bit math                |
| `isize`     | arch     | Copy      | —        | Signed pointer-sized                      |
| `u8`        | 1 byte   | Copy      | —        | Bytes, raw data                           |
| `u16`       | 2 bytes  | Copy      | —        | Port numbers, Unicode code units          |
| `u32`       | 4 bytes  | Copy      | —        | IPv4, pixel colours                       |
| `u64`       | 8 bytes  | Copy      | —        | Hashes, file sizes                        |
| `u128`      | 16 bytes | Copy      | —        | UUIDs, 128-bit hashes                     |
| `usize`     | arch     | Copy      | —        | Lengths, indices                          |
| `f32`       | 4 bytes  | Copy      | —        | GPU/SIMD, ~7 sig. digits                  |
| `f64`       | 8 bytes  | Copy      | ✓ float  | Default float, ~15 sig. digits            |
| `bool`      | 1 byte   | Copy      | —        | true / false only                         |
| `char`      | 4 bytes  | Copy      | —        | Unicode scalar value                      |
| `&str`      | 2×usize  | Copy      | —        | Fat pointer, borrowed UTF-8               |
| `String`    | 3×usize  | Clone     | —        | Heap-owned, growable UTF-8                |
| `()`        | 0 bytes  | Copy      | —        | Unit, zero-sized                          |
| `[T; N]`    | N×sizeof(T) | Copy if T: Copy | — | Fixed stack array              |
| `(T1, T2)`  | sizeof each | Copy if all Copy | — | Heterogeneous fixed record    |

---

*End of PRIMITIVES.md — Rust Zero-to-Hero Curriculum*
