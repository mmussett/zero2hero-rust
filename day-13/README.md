# Day 13: Closures and Iterators

> **Project:** Data Pipeline — transform a `Vec<Student>` using only iterator chains (no explicit `for` loops) to filter, grade, sort, and report.

## Learning Objectives

By the end of today you will be able to:
- Write closures that capture their environment by reference, mutable reference, and by value
- Understand `Fn`, `FnMut`, and `FnOnce` and when each is required
- Compose lazy iterator adapters (`map`, `filter`, `flat_map`, `enumerate`, `zip`)
- Consume iterators with `collect`, `fold`, `sum`, `any`, `all`, `find`, `max`, and `min`
- Rewrite a multi-step `for` loop as a readable single-expression iterator pipeline

---

## Concepts

### 1. Closure Syntax

> **Docs:** [Closures](https://doc.rust-lang.org/book/ch13-01-closures.html) · [Reference — Closure expressions](https://doc.rust-lang.org/reference/expressions/closure-expr.html) · [Rust by Example — Closures](https://doc.rust-lang.org/rust-by-example/fn/closures.html)

A closure is an anonymous function that can capture variables from its surrounding scope — something a regular `fn` cannot do. The syntax uses pipes: `|params| expression` or `|params| { body }`. Type inference usually eliminates the need for explicit types, but you can add them.

```rust
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

fn main() {
    // Shortest form — types inferred
    let double = |x| x * 2;

    // With explicit types
    let triple = |x: i32| -> i32 { x * 3 };

    // Multi-line block
    let describe = |x: i32| {
        if x > 0 { format!("{x} is positive") }
        else if x < 0 { format!("{x} is negative") }
        else { "zero".to_string() }
    };

    println!("{}", double(5));       // 10
    println!("{}", triple(4));       // 12
    println!("{}", describe(-3));    // -3 is negative

    // Closures as arguments — apply accepts any Fn(i32) -> i32
    println!("{}", apply(|x| x + 100, 42)); // 142
    println!("{}", apply(double, 7));        // 14
}
```

**Expected output:**
```
10
12
-3 is negative
142
14
```

#### Exercise 1.1 — Higher-Order Functions

**Goal:** Write a function that takes a closure and returns a closure.

Write `fn make_adder(n: i32) -> impl Fn(i32) -> i32` that returns a closure adding `n` to its argument. Create `add_five` and `add_ten` from it, then use them.

```rust
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    // return a closure that captures n
}

fn main() {
    let add_five = make_adder(5);
    let add_ten  = make_adder(10);
    println!("{}", add_five(3));  // 8
    println!("{}", add_ten(3));   // 13
}
```

**Expected output:**
```
8
13
```

> **Hint:** The closure must use `move` to capture `n` by value, otherwise `n` would be borrowed and the lifetime of the reference would not outlast the function.

---

### 2. Capturing the Environment

> **Docs:** [Fn](https://doc.rust-lang.org/std/ops/trait.Fn.html) · [FnMut](https://doc.rust-lang.org/std/ops/trait.FnMut.html) · [FnOnce](https://doc.rust-lang.org/std/ops/trait.FnOnce.html)

Closures differ from functions precisely because they can reach into the surrounding scope and use variables. How they capture depends on how they use the variable:

- **By reference (`&T`)** — the default when the closure only reads the variable. The closure implements `Fn`.
- **By mutable reference (`&mut T`)** — when the closure modifies the variable. Implements `FnMut`.
- **By value (move)** — when you write `move` before the pipes, or when the closure is sent across threads. Implements `FnOnce` at minimum, `Fn` if the value is `Copy`.

`FnOnce` can be called at most once (it may consume a captured value). `FnMut` can be called multiple times but requires exclusive access. `Fn` can be called any number of times with shared access. Every `Fn` is also `FnMut` is also `FnOnce` — they form a hierarchy.

```rust
fn call_once<F: FnOnce() -> String>(f: F) -> String { f() }
fn call_many<F: Fn() -> i32>(f: F, n: usize) {
    for _ in 0..n { println!("{}", f()); }
}

fn main() {
    let greeting = String::from("Hello");

    // Fn — borrows greeting immutably, can be called repeatedly
    let say_hi = || format!("{}, world!", greeting);
    println!("{}", say_hi());
    println!("{}", say_hi()); // still fine — borrow, not move

    // FnMut — mutably borrows a counter
    let mut count = 0;
    let mut increment = || { count += 1; count };
    println!("{}", increment()); // 1
    println!("{}", increment()); // 2

    // move — takes ownership; useful for threads and returning closures
    let name = String::from("Alice");
    let owned_greeting = move || format!("Hi, {name}!");
    // name is no longer accessible here — it was moved
    println!("{}", owned_greeting()); // Hi, Alice!

    // FnOnce — consumes a captured value when called
    let message = String::from("consumed");
    let consume = move || { let m = message; m.to_uppercase() };
    println!("{}", call_once(consume)); // CONSUMED
    // println!("{}", consume()); // ERROR: FnOnce can only be called once
}
```

---

### 3. The `Iterator` Trait

> **Docs:** [Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html) · [Iterator trait](https://doc.rust-lang.org/std/iter/trait.Iterator.html) · [IntoIterator](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html) · [std::iter](https://doc.rust-lang.org/std/iter/) · [Rust by Example — Iterators](https://doc.rust-lang.org/rust-by-example/trait/iter.html)

Every collection that supports iteration in Rust implements `Iterator`. The trait has one required method:

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

Everything else — `map`, `filter`, `sum`, `collect`, and 70+ other methods — is provided as a default implementation built on top of `next()`. This design means any type implementing `next()` automatically gets the entire iterator toolkit.

`iter()` gives you an iterator over **immutable references** (`&T`). `iter_mut()` gives **mutable references** (`&mut T`). `into_iter()` gives **owned values** (`T`), consuming the collection.

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];

    // iter() — yields &i32
    for x in v.iter() {
        print!("{x} "); // 1 2 3 4 5
    }
    println!();
    println!("v still available: {:?}", v);

    // into_iter() — yields i32, consumes v
    let doubled: Vec<i32> = v.into_iter().map(|x| x * 2).collect();
    // v is gone here
    println!("{:?}", doubled); // [2, 4, 6, 8, 10]

    // Implementing Iterator manually
    struct Counter { current: u32, max: u32 }
    impl Iterator for Counter {
        type Item = u32;
        fn next(&mut self) -> Option<u32> {
            if self.current < self.max {
                self.current += 1;
                Some(self.current)
            } else {
                None
            }
        }
    }

    let sum: u32 = Counter { current: 0, max: 5 }.sum();
    println!("Sum 1..=5: {sum}"); // 15
}
```

---

### 4. Iterator Adapters — Lazy Transformations

> **Docs:** [Iterator trait](https://doc.rust-lang.org/std/iter/trait.Iterator.html) · [std::iter](https://doc.rust-lang.org/std/iter/)

Adapters transform an iterator into another iterator without running any computation yet. Nothing happens until the iterator is consumed. This laziness means you can chain many adapters with no intermediate allocations.

```rust
fn main() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // map: transform each element
    let squares: Vec<i32> = data.iter().map(|x| x * x).collect();
    println!("{:?}", squares);

    // filter: keep elements matching a predicate
    let evens: Vec<&i32> = data.iter().filter(|&&x| x % 2 == 0).collect();
    println!("{:?}", evens);

    // enumerate: attach an index
    for (i, val) in data.iter().enumerate().take(3) {
        println!("data[{i}] = {val}");
    }

    // zip: pair up two iterators
    let letters = ['a', 'b', 'c'];
    let pairs: Vec<_> = data.iter().zip(letters.iter()).collect();
    println!("{:?}", pairs);

    // flat_map: map then flatten (useful for nested iterables)
    let words = vec!["hello world", "foo bar"];
    let all_words: Vec<&str> = words.iter().flat_map(|s| s.split(' ')).collect();
    println!("{:?}", all_words); // ["hello", "world", "foo", "bar"]

    // chain: concatenate two iterators
    let a = [1, 2, 3];
    let b = [4, 5, 6];
    let chained: Vec<i32> = a.iter().chain(b.iter()).copied().collect();
    println!("{:?}", chained); // [1, 2, 3, 4, 5, 6]

    // skip and take
    let middle: Vec<&i32> = data.iter().skip(2).take(4).collect();
    println!("{:?}", middle); // [3, 4, 5, 6]
}
```

#### Exercise 4.1 — Pipeline Composition

**Goal:** Chain at least three adapters in a single expression.

Given `let words = vec!["rust", "python", "go", "java", "typescript"];`, write a single expression that: filters to words longer than 3 characters, maps each to its length, and collects into a `Vec<usize>`.

**Expected output:**
```
[4, 6, 4, 10]
```

> **Hint:** `.filter(|w| w.len() > 3).map(|w| w.len()).collect()`.

---

### 5. Consuming Iterators

> **Docs:** [Iterator trait](https://doc.rust-lang.org/std/iter/trait.Iterator.html) · [std::iter](https://doc.rust-lang.org/std/iter/)

Consumers drive the iteration to completion. They are the "pull" at the end of the lazy pipeline.

```rust
fn main() {
    let numbers = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];

    // collect — gather into any collection (type annotation often required)
    let evens: Vec<i32> = numbers.iter().copied().filter(|x| x % 2 == 0).collect();
    println!("evens: {:?}", evens);

    // sum and product
    let total: i32 = numbers.iter().sum();
    let product: i32 = vec![1, 2, 3, 4, 5].iter().product();
    println!("sum={total}, product={product}");

    // count
    println!("count > 3: {}", numbers.iter().filter(|&&x| x > 3).count());

    // any and all
    println!("any > 8: {}", numbers.iter().any(|&x| x > 8));
    println!("all > 0: {}", numbers.iter().all(|&x| x > 0));

    // find — returns Option<&T>
    let first_big = numbers.iter().find(|&&x| x > 7);
    println!("first > 7: {:?}", first_big);

    // position — returns Option<usize>
    let pos = numbers.iter().position(|&x| x == 9);
    println!("position of 9: {:?}", pos);

    // max and min
    println!("max={:?}, min={:?}", numbers.iter().max(), numbers.iter().min());

    // fold — general accumulator
    let concat = ["a", "b", "c"]
        .iter()
        .fold(String::new(), |mut acc, s| { acc.push_str(s); acc });
    println!("folded: {concat}");
}
```

**Expected output:**
```
evens: [4, 2, 6]
sum=39, product=120
count > 3: 4
any > 8: true
all > 0: true
first > 7: Some(9)
position of 9: Some(5)
max=Some(9), min=Some(1)
folded: abc
```

---

### 6. Chaining Adapters and Consumers

> **Docs:** [Iterator trait](https://doc.rust-lang.org/std/iter/trait.Iterator.html) · [std::iter](https://doc.rust-lang.org/std/iter/)

The real power of iterators is the ability to express complex transformations as a single readable pipeline with no intermediate `Vec` allocations.

```rust
fn main() {
    let sentence = "the quick brown fox jumps over the lazy dog";

    // Multi-step pipeline in one expression
    let result: Vec<String> = sentence
        .split_whitespace()              // &str iterator
        .filter(|w| w.len() > 3)        // only longer words
        .map(|w| {                       // capitalise first letter
            let mut c = w.chars();
            match c.next() {
                None    => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect();

    println!("{}", result.join(", "));
    // Quick, Brown, Jumps, Over, Lazy

    // vs. the verbose loop version:
    let mut result2 = Vec::new();
    for word in sentence.split_whitespace() {
        if word.len() > 3 {
            let mut c = word.chars();
            let cap = match c.next() {
                None    => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            };
            result2.push(cap);
        }
    }
    println!("{}", result2.join(", "));
}
```

Both produce the same output; the iterator version is more declarative and avoids the intermediate `cap` binding cluttering the loop scope.

---

## Day Project: Data Pipeline

### What You're Building

Given a `Vec<Student>` with name, score, and a `passed` flag, you will build a series of iterator pipelines that filter, transform, sort, and report — without a single explicit `for` loop.

### Requirements

1. Define `struct Student { name: String, score: u32, passed: bool }`.
2. Hardcode a `Vec<Student>` with at least 8 students, some with `passed: false`.
3. Using iterator chains only (no `for` loops):
   - Filter to passing students only.
   - Map each to a `(name, grade_letter)` tuple where grade = A (90+), B (80+), C (70+), D (60+), F (below 60).
   - Collect into a `Vec`, then sort by score descending (`sort_by`).
   - Format each as `"Name: Alice — Grade: A (92)"` and print.
4. Compute the **average score** of passing students using `fold` or `sum` + `count`.
5. Find the **student with the highest score** among all students using `max_by_key`.
6. Count how many students **failed** using `filter` + `count`.

### Getting Started

```rust
struct Student {
    name: String,
    score: u32,
    passed: bool,
}

fn letter_grade(score: u32) -> &'static str {
    match score {
        90..=100 => "A",
        80..=89  => "B",
        70..=79  => "C",
        60..=69  => "D",
        _        => "F",
    }
}

fn main() {
    let mut students = vec![
        Student { name: "Alice".to_string(),   score: 92, passed: true  },
        Student { name: "Bob".to_string(),     score: 58, passed: false },
        Student { name: "Carol".to_string(),   score: 85, passed: true  },
        Student { name: "Dave".to_string(),    score: 73, passed: true  },
        Student { name: "Eve".to_string(),     score: 91, passed: true  },
        Student { name: "Frank".to_string(),   score: 45, passed: false },
        Student { name: "Grace".to_string(),   score: 67, passed: true  },
        Student { name: "Heidi".to_string(),   score: 88, passed: true  },
    ];

    // 1. Passing students, graded, sorted, formatted
    let mut passing: Vec<(&Student, &'static str)> = students
        .iter()
        .filter(|s| s.passed)
        .map(|s| (s, letter_grade(s.score)))
        .collect();

    passing.sort_by(|a, b| b.0.score.cmp(&a.0.score));

    println!("=== Passing Students (by score) ===");
    for (student, grade) in &passing {
        println!("  Name: {} — Grade: {} ({})", student.name, grade, student.score);
    }

    // 2. Average score of passing students
    let (total, count) = students
        .iter()
        .filter(|s| s.passed)
        .fold((0u64, 0u64), |(sum, n), s| (sum + s.score as u64, n + 1));
    let average = if count > 0 { total as f64 / count as f64 } else { 0.0 };
    println!("\nAverage passing score: {average:.1}");

    // 3. Highest scoring student overall
    if let Some(top) = students.iter().max_by_key(|s| s.score) {
        println!("Top student: {} with {}", top.name, top.score);
    }

    // 4. Failed count
    let failed = students.iter().filter(|s| !s.passed).count();
    println!("Students who failed: {failed}");
}
```

### Running Your Solution

```bash
cargo run -p day-13
```

Expected output:
```
=== Passing Students (by score) ===
  Name: Alice — Grade: A (92)
  Name: Eve — Grade: A (91)
  Name: Heidi — Grade: B (88)
  Name: Carol — Grade: B (85)
  Name: Dave — Grade: C (73)
  Name: Grace — Grade: D (67)

Average passing score: 82.7
Top student: Alice with 92
Students who failed: 2
```

### Extension Challenges

- **Easy:** Add a pipeline that collects all grade letters into a `HashMap<&str, usize>` counting how many students earned each grade (A, B, C, D, F). Print the grade distribution.
- **Medium:** Use `flat_map` to split each student's name into individual characters, then count total characters across all student names using an iterator (no loops).
- **Hard:** Implement `fn percentile(students: &[Student], p: f64) -> f64` using iterator methods — sort scores, then find the value at the `p`-th percentile. Return the score at which `p`% of students scored at or below.
