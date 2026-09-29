# Day 27: Unsafe Rust

> **Project:** Singly-linked list with raw pointers — implement a safe public API backed by unsafe pointer manipulation

## Learning Objectives

By the end of today you will be able to:
- List the five things `unsafe` enables that safe Rust does not
- Create, use, and free raw pointers (`*const T`, `*mut T`) correctly
- Explain when using `unsafe` is justified and when it is not
- Build a safe public API that wraps unsafe internals
- Implement `Drop` for a type that owns heap memory via raw pointers

---

## Concepts

### 1. What `unsafe` Enables

> **Docs:** [Unsafe Rust Book chapter](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html) · [Rustonomicon](https://doc.rust-lang.org/nomicon/) · [Rust by Example — Unsafe](https://doc.rust-lang.org/rust-by-example/unsafe.html)

`unsafe` is not a mode that disables the borrow checker. The borrow checker, type system, and all other Rust guarantees remain active inside an `unsafe` block. What `unsafe` does is unlock exactly five additional capabilities:

1. **Dereference raw pointers** (`*const T`, `*mut T`) — the compiler cannot verify these are valid
2. **Call `unsafe` functions or methods** — the function declares preconditions the caller must uphold
3. **Implement `unsafe` traits** — like `Send` and `Sync`, where you manually attest to thread safety
4. **Access or modify mutable static variables** — global mutable state is inherently racy
5. **Access fields of `union` types** — unions overlap memory; only one field is valid at a time

Everything else — the borrow checker, lifetime rules, type inference, trait dispatch, match exhaustiveness — still applies inside `unsafe`. Most Rust code that uses `unsafe` is unsafe in a very narrow, well-understood way. The programmer's job is to uphold the invariants that the compiler cannot check.

The most important mental model: `unsafe` is a **promise from the programmer to the compiler**. When you write `unsafe { *ptr }`, you are asserting "I know this pointer is valid, non-null, aligned, and that no other code is mutating the pointee right now." If your assertion is wrong, you get undefined behaviour — the same as C.

```rust
fn safe_function(x: &i32) -> i32 {
    *x  // safe: borrow checker ensures x is valid
}

unsafe fn unsafe_function(ptr: *const i32) -> i32 {
    *ptr  // unsafe: caller must guarantee ptr is valid
}

fn demonstrate() {
    let val = 42_i32;
    let ptr: *const i32 = &val as *const i32;

    // Creating a raw pointer is safe
    // Dereferencing requires unsafe
    let result = unsafe { *ptr };
    println!("{}", result);
}
```

#### Exercise 1.1 — Five Operations

**Goal:** Demonstrate each of the five unsafe capabilities.

Write five separate examples, each in its own function: (1) dereference a raw pointer, (2) call an `unsafe fn`, (3) implement `unsafe trait Zap for MyType {}` (a dummy unsafe trait you define), (4) read from a `static mut` variable, (5) access a union field. Each should compile and run without UB when used correctly.

**Expected output:**
```
raw ptr: 42
unsafe fn: 100
unsafe trait impl: ok
static mut: 0
union field: 3.14
```

> **Hint:** For `static mut`, the access must be inside `unsafe { }`. For unions, define `union FloatOrInt { f: f32, i: u32 }` and read the field you wrote.

---

### 2. Raw Pointers

> **Docs:** [std::ptr](https://doc.rust-lang.org/std/ptr/) · [raw pointer](https://doc.rust-lang.org/std/primitive.pointer.html) · [std::ptr::read](https://doc.rust-lang.org/std/ptr/fn.read.html) · [std::ptr::write](https://doc.rust-lang.org/std/ptr/fn.write.html)

Raw pointers are created by casting a reference: `&val as *const T` or `&mut val as *mut T`. They can also be created with `std::ptr::null()` or `std::ptr::null_mut()` for null pointers. Unlike references, raw pointers:

- Have no lifetime
- Are not guaranteed to be non-null
- Are not guaranteed to be aligned
- Do not enforce exclusive access (`*mut T` doesn't prevent you from having two)
- Are not automatically dereferenced

The programmer is responsible for all of these. The most common bugs with raw pointers are use-after-free (the pointed-to value was dropped), double-free (freeing the same allocation twice), and reading uninitialized memory.

```rust
use std::alloc::{alloc, dealloc, Layout};

unsafe fn allocate_box(value: u32) -> *mut u32 {
    let layout = Layout::new::<u32>();
    let ptr = alloc(layout) as *mut u32;
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    ptr.write(value);  // write the value without reading the uninitialized memory
    ptr
}

unsafe fn free_box(ptr: *mut u32) {
    let layout = Layout::new::<u32>();
    dealloc(ptr as *mut u8, layout);
}

fn main() {
    unsafe {
        let ptr = allocate_box(99);
        println!("value: {}", *ptr);
        free_box(ptr);
        // ptr is now dangling — do NOT dereference it again
    }
}
```

Prefer `Box::into_raw` and `Box::from_raw` over manual `alloc`/`dealloc` — they use the same allocator and are harder to misuse:

```rust
let boxed = Box::new(42_u32);
let ptr: *mut u32 = Box::into_raw(boxed);  // Box is consumed, heap remains
// ... use ptr ...
let _reclaimed = unsafe { Box::from_raw(ptr) };  // Box takes ownership again; drops on scope exit
```

#### Exercise 2.1 — Round-Trip a Box

**Goal:** Practice `Box::into_raw` and `Box::from_raw`.

Create a `Box<String>` with value "hello". Convert it to a raw pointer with `into_raw`. In a separate function that takes `*mut String`, mutate the string by calling `.push_str(" world")`. Back in main, reclaim the box with `from_raw` and print the value.

**Expected output:**
```
hello world
```

> **Hint:** Between `into_raw` and `from_raw`, the memory is neither freed nor managed. It is your responsibility not to lose the pointer or call `from_raw` more than once.

---

### 3. When `unsafe` Is Justified

> **Docs:** [Nomicon — unsafe intro](https://doc.rust-lang.org/nomicon/what-unsafe-does.html) · [Reference — unsafe keyword](https://doc.rust-lang.org/reference/unsafe-keyword.html)

Unsafe Rust is appropriate in these situations — and generally nowhere else:

**FFI (Foreign Function Interface):** Every C function is declared `unsafe` because Rust cannot verify C's contracts. Calling `libc::strlen` requires you to guarantee the pointer points to a null-terminated string.

**Performance-critical code with compiler-opaque invariants:** `std::slice::from_raw_parts` is unsafe because the compiler cannot verify that the pointer and length are consistent. But if you maintain this invariant yourself (e.g., because you're implementing a custom data structure), the unsafe call is sound.

**Implementing core data structures:** Linked lists, lock-free queues, arenas, and similar structures require pointer manipulation that the borrow checker cannot express. This is the canonical use case.

**`std::mem::transmute`:** Reinterprets the bits of a value as a different type. Almost always avoidable. Use only when you have a mathematical proof that the bit representations are compatible.

The test for whether `unsafe` is justified: can you write the same code in safe Rust without unacceptable performance loss or API limitations? If yes, do that. If no, isolate the unsafe code, write it carefully, document the invariants, and wrap it in a safe API.

#### Exercise 3.1 — Justify or Reject

**Goal:** Classify whether each usage is justified.

For each of the following, write a comment explaining whether the unsafe usage is justified and what the invariant is:
1. `unsafe { *ptr }` where `ptr = &local_var as *const i32`
2. `unsafe { std::mem::transmute::<u32, f32>(bits) }` for bit-casting IEEE 754
3. Using `unsafe` to skip bounds checking on a Vec with `get_unchecked(i)` in a hot loop where `i < v.len()` is already checked

**Expected output:** Three annotated comments explaining each case.

> **Hint:** For (2), note that Rust now has `f32::from_bits(u32)` as a safe alternative. For (3), the bound check must genuinely happen before the unchecked access — in the same loop iteration.

---

### 4. `extern` — Foreign Function Interface

> **Docs:** [Reference — extern blocks](https://doc.rust-lang.org/reference/items/external-blocks.html) · [Nomicon — FFI](https://doc.rust-lang.org/nomicon/ffi.html)

`extern` is how Rust calls code written in other languages (usually C) and how it exposes Rust functions to other languages. Every C library function is inherently `unsafe` from Rust's perspective because Rust cannot verify C's contracts.

**Calling C from Rust:**

```rust
// Declare the C functions you want to call
extern "C" {
    fn abs(x: i32) -> i32;
    fn strlen(s: *const u8) -> usize;
}

fn main() {
    // All calls into extern blocks are unsafe
    let result = unsafe { abs(-42) };
    println!("abs(-42) = {}", result);
}
```

The `"C"` string is the **ABI** (application binary interface) — it tells Rust to use C calling conventions. This is the correct choice for virtually all FFI. Other ABIs exist (`"system"`, `"Rust"`) but are rarely needed.

**Exposing Rust functions to C:**

```rust
// #[no_mangle] prevents Rust from mangling the function name
// extern "C" gives it C calling conventions
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**`extern crate` (historical):** In Rust editions before 2018, you wrote `extern crate serde;` to bring an external crate into scope. The 2021 edition (which this curriculum uses) makes this unnecessary — just write `use serde::...` directly. You'll see `extern crate` in older code; it's safe to ignore.

**Linking libraries:** When calling a C library, tell Rust where to find it:

```rust
#[link(name = "m")] // links to libm (the C math library)
extern "C" {
    fn sqrt(x: f64) -> f64;
}
```

For most real FFI work, use the `libc` crate (which provides safe type definitions for C types) and the `bindgen` tool (which auto-generates `extern` blocks from C headers).

#### Exercise 4.1 — Call a C Standard Library Function

**Goal:** Use `extern "C"` to call `abs` from the C standard library.

Write an `extern "C"` block declaring `fn abs(x: i32) -> i32`. In `main`, call it with `-7`, `-100`, and `42` and print the results. Wrap each call in `unsafe`.

**Expected output:**
```
abs(-7)   = 7
abs(-100) = 100
abs(42)   = 42
```

> **Hint:** On Linux/macOS `abs` is in libc which is always linked. On Windows you may need `#[link(name = "msvcrt")]`. If this doesn't compile on your platform, use `unsafe { (-7_i32).wrapping_abs() }` as a safe-Rust equivalent to verify the concept.

---

### 5. Safe Abstractions over Unsafe Code

> **Docs:** [Rustonomicon](https://doc.rust-lang.org/nomicon/) · [Unsafe Rust Book chapter](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)

The goal of writing `unsafe` code is almost always to build a **safe public API**. The `unsafe` block is the implementation detail; users of your type never write `unsafe`. This is exactly how `Vec`, `Box`, `Arc`, and the entire standard library work.

The design principle: **put all invariants in one place**. If your linked list has the invariant "every non-null `next` pointer points to a valid, aligned `Node<T>` on the heap," then that invariant should be upheld by every method that creates or modifies `next` pointers. Users of the list see only `push_front`, `pop_front`, etc. — no pointers, no unsafe.

```rust
pub struct Stack<T> {
    head: *mut Node<T>,
    len: usize,
}

struct Node<T> {
    value: T,
    next: *mut Node<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { head: std::ptr::null_mut(), len: 0 }
    }

    pub fn push(&mut self, value: T) {
        // Box::into_raw: allocate on heap, give us ownership of the pointer
        let node = Box::into_raw(Box::new(Node {
            value,
            next: self.head,
        }));
        self.head = node;
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }
        unsafe {
            // SAFETY: head is non-null (checked above), was allocated by push
            // via Box::into_raw, and we have exclusive access (mut self).
            let node = Box::from_raw(self.head);
            self.head = node.next;
            self.len -= 1;
            Some(node.value)
        }
    }
}
```

Notice the `// SAFETY:` comment. This is a strong convention in the Rust ecosystem — every `unsafe` block should have a comment explaining why the code is sound.

#### Exercise 4.1 — Safe Stack Wrapper

**Goal:** Build and test the Stack from the example above.

Implement `Stack<T>` with `push`, `pop`, and `len`. Write tests for: push one element and pop it, push three elements and pop all three (LIFO order), pop from empty stack returns `None`.

**Expected output:**
```
test stack_push_pop ... ok
test stack_lifo_order ... ok
test stack_empty_pop ... ok
```

> **Hint:** LIFO means "last in, first out." If you push 1, 2, 3, you should pop 3, 2, 1.

---

### 6. `unsafe` Traits — Send and Sync

> **Docs:** [Send trait](https://doc.rust-lang.org/std/marker/trait.Send.html) · [Sync trait](https://doc.rust-lang.org/std/marker/trait.Sync.html)

`Send` and `Sync` are the two most important `unsafe` traits. `Send` means "it is safe to transfer ownership of this type to another thread." `Sync` means "it is safe to share a reference to this type between threads" (equivalent to: `&T` is `Send`).

The compiler derives these automatically for most types: a struct is `Send` if all its fields are `Send`, and `Sync` if all its fields are `Sync`. The main exceptions are raw pointers (which are `!Send + !Sync`) and `Cell`/`RefCell` (which are `!Sync`).

If you write a data structure with raw pointers but design it to be thread-safe (e.g., it internally uses atomic operations), you must manually assert `Send` and `Sync`:

```rust
unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}
```

These impls are `unsafe` because the compiler cannot verify your claim. You are promising that your type upholds the thread-safety invariants. If you're wrong, you have data races.

Similarly, `unsafe fn` signals to callers: "I have preconditions you must verify before calling me." The function cannot check them itself (or it would be safe). The canonical example is `std::slice::from_raw_parts(ptr, len)` — the caller must ensure the pointer is valid for `len` elements.

#### Exercise 5.1 — Manual Send Impl

**Goal:** Understand when you need to manually implement Send.

Create a struct `MyCell<T> { data: *mut T }` with a `new(val: T) -> Self` constructor using `Box::into_raw`. Notice that the compiler refuses to derive `Send` for it (because `*mut T` is not `Send`). Add `unsafe impl<T: Send> Send for MyCell<T> {}` with a comment explaining why this is sound (or when it wouldn't be). Clean up memory in a `Drop` impl.

**Expected output:**
```
MyCell created and dropped correctly
```

> **Hint:** `MyCell` is safe to send if `T: Send` because we have exclusive ownership (no other code can access the pointer). This is the same argument `Box<T>` makes.

---

## Day Project: Singly-Linked List with Raw Pointers

### What You're Building

A `LinkedList<T>` backed entirely by raw pointers. The public API is completely safe — users never write a single `unsafe`. Inside the implementation, you carefully manage heap allocation and pointer chasing. The `Drop` impl ensures no memory leaks when the list is destroyed.

### Requirements

1. Define `struct LinkedList<T>` with a `*mut Node<T>` head and `usize` length
2. Define `struct Node<T> { value: T, next: *mut Node<T> }` (private)
3. `pub fn new() -> Self` — empty list
4. `pub fn push_front(&mut self, val: T)` — add to front
5. `pub fn pop_front(&mut self) -> Option<T>` — remove and return front element
6. `pub fn peek(&self) -> Option<&T>` — borrow the front element without removing
7. `pub fn len(&self) -> usize` — number of elements
8. `pub fn is_empty(&self) -> bool`
9. `impl<T> Drop for LinkedList<T>` — free all nodes to prevent leaks
10. `unsafe impl<T: Send> Send for LinkedList<T> {}`
11. Tests: push/pop correctness, peek, len tracking, LIFO order, empty list, Drop (check with Miri)
12. Every `unsafe` block must have a `// SAFETY:` comment

### Getting Started

```toml
[package]
name = "day-27"
version = "0.1.0"
edition = "2021"
```

No external dependencies needed for the main implementation. Scaffold `src/main.rs`:

```rust
struct Node<T> {
    value: T,
    next: *mut Node<T>,
}

pub struct LinkedList<T> {
    head: *mut Node<T>,
    len: usize,
}

unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: std::ptr::null_mut(),
            len: 0,
        }
    }

    pub fn push_front(&mut self, value: T) {
        let node = Box::into_raw(Box::new(Node {
            value,
            next: self.head,
        }));
        self.head = node;
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }
        // SAFETY: head is non-null (checked above), was allocated via Box::into_raw
        // in push_front, and we hold &mut self so no aliases exist.
        unsafe {
            let node = Box::from_raw(self.head);
            self.head = node.next;
            self.len -= 1;
            Some(node.value)
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.head.is_null() {
            return None;
        }
        // SAFETY: head is non-null, points to a valid Node allocated in push_front.
        // We return a reference tied to &self, preventing mutation during the borrow.
        unsafe { Some(&(*self.head).value) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

fn main() {
    let mut list: LinkedList<i32> = LinkedList::new();
    list.push_front(3);
    list.push_front(2);
    list.push_front(1);

    println!("len: {}", list.len());
    println!("peek: {:?}", list.peek());

    while let Some(val) = list.pop_front() {
        println!("popped: {}", val);
    }
    println!("empty: {}", list.is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut list = LinkedList::new();
        list.push_front(1);
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn test_lifo_order() {
        let mut list = LinkedList::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
    }

    #[test]
    fn test_peek_does_not_remove() {
        let mut list = LinkedList::new();
        list.push_front(42);
        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.len(), 1);  // still there
    }

    #[test]
    fn test_len_tracking() {
        let mut list = LinkedList::new();
        assert_eq!(list.len(), 0);
        list.push_front("a");
        list.push_front("b");
        assert_eq!(list.len(), 2);
        list.pop_front();
        assert_eq!(list.len(), 1);
    }
}
```

### Running Your Solution

```bash
cargo run -p day-27
```

Expected output:
```
len: 3
peek: Some(1)
popped: 1
popped: 2
popped: 3
empty: true
```

Run tests:
```bash
cargo test -p day-27
```

Check for memory errors with Miri (requires nightly):
```bash
rustup toolchain install nightly
rustup component add miri --toolchain nightly
cargo +nightly miri test -p day-27
```

Miri will report use-after-free, double-free, or uninitialized memory reads. A clean Miri run is a strong signal that your unsafe code is sound.

### Extension Challenges

- **Easy:** Add `pub fn push_back(&mut self, val: T)` — append to the tail. This requires either traversing to the tail O(n) or maintaining a `tail: *mut Node<T>` pointer.
- **Medium:** Implement `Iterator` for `LinkedList<T>` (by reference). The iterator holds a `*mut Node<T>` cursor. Each `next()` call advances the cursor and returns `Some(&T)` or `None`.
- **Hard:** Convert `LinkedList<T>` into a doubly-linked list by adding a `prev: *mut Node<T>` field to `Node<T>`. Implement `push_back` and `pop_back`. This requires carefully maintaining both prev and next pointers on every mutation. Check with Miri.
