# Day 10: Traits and Default Implementations

> **Project:** Shape Library — define a `Shape` trait with default methods and implement it for `Circle`, `Rectangle`, and `Triangle` in a library crate.

## Learning Objectives

By the end of today you will be able to:
- Declare a `trait` with required and default method implementations
- Implement standard library traits (`Display`, `PartialEq`) manually and via `#[derive]`
- Write functions that accept any type satisfying a trait bound
- Use `impl Trait` syntax and `where` clauses for readable generic signatures
- Understand when to derive traits vs implement them by hand

> **Note:** Today's project is a **library crate** (`src/lib.rs`). There is no `main`. Run it with `cargo test -p day-10` to verify your implementation.

---

## Concepts

### 1. Declaring a Trait

> **Docs:** [Book — Traits](https://doc.rust-lang.org/book/ch10-02-traits.html) · [Reference — Traits](https://doc.rust-lang.org/reference/items/traits.html) · [Rust by Example — Traits](https://doc.rust-lang.org/rust-by-example/trait.html)

A trait is a contract: any type that implements the trait promises to provide the listed methods. Traits are Rust's primary tool for polymorphism — they let you write code that works across unrelated types without knowing their concrete details.

Think of a trait as an interface from Java or a protocol from Swift. The key difference from those languages is that Rust traits can also carry **default method bodies**, so implementing a trait can mean providing only the customised parts while inheriting the rest for free.

```rust
// A trait with one required method and one default method
pub trait Greet {
    // Required — implementors must provide this
    fn name(&self) -> &str;

    // Default — implementors inherit this unless they override it
    fn hello(&self) -> String {
        format!("Hello, I am {}!", self.name())
    }
}

pub struct Person {
    pub name: String,
}

impl Greet for Person {
    fn name(&self) -> &str {
        &self.name
    }
    // hello() is inherited automatically
}

pub struct Robot {
    pub id: u32,
}

impl Greet for Robot {
    fn name(&self) -> &str {
        "UNIT"
    }

    // Override the default
    fn hello(&self) -> String {
        format!("UNIT-{} ONLINE. GREETINGS.", self.id)
    }
}

fn main() {
    let p = Person { name: "Alice".to_string() };
    let r = Robot { id: 7 };
    println!("{}", p.hello()); // Hello, I am Alice!
    println!("{}", r.hello()); // UNIT-7 ONLINE. GREETINGS.
}
```

#### Exercise 1.1 — Define a Printable Trait

**Goal:** Practice declaring a trait with a required method and a default method.

Define a trait `Summary` with:
- required: `fn summarise(&self) -> String`
- default: `fn preview(&self) -> String` that returns the first 20 characters of `summarise()` followed by `"..."` if longer, or the full string if shorter.

Implement `Summary` for `Article { title: String, body: String }` where `summarise` returns `"title: body"`. Implement it for `Tweet { author: String, content: String }` where `summarise` returns `"@author: content"`. Test both `summarise` and `preview` on instances with a long body.

**Expected output:**
```
Article: Rust 2024 Edition: A new edition of Rust is here with...
Tweet: @ferris: Just rewrote my OS in Rust. No...
```

> **Hint:** Use `s.chars().take(20).collect::<String>()` for the first 20 characters.

---

### 2. Implementing a Trait for a Type

> **Docs:** [Book — Traits](https://doc.rust-lang.org/book/ch10-02-traits.html) · [From/Into](https://doc.rust-lang.org/std/convert/trait.From.html)

The syntax `impl TraitName for TypeName { ... }` attaches the trait to your type. You must provide a body for every required method. If you miss one, the compiler tells you exactly which method is missing — this is a compile-time guarantee.

There is an important rule: you can only implement a trait for a type if at least one of the trait or the type is defined in the current crate. This rule (called the **orphan rule**) prevents two libraries from accidentally providing conflicting implementations.

```rust
use std::fmt;

pub struct Celsius(f64);
pub struct Fahrenheit(f64);

impl fmt::Display for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°C", self.0)
    }
}

impl fmt::Display for Fahrenheit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°F", self.0)
    }
}

impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self {
        Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
    }
}

fn main() {
    let boiling = Celsius(100.0);
    println!("{boiling}"); // 100.0°C
    let boiling_f: Fahrenheit = boiling.into();
    println!("{boiling_f}"); // 212.0°F
}
```

#### Exercise 2.1 — Implement PartialEq Manually

**Goal:** Understand what `#[derive(PartialEq)]` does by writing it yourself.

Define `struct Color { r: u8, g: u8, b: u8 }`. Implement `PartialEq<Color>` for `Color` so that `==` compares all three channels. Do NOT use `#[derive]`. Then demonstrate it in a test or main.

**Expected output:**
```
red == red: true
red == blue: false
```

> **Hint:** `impl PartialEq for Color { fn eq(&self, other: &Self) -> bool { ... } }`.

---

### 3. Default Method Implementations

> **Docs:** [Book — Default implementations](https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations)

Default methods are powerful because they can call the trait's own required methods. This means you can provide rich behaviour "for free" built on a small required surface area — the same pattern used by `Iterator`, which requires only `next()` but provides over 70 adapters built on top of it.

```rust
pub trait Measurable {
    fn meters(&self) -> f64; // required

    fn centimeters(&self) -> f64 {
        self.meters() * 100.0
    }

    fn kilometers(&self) -> f64 {
        self.meters() / 1000.0
    }

    fn describe(&self) -> String {
        format!("{:.2}m / {:.1}cm / {:.4}km",
            self.meters(), self.centimeters(), self.kilometers())
    }
}

pub struct Bridge { pub length_m: f64 }

impl Measurable for Bridge {
    fn meters(&self) -> f64 { self.length_m }
}

fn main() {
    let golden_gate = Bridge { length_m: 2737.0 };
    println!("{}", golden_gate.describe());
    // 2737.00m / 273700.0cm / 2.7370km
}
```

---

### 4. Trait Bounds in Functions

> **Docs:** [Book — Trait bounds](https://doc.rust-lang.org/book/ch10-02-traits.html#trait-bound-syntax) · [std::cmp](https://doc.rust-lang.org/std/cmp/)

To write a function that accepts any type implementing a trait, you use a **trait bound**. There are three equivalent syntaxes — choose the one that reads most clearly for your function's complexity.

```rust
use std::fmt::Display;

pub trait Area {
    fn area(&self) -> f64;
}

// Syntax 1 — inline bound (clearest for one or two bounds)
pub fn print_area<T: Area>(shape: &T) {
    println!("Area: {:.2}", shape.area());
}

// Syntax 2 — impl Trait (cleanest for simple cases, especially arguments)
pub fn print_area_impl(shape: &impl Area) {
    println!("Area: {:.2}", shape.area());
}

// Syntax 3 — where clause (best when bounds grow complex)
pub fn print_area_where<T>(shape: &T)
where
    T: Area + Display,
{
    println!("{shape} has area {:.2}", shape.area());
}

// Multiple bounds with +
pub fn describe<T: Area + Display>(shape: &T) {
    println!("Shape: {shape}, Area: {:.2}", shape.area());
}
```

The difference between `impl Trait` in argument position and a generic `<T: Trait>` is subtle: with `impl Trait`, the caller cannot name the type parameter; with `<T: Trait>`, they can (and can pass the same `T` in multiple positions). For return types, `impl Trait` means "some specific type implementing Trait, decided by the function body" — useful for returning closures or iterators without naming the concrete type.

#### Exercise 4.1 — Bound a Function

**Goal:** Write a function with multiple trait bounds and use it with two different types.

Define a trait `Labelled` with `fn label(&self) -> &str`. Write a function `print_labelled_area<T: Area + Labelled>(shape: &T)` that prints `"[label]: area = X.XX"`. Implement both `Area` and `Labelled` for a `Square { side: f64 }` and a `Circle { radius: f64 }`. Call `print_labelled_area` with both.

**Expected output:**
```
[square]: area = 25.00
[circle]: area = 78.54
```

> **Hint:** `std::f64::consts::PI` gives you pi.

---

### 5. Standard Library Traits — Display, PartialEq, PartialOrd

> **Docs:** [Display](https://doc.rust-lang.org/std/fmt/trait.Display.html) · [Debug](https://doc.rust-lang.org/std/fmt/trait.Debug.html) · [PartialEq](https://doc.rust-lang.org/std/cmp/trait.PartialEq.html) · [PartialOrd](https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html) · [Ord](https://doc.rust-lang.org/std/cmp/trait.Ord.html) · [Default trait](https://doc.rust-lang.org/std/default/trait.Default.html)

The standard library defines dozens of traits you can implement to plug your types into Rust's ecosystem. The most important ones for daily coding:

- `Display` (`use std::fmt`): enables `{}` in format strings. Implement `fn fmt(&self, f: &mut Formatter) -> fmt::Result`.
- `Debug`: enables `{:?}`. Almost always use `#[derive(Debug)]` unless you want custom output.
- `PartialEq` / `Eq`: enables `==` and `!=`. Use `#[derive]` when all fields implement it; implement manually for custom equality logic.
- `PartialOrd` / `Ord`: enables `<`, `>`, `<=`, `>=` and sorting. `#[derive]` compares fields in declaration order.
- `Clone` / `Copy`: controls duplication. Use `#[derive]` for types whose fields are all `Clone`/`Copy`.

```rust
use std::fmt;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Weight {
    pub kg: f64,
}

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1} kg", self.kg)
    }
}

fn main() {
    let a = Weight { kg: 72.5 };
    let b = Weight { kg: 68.0 };
    println!("{a}");                     // 72.5 kg
    println!("a == a: {}", a == a);      // true
    println!("a > b:  {}", a > b);       // true
    println!("{:?}", a.clone());         // Weight { kg: 72.5 }
}
```

---

## Day Project: Shape Library

### What You're Building

A library crate that defines a `Shape` trait with area, perimeter, name, and a default `describe` method. Three types — `Circle`, `Rectangle`, and `Triangle` — implement the trait. A helper function accepts `&impl Shape`. Every shape also implements `Display`. The crate is documented with doc comments.

### Requirements

1. Define `pub mod shapes` containing all types and the trait.
2. `pub trait Shape` with: `fn area(&self) -> f64`, `fn perimeter(&self) -> f64`, `fn name(&self) -> &str`, and a default `fn describe(&self) -> String` returning `"[name]: area=X.XX, perimeter=Y.YY"`.
3. `pub struct Circle { pub radius: f64 }` — area = π r², perimeter = 2πr.
4. `pub struct Rectangle { pub width: f64, pub height: f64 }` — area = w×h, perimeter = 2(w+h).
5. `pub struct Triangle { pub a: f64, pub b: f64, pub c: f64 }` — use Heron's formula for area, perimeter = a+b+c.
6. Implement `Shape` for all three. Implement `std::fmt::Display` for all three.
7. Write `pub fn print_shape_info(shape: &impl Shape)` that prints the `describe()` output.
8. Include `///` doc comments on the trait and each struct.

### Getting Started

```rust
// src/lib.rs
pub mod shapes {
    use std::fmt;
    use std::f64::consts::PI;

    /// A geometric shape with computable area and perimeter.
    pub trait Shape {
        fn area(&self) -> f64;
        fn perimeter(&self) -> f64;
        fn name(&self) -> &str;

        fn describe(&self) -> String {
            format!(
                "[{}]: area={:.2}, perimeter={:.2}",
                self.name(),
                self.area(),
                self.perimeter()
            )
        }
    }

    /// A circle defined by its radius.
    pub struct Circle {
        pub radius: f64,
    }

    impl Shape for Circle {
        fn area(&self) -> f64 { PI * self.radius * self.radius }
        fn perimeter(&self) -> f64 { 2.0 * PI * self.radius }
        fn name(&self) -> &str { "circle" }
    }

    impl fmt::Display for Circle {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Circle(r={})", self.radius)
        }
    }

    /// A rectangle defined by width and height.
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    impl Shape for Rectangle {
        fn area(&self) -> f64 { self.width * self.height }
        fn perimeter(&self) -> f64 { 2.0 * (self.width + self.height) }
        fn name(&self) -> &str { "rectangle" }
    }

    impl fmt::Display for Rectangle {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Rectangle({}x{})", self.width, self.height)
        }
    }

    /// A triangle defined by its three side lengths.
    pub struct Triangle {
        pub a: f64,
        pub b: f64,
        pub c: f64,
    }

    impl Shape for Triangle {
        fn area(&self) -> f64 {
            let s = self.perimeter() / 2.0;
            (s * (s - self.a) * (s - self.b) * (s - self.c)).sqrt()
        }
        fn perimeter(&self) -> f64 { self.a + self.b + self.c }
        fn name(&self) -> &str { "triangle" }
    }

    impl fmt::Display for Triangle {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Triangle({}, {}, {})", self.a, self.b, self.c)
        }
    }
}

pub fn print_shape_info(shape: &impl shapes::Shape) {
    println!("{}", shape.describe());
}
```

### Running Your Solution

```bash
cargo test -p day-10
```

Add a test module to verify your shapes:

```rust
#[cfg(test)]
mod tests {
    use super::shapes::*;
    use std::f64::consts::PI;

    #[test]
    fn circle_area() {
        let c = Circle { radius: 5.0 };
        assert!((c.area() - PI * 25.0).abs() < 1e-9);
    }

    #[test]
    fn rectangle_perimeter() {
        let r = Rectangle { width: 4.0, height: 3.0 };
        assert_eq!(r.perimeter(), 14.0);
    }

    #[test]
    fn triangle_describe() {
        let t = Triangle { a: 3.0, b: 4.0, c: 5.0 };
        let d = t.describe();
        assert!(d.starts_with("[triangle]"));
    }
}
```

### Extension Challenges

- **Easy:** Add a `Square { side: f64 }` type that implements `Shape` without duplicating the `Rectangle` logic (hint: construct a `Rectangle` internally in the area and perimeter methods).
- **Medium:** Write `fn largest_shape<'a>(shapes: &'a [&'a dyn Shape]) -> &'a dyn Shape` that returns a reference to the shape with the greatest area. (Preview of Day 15's trait objects!)
- **Hard:** Add a `Polygon { vertices: Vec<(f64, f64)> }` that implements `Shape` using the Shoelace formula for area and the sum of Euclidean distances between consecutive vertices for perimeter.
