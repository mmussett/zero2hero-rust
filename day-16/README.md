# Day 16: Smart Pointers — Box, Rc, RefCell

> **Project:** Binary Search Tree and Shared Graph — implement a BST with `Box<Node>` then model shared graph nodes with `Rc<RefCell<GraphNode>>`.

## Learning Objectives

By the end of today you will be able to:
- Use `Box<T>` to heap-allocate values and resolve recursive type size errors.
- Build a recursive data structure (binary search tree) using `Box`.
- Share ownership across multiple variables with `Rc<T>` and understand its single-threaded limitation.
- Use `RefCell<T>` to achieve interior mutability when the compiler cannot verify borrows at compile time.
- Combine `Rc<RefCell<T>>` for shared mutable state in single-threaded programs.

---

## Concepts

### 1. `Box<T>` — Heap Allocation and Deref Coercion

> **Docs:** [Box<T>](https://doc.rust-lang.org/std/boxed/struct.Box.html) · [Book — Box<T>](https://doc.rust-lang.org/book/ch15-01-box.html) · [Deref trait](https://doc.rust-lang.org/std/ops/trait.Deref.html) · [Book — Deref](https://doc.rust-lang.org/book/ch15-02-deref.html)

`Box<T>` is the simplest smart pointer: it allocates `T` on the heap and gives you a stack-resident pointer to it. When the `Box` goes out of scope, it drops both itself and the heap allocation. There is no reference counting or locking — just a single owner.

You use `Box<T>` for three reasons: to fix *recursive type* size errors (the next section), to move large values cheaply by passing a pointer instead of copying bytes, and to store trait objects (`Box<dyn Trait>` from Day 15). Because `Box<T>` implements `Deref<Target = T>`, Rust's deref coercions let you use it almost everywhere you would use a `&T`.

```rust
fn main() {
    // Allocate an i32 on the heap
    let boxed: Box<i32> = Box::new(42);

    // Deref to read the inner value
    println!("value: {}", *boxed);

    // Deref coercion — works anywhere &i32 is expected
    let r: &i32 = &boxed;
    println!("via coercion: {}", r);

    // Large struct — moving the Box is just copying a pointer
    let big = Box::new([0u8; 1_000_000]);
    let also_big = big; // moved: only the pointer moved, not 1 MB of data
    println!("first byte: {}", also_big[0]);
} // also_big drops here; heap memory freed automatically
```

#### Exercise 1.1 — Box Basics

**Goal:** Practice allocating, dereferencing, and moving `Box<T>` values.

Create a `struct Heavyweight { data: Vec<u64> }` that holds `1_000` elements. Create it on the stack, move it into a `Box`, then pass the `Box` to a function `fn print_sum(hw: Box<Heavyweight>)` that prints the sum of `hw.data`. Confirm that `hw` is moved into the function and dropped there.

```rust
struct Heavyweight {
    data: Vec<u64>,
}

fn print_sum(hw: Box<Heavyweight>) {
    // your code here
}

fn main() {
    let hw = Heavyweight { data: (1..=1_000).collect() };
    let boxed = Box::new(hw);
    print_sum(boxed);
    // hw and boxed are both moved — you cannot use them here
}
```

**Expected output:**
```
Sum: 500500
```

> **Hint:** `hw.data.iter().sum::<u64>()` computes the sum through the `Box` via deref coercion.

---

### 2. Recursive Types with `Box`

> **Docs:** [Box<T>](https://doc.rust-lang.org/std/boxed/struct.Box.html) · [Book — Box<T>](https://doc.rust-lang.org/book/ch15-01-box.html)

A recursive type is one whose definition includes itself: a tree node that contains child nodes of the same type, a linked list whose `Cons` variant holds another list. The compiler must know the size of every type at compile time, and a naively recursive type has infinite size — so Rust rejects it.

The fix is to insert a `Box` at the recursive position. `Box<T>` has a known, fixed size (one pointer-width), regardless of how large `T` is. This breaks the infinite recursion in the size calculation.

```rust
// This does NOT compile — infinite size
// enum List {
//     Cons(i32, List),
//     Nil,
// }

// This compiles — Box has a known size
#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    fn new() -> Self { List::Nil }

    fn prepend(self, value: i32) -> Self {
        List::Cons(value, Box::new(self))
    }

    fn sum(&self) -> i32 {
        match self {
            List::Nil => 0,
            List::Cons(val, rest) => val + rest.sum(),
        }
    }
}

fn main() {
    let list = List::new()
        .prepend(3)
        .prepend(2)
        .prepend(1);

    println!("List: {:?}", list);
    println!("Sum: {}", list.sum());
}
```

**Expected output:**
```
List: Cons(1, Cons(2, Cons(3, Nil)))
Sum: 6
```

#### Exercise 2.1 — Recursive Length

**Goal:** Extend the linked list above with a `len` method.

Add `fn len(&self) -> usize` to the `List` implementation above. It should return `0` for `Nil` and `1 + rest.len()` for `Cons`. Test it with a list of 5 elements.

**Expected output:**
```
Length: 5
```

> **Hint:** The structure mirrors `sum` — replace `val +` with `1 +` and ignore `val`.

---

### 3. `Rc<T>` — Shared Ownership

> **Docs:** [Rc<T>](https://doc.rust-lang.org/std/rc/struct.Rc.html) · [Book — Rc<T>](https://doc.rust-lang.org/book/ch15-04-rc.html)

Rust's ownership model normally allows exactly one owner per value. `Rc<T>` (Reference Counted) relaxes this: it lets multiple variables be *co-owners* of the same heap allocation. The allocation is freed when the last `Rc` pointing to it is dropped.

`Rc::clone(&rc)` increments the reference count and returns a new `Rc` pointing to the same data — it does *not* clone the data. This is cheap. `Rc::strong_count(&rc)` lets you inspect the count. Because updating the count uses ordinary (non-atomic) CPU instructions, `Rc<T>` is not safe to share across threads — use `Arc<T>` for that.

Data accessed through `Rc<T>` is immutable — you can only get a `&T`. If you need mutation, combine it with `RefCell<T>` (Section 4).

```rust
use std::rc::Rc;

#[derive(Debug)]
struct Config {
    debug: bool,
    max_retries: u32,
}

fn main() {
    let config = Rc::new(Config { debug: true, max_retries: 3 });

    println!("Initial count: {}", Rc::strong_count(&config)); // 1

    let module_a = Rc::clone(&config);
    let module_b = Rc::clone(&config);

    println!("After cloning: {}", Rc::strong_count(&config)); // 3

    // All three point to the same Config on the heap
    println!("module_a debug: {}", module_a.debug);
    println!("module_b retries: {}", module_b.max_retries);

    drop(module_a);
    println!("After dropping module_a: {}", Rc::strong_count(&config)); // 2
} // module_b and config drop here; count reaches 0; Config freed
```

**Expected output:**
```
Initial count: 1
After cloning: 3
module_a debug: true
module_b retries: 3
After dropping module_a: 2
```

#### Exercise 3.1 — Shared Catalog

**Goal:** Practice sharing ownership with `Rc`.

Create a `Vec<String>` catalog of 3 book titles. Wrap it in `Rc::new(...)`. Clone the `Rc` into two "library branches" (`branch_a` and `branch_b`). In a separate function `fn count_books(catalog: &Rc<Vec<String>>) -> usize`, return the length of the catalog. Call it from both branches and assert the results are equal.

**Expected output:**
```
Branch A has 3 books
Branch B has 3 books
Reference count: 3
```

> **Hint:** Pass `&Rc<Vec<String>>` to the function — dereferencing gives you `&Vec<String>` through deref coercion.

---

### 4. `RefCell<T>` — Interior Mutability

> **Docs:** [RefCell<T>](https://doc.rust-lang.org/std/cell/struct.RefCell.html) · [Book — RefCell](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) · [Cell<T>](https://doc.rust-lang.org/std/cell/struct.Cell.html)

Rust's borrow checker enforces at compile time that you have either one mutable reference or any number of immutable references. Sometimes this is too restrictive: you know the borrows are valid, but the compiler cannot see it (e.g., a single-threaded callback that needs to mutate shared state).

`RefCell<T>` defers the borrow check to runtime. `borrow()` returns a `Ref<T>` guard (like a `&T`), and `borrow_mut()` returns a `RefMut<T>` guard (like a `&mut T`). If you violate the rules — two concurrent `borrow_mut()` calls — `RefCell` *panics* at runtime rather than failing at compile time.

Use `RefCell` sparingly and as close to the smallest possible scope as you can, to minimise the window where a panic is possible.

```rust
use std::cell::RefCell;

fn main() {
    let data = RefCell::new(vec![1, 2, 3]);

    // Immutable borrow
    {
        let view = data.borrow();
        println!("Read: {:?}", *view);
    } // view dropped here — borrow ends

    // Mutable borrow
    {
        let mut view = data.borrow_mut();
        view.push(4);
    } // view dropped here — mutable borrow ends

    println!("After push: {:?}", data.borrow());

    // This would PANIC at runtime (two borrow_muts at once):
    // let _a = data.borrow_mut();
    // let _b = data.borrow_mut(); // panic!
}
```

**Expected output:**
```
Read: [1, 2, 3]
After push: [1, 2, 3, 4]
```

#### Exercise 4.1 — Mock Logger

**Goal:** Use `RefCell` to implement a logger that records messages for later inspection.

Create a `struct MockLogger { messages: RefCell<Vec<String>> }`. Implement `fn log(&self, msg: &str)` (note: `&self`, not `&mut self`) that appends to `messages` via `borrow_mut()`. Implement `fn dump(&self)` that prints all recorded messages. In `main`, call `log` three times and then `dump`.

```rust
use std::cell::RefCell;

struct MockLogger {
    messages: RefCell<Vec<String>>,
}

impl MockLogger {
    fn new() -> Self { MockLogger { messages: RefCell::new(Vec::new()) } }
    fn log(&self, msg: &str) { /* your code */ }
    fn dump(&self) { /* your code */ }
}
```

**Expected output:**
```
[0] Starting up
[1] Processing request
[2] Shutting down
```

> **Hint:** `for (i, msg) in self.messages.borrow().iter().enumerate()` iterates the logged messages inside the borrow guard.

---

### 5. `Rc<RefCell<T>>` — Shared Mutable State

> **Docs:** [Rc<T>](https://doc.rust-lang.org/std/rc/struct.Rc.html) · [RefCell<T>](https://doc.rust-lang.org/std/cell/struct.RefCell.html) · [Weak<T>](https://doc.rust-lang.org/std/rc/struct.Weak.html)

`Rc<T>` gives shared ownership; `RefCell<T>` gives interior mutability. Combined as `Rc<RefCell<T>>`, you get multiple owners who can all mutate the shared value — the standard pattern for shared mutable state in single-threaded Rust.

Graph and tree structures where nodes need to know about each other are the textbook example. A parent might hold `Rc<RefCell<Child>>` so that it can mutate the child later, and the child might hold a weak reference back to the parent to avoid reference cycles.

```rust
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct Counter {
    value: i32,
}

fn main() {
    let counter = Rc::new(RefCell::new(Counter { value: 0 }));

    let c1 = Rc::clone(&counter);
    let c2 = Rc::clone(&counter);

    // Both c1 and c2 can mutate the shared Counter
    c1.borrow_mut().value += 10;
    c2.borrow_mut().value += 5;

    println!("Final value: {}", counter.borrow().value); // 15
    println!("Owners: {}", Rc::strong_count(&counter));  // 3
}
```

**Expected output:**
```
Final value: 15
Owners: 3
```

#### Exercise 5.1 — Shared Budget

**Goal:** Practice `Rc<RefCell<T>>` with a realistic scenario.

Create a `struct Budget { remaining: f64 }`. Wrap it in `Rc<RefCell<Budget>>`. Create two "department" variables that each hold a clone of the `Rc`. Have department A deduct `500.0` and department B deduct `300.0`. After both deductions, print the remaining budget from the original `Rc`. Assert it equals `200.0` (start with `1000.0`).

**Expected output:**
```
Remaining budget: 200.00
```

> **Hint:** `budget.borrow_mut().remaining -= 500.0;` modifies through the combined smart pointer.

---

## Day Project: BST and Graph Nodes

### What You're Building

You will build two data structures that showcase why different smart pointers exist. Part A implements a binary search tree using `Box<Node>` — each node owns its children exclusively. Part B models graph nodes using `Rc<RefCell<GraphNode>>` — nodes can be shared between multiple neighbours because a graph may have multiple paths to the same node.

### Requirements

**Part A — Binary Search Tree:**
1. Define `struct Node { value: i32, left: Option<Box<Node>>, right: Option<Box<Node>> }`.
2. Define `struct BinarySearchTree { root: Option<Box<Node>> }`.
3. Implement `fn insert(&mut self, value: i32)` using iterative or recursive logic.
4. Implement `fn contains(&self, value: i32) -> bool`.
5. Implement `fn in_order(&self) -> Vec<i32>` returning values in sorted order.
6. In `main`, insert values `[5, 3, 7, 1, 4, 6, 8]` and print the in-order traversal.

**Part B — Graph Nodes:**
1. Define `struct GraphNode { value: i32, neighbours: Vec<Rc<RefCell<GraphNode>>> }`.
2. Implement `fn add_neighbour(&mut self, node: Rc<RefCell<GraphNode>>)`.
3. Implement `fn print_neighbours(&self)` that prints each neighbour's value.
4. In `main`, create nodes A(1), B(2), C(3). Add B and C as neighbours of A. Add A as a neighbour of B. Print neighbours for both A and B to show sharing.

### Getting Started

```bash
cargo new day-16
cd day-16
```

No external dependencies needed.

`src/main.rs` structure:

```rust
use std::rc::Rc;
use std::cell::RefCell;

// Part A — BST
struct Node { /* ... */ }
struct BinarySearchTree { /* ... */ }

// Part B — Graph
struct GraphNode { /* ... */ }

fn main() {
    println!("=== Part A: Binary Search Tree ===");
    // BST demo

    println!("\n=== Part B: Graph Nodes ===");
    // Graph demo
}
```

### Running Your Solution

```bash
cargo run -p day-16
```

Successful output:

```
=== Part A: Binary Search Tree ===
In-order: [1, 3, 4, 5, 6, 7, 8]
Contains 4: true
Contains 9: false

=== Part B: Graph Nodes ===
Node 1 neighbours: [2, 3]
Node 2 neighbours: [1]
```

### Extension Challenges

- **Easy:** Add a `fn height(&self) -> usize` method to `BinarySearchTree` that returns the height of the tree.
- **Medium:** Add a `fn delete(&mut self, value: i32)` method to `BinarySearchTree`.
- **Hard:** Use `Weak<RefCell<GraphNode>>` for back-edges to avoid reference cycles. Implement a graph with a cycle (A -> B -> A) and verify it does not leak memory by printing a message in `GraphNode`'s `Drop` implementation.
