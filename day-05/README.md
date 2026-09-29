# Day 05: Borrowing and References

> **Project:** String Statistics Tool — functions that borrow a text string and compute word count, character count, longest word, and most common character — all without taking ownership

## Learning Objectives

By the end of today you will be able to:
- Create shared references (`&T`) that borrow without moving
- Explain the borrow rules and show what each violation looks like as a compile error
- Create mutable references (`&mut T`) and modify data through them
- Write functions that accept `&str` instead of `String` for maximum flexibility
- Use slices (`&[T]` and `&str`) to reference a portion of a collection
- Read and write a basic lifetime annotation (`'a`) and explain what it tells the compiler

---

## Concepts

### 1. Shared References (`&T`)

> **Docs:** [The Rust Book — References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) · [Reference — Reference types](https://doc.rust-lang.org/reference/types/pointer.html)

Yesterday you learned that passing a `String` to a function moves it — the caller loses ownership. That is inconvenient: you would have to return the `String` back just to keep using it. **Borrowing** solves this. A reference lets you give a function access to a value without transferring ownership. The function "borrows" the value; when it is done, the original owner still has it.

A shared reference is written as `&T` (pronounced "reference to T" or "borrow of T"). You can have **as many shared references as you want** simultaneously — they are read-only, so there is no risk of two borrowers interfering with each other. This is safe for the same reason you can hand a book to multiple readers simultaneously: if nobody is writing, they can all read at the same time.

A note on `&String` vs. `&str`: `String` is an owned, heap-allocated, growable string. `&String` is a reference to that owned string. `&str` is a string **slice** — a view into some string data (could be a `String`, a string literal, or part of either). When writing function parameters, prefer `&str` over `&String` — it accepts both owned strings (auto-deref) and literals, making your function more flexible.

```rust
fn print_length(s: &str) {
    // We have read access to the string data — no ownership transfer
    println!("\"{}\" has {} characters", s, s.len());
} // The borrow ends here. The caller still owns the data.

fn main() {
    let owned = String::from("hello, rust");

    // Pass a reference — owned is NOT moved
    print_length(&owned);

    // owned is still valid
    println!("Still have: {}", owned);

    // String literals are &str already — no & needed
    print_length("a literal");

    // Multiple shared borrows are fine simultaneously
    let r1 = &owned;
    let r2 = &owned;
    let r3 = &owned;
    println!("{} {} {}", r1, r2, r3); // all three reading at once
}
```

#### Exercise 1.1 — Borrow Don't Move

**Goal:** Rewrite a function to take `&str` instead of `String`, so the caller keeps ownership.

The function below takes ownership of its parameter. Rewrite it to borrow instead. Verify that `main` can call the function and then use `greeting` again after the call.

```rust
fn shout(s: String) {
    println!("{}", s.to_uppercase());
}

fn main() {
    let greeting = String::from("hello world");
    shout(greeting);
    // Try printing greeting here — it will fail until you fix shout
    println!("still have: {}", greeting);
}
```

**Expected output:**
```
HELLO WORLD
still have: hello world
```

> **Hint:** Change `s: String` to `s: &str` and the call site to `shout(&greeting)`. The `.to_uppercase()` method works on `&str` just as well as `String`.

---

### 2. The Borrow Rules

> **Docs:** [The Rust Book — References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) · [Rustonomicon — Ownership](https://doc.rust-lang.org/nomicon/ownership.html)

Rust enforces two rules about references at compile time:

1. **At any given time, you can have either many shared references (`&T`) OR exactly one mutable reference (`&mut T`), but never both simultaneously.**
2. **References must always be valid** — a reference cannot outlive the data it points to.

Rule 1 prevents data races. A data race happens when two pieces of code access the same memory concurrently and at least one is writing. By allowing either many readers or exactly one writer (but never both), Rust makes data races impossible — not just unlikely, but structurally impossible.

Rule 2 prevents dangling pointers — references to memory that has already been freed. Rust tracks lifetimes at compile time and refuses to compile code where a reference could outlive its referent.

These rules are enforced by the **borrow checker**, a part of the Rust compiler. When you violate these rules, the error messages are usually very precise about what went wrong and where.

```rust
fn main() {
    let mut data = String::from("hello");

    // Multiple shared borrows — fine
    let r1 = &data;
    let r2 = &data;
    println!("{} and {}", r1, r2);
    // r1 and r2 are no longer used after this point

    // Mutable borrow — fine, because r1 and r2 are no longer active
    let r3 = &mut data;
    r3.push_str(", world");
    println!("{}", r3);

    // This would be a compile error if r1 were still active:
    // let r1 = &data;
    // let r3 = &mut data; // ERROR: cannot borrow as mutable because it is also borrowed as immutable
    // println!("{}", r1);
}
```

#### Exercise 2.1 — Spot the Borrow Violation

**Goal:** See the borrow checker error for mixing shared and mutable references, then fix the code.

The code below violates the borrow rules. Run it, read the error carefully, then fix it by rearranging the order of operations so the shared borrows are no longer active when the mutable borrow is taken.

```rust
fn main() {
    let mut v = vec![1, 2, 3];

    let first = &v[0]; // shared borrow
    v.push(4);         // mutable borrow — ERROR if first is still active
    println!("first element: {}", first);
}
```

**Expected output (after the fix):**
```
first element: 1
```

> **Hint:** Move `println!("first element: {}", first)` to before the `v.push(4)` line. Once `first` is no longer used, the shared borrow ends, and the mutable borrow for `push` is allowed.

---

### 3. Mutable References (`&mut T`)

> **Docs:** [The Rust Book — Mutable References](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#mutable-references) · [Rust by Example — Mutability](https://doc.rust-lang.org/rust-by-example/scope/borrow/mut.html)

A mutable reference allows you to modify the borrowed data. The key constraints are: only one mutable reference at a time, and no shared references active simultaneously. You signal that you want to borrow mutably with `&mut` at both the borrow site and the function parameter.

Mutable references in function signatures are a clear, explicit contract: this function modifies the data you pass it. Callers know upfront what they are agreeing to.

```rust
fn double_all(numbers: &mut Vec<i32>) {
    // Modifies the Vec through the mutable reference
    for n in numbers.iter_mut() {
        *n *= 2; // dereference with * to modify the value behind the reference
    }
}

fn append_exclamation(s: &mut String) {
    s.push('!'); // String methods work directly on &mut String
}

fn main() {
    let mut scores = vec![1, 2, 3, 4, 5];
    println!("Before: {:?}", scores);
    double_all(&mut scores); // pass a mutable reference
    println!("After:  {:?}", scores);

    let mut message = String::from("Hello");
    append_exclamation(&mut message);
    println!("{}", message); // Hello!
}
```

#### Exercise 3.1 — Mutate Through Reference

**Goal:** Write a function that accepts `&mut Vec<i32>` and removes all negative numbers.

Write `fn remove_negatives(v: &mut Vec<i32>)` that modifies the vector in place to remove all elements less than zero. Use `.retain(|&x| ...)` — look it up, or use a `while` loop with `.remove(i)`.

```rust
fn remove_negatives(v: &mut Vec<i32>) {
    // your code here
}

fn main() {
    let mut numbers = vec![3, -1, 4, -1, 5, -9, 2, 6];
    remove_negatives(&mut numbers);
    println!("{:?}", numbers);
}
```

**Expected output:**
```
[3, 4, 5, 2, 6]
```

> **Hint:** `v.retain(|&x| x >= 0)` is the most concise solution. The closure `|&x|` pattern-matches each element by value (using destructuring). Alternatively, iterate with index from the end and call `v.remove(i)` for each negative.

---

### 4. References in Function Parameters

> **Docs:** [The Rust Book — References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) · [Rust by Example — Borrowing](https://doc.rust-lang.org/rust-by-example/scope/borrow.html)

Function parameters are the main place you will encounter references in everyday Rust. The guideline is simple: **take a reference to what you need, not ownership of it**, unless the function genuinely needs to own the data.

For strings: use `&str` (not `&String`) for read-only access. `&str` is more general: it accepts `&String` (via automatic deref coercion), string literals, and string slices.

For vectors: use `&[T]` (not `&Vec<T>`) for read-only access. `&[T]` is a slice and accepts `&Vec<T>` via deref coercion, plus arrays like `&[1, 2, 3]`.

Use `&mut String` or `&mut Vec<T>` only when the function needs to modify the data. Use owned `String` or `Vec<T>` only when the function needs to take permanent ownership (store it somewhere, return it as a different type, etc.).

```rust
// Accepts any string-like data — &str, &String, literals
fn count_vowels(s: &str) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}

// Accepts any slice-like data — &[i32], &Vec<i32>, &arrays
fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

// Mutates through a mutable reference
fn normalise(v: &mut Vec<f64>) {
    let max = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if max != 0.0 {
        for x in v.iter_mut() {
            *x /= max;
        }
    }
}

fn main() {
    let text = String::from("Hello, Rustaceans!");
    println!("Vowels in \"{}\": {}", text, count_vowels(&text));
    println!("Vowels in \"rust\": {}", count_vowels("rust"));

    let nums = vec![10, 20, 30, 40];
    println!("Sum: {}", sum(&nums));
    println!("Sum of literal: {}", sum(&[1, 2, 3]));

    let mut values = vec![2.0, 8.0, 4.0, 6.0];
    normalise(&mut values);
    println!("Normalised: {:?}", values);
}
```

#### Exercise 4.1 — Flexible String Functions

**Goal:** Write two functions using `&str` parameters and verify they work with both `String` and string literals.

Write `fn starts_with_uppercase(s: &str) -> bool` and `fn word_count(s: &str) -> usize`. Test each with both a `String::from(...)` and a string literal `"like this"`.

**Expected output:**
```
"Hello world" starts with uppercase: true
"hello world" starts with uppercase: false
"Hello world" word count: 2
"one two three four" word count: 4
```

> **Hint:** `s.chars().next()` gives the first character as an `Option<char>`. Use `.map(|c| c.is_uppercase()).unwrap_or(false)` on it. For word count, `s.split_whitespace().count()` is idiomatic.

---

### 5. Slices — Views into Collections

> **Docs:** [The Rust Book — The Slice Type](https://doc.rust-lang.org/book/ch04-03-slices.html) · [`str` primitive](https://doc.rust-lang.org/std/primitive.str.html) · [`slice` primitive](https://doc.rust-lang.org/std/primitive.slice.html)

A **slice** is a reference to a contiguous sequence of elements in some collection. It does not own the data — it is a view. Slices are written as `&[T]` for a slice of type `T`, or `&str` for a string slice.

You can create a slice of a `Vec` or array using range syntax: `&v[1..4]` gives a slice of elements at indices 1, 2, and 3. `&v[..]` is a slice of the entire collection (equivalent to `&v`). String slices work the same way on `String`: `&s[0..5]` gives the first 5 bytes (careful with multi-byte Unicode — slicing in the middle of a character panics).

Slices are the reason `&str` and `&[T]` are preferred over `&String` and `&Vec<T>` in function parameters: a slice can point to a subset, while `&Vec<T>` always means "the whole thing."

```rust
fn first_half(numbers: &[i32]) -> &[i32] {
    let mid = numbers.len() / 2;
    &numbers[..mid] // slice of the first half
}

fn contains_zero(numbers: &[i32]) -> bool {
    numbers.iter().any(|&n| n == 0)
}

fn main() {
    let data = vec![10, 20, 30, 40, 50, 60];

    let half = first_half(&data);
    println!("First half: {:?}", half);

    // Slice of an array (no Vec needed)
    let arr = [1, 2, 0, 4, 5];
    println!("Array contains zero: {}", contains_zero(&arr));
    println!("Vec contains zero: {}", contains_zero(&data));

    // String slices
    let s = String::from("hello world");
    let hello: &str = &s[..5];  // first 5 bytes: "hello"
    let world: &str = &s[6..];  // from byte 6 to end: "world"
    println!("{} | {}", hello, world);
}
```

#### Exercise 5.1 — Slice Statistics

**Goal:** Write functions that compute statistics on `&[i32]` slices.

Write three functions: `fn minimum(nums: &[i32]) -> i32`, `fn maximum(nums: &[i32]) -> i32`, and `fn average(nums: &[i32]) -> f64`. All should take a slice (work with both arrays and Vecs). Call them from `main` with a `Vec<i32>` and a raw array.

```rust
fn minimum(nums: &[i32]) -> i32 { todo!() }
fn maximum(nums: &[i32]) -> i32 { todo!() }
fn average(nums: &[i32]) -> f64 { todo!() }

fn main() {
    let data = vec![15, 3, 27, 8, 42, 1, 19];
    println!("min={}, max={}, avg={:.1}", minimum(&data), maximum(&data), average(&data));

    let arr = [10, 20, 30];
    println!("min={}, max={}, avg={:.1}", minimum(&arr), maximum(&arr), average(&arr));
}
```

**Expected output:**
```
min=1, max=42, avg=16.4
min=10, max=30, avg=20.0
```

> **Hint:** Use `.iter().min().copied().unwrap_or(0)` for minimum, and similarly for maximum. For average, compute `sum` as `i64` to avoid overflow, then divide by `nums.len()` casting to `f64`.

---

### 6. Introduction to Lifetimes

> **Docs:** [The Rust Book — Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html) · [Rust by Example — Lifetimes](https://doc.rust-lang.org/rust-by-example/scope/lifetime.html)

When a function returns a reference, the compiler needs to know: *how long is that reference valid?* This is the question **lifetimes** answer.

A lifetime is not something you control at runtime — it is a name the compiler uses to track how long a borrow lasts. In most code the compiler infers lifetimes automatically (called *lifetime elision*). When inference cannot work it out — typically when a function returns a reference that came from one of its parameters — you must annotate it explicitly.

The syntax uses a tick followed by a lowercase name: `'a`. It is placed after the `&`:

```rust
// The returned &str lives as long as the input &str.
// Both the input and output share the lifetime 'a.
fn first_word<'a>(s: &'a str) -> &'a str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None    => s,
    }
}

fn main() {
    let sentence = String::from("hello world");
    let word = first_word(&sentence); // 'a is the borrow of `sentence`
    println!("First word: {}", word);
    // `word` cannot outlive `sentence` — the compiler enforces this
}
```

**Reading the annotation:** `fn first_word<'a>(s: &'a str) -> &'a str` says: "for some lifetime `'a`, take a `&str` that lives at least `'a` and return a `&str` that also lives at most `'a`." The compiler uses this to reject programs where the returned reference would outlive the data it points to.

```rust
// Lifetime elision — the compiler infers the lifetime for you.
// This is identical to the annotated version above:
fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None    => s,
    }
}

// When there are multiple input references, elision may fail.
// You must be explicit about which input the output borrows from:
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

fn main() {
    let s1 = String::from("long string");
    let result;
    {
        let s2 = String::from("xy");
        result = longest(s1.as_str(), s2.as_str());
        println!("Longest: {}", result); // fine — result used before s2 drops
    }
}
```

**What lifetimes do NOT do:**
- They do not extend how long a value lives. They only describe relationships.
- They are purely compile-time. There is zero runtime cost.

**Day 12** is the mastery day for lifetimes: struct lifetime annotations, `'static`, the three elision rules, and higher-ranked trait bounds. For now, the key insight is: *a lifetime annotation says "this reference cannot outlive that other reference," and the compiler uses this to prevent dangling pointers.*

#### Exercise 6.1 — Annotate longest

**Goal:** Write a `longest` function with an explicit lifetime annotation.

```rust
// Complete the signature with a lifetime annotation
fn longest(/* your signature here */) -> /* return type */ {
    if x.len() >= y.len() { x } else { y }
}

fn main() {
    let a = String::from("apple");
    let b = String::from("kiwi");
    println!("Longest: {}", longest(&a, &b));
}
```

**Expected output:**
```
Longest: apple
```

> **Hint:** Both parameters and the return type need `&'a str` where `'a` is the shared lifetime. Declare `<'a>` after the function name: `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str`.

---

## Day Project: String Statistics Tool

### What You're Building

A program that analyses a paragraph of text and reports four statistics: word count, character count (excluding whitespace), the longest word, and the most common character. The twist is that every function borrows the text — none of them take ownership. This is the pattern you will use in almost all real Rust code: the caller owns the data, and functions borrow it to compute results.

The `longest_word` function is the most interesting one: it returns a `&str` that is a slice of the input. This requires a **lifetime annotation** — `<'a>` — to tell the compiler that the returned slice lives as long as the input slice. This is your first glimpse of lifetimes; a full explanation comes later in the curriculum. For now, just follow the signature shown below and understand that the annotation is saying "the output slice cannot outlive the input."

### Requirements

1. Write `fn word_count(text: &str) -> usize` — count whitespace-separated words.
2. Write `fn char_count(text: &str) -> usize` — count non-whitespace characters.
3. Write `fn longest_word<'a>(text: &'a str) -> &'a str` — return the longest word as a slice of the input. If there is a tie, return the first.
4. Write `fn most_common_char(text: &str) -> char` — return the most frequently occurring non-whitespace, lowercase character.
5. Call all four functions from `main` with a sample paragraph of at least 20 words.
6. Print each statistic on its own line with a clear label.

### Getting Started

```rust
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn char_count(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

fn longest_word<'a>(text: &'a str) -> &'a str {
    text.split_whitespace()
        .max_by_key(|w| w.len())
        .unwrap_or("")
}

fn most_common_char(text: &str) -> char {
    let mut counts = [0u32; 26];
    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            let idx = (c.to_ascii_lowercase() as u8 - b'a') as usize;
            counts[idx] += 1;
        }
    }
    let max_idx = counts
        .iter()
        .enumerate()
        .max_by_key(|&(_, count)| count)
        .map(|(i, _)| i)
        .unwrap_or(0);
    (b'a' + max_idx as u8) as char
}

fn main() {
    let text = "Rust is a systems programming language focused on three goals: \
                safety, speed, and concurrency. It accomplishes these goals without \
                a garbage collector, making it useful for a number of use cases other \
                languages are not good at.";

    println!("Text analysis");
    println!("=============");
    println!("Word count:        {}", word_count(text));
    println!("Character count:   {}", char_count(text));
    println!("Longest word:      {}", longest_word(text));
    println!("Most common char:  {}", most_common_char(text));
}
```

### Running Your Solution

```bash
cargo run -p day-05
```

**Expected output:**
```
Text analysis
=============
Word count:        38
Character count:   177
Longest word:      programming
Most common char:  a
```

(Exact numbers depend on your chosen paragraph.)

### Extension Challenges

- **Easy:** Add `fn average_word_length(text: &str) -> f64` that computes the mean length of all words. Print it with two decimal places.
- **Medium:** Add `fn unique_words(text: &str) -> usize` that counts how many distinct words appear (case-insensitive). You will need a `std::collections::HashSet<String>` — look up how to insert into one and call `.len()` on it.
- **Hard:** Implement `fn top_three_chars(text: &str) -> [char; 3]` that returns the three most common alphabetic characters in descending order. Return them as a fixed-size array of three `char` values and print all three in `main`.
