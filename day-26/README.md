# Day 26: Benchmarking and Performance with criterion

> **Project:** Word Frequency Benchmark — compare three implementations of a word counter using statistically rigorous criterion benchmarks

## Learning Objectives

By the end of today you will be able to:
- Explain what criterion measures and why standard tests are not appropriate for timing
- Set up a criterion benchmark suite with the correct Cargo.toml configuration
- Write benchmark functions using `Criterion`, `black_box`, and `BenchmarkGroup`
- Read and interpret criterion's statistical output
- Compare multiple implementations side-by-side to make informed performance decisions

---

## Concepts

### 1. Why Microbenchmarks

> **Docs:** [criterion](https://docs.rs/criterion/latest/criterion/) · [Criterion guide](https://bheisler.github.io/criterion.rs/book/)

Cargo's built-in test runner (`cargo test`) is designed to verify correctness, not measure performance. It runs each test once or a handful of times, applies no warm-up, and reports no statistics. Timing a function in a test is meaningless: the first run incurs JIT-style warm-up costs, the OS scheduler interferes, and a single data point has no statistical validity.

Criterion addresses all of these problems. For each benchmark function it:
1. Runs a **warm-up phase** to fill CPU caches and stabilize branch predictors
2. Runs the benchmark many times to collect a **sample distribution**
3. Applies **outlier rejection** (removes anomalous samples caused by OS interrupts)
4. Reports **mean, median, standard deviation, and confidence intervals**
5. Compares against a baseline from the previous run and reports **regressions or improvements**

The result is a measurement you can trust and reproduce. A 5% speedup that criterion reports is real; a 5% speedup you measured with `Instant::now()` in a test might be noise.

Criterion also generates **HTML reports** in `target/criterion/` with charts showing the distribution of sample times. These are invaluable for spotting bimodal distributions (which indicate interference from another source) or long tails.

#### Exercise 1.1 — Naive Timing vs Criterion

**Goal:** Understand why `Instant::now()` in a test is unreliable.

Write a test that times `"hello world".to_string()` using `std::time::Instant`. Run it 5 times with `cargo test -- --nocapture`. Note the variance. Then imagine making a decision based on those numbers. Write a comment explaining why the measurement is untrustworthy.

**Expected output (times will vary wildly):**
```
elapsed: 1µs
elapsed: 3µs
elapsed: 0µs
elapsed: 2µs
elapsed: 0µs
```

> **Hint:** The variance you see is the OS scheduler, cache effects, and the overhead of the test harness itself — none of which reflect the actual cost of `to_string()`.

---

### 2. Setting Up Criterion

> **Docs:** [criterion::Criterion](https://docs.rs/criterion/latest/criterion/struct.Criterion.html) · [cargo bench](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)

Criterion benchmarks live in the `benches/` directory alongside `src/`. Each benchmark file corresponds to a `[[bench]]` section in `Cargo.toml`. The `harness = false` key tells Cargo not to use the built-in test harness — criterion provides its own.

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "word_count"
harness = false
```

The file `benches/word_count.rs` must use the `criterion_group!` and `criterion_main!` macros to register and run your benchmarks:

```rust
// benches/word_count.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn my_benchmark(c: &mut Criterion) {
    c.bench_function("example", |b| {
        b.iter(|| {
            // code to benchmark
            "hello".to_string()
        })
    });
}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);
```

Run all benchmarks with:
```bash
cargo bench
```

Run a specific benchmark by name:
```bash
cargo bench -- word_count
```

#### Exercise 2.1 — Set Up the Harness

**Goal:** Get a "hello world" benchmark running.

Create `benches/hello.rs` with a benchmark that calls `String::from("hello world")` inside `b.iter()`. Add the corresponding `[[bench]]` section to `Cargo.toml`. Run `cargo bench` and confirm criterion runs the benchmark and prints timing output.

**Expected output (approximate):**
```
hello_world             time:   [3.2 ns 3.3 ns 3.4 ns]
```

> **Hint:** The three numbers are the lower bound, mean, and upper bound of the 95% confidence interval. They should all be very close to each other for a stable operation like this.

---

### 3. Writing a Benchmark

> **Docs:** [std::hint::black_box](https://doc.rust-lang.org/std/hint/fn.black_box.html) · [Rust performance book](https://nnethercote.github.io/perf-book/)

The core of a criterion benchmark is the `b.iter(|| ...)` call. The closure is the code under measurement. criterion calls it thousands of times, adjusting the iteration count until it has enough samples for a statistically valid measurement.

`black_box` is essential. Rust's compiler is extremely aggressive about dead-code elimination and constant folding. Without `black_box`, a call to `count_words("hello world")` might be optimized into a single `return 2` at compile time, making your benchmark measure nothing. `black_box` creates an opaque barrier that prevents the optimizer from looking through it:

```rust
use criterion::black_box;

fn bench_count(c: &mut Criterion) {
    let input = "the quick brown fox jumps over the lazy dog";
    c.bench_function("count_words", |b| {
        b.iter(|| {
            // black_box prevents the compiler from caching the result
            count_words(black_box(input))
        })
    });
}
```

Apply `black_box` to inputs (prevents constant propagation) and optionally to outputs (prevents discarding the return value). For most benchmarks, wrapping the input is sufficient.

If your benchmark needs expensive setup that should not be timed (like generating test data), use `b.iter_batched`:

```rust
b.iter_batched(
    || generate_large_input(),    // setup, not timed
    |input| count_words(input),   // what you're measuring
    criterion::BatchSize::SmallInput,
);
```

#### Exercise 3.1 — Benchmark a Real Function

**Goal:** Benchmark two simple string operations side by side.

Write benchmarks for `str::to_uppercase()` and `str::chars().count()` on a 100-character string. Observe which is faster and by how much.

**Expected output (approximate):**
```
to_uppercase            time:   [45 ns 46 ns 47 ns]
char_count              time:   [12 ns 13 ns 14 ns]
```

> **Hint:** Wrap the string literal in `black_box(...)` at the start of the iter closure to prevent the compiler from evaluating it at compile time.

---

### 4. Reading Criterion Output

> **Docs:** [criterion](https://docs.rs/criterion/latest/criterion/) · [Criterion guide](https://bheisler.github.io/criterion.rs/book/)

A typical criterion output line looks like:

```
count_words/hashmap     time:   [412.34 ns 415.11 ns 418.02 ns]
                        change: [-2.3456% -1.2345% -0.1234%] (p = 0.04 < 0.05)
                        Performance has improved.
```

The three time values in brackets are the **lower bound**, **mean**, and **upper bound** of the 95% confidence interval. Tighter intervals indicate a more stable measurement.

The `change` line appears when criterion has a baseline from a previous `cargo bench` run. It shows the percentage change in performance:
- Negative percentages mean improvement (the code got faster)
- Positive percentages mean regression (the code got slower)
- `p < 0.05` means the change is statistically significant at the 95% confidence level
- "No change detected" means the difference is within the noise floor

HTML reports live at `target/criterion/<benchmark-name>/report/index.html`. Open them in a browser to see:
- A **PDF/KDE plot** of sample times (normal distribution indicates a clean measurement)
- **Iteration count** per sample (higher means faster functions, since criterion adjusts for minimum wall-clock time)
- **Regression plots** showing change over time (if you run benchmarks repeatedly)

#### Exercise 4.1 — Introduce a Regression

**Goal:** Observe criterion detecting a performance change.

Run `cargo bench` once to establish a baseline. Then change your `count_words` function to add a `thread::sleep(Duration::from_nanos(100))` call. Run `cargo bench` again. Observe the "Performance has regressed" message and the change percentages.

**Expected output (second run):**
```
count_words             time:   [512.34 ns 515.11 ns 518.02 ns]
                        change: [+22.345% +24.123% +25.987%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

> **Hint:** Remove the sleep before committing. The baseline is stored in `target/criterion/`. Delete the directory to reset baselines.

---

### 5. Comparing Implementations

> **Docs:** [criterion::BenchmarkId](https://docs.rs/criterion/latest/criterion/struct.BenchmarkId.html) · [criterion::Criterion](https://docs.rs/criterion/latest/criterion/struct.Criterion.html)

`BenchmarkGroup` lets you compare multiple implementations of the same function in a single benchmark run. criterion then produces a comparison report showing all implementations on the same chart.

```rust
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_implementations(c: &mut Criterion) {
    let input = "the quick brown fox".repeat(100);

    let mut group = c.benchmark_group("word_count");

    group.bench_function("hashmap", |b| {
        b.iter(|| count_words_hashmap(black_box(&input)))
    });

    group.bench_function("btreemap", |b| {
        b.iter(|| count_words_btreemap(black_box(&input)))
    });

    group.bench_function("sort_dedup", |b| {
        b.iter(|| count_words_sort_dedup(black_box(&input)))
    });

    group.finish();
}
```

You can also parameterize over input sizes using `BenchmarkId`:

```rust
for size in [100, 1_000, 10_000].iter() {
    let input = generate_words(*size);
    group.bench_with_input(BenchmarkId::new("hashmap", size), &input, |b, input| {
        b.iter(|| count_words_hashmap(black_box(input)))
    });
}
```

This produces separate measurements for each `(implementation, size)` pair, letting you see how performance scales.

#### Exercise 5.1 — Group Benchmark

**Goal:** Benchmark two different sorting algorithms in a group.

Write two sort implementations: `fn bubble_sort(v: &mut Vec<u32>)` and `fn std_sort(v: &mut Vec<u32>) { v.sort() }`. Benchmark both in a `BenchmarkGroup` with a 1,000-element random input. Use `iter_batched` so the sort's input is fresh each iteration (sorting an already-sorted array is much faster, which would bias the benchmark).

**Expected output (approximate):**
```
sort_comparison/bubble_sort  time: [1.23 ms 1.25 ms 1.27 ms]
sort_comparison/std_sort     time: [12.3 µs 12.5 µs 12.7 µs]
```

> **Hint:** `iter_batched` with `BatchSize::SmallInput` clones the setup data before each iteration. The setup closure `|| v.clone()` runs outside the timed section.

---

## Day Project: Word Frequency Benchmark

### What You're Building

Implement three versions of a word frequency counter and benchmark them rigorously with criterion. The goal is to understand the real performance trade-offs between `HashMap`, `BTreeMap`, and a sort-based approach, and to practice the discipline of making evidence-based performance decisions.

Additionally, benchmark `String::clone()` vs working with `&str` to demonstrate the cost of heap allocation in a tight loop.

### Requirements

1. Implement `count_words_hashmap(text: &str) -> HashMap<&str, usize>` — split on whitespace, count occurrences
2. Implement `count_words_btreemap(text: &str) -> BTreeMap<&str, usize>` — same logic but sorted keys
3. Implement `count_words_sort_dedup(text: &str) -> Vec<(&str, usize)>` — collect into Vec, sort, then count runs
4. Generate a 10,000-word input string at the start of each benchmark using a helper function
5. Create a `BenchmarkGroup` comparing all three implementations
6. Add a separate benchmark comparing `String::clone()` vs using `&str` in a 1,000-iteration loop
7. Run `cargo bench` and record the results in a comment block at the top of `benches/word_count.rs`

### Getting Started

```toml
[package]
name = "day-26"
version = "0.1.0"
edition = "2021"

[lib]
name = "day_26"
path = "src/lib.rs"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "word_count"
harness = false
```

In `src/lib.rs`, implement the three word counting functions:

```rust
use std::collections::{BTreeMap, HashMap};

pub fn generate_text(words: usize) -> String {
    // Generate a deterministic large string for benchmarking
    let vocabulary = [
        "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
        "hello", "world", "rust", "is", "fast", "and", "safe", "memory",
        "ownership", "borrowing", "traits", "generics", "async", "await",
    ];
    (0..words)
        .map(|i| vocabulary[i % vocabulary.len()])
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn count_words_hashmap(text: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

pub fn count_words_btreemap(text: &str) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

pub fn count_words_sort_dedup(text: &str) -> Vec<(&str, usize)> {
    let mut words: Vec<&str> = text.split_whitespace().collect();
    words.sort_unstable();
    let mut result: Vec<(&str, usize)> = Vec::new();
    for word in words {
        match result.last_mut() {
            Some(last) if last.0 == word => last.1 += 1,
            _ => result.push((word, 1)),
        }
    }
    result
}
```

In `benches/word_count.rs`:

```rust
// Results recorded on [your machine] at [date]:
// word_count/hashmap     time: [~X µs]
// word_count/btreemap    time: [~X µs]
// word_count/sort_dedup  time: [~X µs]
//
// Takeaway: hashmap is fastest for counting, btreemap pays ~30% overhead
// for maintaining sorted order, sort_dedup is fastest when output must be sorted.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use day_26::{count_words_btreemap, count_words_hashmap, count_words_sort_dedup, generate_text};

fn bench_word_count(c: &mut Criterion) {
    let text = generate_text(10_000);

    let mut group = c.benchmark_group("word_count");

    group.bench_function("hashmap", |b| {
        b.iter(|| count_words_hashmap(black_box(&text)))
    });

    group.bench_function("btreemap", |b| {
        b.iter(|| count_words_btreemap(black_box(&text)))
    });

    group.bench_function("sort_dedup", |b| {
        b.iter(|| count_words_sort_dedup(black_box(&text)))
    });

    group.finish();
}

fn bench_allocation(c: &mut Criterion) {
    let s = "hello world from rust".to_string();

    let mut group = c.benchmark_group("allocation");

    group.bench_function("string_clone", |b| {
        b.iter(|| {
            let _owned: String = black_box(&s).clone();
        })
    });

    group.bench_function("str_ref", |b| {
        b.iter(|| {
            let _borrowed: &str = black_box(s.as_str());
        })
    });

    group.finish();
}

criterion_group!(benches, bench_word_count, bench_allocation);
criterion_main!(benches);
```

### Running Your Solution

```bash
cargo bench -p day-26
```

Successful output:
```
word_count/hashmap      time:   [XXX µs XXX µs XXX µs]
word_count/btreemap     time:   [XXX µs XXX µs XXX µs]
word_count/sort_dedup   time:   [XXX µs XXX µs XXX µs]
allocation/string_clone time:   [XX ns XX ns XX ns]
allocation/str_ref      time:   [X ns X ns X ns]
```

Open the HTML report:
```bash
# On macOS/Linux:
open target/criterion/word_count/hashmap/report/index.html
# On Windows:
start target/criterion/word_count/hashmap/report/index.html
```

### Extension Challenges

- **Easy:** Add a fourth implementation `count_words_fnv` using the `fnv` crate's `FnvHashMap` (a faster hash for short keys). Benchmark it against the standard `HashMap`.
- **Medium:** Parameterize the benchmark over input sizes `[1_000, 5_000, 10_000, 50_000]` using `BenchmarkId`. Plot how each implementation scales with input size and identify at what point `sort_dedup` overtakes `hashmap`.
- **Hard:** Profile the `hashmap` implementation using `perf` (Linux) or Instruments (macOS). Identify the hottest lines. Then write a SIMD-optimized whitespace splitter using `std::arch` or the `memchr` crate and benchmark it against the standard `split_whitespace`.
