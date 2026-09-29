# Day 03: Functions and Structs

> **Project:** Contact Card — a `ContactCard` struct with a `new()` constructor, `greet()` and `is_adult()` methods; create three contacts and demonstrate them with debug printing

## Learning Objectives

By the end of today you will be able to:
- Define functions with parameters and return types, and explain why forward declarations are not needed
- Distinguish expressions from statements and use implicit returns correctly
- Define a `struct` with named fields and explain when to group data this way
- Implement methods on a struct using an `impl` block, including `&self`, `&mut self`, and associated functions
- Use `#[derive(Debug)]` to print struct values with `{:?}` and `{:#?}`
- Return `Result<T, E>` from a function and handle both `Ok` and `Err` variants with `match`

---

## Concepts

### 1. Defining Functions

> **Docs:** [The Rust Book — Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html) · [Reference — Functions](https://doc.rust-lang.org/reference/items/functions.html)

You have already seen `fn main()`. Every other function you write follows the same pattern: `fn name(param: Type, ...) -> ReturnType { body }`. A few things that might surprise you coming from other languages:

**You do not need forward declarations.** In C or C++, a function must be declared before it is called. Rust has no such requirement — you can define `main` first and all your helpers after it, and the compiler resolves them without complaint. This means you can organise code in any order that reads well.

**Types are always explicit in function signatures.** Inside a function body, Rust can infer types. In function parameters and return types, you must write them out. This is intentional: function signatures serve as documentation, and requiring types makes that documentation precise and compiler-enforced.

**Multiple parameters** are separated by commas, each with its own `: Type` annotation. There is no shorthand for "two parameters of the same type" — write each out fully.

```rust
// A function that takes two i32s and returns their sum
fn add(a: i32, b: i32) -> i32 {
    a + b // implicit return — no semicolon, no `return` keyword
}

// A function that takes a &str and an i32
fn greet_user(name: &str, times: i32) {
    // No -> means the function returns () (unit — i.e., nothing)
    for _ in 0..times {
        println!("Hello, {}!", name);
    }
}

fn main() {
    let sum = add(3, 4);
    println!("3 + 4 = {}", sum);

    greet_user("Alice", 2);
}
```

#### Exercise 1.1 — Area Calculator

**Goal:** Define and call a function with two parameters and a return value.

Write a function `rectangle_area(width: f64, height: f64) -> f64` that returns the area of a rectangle. Call it from `main` with at least two different pairs of dimensions and print each result.

```rust
fn rectangle_area(width: f64, height: f64) -> f64 {
    // your code here
}

fn main() {
    // call rectangle_area with (5.0, 3.0) and (10.0, 2.5)
}
```

**Expected output:**
```
Area of 5 x 3 rectangle: 15
Area of 10 x 2.5 rectangle: 25
```

> **Hint:** The formula is simply `width * height`. No `return` keyword needed — just write the expression as the last line without a semicolon.

---

### 2. Expressions vs. Statements

> **Docs:** [The Rust Book — Statements and Expressions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html#statements-and-expressions) · [Reference — Statements](https://doc.rust-lang.org/reference/statements.html)

This distinction trips up nearly every Rust beginner and is worth spending time on.

A **statement** performs an action and does not produce a value. `let x = 5;` is a statement — the semicolon at the end signals "this is a complete action, discard any value." Function definitions are also statements.

An **expression** evaluates to a value. `5 + 3` is an expression. A block `{ ... }` is an expression — it evaluates to the value of its last expression. An `if`/`else` is an expression. A `match` is an expression. This is why you can write `let y = if cond { 1 } else { 2 };` — the whole `if`/`else` is an expression that produces a value.

The rule: **adding a semicolon to an expression turns it into a statement** by discarding its value. This is the most common source of the confusing error `expected X, found ()` — you accidentally added a semicolon to what should have been a returning expression.

```rust
fn implicit_return(x: i32) -> i32 {
    x * 2 // No semicolon: this expression's value is returned
}

fn explicit_return(x: i32) -> i32 {
    if x > 0 {
        return x; // early return with the `return` keyword
    }
    -x // implicit return for the non-early path
}

fn block_expression() -> i32 {
    // A block is an expression; it evaluates to its last expression
    let y = {
        let a = 3;
        let b = 4;
        a * a + b * b // 9 + 16 = 25
    };
    y // returns 25
}

fn main() {
    println!("{}", implicit_return(6));     // 12
    println!("{}", explicit_return(-3));    // 3
    println!("{}", block_expression());     // 25
}
```

#### Exercise 2.1 — Spot the Bug

**Goal:** Understand how a stray semicolon breaks return type inference.

The function below has a bug. Figure out what is wrong by reading the error message from `cargo run`, explain in a comment what went wrong, and fix it.

```rust
fn double(x: i32) -> i32 {
    x * 2; // BUG: why does the compiler complain about this?
}

fn main() {
    println!("{}", double(5));
}
```

**Expected output (after the fix):**
```
10
```

> **Hint:** The semicolon turns the expression into a statement, making the function return `()` instead of `i32`. Remove the semicolon to let the expression's value propagate as the return value.

---

### 3. Defining Structs

> **Docs:** [The Rust Book — Defining Structs](https://doc.rust-lang.org/book/ch05-01-defining-structs.html) · [Reference — Struct types](https://doc.rust-lang.org/reference/types/struct.html)

A **struct** (short for "structure") groups related data under a single name. Think of it like a row in a database table, or a simple object without behaviour (we add behaviour with `impl` in the next section). Each piece of data is called a **field**, and each field has a name and a type.

Structs solve a practical problem: if a function needs to work with, say, a person's name, email, and age, you could pass three separate parameters — but that becomes unwieldy. Wrapping them in a struct gives the group a meaningful name, makes the function signature cleaner, and keeps related data together.

When you create a struct value (instantiate it), you list each field and its value in `FieldName: value` pairs. The **field init shorthand** lets you skip the `FieldName:` part when a local variable has exactly the same name as the field.

```rust
// Define the struct (think of this as the blueprint)
struct Point {
    x: f64,
    y: f64,
}

// A richer struct
struct Person {
    name: String,
    age: u32,
    active: bool,
}

fn main() {
    // Instantiate Point — must provide all fields
    let origin = Point { x: 0.0, y: 0.0 };
    let p = Point { x: 3.0, y: 4.5 };

    // Access fields with `.`
    println!("origin: ({}, {})", origin.x, origin.y);
    println!("p: ({}, {})", p.x, p.y);

    // Field init shorthand
    let name = String::from("Alice");
    let age = 30;
    let active = true;
    let alice = Person { name, age, active }; // shorthand: same as name: name, age: age, etc.

    println!("{} is {} years old", alice.name, alice.age);
}
```

#### Exercise 3.1 — Rectangle Struct

**Goal:** Define a struct and access its fields.

Define a struct `Rectangle` with `width: f64` and `height: f64` fields. Write a function `area(rect: Rectangle) -> f64` that computes and returns its area. Create two `Rectangle` values in `main` and print each area. (Do not worry about ownership of the struct yet — Day 4 covers that fully.)

**Expected output:**
```
Rectangle 1 area: 15
Rectangle 2 area: 25
```

> **Hint:** Pass the struct to the function using the struct's name. The field access syntax `rect.width` and `rect.height` works inside the function.

---

### 4. `impl` Blocks — Adding Methods

> **Docs:** [The Rust Book — Method Syntax](https://doc.rust-lang.org/book/ch05-03-method-syntax.html) · [Reference — Implementations](https://doc.rust-lang.org/reference/items/implementations.html)

An **`impl` block** is where you attach behaviour to a struct. The functions inside an `impl` block are called **methods** when they take `self` as their first parameter.

There are three self flavours:
- `&self` — borrows the struct immutably. The method can read fields but not change them. Most methods use this.
- `&mut self` — borrows the struct mutably. The method can change fields.
- `self` — takes ownership of the struct. The struct is consumed after this call. Used rarely.

Functions in `impl` that do *not* take any form of `self` are called **associated functions** (or sometimes "static methods" in other languages). `new()` is the conventional name for an associated function that constructs and returns an instance of the struct. Call them with `StructName::function_name(...)` (using `::` not `.`).

**`Self` (capital S)** is a type alias for the type currently being implemented. Inside `impl Circle`, `Self` means `Circle`. Writing `-> Self` instead of `-> Circle` keeps the code correct if you ever rename the struct, and is the idiomatic choice for constructors and builder-pattern methods.

```rust
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn origin() -> Self {        // Self = Point here
        Self { x: 0.0, y: 0.0 } // Self { ... } constructs a Point
    }

    fn translate(&self, dx: f64, dy: f64) -> Self {
        Self { x: self.x + dx, y: self.y + dy }
    }
}
```

```rust
struct Circle {
    radius: f64,
}

impl Circle {
    // Associated function — called as Circle::new(...)
    fn new(radius: f64) -> Circle {
        Circle { radius }
    }

    // Method — called on an instance as circle.area()
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    // Mutable method — changes the struct
    fn scale(&mut self, factor: f64) {
        self.radius *= factor;
    }

    fn describe(&self) -> String {
        format!("Circle with radius {:.2}", self.radius)
    }
}

fn main() {
    let mut c = Circle::new(5.0);
    println!("{}", c.describe());
    println!("Area: {:.4}", c.area());
    println!("Circumference: {:.4}", c.circumference());

    c.scale(2.0);
    println!("After scaling: {}", c.describe());
}
```

#### Exercise 4.1 — Rectangle Methods

**Goal:** Add an `impl` block to your `Rectangle` struct from Exercise 3.1.

Add these methods: `new(width: f64, height: f64) -> Rectangle`, `area(&self) -> f64`, `perimeter(&self) -> f64`, `is_square(&self) -> bool`. Create two rectangles using `Rectangle::new(...)` and call all methods on each.

**Expected output:**
```
Rectangle(5 x 3): area=15, perimeter=16, square=false
Rectangle(4 x 4): area=16, perimeter=16, square=true
```

> **Hint:** Perimeter is `2 * (width + height)`. For `is_square`, compare `self.width == self.height` and return the boolean directly.

---

### 5. `#[derive(Debug)]` and Debug Printing

> **Docs:** [`std::fmt::Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html) · [Reference — Derive](https://doc.rust-lang.org/reference/attributes/derive.html) · [Rust by Example — Debug](https://doc.rust-lang.org/rust-by-example/hello/print/print_debug.html)

By default, Rust does not know how to print your custom types. If you try `println!("{}", my_struct)` you get a compile error because the `Display` trait (the `{}` format) is not implemented. The easy shortcut for development and debugging is to derive the `Debug` trait.

Add `#[derive(Debug)]` on the line immediately before your `struct` definition. This attribute tells the compiler to automatically generate a debug representation. Then use `{:?}` for a compact one-line format or `{:#?}` for a pretty-printed multi-line format.

The `#[...]` syntax is called an **attribute** in Rust. Attributes annotate items (functions, structs, modules) with extra instructions for the compiler. `derive` is a built-in attribute that automatically implements specified traits.

```rust
#[derive(Debug)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Debug)]
struct Palette {
    name: String,
    primary: Color,
    secondary: Color,
}

fn main() {
    let red = Color { red: 255, green: 0, blue: 0 };
    let blue = Color { red: 0, green: 0, blue: 255 };

    let palette = Palette {
        name: String::from("Classic"),
        primary: red,
        secondary: blue,
    };

    // Compact debug format — all on one line
    println!("{:?}", palette.primary);

    // Pretty-printed debug format — indented, one field per line
    println!("{:#?}", palette);
}
```

**Expected output:**
```
Color { red: 255, green: 0, blue: 0 }
Palette {
    name: "Classic",
    primary: Color {
        red: 0,
        green: 0,
        blue: 255,
    },
    secondary: Color {
        red: 0,
        green: 0,
        blue: 255,
    },
}
```

#### Exercise 5.1 — Debug Your Rectangle

**Goal:** Add `#[derive(Debug)]` to your `Rectangle` struct and print instances with `{:#?}`.

Add the derive attribute to `Rectangle`, then in `main` print two rectangles using `{:#?}` before calling their methods.

**Expected output:**
```
Rectangle {
    width: 5.0,
    height: 3.0,
}
```

> **Hint:** Just add `#[derive(Debug)]` on the line before `struct Rectangle {`. No changes to the struct body or `impl` block are needed.

---

### 6. `Result<T, E>` — Functions That Can Fail

> **Docs:** [`std::result::Result`](https://doc.rust-lang.org/std/result/enum.Result.html) · [The Rust Book — Recoverable Errors with Result](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)

Now that you know how to write functions, you can write functions that *fail* in a meaningful way. `Result<T, E>` is Rust's built-in type for expressing recoverable failure. It is an enum with two variants:

- `Ok(value)` — the function succeeded, carrying a value of type `T`
- `Err(error)` — the function failed, carrying an error of type `E`

The caller is forced to handle both cases — the compiler will warn if a `Result` is ignored. This makes errors visible in function signatures and impossible to silently swallow. Compare to exceptions (invisible in the signature) or returning `-1` as an error sentinel (easy to miss).

```rust
// A function that can fail returns Result<SuccessType, ErrorType>
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("cannot divide by zero"))
    } else {
        Ok(a / b)
    }
}

fn parse_age(s: &str) -> Result<u32, String> {
    // str::parse returns Result; map_err converts the error to our error type
    s.trim()
     .parse::<u32>()
     .map_err(|e| format!("'{}' is not a valid age: {}", s.trim(), e))
}

fn main() {
    // Pattern match to handle both variants
    match divide(10.0, 2.0) {
        Ok(result) => println!("10 / 2 = {}", result),
        Err(e)     => println!("Error: {}", e),
    }

    match divide(10.0, 0.0) {
        Ok(result) => println!("Result: {}", result),
        Err(e)     => println!("Error: {}", e),     // this branch runs
    }

    // unwrap() extracts Ok or panics on Err — fine in examples, avoid in production
    let age = parse_age("25").unwrap();
    println!("Age: {}", age);

    // is_ok() / is_err() for quick checks without consuming the value
    println!("'abc' is a valid age: {}", parse_age("abc").is_ok());
}
```

`Result` is not just for I/O. Any function with a precondition that might be violated is a candidate: parsing, validation, lookups that might miss. Day 09 goes deeper — combinators (`map`, `and_then`), the `?` operator for propagation, and building proper error types.

#### Exercise 6.1 — Validated Constructor

**Goal:** Add a `new` constructor to a struct that validates its input and returns `Result`.

Write `struct Rectangle { width: f64, height: f64 }` with `fn new(width: f64, height: f64) -> Result<Rectangle, String>` that returns `Err` if either dimension is `<= 0.0`. Call it from `main` with valid and invalid inputs and match on the result.

**Expected output:**
```
Created: Rectangle { width: 5.0, height: 3.0 }
Error: width must be positive, got -1
```

> **Hint:** Return `Err(format!("width must be positive, got {}", width))` when the check fails, and `Ok(Rectangle { width, height })` when it passes.

---

## Day Project: Contact Card

### What You're Building

A `ContactCard` struct that represents a person's contact information, complete with a constructor, methods, and debug printing. You will create three different contacts and call methods on each one. This project ties together everything from today: struct definition, field access, `impl` with multiple methods (including an associated function), and debug printing. It also previews a topic from tomorrow: storing a `String` inside a struct requires understanding ownership, so pay attention to how `String::from(...)` is used.

### Requirements

1. Define `struct ContactCard` with three fields: `name: String`, `email: String`, `age: u32`.
2. Add `#[derive(Debug)]` to the struct.
3. In an `impl ContactCard` block, implement:
   - `new(name: &str, email: &str, age: u32) -> ContactCard` — constructor that converts `&str` to `String` using `.to_string()` or `String::from(...)`.
   - `greet(&self) -> String` — returns a greeting string like `"Hi, I'm Alice and I can be reached at alice@example.com"`.
   - `is_adult(&self) -> bool` — returns `true` if `age >= 18`.
4. Create three `ContactCard` instances using `ContactCard::new(...)`.
5. Print each with `{:#?}`.
6. Call `greet()` and `is_adult()` on each and print the results.

### Getting Started

```rust
#[derive(Debug)]
struct ContactCard {
    name: String,
    email: String,
    age: u32,
}

impl ContactCard {
    fn new(name: &str, email: &str, age: u32) -> ContactCard {
        ContactCard {
            name: name.to_string(),
            email: email.to_string(),
            age,
        }
    }

    fn greet(&self) -> String {
        format!("Hi, I'm {} and I can be reached at {}", self.name, self.email)
    }

    fn is_adult(&self) -> bool {
        self.age >= 18
    }
}

fn main() {
    let alice = ContactCard::new("Alice", "alice@example.com", 30);
    let bob = ContactCard::new("Bob", "bob@example.com", 16);
    let carol = ContactCard::new("Carol", "carol@example.com", 42);

    println!("{:#?}", alice);
    println!("{}", alice.greet());
    println!("Is adult: {}", alice.is_adult());

    println!("---");
    println!("{:#?}", bob);
    println!("{}", bob.greet());
    println!("Is adult: {}", bob.is_adult());

    println!("---");
    println!("{:#?}", carol);
    println!("{}", carol.greet());
    println!("Is adult: {}", carol.is_adult());
}
```

### Running Your Solution

```bash
cargo run -p day-03
```

**Expected output:**
```
ContactCard {
    name: "Alice",
    email: "alice@example.com",
    age: 30,
}
Hi, I'm Alice and I can be reached at alice@example.com
Is adult: true
---
ContactCard {
    name: "Bob",
    email: "bob@example.com",
    age: 16,
}
Hi, I'm Bob and I can be reached at bob@example.com
Is adult: false
---
ContactCard {
    name: "Carol",
    email: "carol@example.com",
    age: 42,
}
Hi, I'm Carol and I can be reached at carol@example.com
Is adult: true
```

### Extension Challenges

- **Easy:** Add a `birthday(&mut self)` method that increments `age` by 1. Call it on Alice and print her age before and after.
- **Medium:** Add a `summary(&self) -> String` method that returns a compact string like `"Alice (30) <alice@example.com>"`. Store three contacts in an array (`let contacts = [alice, bob, carol];`) and use a `for` loop to print all summaries.
- **Hard:** Add a `pub fn from_parts(parts: &[&str]) -> ContactCard` associated function that takes a slice of three string parts (name, email, age-as-string) and parses the age with `parts[2].parse::<u32>().unwrap_or(0)`. This is a preview of error handling and parsing you will develop further in later days.
