# Day 11: Generics

> **Project:** Generic `Stack<T>` — build a fully tested, type-safe stack data structure from scratch.

## Learning Objectives

By the end of today you will be able to:
- Write generic functions that work over any type satisfying given trait bounds
- Define generic structs and implement methods on them with and without additional bounds
- Explain monomorphization and why Rust generics have zero runtime overhead
- Use multiple type parameters in a struct
- Write generic code that compiles and test it with multiple concrete types

> **Note:** Today's project is a **library crate** (`src/lib.rs`) with no `main`. Run `cargo test -p day-11` to verify.

---

## Concepts

### 1. Generic Functions

> **Docs:** [Book — Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html) · [Book — Generic functions](https://doc.rust-lang.org/book/ch10-01-syntax.html#in-function-definitions) · [Rust by Example — Generics](https://doc.rust-lang.org/rust-by-example/generics.html)

A generic function is parameterised over one or more types written in angle brackets: `fn name<T>(arg: &[T]) -> &T`. The letter `T` is a convention; you can use any identifier (`Key`, `Item`, `A`). When you call the function, the compiler infers — or you specify explicitly — what `T` is, and generates a dedicated, fully optimised version for that concrete type.

Trait bounds restrict which types are allowed. A bound like `T: PartialOrd` tells the compiler "T must support comparison with `<`". Without a bound, you can only do things that work for every type (move it, copy a reference — nothing type-specific).

```rust
// T must implement PartialOrd to use < and >
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut biggest = &list[0];
    for item in &list[1..] {
        if item > biggest {
            biggest = item;
        }
    }
    biggest
}

fn main() {
    let numbers = vec![34, 50, 25, 100, 65];
    println!("Largest number: {}", largest(&numbers)); // 100

    let chars = vec!['y', 'm', 'a', 'q'];
    println!("Largest char: {}", largest(&chars)); // y

    // Explicit turbofish syntax when inference fails
    let result = largest::<f64>(&[1.5, 2.7, 0.3]);
    println!("Largest float: {result}"); // 2.7
}
```

The same source code, three different concrete functions generated at compile time. No dynamic dispatch, no virtual table — the generated machine code is identical to what you would write by hand for each type.

#### Exercise 1.1 — Generic Min and Max

**Goal:** Write two generic functions sharing a type parameter.

Write `fn min<T: PartialOrd>(a: T, b: T) -> T` and `fn max<T: PartialOrd>(a: T, b: T) -> T`. Call each with `i32`, `f64`, and `&str` values to confirm they work across types.

```rust
fn min<T: PartialOrd>(a: T, b: T) -> T {
    // your implementation
}

fn max<T: PartialOrd>(a: T, b: T) -> T {
    // your implementation
}

fn main() {
    println!("{}", min(3, 7));          // 3
    println!("{}", max(3.14, 2.72));    // 3.14
    println!("{}", min("apple", "zoo")); // apple
}
```

**Expected output:**
```
3
3.14
apple
```

> **Hint:** `if a < b { a } else { b }` works when `T: PartialOrd`. The function takes ownership of `a` and `b`, so returning one of them is fine.

---

### 2. Generic Structs

> **Docs:** [Book — Generic structs](https://doc.rust-lang.org/book/ch10-01-syntax.html#in-struct-definitions) · [Book — Trait bounds](https://doc.rust-lang.org/book/ch10-02-traits.html#trait-bound-syntax)

You can parameterise structs over types too. The type parameter appears in the `struct` definition and must be repeated on `impl` blocks.

When you write `impl<T> Pair<T>`, you are saying "for any type T, here are methods that every `Pair<T>` has." When you write `impl<T: Display + PartialOrd> Pair<T>`, you are adding methods that only exist for `Pair<T>` when `T` also satisfies those bounds — the struct itself compiles for any `T`, but those extra methods are only available when the bounds hold.

```rust
use std::fmt::Display;

pub struct Pair<T> {
    pub first: T,
    pub second: T,
}

// Methods available for ALL Pair<T>
impl<T> Pair<T> {
    pub fn new(first: T, second: T) -> Self {
        Pair { first, second }
    }
}

// Methods only available when T: Display + PartialOrd
impl<T: Display + PartialOrd> Pair<T> {
    pub fn print_largest(&self) {
        if self.first >= self.second {
            println!("Largest: {}", self.first);
        } else {
            println!("Largest: {}", self.second);
        }
    }
}

fn main() {
    let pair = Pair::new(5, 10);
    pair.print_largest(); // Largest: 10

    let words = Pair::new("hello", "world");
    words.print_largest(); // Largest: world

    // This would work even without Display + PartialOrd:
    let _data: Pair<Vec<i32>> = Pair::new(vec![1, 2], vec![3, 4]);
    // _data.print_largest(); // ERROR: Vec<i32> doesn't implement Display
}
```

#### Exercise 2.1 — Generic Triple

**Goal:** Define a generic struct with a method under a trait bound.

Define `pub struct Triple<T> { pub a: T, pub b: T, pub c: T }`. Add a method `pub fn sum(&self) -> T` available only when `T: Copy + std::ops::Add<Output = T>`. Test with `Triple<i32>` and `Triple<f64>`.

**Expected output:**
```
i32 sum: 60
f64 sum: 6.6
```

> **Hint:** `self.a + self.b + self.c` requires `T: Copy + Add<Output = T>`. The `Copy` bound lets you use `self.a` multiple times without moving.

---

### 3. Generic Enums

> **Docs:** [Book — Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html)

You already use generic enums every day in Rust — `Option<T>` and `Result<T, E>` are both defined as generic enums in the standard library. Here is a simplified version:

```rust
// The standard library's Option, written out
pub enum MyOption<T> {
    Some(T),
    None,
}

// Result has two type parameters
pub enum MyResult<T, E> {
    Ok(T),
    Err(E),
}

fn divide(a: f64, b: f64) -> MyResult<f64, String> {
    if b == 0.0 {
        MyResult::Err("division by zero".to_string())
    } else {
        MyResult::Ok(a / b)
    }
}

fn main() {
    match divide(10.0, 3.0) {
        MyResult::Ok(v)  => println!("Result: {v:.4}"),
        MyResult::Err(e) => println!("Error: {e}"),
    }
}
```

Understanding that `Option` and `Result` are just generic enums demystifies them — they are not magic keywords, just types defined in the standard library using the same generics feature you are learning today.

---

### 4. Monomorphization — Zero-Cost Generics

> **Docs:** [Reference — Generic parameters](https://doc.rust-lang.org/reference/items/generics.html)

Rust's generics are **monomorphized** at compile time. When the compiler sees `largest::<i32>` and `largest::<char>`, it generates two separate functions in the binary — one specialised for `i32`, one for `char`. The generated machine code is exactly what you would write if you had written two separate, type-specific functions by hand.

This is fundamentally different from Java's erasure (which loses type info at runtime) or Python's dynamic approach (which resolves methods at runtime). In Rust, you pay for generics only in compile time and binary size, never in runtime performance.

```rust
// You write this once:
fn identity<T>(x: T) -> T { x }

// The compiler generates (conceptually):
// fn identity_i32(x: i32) -> i32 { x }
// fn identity_String(x: String) -> String { x }

fn main() {
    let a = identity(42_i32);      // monomorphized to i32 version
    let b = identity("hello");     // monomorphized to &str version
    let c = identity(3.14_f64);    // monomorphized to f64 version
    println!("{a} {b} {c}");
}
```

The alternative to monomorphization is **dynamic dispatch** via trait objects (`&dyn Trait`), which you will explore on Day 15. The trade-off: trait objects have a small runtime overhead (one pointer indirection) but avoid code bloat and enable heterogeneous collections. Monomorphized generics are faster but generate larger binaries.

---

### 5. Multiple Type Parameters

> **Docs:** [Book — Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html)

Structs and functions can have more than one type parameter. Each gets its own letter and its own optional bounds.

```rust
use std::fmt::Display;

pub struct KeyValue<K, V> {
    pub key: K,
    pub value: V,
}

impl<K: Display, V: Display> KeyValue<K, V> {
    pub fn new(key: K, value: V) -> Self {
        KeyValue { key, value }
    }

    pub fn display(&self) {
        println!("{} => {}", self.key, self.value);
    }
}

fn swap<A, B>(pair: (A, B)) -> (B, A) {
    (pair.1, pair.0)
}

fn main() {
    let kv = KeyValue::new("name", "Alice");
    kv.display(); // name => Alice

    let kv2 = KeyValue::new(42_i32, 3.14_f64);
    kv2.display(); // 42 => 3.14

    let swapped = swap((1, "one"));
    println!("{:?}", swapped); // ("one", 1)
}
```

---

### 6. Queue — First In, First Out

> **Docs:** [std::collections](https://doc.rust-lang.org/std/collections/)

A **queue** is a FIFO (first in, first out) data structure: items are added at the back and removed from the front. Real-world analogy: a ticket queue — you join the back, you're served from the front. Operations: `enqueue` (add to back), `dequeue` (remove from front), `peek` (look at front without removing), `is_empty`, `len`.

Implement a `Queue<T>` backed by `VecDeque<T>` from the standard library. `VecDeque` is a double-ended queue — both `push_back` and `pop_front` are O(1), unlike a plain `Vec` where `remove(0)` is O(n).

```rust
use std::collections::VecDeque;

pub struct Queue<T> {
    data: VecDeque<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { data: VecDeque::new() }
    }

    pub fn enqueue(&mut self, item: T) {
        self.data.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.front()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

fn main() {
    let mut q: Queue<&str> = Queue::new();
    q.enqueue("Alice");
    q.enqueue("Bob");
    q.enqueue("Carol");

    println!("Front of queue: {:?}", q.peek()); // Some("Alice")

    while let Some(person) = q.dequeue() {
        println!("Serving: {}", person);
    }
    // Alice served first — FIFO
}
```

**Queue vs Stack:** The only difference is where you remove — stack removes from the top (LIFO), queue removes from the front (FIFO). Both can be implemented with `VecDeque`.

#### Exercise 6.1 — Breadth-First Print

**Goal:** Use a `Queue<i32>` to print the numbers 1–10 in FIFO order, then drain a second queue in FIFO order to verify ordering.

Enqueue numbers 1 through 10. Print each as you dequeue. Then enqueue the letters `'a'` through `'e'` into a `Queue<char>` and drain it — confirm FIFO ordering.

**Expected output:**
```
1 2 3 4 5 6 7 8 9 10
a b c d e
```

> **Hint:** Use a `while let Some(n) = q.dequeue()` loop. Print with `print!("{} ", n)` inside the loop, then `println!()` after.

---

### 7. Singly Linked List — Enum + Box Approach

> **Docs:** [LinkedList](https://doc.rust-lang.org/std/collections/struct.LinkedList.html)

A **linked list** is a sequence of nodes where each node holds a value and a pointer to the next node. Unlike a `Vec`, a linked list does not require contiguous memory — each node can be anywhere on the heap. Trade-offs: O(1) prepend, O(n) access by index, poor cache locality.

In safe Rust, a recursive enum with `Box` is the clean way to represent a singly linked list. `Box<T>` gives the `next` field a known size (a pointer), solving the "infinitely-sized type" problem.

```rust
// A list is either empty (Nil) or a value followed by the rest of the list (Cons)
#[derive(Debug)]
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T: std::fmt::Debug> List<T> {
    fn new() -> Self {
        List::Nil
    }

    // Prepend: O(1) — creates a new head node
    fn prepend(self, value: T) -> Self {
        List::Cons(value, Box::new(self))
    }

    // Count nodes: O(n) — must walk the whole list
    fn len(&self) -> usize {
        match self {
            List::Nil => 0,
            List::Cons(_, tail) => 1 + tail.len(),
        }
    }

    // Print all elements
    fn print(&self) {
        match self {
            List::Nil => println!(),
            List::Cons(val, tail) => {
                print!("{:?} → ", val);
                tail.print();
            }
        }
    }
}

fn main() {
    let list = List::new()
        .prepend(3)
        .prepend(2)
        .prepend(1);

    list.print();              // 1 → 2 → 3 →
    println!("len: {}", list.len()); // 3
}
```

**Why this beats unsafe for most uses:** This approach is fully safe, easy to reason about, and covers the common cases. Day 27 shows how to implement a linked list with raw pointers when you need mutable access from both ends or O(1) append — at the cost of `unsafe` code.

#### Exercise 7.1 — String Stack via Linked List

**Goal:** Build a `Stack<String>` using the `List<T>` enum instead of `Vec<T>`. Implement `push` (prepend), `pop` (deconstruct the head), and `peek` (return a reference to the head value).

```rust
struct Stack<T> {
    head: List<T>,
}

impl<T: std::fmt::Debug> Stack<T> {
    fn new() -> Self { Stack { head: List::Nil } }
    fn push(&mut self, val: T) { /* replace head with Cons(val, old_head) */ }
    fn pop(&mut self) -> Option<T> { /* deconstruct head, return the value */ }
    fn peek(&self) -> Option<&T> { /* return reference to head value */ }
}
```

Push `"hello"`, `"world"`, `"!"` then pop all three and print them.

**Expected output:**
```
Popped: !
Popped: world
Popped: hello
```

> **Hint:** For `push`, use `std::mem::replace(&mut self.head, List::Nil)` to take ownership of the current head, then build `List::Cons(val, Box::new(old_head))` as the new head.

---

### 8. Priority Queue — `BinaryHeap`

> **Docs:** [BinaryHeap](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html)

A **priority queue** dequeues the *highest-priority* item first, regardless of insertion order. Rust's standard library provides `std::collections::BinaryHeap<T>`, a max-heap: `pop()` always returns the largest element. For a min-heap (smallest first), wrap values in `std::cmp::Reverse`.

`BinaryHeap` requires `T: Ord`. For custom types, implement `PartialOrd` and `Ord` — or `#[derive]` them.

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

fn main() {
    // Max-heap (largest first)
    let mut max_heap = BinaryHeap::new();
    max_heap.push(3);
    max_heap.push(1);
    max_heap.push(4);
    max_heap.push(1);
    max_heap.push(5);

    print!("Max-heap order: ");
    while let Some(n) = max_heap.pop() {
        print!("{} ", n); // 5 4 3 1 1
    }
    println!();

    // Min-heap using Reverse
    let mut min_heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();
    for n in [3, 1, 4, 1, 5] {
        min_heap.push(Reverse(n));
    }

    print!("Min-heap order: ");
    while let Some(Reverse(n)) = min_heap.pop() {
        print!("{} ", n); // 1 1 3 4 5
    }
    println!();
}
```

**Custom priority with a struct:**

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Eq, PartialEq)]
struct Task {
    priority: u32,
    name: String,
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority) // higher priority = larger
    }
}
impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut queue = BinaryHeap::new();
    queue.push(Task { priority: 1, name: "Low".into() });
    queue.push(Task { priority: 10, name: "Critical".into() });
    queue.push(Task { priority: 5, name: "Medium".into() });

    while let Some(task) = queue.pop() {
        println!("[{}] {}", task.priority, task.name);
    }
    // [10] Critical
    // [5] Medium
    // [1] Low
}
```

#### Exercise 8.1 — Top K Elements

**Goal:** Given a `Vec<i32>` of 10 numbers, use a `BinaryHeap` to find the top 3 largest without sorting the whole Vec.

Push all numbers into a `BinaryHeap`, then call `pop()` three times and print the results.

**Expected output (for input `[3,1,4,1,5,9,2,6,5,3]`):**
```
Top 3: 9, 6, 5
```

> **Hint:** `BinaryHeap::from(vec)` converts a Vec into a heap in O(n) — faster than sorting.

---

### 9. Graph — Adjacency List

> **Docs:** [std::collections](https://doc.rust-lang.org/std/collections/)

A **graph** is a set of nodes (vertices) connected by edges. An **adjacency list** represents the graph as a `HashMap` where each key is a node and its value is a list of its neighbours — memory-efficient for sparse graphs.

Directed vs undirected: in a directed graph, an edge from A to B does not imply an edge from B to A. The adjacency list only stores outgoing edges.

```rust
use std::collections::{HashMap, HashSet, VecDeque};

struct Graph {
    edges: HashMap<String, Vec<String>>,
}

impl Graph {
    fn new() -> Self {
        Graph { edges: HashMap::new() }
    }

    fn add_edge(&mut self, from: &str, to: &str) {
        self.edges.entry(from.to_string()).or_default().push(to.to_string());
        // For undirected: also add the reverse
        self.edges.entry(to.to_string()).or_default().push(from.to_string());
    }

    fn neighbours(&self, node: &str) -> &[String] {
        self.edges.get(node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    // Breadth-first search — returns nodes in visit order
    fn bfs(&self, start: &str) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut order = Vec::new();

        queue.push_back(start.to_string());
        visited.insert(start.to_string());

        while let Some(node) = queue.pop_front() {
            order.push(node.clone());
            for neighbour in self.neighbours(&node) {
                if !visited.contains(neighbour) {
                    visited.insert(neighbour.clone());
                    queue.push_back(neighbour.clone());
                }
            }
        }
        order
    }

    // Depth-first search — recursive
    fn dfs(&self, start: &str, visited: &mut HashSet<String>, order: &mut Vec<String>) {
        if visited.contains(start) { return; }
        visited.insert(start.to_string());
        order.push(start.to_string());
        for neighbour in self.neighbours(start) {
            self.dfs(neighbour, visited, order);
        }
    }
}

fn main() {
    let mut g = Graph::new();
    g.add_edge("A", "B");
    g.add_edge("A", "C");
    g.add_edge("B", "D");
    g.add_edge("C", "D");
    g.add_edge("D", "E");

    println!("BFS from A: {:?}", g.bfs("A"));
    // ["A", "B", "C", "D", "E"]

    let mut visited = std::collections::HashSet::new();
    let mut dfs_order = Vec::new();
    g.dfs("A", &mut visited, &mut dfs_order);
    println!("DFS from A: {:?}", dfs_order);
    // ["A", "B", "D", "C", "E"] (order may vary by neighbour insertion)
}
```

**Time complexity:** BFS and DFS are both O(V + E) where V = vertices, E = edges. The adjacency list representation uses O(V + E) space. A full adjacency matrix uses O(V²) space — better for dense graphs, worse for sparse ones.

#### Exercise 9.1 — Shortest Path

**Goal:** Extend the `Graph` with a `shortest_path(start: &str, end: &str) -> Option<Vec<String>>` method that uses BFS to find the shortest path between two nodes. BFS guarantees the shortest path in an unweighted graph because it explores nodes level by level.

**Expected output (for the graph above):**
```
Shortest path A → E: ["A", "B", "D", "E"]
No path found: None
```

> **Hint:** During BFS, store a `HashMap<String, String>` tracking each node's parent. When you reach the destination, trace back through parents to reconstruct the path. Reverse the path before returning.

---

## Day Project: Data Structures Library

### What You're Building

You will implement a small data structures library with four types: `Stack<T>`, `Queue<T>`, a singly linked `List<T>`, and a weighted `Graph`. Each type is generic (where applicable) and lives in its own module. A `main.rs` (or `examples/demo.rs` for a library crate) demonstrates all four with realistic usage scenarios.

### Requirements

1. **`Stack<T>`** — generic, Vec-backed. Methods: `push`, `pop`, `peek`, `is_empty`, `len`. Implement `Default`.
2. **`Queue<T>`** — generic, VecDeque-backed. Methods: `enqueue`, `dequeue`, `peek`, `is_empty`, `len`. Implement `Default`.
3. **`List<T>`** — recursive enum (`Cons(T, Box<List<T>>)` / `Nil`). Methods: `prepend`, `len`, `to_vec() -> Vec<&T>`. Implement `Display` to print `1 → 2 → 3 → Nil`.
4. **`Graph`** — adjacency list (`HashMap<String, Vec<String>>`). Methods: `add_edge`, `neighbours`, `bfs`, `dfs`, `shortest_path`.
5. All four types must have unit tests (`#[cfg(test)]`).
6. The demo shows each structure being built, used, and printed.

### Getting Started

```rust
// src/lib.rs
pub mod stack;
pub mod queue;
pub mod list;
pub mod graph;
```

Start by scaffolding the four modules in separate files, then implement each one.

### Running Your Solution

```bash
cargo test -p day-11          # run all unit tests
cargo run -p day-11           # run the demo (if binary crate)
```

Expected output (abridged):
```
=== Stack ===
Pushed: 1, 2, 3 | Peek: 3
Popped: 3, 2, 1

=== Queue ===
Enqueued: A, B, C | Front: A
Dequeued: A, B, C

=== Linked List ===
1 → 2 → 3 → Nil  (len: 3)

=== Graph BFS from A ===
A → B → C → D → E

Shortest path A → E: [A, B, D, E]
```

### Extension Challenges

- **Easy:** Add a `size_hint` method to `Stack` and `Queue` that returns the current memory usage in bytes (`len() * std::mem::size_of::<T>()`).
- **Medium:** Implement a `DoublyLinkedList<T>` using `Option<Rc<RefCell<Node<T>>>>` for both `prev` and `next` pointers (preview of Day 16's smart pointer patterns).
- **Hard:** Add weighted edges to `Graph` — `add_weighted_edge(from, to, weight: u32)` — and implement Dijkstra's shortest path algorithm using `BinaryHeap` (preview of Day 11's priority queue).
