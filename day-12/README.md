# Day 12: Lifetimes

> **Project:** Text Analyser — implement functions with explicit lifetime annotations, define a struct that borrows text, and demonstrate correct scoping in `main`.

## Learning Objectives

By the end of today you will be able to:
- Explain WHY the compiler needs lifetime annotations (preventing dangling references)
- Read and write lifetime annotation syntax (`'a`) on functions and structs
- Identify when annotations are required vs when elision rules apply
- Define a struct that holds a reference with a lifetime parameter
- Understand what `'static` means and when it appears

---

## Concepts

### 1. Why Lifetimes Exist

> **Docs:** [Book — Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html) · [Rustonomicon — Lifetimes](https://doc.rust-lang.org/nomicon/lifetimes.html)

Every reference in Rust has a **lifetime**: the span of code during which that reference is valid. The borrow checker already tracks these lifetimes automatically — when you write `let x = 5; let r = &x;`, the compiler knows `r` must not outlive `x`. Most of the time, it figures this out on its own.

Lifetimes become explicit when the compiler cannot determine, from a function's signature alone, how long a returned reference is valid. Consider: `fn pick(a: &str, b: &str) -> &str`. The compiler sees a reference going out — but which input does it come from? Without knowing, it cannot enforce that the returned reference does not outlive its source. This is the exact scenario where you must annotate.

The following code shows the kind of error lifetimes prevent. The compiler rejects this before it runs:

```rust
fn dangle() -> &String {          // ERROR: missing lifetime specifier
    let s = String::from("hello");
    &s                            // s is dropped at end of this scope
}                                 // returning a reference to freed memory

fn main() {
    // let reference = dangle(); // this would be a dangling pointer — rejected at compile time
    println!("The compiler saves you here.");
}
```

In C or C++, returning a pointer to a local variable compiles and produces undefined behaviour at runtime. Rust's borrow checker catches this at compile time — it is not a runtime check.

#### Exercise 1.1 — Spot the Dangling Reference

**Goal:** Understand what the compiler is protecting against.

Read this code and predict the error before running it. Then add a comment explaining which variable goes out of scope and why the reference would be invalid.

```rust
fn main() {
    let r;
    {
        let x = 5;
        r = &x;
    } // x is dropped here
    // println!("{r}"); // what would happen here?
}
```

**Expected output:**
```
(compile error: `x` does not live long enough)
```

> **Hint:** Uncomment the `println!` line, try `cargo build`, and read the error message carefully. The compiler points to the exact line where the problem is.

---

### 2. Lifetime Annotation Syntax

> **Docs:** [Book — Lifetime annotations in functions](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-annotations-in-function-signatures) · [Rust by Example — Lifetimes](https://doc.rust-lang.org/rust-by-example/scope/lifetime.html)

Lifetime annotations **describe relationships between references** — they do not change how long data lives. The syntax uses a tick and a lowercase name: `'a`, `'b`, `'input`. By convention, `'a` is the first lifetime, `'b` the second.

The annotation `fn foo<'a>(x: &'a str) -> &'a str` says: "the returned reference lives at least as long as the reference `x`." It is a contract you declare so the compiler can verify callers honour it.

```rust
// 'a says: the return value's lifetime is tied to BOTH inputs.
// The compiler picks the shorter of the two lifetimes.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let string1 = String::from("long string");
    let result;
    {
        let string2 = String::from("xyz");
        result = longest(string1.as_str(), string2.as_str());
        println!("Longest: {result}"); // fine — both strings alive here
    }
    // println!("{result}"); // ERROR — string2 no longer alive
}
```

If you change `longest` to always return `x`, you could annotate it as `fn longest<'a, 'b>(x: &'a str, _y: &'b str) -> &'a str` — the output is only tied to `x`. Lifetime annotations express truth about your code; they are not a workaround.

#### Exercise 2.1 — Annotate Longest

**Goal:** Write `longest` from scratch with correct lifetime annotations.

Implement `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str`. Call it in `main` with two `String` values that have different scopes — one created outside a block, one inside — and verify the compiler accepts your usage.

**Expected output:**
```
The longest word is: programming
```

> **Hint:** Call it while both strings are alive. The lifetime `'a` is inferred as the overlap of the two strings' lifetimes.

---

### 3. Lifetime Annotations in Functions — When Are They Required?

> **Docs:** [Book — Lifetime annotations in functions](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-annotations-in-function-signatures)

You only need to write lifetime annotations when the compiler cannot figure them out automatically. The rule of thumb: **if a function returns a reference and takes multiple input references, you usually need to annotate**.

```rust
// No annotation needed — only one input reference, return must come from it
fn first_char(s: &str) -> &str {
    &s[..1]
}

// No annotation needed — returns a reference to 'static data, not an input
fn greeting() -> &'static str {
    "Hello!"
}

// Annotation required — two inputs, one output, ambiguous source
fn first_of_two<'a>(a: &'a str, b: &'a str) -> &'a str {
    if !a.is_empty() { a } else { b }
}

// Annotation for each parameter independently — they do not need to match
fn pick_short<'a, 'b>(short: &'a str, _long: &'b str) -> &'a str {
    short
}

fn main() {
    println!("{}", first_char("hello"));          // h
    println!("{}", greeting());                    // Hello!
    println!("{}", first_of_two("", "fallback")); // fallback
    println!("{}", pick_short("hi", "much longer string")); // hi
}
```

**Expected output:**
```
h
Hello!
fallback
hi
```

#### Exercise 3.1 — First Word

**Goal:** Practice elision — writing a function that does NOT need an explicit lifetime.

Write `fn first_word(s: &str) -> &str` that returns the first space-delimited word, or the whole string if there is no space. Verify it compiles without any lifetime annotations.

**Expected output:**
```
First word of "Hello world": Hello
First word of "Rust": Rust
```

> **Hint:** Find the index of the first space with `s.find(' ')` and slice accordingly. The elision rules handle the lifetime automatically here.

---

### 4. Lifetime Elision Rules

> **Docs:** [Book — Lifetime elision](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-elision)

The compiler applies three rules before requiring you to annotate. You do not need to memorise them — just know they exist, so you understand why many functions compile without annotations:

1. **Each reference parameter gets its own distinct lifetime.** `fn f(x: &str, y: &str)` becomes `fn f<'a, 'b>(x: &'a str, y: &'b str)`.
2. **If there is exactly one input lifetime, it is applied to all output lifetimes.** `fn f(x: &str) -> &str` becomes `fn f<'a>(x: &'a str) -> &'a str`.
3. **If one of the input lifetimes is `&self` or `&mut self`, its lifetime is applied to all output lifetimes.** This is why methods usually do not need annotations.

When the rules leave any output lifetime ambiguous after applying all three, the compiler requires you to annotate explicitly.

```rust
struct Sentence {
    words: Vec<String>,
}

impl Sentence {
    // Rule 3: &self's lifetime flows to the return value — no annotation needed
    fn first_word(&self) -> &str {
        self.words.first().map(String::as_str).unwrap_or("")
    }

    fn nth_word(&self, n: usize) -> Option<&str> {
        self.words.get(n).map(String::as_str)
    }
}

fn main() {
    let sentence = Sentence {
        words: vec!["Rust".to_string(), "is".to_string(), "great".to_string()],
    };
    println!("{}", sentence.first_word()); // Rust
    println!("{:?}", sentence.nth_word(1)); // Some("is")
}
```

---

### 5. Lifetimes in Structs

> **Docs:** [Book — Lifetime annotations in structs](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-annotations-in-struct-definitions)

When a struct stores a reference, it must declare a lifetime parameter. The annotation says: "this struct cannot outlive the data that the reference points to."

```rust
// The struct lives no longer than the text it borrows
struct Excerpt<'a> {
    text: &'a str,
}

impl<'a> Excerpt<'a> {
    fn new(novel: &'a str) -> Self {
        let first_sentence = novel
            .split('.')
            .next()
            .unwrap_or(novel);
        Excerpt { text: first_sentence }
    }

    fn content(&self) -> &str {
        self.text // Rule 3 elides the lifetime here
    }
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let excerpt = Excerpt::new(&novel);
    println!("Excerpt: {}", excerpt.content()); // Call me Ishmael
    // novel is still alive here, so excerpt is valid
}
```

If you tried to drop `novel` before `excerpt` goes out of scope, the compiler would reject it. The lifetime parameter on `Excerpt<'a>` encodes this dependency directly in the type.

---

### 6. `'static` — References That Last Forever

> **Docs:** [Book — 'static lifetime](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#the-static-lifetime) · [Reference — Lifetime bounds](https://doc.rust-lang.org/reference/trait-bounds.html) · [HRTB](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds)

The `'static` lifetime means the reference is valid for the entire duration of the program. String literals like `"hello"` have type `&'static str` because they are compiled directly into the binary's read-only data segment — they never go away.

You will commonly encounter `'static` in error messages when the compiler cannot fit your reference into a shorter lifetime. The fix is usually not to use `'static` but to change the ownership or lifetime of your data.

```rust
// String literals are 'static
let s: &'static str = "I live forever";

// A constant is 'static
const GREETING: &str = "Hello"; // implicitly &'static str

// You can use 'static as a bound: means "T has no non-static references"
fn print_static<T: std::fmt::Display + 'static>(val: T) {
    println!("{val}");
}

fn main() {
    print_static(42);
    print_static("hello");
    print_static(3.14_f64);
    // print_static(&42); // ERROR: &i32 is not 'static (it borrows local memory)
}
```

---

## Day Project: Text Analyser

### What You're Building

A set of functions and a struct that analyse text using explicit lifetime annotations. The goal is to write code that demonstrably requires lifetime annotations and to understand why the compiler needs each one.

### Requirements

1. `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str` — returns the longer of two string slices.
2. `fn first_word(s: &str) -> &str` — returns the first whitespace-delimited word (uses elision, no annotation needed).
3. `fn first_line<'a>(text: &'a str) -> &'a str` — returns the first newline-delimited line.
4. `struct Excerpt<'a> { novel: &'a str, first_sentence: &'a str }` with `impl<'a> Excerpt<'a> { fn new(novel: &'a str) -> Self }` that extracts the first sentence (up to the first `.`).
5. In `main`, demonstrate all four with multiple `String` values, some with different scopes, to show the lifetime constraints in action.
6. Add a `fn count_words(s: &str) -> usize` with no lifetime annotations to practice elision.

### Getting Started

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn first_word(s: &str) -> &str {
    match s.find(|c: char| c.is_whitespace()) {
        Some(idx) => &s[..idx],
        None      => s,
    }
}

fn first_line<'a>(text: &'a str) -> &'a str {
    match text.find('\n') {
        Some(idx) => &text[..idx],
        None      => text,
    }
}

fn count_words(s: &str) -> usize {
    s.split_whitespace().count()
}

struct Excerpt<'a> {
    novel: &'a str,
    first_sentence: &'a str,
}

impl<'a> Excerpt<'a> {
    fn new(novel: &'a str) -> Self {
        let first_sentence = novel.split('.').next().unwrap_or(novel).trim();
        Excerpt { novel, first_sentence }
    }

    fn announce(&self) -> String {
        format!(
            "First sentence: '{}' (novel is {} words long)",
            self.first_sentence,
            count_words(self.novel)
        )
    }
}

fn main() {
    // longest: demonstrate with different scope depths
    let string1 = String::from("Rust programming");
    let result;
    {
        let string2 = String::from("Python");
        result = longest(&string1, &string2);
        println!("longest('{}', '{}') = '{}'", string1, string2, result);
    }

    // first_word: elision in action
    let sentence = "The quick brown fox";
    println!("first_word('{}') = '{}'", sentence, first_word(sentence));
    println!("first_word('Rust') = '{}'", first_word("Rust"));

    // first_line
    let multiline = "First line\nSecond line\nThird line";
    println!("first_line = '{}'", first_line(multiline));

    // Excerpt: struct with lifetime parameter
    let novel = String::from(
        "Call me Ishmael. Some years ago, having little money. \
         It is a way I have of driving off the spleen."
    );
    let excerpt = Excerpt::new(&novel);
    println!("{}", excerpt.announce());

    // count_words
    let poem = "Two roads diverged in a yellow wood";
    println!("Word count: {}", count_words(poem));
}
```

### Running Your Solution

```bash
cargo run -p day-12
```

Expected output:
```
longest('Rust programming', 'Python') = 'Rust programming'
first_word('The quick brown fox') = 'The'
first_word('Rust') = 'Rust'
first_line = 'First line'
First sentence: 'Call me Ishmael' (novel is 24 words long)
Word count: 7
```

### Extension Challenges

- **Easy:** Add `fn longest_word<'a>(text: &'a str) -> &'a str` that returns the longest single word in the string (no whitespace). Test with a sentence containing words of varying lengths.
- **Medium:** Define `struct TextWindow<'a> { source: &'a str, start: usize, end: usize }` with a method `fn content(&self) -> &str` and a method `fn next_word(&mut self) -> Option<&str>` that advances the window word by word.
- **Hard:** Write a function `fn find_common_prefix<'a>(strings: &[&'a str]) -> &'a str` that returns the longest common prefix shared by all strings in the slice. Correctly annotate the lifetime — the result must borrow from the slice's data, not from the slice itself.
