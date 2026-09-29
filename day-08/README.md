# Day 08: Collections — Vec, HashMap, HashSet

> **Project:** Word Frequency Counter — parse a text passage, count every word, and print the top 10 most frequent.

## Learning Objectives

By the end of today you will be able to:
- Create and manipulate `Vec<T>` using indexed access, slices, and iteration
- Store and retrieve key-value pairs in a `HashMap<K, V>` using the entry API
- Use `HashSet<T>` for efficient membership testing and set operations
- Choose the right collection for a given problem based on access patterns

---

## Concepts

### 1. `Vec<T>` — Rust's Growable Array

> **Docs:** [Book — Vectors](https://doc.rust-lang.org/book/ch08-01-vectors.html) · [Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html)

A `Vec<T>` (short for "vector") is the workhorse of Rust collections. It stores a sequence of elements of the same type in contiguous heap memory and grows automatically as you add items. Think of it as the Rust equivalent of a Python list or a C++ `std::vector`.

The critical thing to understand about `Vec` indexing is that `vec[i]` and `vec.get(i)` behave very differently. The bracket syntax panics at runtime if the index is out of bounds — the compiler cannot know at compile time whether index 42 is valid for a vector of unknown length. The `.get(i)` method returns `Option<&T>`, forcing you to handle the "index does not exist" case explicitly. In production code, prefer `.get()` whenever the index comes from user input or external data.

Iterating with `for x in &vec` borrows each element immutably. Using `for x in &mut vec` borrows mutably, allowing in-place modification. Using `for x in vec` moves the vector, consuming it — after this loop, `vec` is gone. Slices (`&vec[1..3]`) give you a borrowed window into the vector without copying.

```rust
fn main() {
    // Two ways to create a Vec
    let mut scores: Vec<i32> = Vec::new();
    let names = vec!["Alice", "Bob", "Carol"];

    // push appends; pop removes from the back (returns Option<T>)
    scores.push(95);
    scores.push(82);
    scores.push(74);
    let last = scores.pop(); // Some(74)

    // Indexed access — panics if out of bounds
    println!("First score: {}", scores[0]);

    // Safe access — returns Option
    match scores.get(10) {
        Some(s) => println!("Score: {s}"),
        None    => println!("Index 10 does not exist"),
    }

    // Immutable iteration — x is &i32
    for score in &scores {
        println!("Score: {score}");
    }

    // Mutable iteration — double every score in-place
    for score in &mut scores {
        *score *= 2;
    }

    // Slicing — a borrowed view of elements 0..2 (not including 2)
    let first_two: &[&str] = &names[0..2];
    println!("First two: {:?}", first_two);

    println!("scores has {} elements, is_empty: {}", scores.len(), scores.is_empty());
    println!("last was: {:?}", last);
}
```

**Expected output:**
```
First score: 95
Index 10 does not exist
Score: 190
Score: 164
First two: ["Alice", "Bob"]
scores has 2 elements, is_empty: false
last was: Some(74)
```

#### Exercise 1.1 — Vec Manipulation

**Goal:** Practice building, modifying, and iterating a `Vec`.

Create a `Vec<i32>` containing the numbers 1 through 10. Remove all even numbers using `retain(|x| ...)`. Then print the remaining elements on one line separated by commas.

```rust
fn main() {
    let mut numbers: Vec<i32> = (1..=10).collect();
    // remove even numbers with retain
    // print remaining
}
```

**Expected output:**
```
Odd numbers: 1, 3, 5, 7, 9
```

> **Hint:** `retain` keeps elements where the closure returns `true`. Use `join` on a `Vec<String>` to build the comma-separated output, or collect into strings with `map`.

---

### 2. `HashMap<K, V>` — Key-Value Lookups

> **Docs:** [Book — HashMaps](https://doc.rust-lang.org/book/ch08-03-hash-maps.html) · [HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html) · [Entry API](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html)

A `HashMap` stores pairs of (key, value) and gives you O(1) average-case lookup by key. The keys must implement `Eq` and `Hash` — `String`, `&str`, integers, and tuples of those all work. Rust's standard `HashMap` uses a cryptographically secure hasher by default to resist denial-of-service attacks, which is slightly slower than common hash functions. For performance-critical inner loops, you can swap in `ahash` or `rustc-hash`, but the default is the right choice until you profile.

The `.get(key)` method returns `Option<&V>` — you get a reference, not ownership. If you need to mutate a value in place, `.get_mut(key)` returns `Option<&mut V>`.

The **entry API** is the idiomatic way to insert-or-update without doing two lookups. `entry(key).or_insert(default)` returns a `&mut V` pointing at the existing value if the key exists, or inserts `default` and returns a reference to it. This is far more efficient than the common beginner pattern of `if contains_key { update } else { insert }`.

```rust
use std::collections::HashMap;

fn main() {
    // Create and populate
    let mut capitals: HashMap<String, String> = HashMap::new();
    capitals.insert("France".to_string(), "Paris".to_string());
    capitals.insert("Japan".to_string(),  "Tokyo".to_string());
    capitals.insert("Egypt".to_string(),  "Cairo".to_string());

    // Literal initialisation with HashMap::from
    let scores: HashMap<&str, i32> = HashMap::from([
        ("Alice", 92),
        ("Bob",   85),
        ("Carol", 78),
    ]);

    // get returns Option<&V>
    if let Some(capital) = capitals.get("France") {
        println!("Capital of France: {capital}");
    }

    // contains_key for existence check
    println!("Has Germany: {}", capitals.contains_key("Germany"));

    // Iterating — order is not guaranteed
    for (country, capital) in &capitals {
        println!("{country} → {capital}");
    }

    // Entry API: word counting
    let text = "the cat sat on the mat the cat";
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for word in text.split_whitespace() {
        let count = counts.entry(word).or_insert(0);
        *count += 1;
    }
    println!("'the' appears {} times", counts["the"]);

    // remove returns Option<V>
    let removed = capitals.remove("Egypt");
    println!("Removed: {:?}", removed);
}
```

**Expected output (country order may vary):**
```
Capital of France: Paris
Has Germany: false
France → Paris
Japan → Tokyo
Egypt → Cairo
'the' appears 3 times
Removed: Some("Cairo")
```

#### Exercise 2.1 — Inventory Counter

**Goal:** Use the entry API to build a frequency map.

Given the list `["apple", "banana", "apple", "cherry", "banana", "apple"]`, build a `HashMap<&str, usize>` counting occurrences. Then print each fruit and count in alphabetical order.

```rust
use std::collections::HashMap;

fn main() {
    let fruits = vec!["apple", "banana", "apple", "cherry", "banana", "apple"];
    let mut counts: HashMap<&str, usize> = HashMap::new();
    // count with entry API
    // sort keys and print
}
```

**Expected output:**
```
apple: 3
banana: 2
cherry: 1
```

> **Hint:** `HashMap` iteration order is random; collect the keys into a `Vec`, sort it, then iterate.

---

### 3. `HashSet<T>` — Efficient Membership Testing

> **Docs:** [HashSet](https://doc.rust-lang.org/std/collections/struct.HashSet.html) · [Hash trait](https://doc.rust-lang.org/std/hash/trait.Hash.html)

A `HashSet<T>` is a `HashMap<T, ()>` under the hood — it stores unique elements with O(1) lookup. Whenever you find yourself asking "is this item already in my Vec?", that is a signal to reach for a `HashSet` instead, because Vec linear search is O(n).

`HashSet` also supports the classic mathematical set operations: `union`, `intersection`, `difference`, and `symmetric_difference`. These return iterators of references, so you usually need to `collect` them.

Choose `HashSet` over `Vec` when: order does not matter, you need fast `contains` checks, and all elements are unique. If you need sorted iteration, use `BTreeSet` instead.

```rust
use std::collections::HashSet;

fn main() {
    let mut seen: HashSet<i32> = HashSet::new();
    seen.insert(1);
    seen.insert(2);
    seen.insert(3);
    seen.insert(2); // duplicate — silently ignored, returns false

    println!("Contains 2: {}", seen.contains(&2));
    println!("Contains 9: {}", seen.contains(&9));
    seen.remove(&1);
    println!("After removing 1, contains 1: {}", seen.contains(&1));

    // Set operations
    let set_a: HashSet<i32> = [1, 2, 3, 4].into_iter().collect();
    let set_b: HashSet<i32> = [3, 4, 5, 6].into_iter().collect();

    let mut union_sorted: Vec<i32> = set_a.union(&set_b).copied().collect();
    union_sorted.sort();
    println!("Union: {:?}", union_sorted);

    let mut intersection_sorted: Vec<i32> = set_a.intersection(&set_b).copied().collect();
    intersection_sorted.sort();
    println!("Intersection: {:?}", intersection_sorted);

    let mut diff_sorted: Vec<i32> = set_a.difference(&set_b).copied().collect();
    diff_sorted.sort();
    println!("A - B: {:?}", diff_sorted);
}
```

**Expected output:**
```
Contains 2: true
Contains 9: false
After removing 1, contains 1: false
Union: [1, 2, 3, 4, 5, 6]
Intersection: [3, 4]
A - B: [1, 2]
```

#### Exercise 3.1 — Duplicate Finder

**Goal:** Use a `HashSet` to find duplicate words in a sentence.

Given the sentence `"to be or not to be that is the question to"`, find all words that appear more than once. Print them in sorted order.

```rust
use std::collections::HashSet;

fn main() {
    let sentence = "to be or not to be that is the question to";
    // find duplicates using a HashSet to track seen words
}
```

**Expected output:**
```
Duplicates: ["be", "to"]
```

> **Hint:** Use two `HashSet`s — one for words you have seen once, one for confirmed duplicates.

---

### 4. `BTreeMap<K, V>` and `BTreeSet<T>` — Sorted Collections

> **Docs:** [BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) · [BTreeSet](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html)

`BTreeMap` and `BTreeSet` are the sorted siblings of `HashMap` and `HashSet`. They use a B-tree internally, keeping keys in sorted order at all times. Operations are O(log n) rather than O(1), but they enable two things `HashMap` cannot: **sorted iteration** and **range queries**.

```rust
use std::collections::{BTreeMap, BTreeSet};

fn main() {
    // BTreeMap — keys always iterated in sorted order
    let mut scores: BTreeMap<&str, u32> = BTreeMap::new();
    scores.insert("Charlie", 85);
    scores.insert("Alice", 92);
    scores.insert("Bob", 78);

    // Iterates alphabetically — Alice, Bob, Charlie
    for (name, score) in &scores {
        println!("{}: {}", name, score);
    }

    // Range query — everyone from "Alice" to "Bob" inclusive
    for (name, score) in scores.range("Alice"..="Bob") {
        println!("Range: {}: {}", name, score);
    }

    // BTreeSet — sorted, unique elements
    let mut tags: BTreeSet<&str> = BTreeSet::new();
    tags.insert("rust");
    tags.insert("async");
    tags.insert("cli");
    tags.insert("rust"); // duplicate ignored

    println!("{:?}", tags); // {"async", "cli", "rust"} — sorted
}
```

**CS connection:** BTreeMap/BTreeSet implement a **balanced search tree** (specifically a B-tree), the same structure covered conceptually in Day 11 and Day 16. The difference: the std library's B-tree is self-balancing, guaranteeing O(log n) worst case. A hand-rolled BST without balancing degrades to O(n).

#### Exercise 4.1 — Sorted Word Index

**Goal:** Build an alphabetically sorted index of which words appear on each "line" (use an array of strings as lines). Use a `BTreeMap<String, Vec<usize>>` where the key is a word and the value is the list of line numbers it appears on.

**Expected output (sorted alphabetically):**
```
hello: [1, 3]
rust: [1, 2]
world: [2]
```

> **Hint:** `scores.entry(word).or_default().push(line_num)` — `or_default()` inserts an empty `Vec` if the key doesn't exist yet.

---

### 5. `VecDeque<T>` — Double-Ended Queue

> **Docs:** [VecDeque](https://doc.rust-lang.org/std/collections/struct.VecDeque.html)

`VecDeque<T>` (vector deque) is a ring buffer that supports O(1) push and pop at **both** ends. `Vec` can only efficiently push/pop at the back — `remove(0)` is O(n) because every element shifts. `VecDeque` solves this.

```rust
use std::collections::VecDeque;

fn main() {
    let mut deque: VecDeque<i32> = VecDeque::new();

    // Push to both ends
    deque.push_back(1);
    deque.push_back(2);
    deque.push_front(0);   // [0, 1, 2]

    println!("Front: {:?}", deque.front()); // Some(0)
    println!("Back:  {:?}", deque.back());  // Some(2)

    deque.pop_front(); // removes 0 → [1, 2]
    deque.pop_back();  // removes 2 → [1]

    // Use as a FIFO queue
    let mut queue: VecDeque<&str> = VecDeque::with_capacity(10);
    queue.push_back("first");
    queue.push_back("second");
    queue.push_back("third");
    while let Some(item) = queue.pop_front() {
        println!("{}", item); // first, second, third
    }

    // Convert from Vec
    let v = vec![1, 2, 3, 4, 5];
    let mut dq: VecDeque<i32> = VecDeque::from(v);
    dq.push_front(0);
    println!("{:?}", dq); // [0, 1, 2, 3, 4, 5]
}
```

**CS connection:** `VecDeque` implements the **Deque** (double-ended queue) abstract data type — the most flexible sequential structure. A Stack (LIFO) is a Deque restricted to one end; a Queue (FIFO) uses opposite ends. Day 11 builds Queue and Stack wrappers around `VecDeque`.

#### Exercise 5.1 — Sliding Window

**Goal:** Use a `VecDeque<i32>` to compute the maximum value in every window of size 3 over the slice `[1, 3, -1, -3, 5, 3, 6, 7]`.

**Expected output:**
```
Windows of size 3: [3, 3, 5, 5, 6, 7]
```

> **Hint:** Push new elements to the back, pop old elements from the front when the window is full. Track the max in each window with `.iter().max()`.

---

### 6. `BinaryHeap<T>` — Priority Queue

> **Docs:** [BinaryHeap](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html)

`BinaryHeap<T>` is a **max-heap**: `pop()` always removes and returns the largest element. It is the standard library's priority queue. Requires `T: Ord`. For a min-heap, wrap in `std::cmp::Reverse<T>`.

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

fn main() {
    // Max-heap
    let mut heap = BinaryHeap::from([5, 1, 8, 3, 9, 2]);
    println!("peek (max): {:?}", heap.peek()); // Some(9)
    while let Some(n) = heap.pop() {
        print!("{} ", n); // 9 8 5 3 2 1 — descending
    }
    println!();

    // Min-heap with Reverse
    let mut min_heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();
    for n in [5, 1, 8, 3] { min_heap.push(Reverse(n)); }
    while let Some(Reverse(n)) = min_heap.pop() {
        print!("{} ", n); // 1 3 5 8 — ascending
    }
    println!();
}
```

**CS connection:** `BinaryHeap` implements a **binary heap** — a complete binary tree stored in an array. Used internally by Dijkstra's algorithm, heap sort, and task schedulers. Day 11 uses it to build a priority task queue. O(log n) push/pop, O(1) peek.

#### Exercise 6.1 — K Largest Elements

**Goal:** Given `[3, 1, 4, 1, 5, 9, 2, 6, 5, 3]`, use `BinaryHeap` to find the 3 largest elements without sorting.

**Expected output:**
```
Top 3 largest: [9, 6, 5]
```

> **Hint:** `BinaryHeap::from(vec)` builds a heap in O(n). Call `.pop()` three times.

---

### 7. The Complete `std::collections` Map

> **Docs:** [std::collections overview](https://doc.rust-lang.org/std/collections/) · [LinkedList](https://doc.rust-lang.org/std/collections/struct.LinkedList.html)

Every collection in the standard library, its CS equivalent, and when to reach for it:

| `std::collections` type | CS data structure | Best for |
|------------------------|-------------------|----------|
| `Vec<T>` | Dynamic array | Indexed access, push/pop at back, most sequences |
| `VecDeque<T>` | Deque / ring buffer | FIFO queues, sliding window, push/pop both ends |
| `LinkedList<T>` | Doubly linked list | O(1) insert/delete at known cursor (rare in practice) |
| `HashMap<K, V>` | Hash table | O(1) key lookup, counting, caching |
| `HashSet<T>` | Hash set | O(1) membership, deduplication, set operations |
| `BTreeMap<K, V>` | B-tree (balanced BST) | Sorted iteration, range queries |
| `BTreeSet<T>` | B-tree set | Sorted unique elements, range membership |
| `BinaryHeap<T>` | Binary max-heap | Priority queue, finding min/max repeatedly |

> `std::collections::LinkedList<T>` exists but is almost never the right choice — cache misses make it slower than `Vec` for most workloads. Prefer `VecDeque` for deque semantics. Use a hand-rolled linked list (Day 11 / Day 27) only when you need cursor-based insertion into a structure you control.

See [`DATA_STRUCTURES.md`](../DATA_STRUCTURES.md) for full coverage of each structure including complexity tables and implementation guidance.

---

## Day Project: Word Frequency Counter

### What You're Building

You will parse a hardcoded multi-line text passage, clean each word by stripping non-alphabetic characters, count occurrences with a `HashMap`, sort the results by frequency, and print the top 10 most common words alongside their counts. This is a classic text-analysis task that exercises all three collection types.

### Requirements

1. Define the source text as a `const` or `let` multi-line string literal inside `main`.
2. Split the text into words, convert each word to lowercase, and strip non-alphabetic characters using `.chars().filter(|c| c.is_alphabetic()).collect::<String>()`.
3. Skip empty strings (words that were pure punctuation become empty after stripping).
4. Count word frequencies using a `HashMap<String, usize>` and the entry API.
5. Collect the map's entries into a `Vec<(&String, &usize)>`, sort descending by count, then alphabetically for ties.
6. Print the top 10 words in the format `"  1. the           — 12 times"` (right-aligned rank, left-aligned word padded to 15 chars).

### Getting Started

```rust
fn main() {
    let text = "
        Two roads diverged in a yellow wood,
        And sorry I could not travel both
        And be one traveler, long I stood
        And looked down one as far as I could
        To where it bent in the undergrowth;
        Then took the other, as just as fair,
        And having perhaps the better claim,
        Because it was grassy and wanted wear;
        Though as for that the passing there
        Had worn them really about the same,
        And both that morning equally lay
        In leaves no step had trodden black.
        Oh, I kept the first for another day!
        Yet knowing how way leads on to way,
        I doubted if I should ever come back.
        I shall be telling this with a sigh
        Somewhere ages and ages hence:
        Two roads diverged in a wood, and I,
        I took the one less traveled by,
        And that has made all the difference.
    ";

    let mut counts: HashMap<String, usize> = HashMap::new();

    for word in text.split_whitespace() {
        let clean: String = word
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect::<String>()
            .to_lowercase();
        if clean.is_empty() {
            continue;
        }
        *counts.entry(clean).or_insert(0) += 1;
    }

    let mut freq_list: Vec<(&String, &usize)> = counts.iter().collect();
    freq_list.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));

    println!("Top 10 words in the poem:\n");
    for (rank, (word, count)) in freq_list.iter().take(10).enumerate() {
        println!("  {:2}. {:<15} — {} times", rank + 1, word, count);
    }
}
```

### Running Your Solution

```bash
cargo run -p day-08
```

Successful output will look like:
```
Top 10 words in the poem:

   1. and             — 8 times
   2. i               — 7 times
   3. the             — 6 times
   4. as              — 4 times
   ...
```

### Extension Challenges

- **Easy:** Also print the total number of unique words and the total word count at the bottom.
- **Medium:** Read the text from a file path provided as a command-line argument instead of a hardcoded string (`std::env::args` and `std::fs::read_to_string`).
- **Hard:** Build a full report: unique words, average frequency, median frequency, words that appear exactly once (hapax legomena), and a simple histogram using terminal bar characters.

---

## Quick Reference — Collection Methods Cheat Sheet

### `Vec<T>`

| Method | Returns | Notes |
|--------|---------|-------|
| `Vec::new()` | `Vec<T>` | empty vector |
| `vec![a, b, c]` | `Vec<T>` | initialise with values |
| `v.push(x)` | `()` | appends to back |
| `v.pop()` | `Option<T>` | removes from back |
| `v[i]` | `T` (ref) | panics if out of bounds |
| `v.get(i)` | `Option<&T>` | safe access |
| `v.len()` | `usize` | element count |
| `v.is_empty()` | `bool` | true when len == 0 |
| `v.contains(&x)` | `bool` | linear search |
| `v.retain(\|x\| pred)` | `()` | remove non-matching in place |
| `v.sort()` | `()` | requires `Ord` |
| `v.dedup()` | `()` | removes consecutive duplicates |
| `&v[a..b]` | `&[T]` | slice (borrows) |

### `HashMap<K, V>`

| Method | Returns | Notes |
|--------|---------|-------|
| `HashMap::new()` | `HashMap<K,V>` | |
| `map.insert(k, v)` | `Option<V>` | returns old value if key existed |
| `map.get(&k)` | `Option<&V>` | |
| `map.get_mut(&k)` | `Option<&mut V>` | |
| `map.remove(&k)` | `Option<V>` | |
| `map.contains_key(&k)` | `bool` | |
| `map.entry(k).or_insert(v)` | `&mut V` | insert if missing, return ref |
| `map.len()` | `usize` | |
| `map.keys()` | iterator | |
| `map.values()` | iterator | |
| `map.iter()` | `(&K, &V)` iterator | |

### `HashSet<T>`

| Method | Returns | Notes |
|--------|---------|-------|
| `HashSet::new()` | `HashSet<T>` | |
| `set.insert(x)` | `bool` | true if newly inserted |
| `set.contains(&x)` | `bool` | O(1) |
| `set.remove(&x)` | `bool` | true if was present |
| `set.union(&other)` | iterator | elements in either |
| `set.intersection(&other)` | iterator | elements in both |
| `set.difference(&other)` | iterator | in self but not other |
| `set.is_subset(&other)` | `bool` | |
| `set.is_disjoint(&other)` | `bool` | no elements in common |
