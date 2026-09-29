# Day 06: Enums and Pattern Matching

> **Project:** Traffic Light State Machine — a `TrafficLight` enum with transition logic, durations, and a `Display` implementation; simulate 8 transitions from `main`

## Learning Objectives

By the end of today you will be able to:
- Define a basic `enum` and use its variants as values
- Define enums whose variants carry different types of data
- Write exhaustive `match` expressions with destructuring and wildcard patterns
- Use match guards to add conditions to match arms
- Use `if let` for concise single-variant matching
- Explain `Option<T>` and use its common methods instead of null checks

---

## Concepts

### 1. Basic Enums

> **Docs:** [The Rust Book — Defining an Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html) · [Rust by Example — Enums](https://doc.rust-lang.org/rust-by-example/custom_types/enum.html) · [Reference — Enumerations](https://doc.rust-lang.org/reference/items/enumerations.html)

An **enum** (short for enumeration) defines a type that can be one of a fixed set of named variants. Enums are the right tool when a value can be exactly one of several discrete possibilities: a cardinal direction, a day of the week, a coin denomination, or — today's project — a traffic light colour.

In Rust, enums are first-class types. You cannot accidentally mix them with integers or strings — the type system tracks which variant you have. Every variant is part of the enum's namespace: you write `Direction::North`, not just `North` (unless you `use Direction::*;`).

Enums are also `match`able, which is where their real power appears. But first, let's see the basic form.

```rust
// Define an enum with four variants
enum Direction {
    North,
    South,
    East,
    West,
}

fn describe(dir: Direction) {
    // match is how you branch on an enum variant
    match dir {
        Direction::North => println!("Heading north"),
        Direction::South => println!("Heading south"),
        Direction::East  => println!("Heading east"),
        Direction::West  => println!("Heading west"),
    }
}

fn main() {
    let d = Direction::North;
    describe(d);

    // You can use `use` to bring variants into scope without the prefix
    use Direction::*;
    describe(South);
    describe(East);
}
```

#### Exercise 1.1 — Coin Enum

**Goal:** Define an enum and write a function that maps variants to values.

Define `enum Coin { Penny, Nickel, Dime, Quarter }`. Write `fn value_in_cents(coin: Coin) -> u32` that returns `1`, `5`, `10`, or `25` for each variant. Call it with all four variants in `main` and print each result.

```rust
enum Coin { Penny, Nickel, Dime, Quarter }

fn value_in_cents(coin: Coin) -> u32 {
    // match here
    todo!()
}

fn main() {
    println!("Penny:   {}", value_in_cents(Coin::Penny));
    println!("Nickel:  {}", value_in_cents(Coin::Nickel));
    println!("Dime:    {}", value_in_cents(Coin::Dime));
    println!("Quarter: {}", value_in_cents(Coin::Quarter));
}
```

**Expected output:**
```
Penny:   1
Nickel:  5
Dime:    10
Quarter: 25
```

> **Hint:** Each arm of the `match` should look like `Coin::Penny => 1,`. The match expression itself is the return value — no explicit `return` needed.

---

### 2. Enums with Data

> **Docs:** [The Rust Book — Defining an Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html) · [Rust by Example — Enums](https://doc.rust-lang.org/rust-by-example/custom_types/enum.html)

Rust's enums go far beyond a simple list of names. Each variant can carry its own data — and different variants can carry different data. This is sometimes called a **sum type** or **tagged union** in programming language theory. It is one of Rust's most powerful features.

Think of a `Shape` that can be either a `Circle` (described by one radius), a `Rectangle` (described by width and height), or a `Triangle` (described by three sides). Each variant has a different "shape" of data. With an enum, you group all of these under one type name, and the compiler ensures you handle all cases.

Variants can carry: nothing (unit variant, like `Direction::North`), a single unnamed value (`Circle(f64)`), multiple unnamed values (`Rectangle(f64, f64)`), or named fields (`Rectangle { width: f64, height: f64 }`).

```rust
enum Shape {
    Circle(f64),                          // one unnamed field: radius
    Rectangle(f64, f64),                  // two unnamed fields: width, height
    Triangle { base: f64, height: f64 },  // named fields
}

fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle(r) => std::f64::consts::PI * r * r,
        Shape::Rectangle(w, h) => w * h,
        Shape::Triangle { base, height } => 0.5 * base * height,
    }
}

fn main() {
    let shapes = vec![
        Shape::Circle(3.0),
        Shape::Rectangle(4.0, 6.0),
        Shape::Triangle { base: 5.0, height: 8.0 },
    ];

    for shape in &shapes {
        println!("Area: {:.2}", area(shape));
    }
}
```

#### Exercise 2.1 — Message Enum

**Goal:** Define an enum with variants that carry different data types.

Define `enum Message` with variants: `Quit` (no data), `Move { x: i32, y: i32 }` (named fields), `Write(String)` (one string), and `ChangeColour(u8, u8, u8)` (three u8 values — RGB). Write `fn process(msg: Message)` that uses `match` and prints what each message does. Call it with all four variants.

**Expected output:**
```
Quit: shutting down
Move to (10, -5)
Write: "hello from enum"
Colour changed to rgb(255, 128, 0)
```

> **Hint:** For the named-field variant, destructure it in the match arm as `Message::Move { x, y } => ...`. For the tuple variant, use `Message::ChangeColour(r, g, b) => ...`.

---

### 3. `match` — Exhaustive Pattern Matching

> **Docs:** [The Rust Book — The `match` Control Flow Construct](https://doc.rust-lang.org/book/ch06-02-match.html) · [Reference — match expressions](https://doc.rust-lang.org/reference/expressions/match-expr.html) · [Rust by Example — match](https://doc.rust-lang.org/rust-by-example/flow_control/match.html)

`match` is Rust's most powerful control flow construct. It works like `switch` in other languages, but with three critical differences:

1. **It is exhaustive** — the compiler requires you to handle every possible case. If you forget a variant, the code does not compile. This eliminates entire classes of bugs.
2. **It is an expression** — a `match` produces a value, just like `if`.
3. **It destructures** — match arms can unpack the data inside enum variants, binding the inner values to names.

The `_` wildcard pattern matches anything and discards the value. It is the idiomatic catch-all when you do not need to name what you matched.

```rust
enum Command {
    Quit,
    Move(i32, i32),
    Print(String),
    Volume(u8),
}

fn execute(cmd: Command) -> String {
    match cmd {
        Command::Quit => String::from("Quitting"),
        Command::Move(x, y) => format!("Moving to ({}, {})", x, y),
        Command::Print(text) => format!("Printing: {}", text),
        Command::Volume(level) => format!("Volume set to {}", level),
    }
}

fn classify_number(n: i32) -> &'static str {
    match n {
        i32::MIN..=-1 => "negative",
        0             => "zero",
        1..=9         => "single digit",
        10..=99       => "double digit",
        _             => "large",
    }
}

fn main() {
    println!("{}", execute(Command::Move(5, -3)));
    println!("{}", execute(Command::Print(String::from("hello"))));

    for n in [-5, 0, 7, 42, 200] {
        println!("{} is {}", n, classify_number(n));
    }
}
```

#### Exercise 3.1 — Shape Perimeter

**Goal:** Add a `perimeter` function to the `Shape` enum from Section 2.

Write `fn perimeter(shape: &Shape) -> f64` using `match`. For `Circle`, perimeter is `2 * PI * r`. For `Rectangle`, it is `2 * (w + h)`. For `Triangle { base, height }`, add a third field `hypotenuse` to the variant and compute `base + height + hypotenuse`.

**Expected output:**
```
Circle perimeter:    18.85
Rectangle perimeter: 20.00
Triangle perimeter:  22.00
```

> **Hint:** You will need to add the `hypotenuse` field to the `Triangle` variant and update both the `area` function and any existing `Triangle` instantiation to include it.

---

### 4. Match Guards

> **Docs:** [The Rust Book — Patterns and Matching](https://doc.rust-lang.org/book/ch18-00-patterns.html) · [Reference — match expressions](https://doc.rust-lang.org/reference/expressions/match-expr.html)

A **match guard** adds an extra `if` condition to a match arm. The arm only matches if both the pattern matches and the guard condition is true. This is useful when you need finer-grained control than pattern shapes alone can provide.

```rust
fn describe_number(n: i32) -> &'static str {
    match n {
        // Match guard: arm only fires if n > 0 AND n % 2 == 0
        n if n > 0 && n % 2 == 0 => "positive even",
        n if n > 0                => "positive odd",
        0                         => "zero",
        n if n % 2 == 0           => "negative even",
        _                         => "negative odd",
    }
}

enum Temperature {
    Celsius(f64),
    Fahrenheit(f64),
}

fn comfort_level(temp: &Temperature) -> &'static str {
    match temp {
        Temperature::Celsius(c) if *c < 0.0  => "freezing",
        Temperature::Celsius(c) if *c < 15.0 => "cold",
        Temperature::Celsius(c) if *c < 25.0 => "comfortable",
        Temperature::Celsius(_)              => "hot",
        Temperature::Fahrenheit(f) if *f < 32.0 => "freezing",
        Temperature::Fahrenheit(f) if *f < 60.0 => "cold",
        Temperature::Fahrenheit(f) if *f < 77.0 => "comfortable",
        Temperature::Fahrenheit(_)              => "hot",
    }
}

fn main() {
    for n in [-3, 0, 4, 7] {
        println!("{}: {}", n, describe_number(n));
    }

    let t = Temperature::Celsius(18.5);
    println!("18.5°C feels {}", comfort_level(&t));
}
```

#### Exercise 4.1 — Fizz Buzz with Match Guards

**Goal:** Rewrite FizzBuzz using a `match` expression with guards.

Write a `match` on a number `n` that produces `"FizzBuzz"` if divisible by both 3 and 5, `"Fizz"` if divisible by 3 only, `"Buzz"` if divisible by 5 only, and the number itself otherwise. Use it in a loop to print results for 1 through 20.

**Expected output:**
```
1 2 Fizz 4 Buzz Fizz 7 8 Fizz Buzz 11 Fizz 13 14 FizzBuzz 16 17 Fizz 19 Buzz
```

> **Hint:** Match on `n` with guards like `n if n % 15 == 0 => "FizzBuzz"`. Make sure divisible-by-15 comes first — guards are checked top to bottom.

---

### 5. `if let` — Concise Single-Variant Matching

> **Docs:** [The Rust Book — Concise Control Flow with `if let`](https://doc.rust-lang.org/book/ch06-03-if-let.html) · [Reference — if expressions](https://doc.rust-lang.org/reference/expressions/if-expr.html)

Sometimes you only care about one variant of an enum and want to do nothing for all others. Writing a full `match` with a `_ => {}` catch-all is technically correct but verbose. `if let` is syntactic sugar for exactly this pattern: "if the value matches this one pattern, extract the data and run this block."

Use `if let` when you are only handling one case and ignoring the rest. Use `match` when you care about multiple cases, or when exhaustiveness checking is valuable.

```rust
enum Colour {
    Red,
    Green,
    Blue,
    Custom(u8, u8, u8),
}

fn main() {
    let c = Colour::Custom(100, 200, 50);

    // Full match — works, but verbose when you only care about one arm
    match &c {
        Colour::Custom(r, g, b) => println!("Custom colour: rgb({}, {}, {})", r, g, b),
        _ => {} // do nothing for other variants
    }

    // if let — more concise for a single pattern
    if let Colour::Custom(r, g, b) = &c {
        println!("Custom colour via if let: rgb({}, {}, {})", r, g, b);
    }

    // if let with else — combines with an else block for the non-matching case
    if let Colour::Red = &c {
        println!("It's red!");
    } else {
        println!("Not red.");
    }
}
```

#### Exercise 5.1 — Optional Greeting

**Goal:** Use `if let` to unpack an `Option<String>` and print the greeting if present.

Write a function `maybe_greet(name: Option<String>)` that uses `if let` to print `"Hello, NAME!"` if `name` is `Some(...)`, or `"Hello, stranger!"` if it is `None`. Call it twice: once with `Some(String::from("Alice"))` and once with `None`.

**Expected output:**
```
Hello, Alice!
Hello, stranger!
```

> **Hint:** `if let Some(name) = name { ... } else { ... }` is the pattern. No need for a full `match`.

#### `ref` — Binding by Reference in Patterns

When you pattern-match a value, Rust normally moves or copies it out of the match. `ref` tells the pattern to bind the matched value *by reference* instead of by value, giving you a `&T` without consuming the original.

In modern Rust (edition 2018+) **match ergonomics** handles most cases automatically: matching on `&val` often doesn't require an explicit `ref`. You'll encounter `ref` mainly in older code or in situations where the compiler cannot infer the reference automatically.

```rust
fn main() {
    let name = String::from("Alice");

    // Without ref — this would MOVE name into the pattern:
    // match name { n => println!("{}", n) }  // name is gone after here

    // With ref — bind by reference, name stays valid
    match name {
        ref n => println!("Hello, {}!", n), // n is &String
    }
    println!("name is still: {}", name); // still accessible

    // In practice, matching on a reference achieves the same thing:
    match &name {
        n => println!("Also works: {}", n), // n is &&String, auto-deref'd
    }
}
```

**Rule of thumb:** Prefer matching on `&value` over using `ref` in new code. Reserve `ref` for cases where matching on a reference isn't possible (e.g. matching a field of a struct you don't own).

---

### 6. `Option<T>` — Rust's Answer to Null

> **Docs:** [`std::option::Option`](https://doc.rust-lang.org/std/option/enum.Option.html) · [The Rust Book — Defining an Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html)

Most languages have `null` (or `nil` or `None` in Python). The problem with null is that any variable might be null, and forgetting to check causes runtime crashes. Rust eliminates null entirely and replaces it with `Option<T>`: a built-in enum that is either `Some(value)` (containing a value of type `T`) or `None` (containing nothing).

Because `Option<T>` is a regular enum, you cannot accidentally use a `Some` value as if it were a `T` — the type system forces you to unwrap it first, making the "check for null" mandatory rather than optional.

```rust
fn find_first_even(numbers: &[i32]) -> Option<i32> {
    for &n in numbers {
        if n % 2 == 0 {
            return Some(n); // wrap the value in Some
        }
    }
    None // no even number found
}

fn main() {
    let data = vec![1, 3, 7, 4, 9];
    let empty: Vec<i32> = vec![1, 3, 5];

    // Pattern matching — the safe, explicit way
    match find_first_even(&data) {
        Some(n) => println!("First even: {}", n),
        None    => println!("No even numbers"),
    }

    // unwrap_or — provide a fallback value
    let result = find_first_even(&empty).unwrap_or(-1);
    println!("Result: {}", result);

    // is_some / is_none — check without extracting
    println!("Found even: {}", find_first_even(&data).is_some());

    // map — transform the inner value if present
    let doubled = find_first_even(&data).map(|n| n * 2);
    println!("Doubled first even: {:?}", doubled); // Some(8)

    // unwrap — use only when you are CERTAIN it is Some; panics on None
    // let dangerous = find_first_even(&empty).unwrap(); // would panic!
}
```

#### Exercise 6.1 — Safe Division

**Goal:** Use `Option<f64>` to represent a result that may not exist.

Write `fn safe_divide(a: f64, b: f64) -> Option<f64>` that returns `None` if `b == 0.0`, and `Some(a / b)` otherwise. In `main`, call it with three pairs: `(10.0, 2.0)`, `(7.0, 0.0)`, `(9.0, 3.0)`. Use `match` to print either the result or an error message.

**Expected output:**
```
10 / 2 = 5
7 / 0 = Error: division by zero
9 / 3 = 3
```

> **Hint:** Return `None` with a plain `return None;` for the zero case, and `Some(a / b)` for the normal case. The `match` in `main` should destructure `Some(result)` to get the `f64` value.

---

## Day Project: Traffic Light State Machine

### What You're Building

A `TrafficLight` enum that models the three states of a traffic signal, complete with:
- A `next()` method that transitions to the next state (Green → Yellow → Red → Green)
- A `duration()` method that returns how many seconds each light holds
- A `Display` trait implementation so you can print the light with `{}`

Then in `main`, you simulate 8 transitions, printing the current state and its duration after each change. This project exercises every enum concept from today: basic enum definition, `impl` blocks on enums, `match` for logic dispatch, and trait implementation.

Implementing `Display` is new but straightforward: it is a trait from `std::fmt` with one required method, `fmt`, that writes to a formatter. You will use it again throughout the curriculum.

### Requirements

1. Define `enum TrafficLight` with three variants: `Red`, `Yellow`, `Green`.
2. Implement `fn next(&self) -> TrafficLight` — transitions: `Green → Yellow → Red → Green`.
3. Implement `fn duration(&self) -> u64` — durations: `Red = 60`, `Yellow = 5`, `Green = 45` (seconds).
4. Implement `std::fmt::Display` for `TrafficLight` so `{}` prints `"Red"`, `"Yellow"`, or `"Green"`.
5. In `main`, start with `TrafficLight::Red`. Loop 8 times: print the current state and its duration, then transition to the next state.

### Getting Started

```rust
use std::fmt;

enum TrafficLight {
    Red,
    Yellow,
    Green,
}

impl TrafficLight {
    fn next(&self) -> TrafficLight {
        match self {
            TrafficLight::Green  => TrafficLight::Yellow,
            TrafficLight::Yellow => TrafficLight::Red,
            TrafficLight::Red    => TrafficLight::Green,
        }
    }

    fn duration(&self) -> u64 {
        match self {
            TrafficLight::Red    => 60,
            TrafficLight::Yellow => 5,
            TrafficLight::Green  => 45,
        }
    }
}

impl fmt::Display for TrafficLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TrafficLight::Red    => "Red",
            TrafficLight::Yellow => "Yellow",
            TrafficLight::Green  => "Green",
        };
        write!(f, "{}", name)
    }
}

fn main() {
    let mut light = TrafficLight::Red;

    for step in 1..=8 {
        println!("Step {}: {} ({}s)", step, light, light.duration());
        light = light.next();
    }
}
```

### Running Your Solution

```bash
cargo run -p day-06
```

**Expected output:**
```
Step 1: Red (60s)
Step 2: Green (45s)
Step 3: Yellow (5s)
Step 4: Red (60s)
Step 5: Green (45s)
Step 6: Yellow (5s)
Step 7: Red (60s)
Step 8: Green (45s)
```

### Extension Challenges

- **Easy:** Add a `is_safe_to_cross(&self) -> bool` method that returns `true` only for `Green`. Print this alongside duration in the simulation loop.
- **Medium:** Add `#[derive(Debug, PartialEq)]` to `TrafficLight` and write a test function `fn test_transitions()` (not a `#[test]` yet — just a normal function) that uses `assert_eq!` to verify that `next()` returns the correct variant for each starting state. Call it from `main`.
- **Hard:** Model a full intersection: create a `struct Intersection` holding four `TrafficLight` values (one per road direction — `north`, `south`, `east`, `west`). Implement logic so that `north`/`south` and `east`/`west` alternate: when N/S are Green, E/W are Red, and vice versa, with a Yellow phase in between. Simulate 12 steps and print all four lights each step.
