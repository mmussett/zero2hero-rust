# Data Structures in Rust

A reference for classic computer science data structures and their idiomatic Rust implementations.
Each entry covers: definition, Rust implementation approach, time complexity, and when to use it.

| Structure | Rust Type / Approach | Day Covered |
|-----------|---------------------|-------------|
| Stack | `Vec<T>` or custom `Stack<T>` | Day 11 |
| Queue | `VecDeque<T>` | Day 11 |
| Deque | `VecDeque<T>` | Day 11 |
| Singly Linked List | `enum List<T>` + `Box` | Day 11 |
| Doubly Linked List | `Rc<RefCell<Node<T>>>` | Day 16 |
| Binary Search Tree | `Box<Node<T>>` | Day 16 |
| Graph (safe) | `HashMap<String, Vec<String>>` | Day 11 |
| Graph (shared nodes) | `Rc<RefCell<GraphNode>>` | Day 16 |
| Hash Map | `std::collections::HashMap<K,V>` | Day 08 |
| Hash Set | `std::collections::HashSet<T>` | Day 08 |
| Priority Queue | `std::collections::BinaryHeap<T>` | Day 11 |
| Sorted Map | `std::collections::BTreeMap<K,V>` | Day 08 |
| Unsafe Linked List | Raw pointers | Day 27 |

---

## Stack (LIFO)

**Definition:** A stack is a last-in, first-out (LIFO) collection. Think of a stack of plates: you can only add or remove from the top. The last item pushed is the first item popped.

**Rust implementation:** Wrap `Vec<T>` in a newtype. `Vec` already supports O(1) push/pop at the end, so it is the natural backing store. Avoid using `std::collections::LinkedList` — it has worse cache performance.

**Time complexity:**
| Operation | Complexity |
|-----------|------------|
| push      | O(1) amortized |
| pop       | O(1) |
| peek      | O(1) |
| is_empty  | O(1) |

**When to use:**
- Undo/redo history (each action is pushed; undo pops)
- Simulating a call stack for recursive algorithms iteratively
- Depth-first search (DFS) — push neighbours, pop to visit
- Balanced parentheses / bracket matching
- Reverse a sequence

**Complete example:**

```rust
struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { data: Vec::new() }
    }

    fn push(&mut self, value: T) {
        self.data.push(value);
    }

    fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    fn peek(&self) -> Option<&T> {
        self.data.last()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

fn is_balanced(s: &str) -> bool {
    let mut stack = Stack::new();
    for ch in s.chars() {
        match ch {
            '(' | '[' | '{' => stack.push(ch),
            ')' => {
                if stack.pop() != Some('(') {
                    return false;
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return false;
                }
            }
            '}' => {
                if stack.pop() != Some('{') {
                    return false;
                }
            }
            _ => {}
        }
    }
    stack.is_empty()
}

fn main() {
    println!("{}", is_balanced("({[]})")); // true
    println!("{}", is_balanced("({[}])"));  // false
    println!("{}", is_balanced("((()))")); // true
}
```

**Common pitfalls:**
- `pop()` returns `Option<T>` — always handle the `None` case (empty stack).
- Do not use `Vec::remove(0)` as a stack operation; that is O(n). Always use `push`/`pop` at the tail.
- If you need a stack shared across threads, use `Mutex<Vec<T>>` instead.

---

## Queue (FIFO)

**Definition:** A queue is a first-in, first-out (FIFO) collection. Think of a checkout line: the first person in line is served first. Items are added at the back (enqueue) and removed from the front (dequeue).

**Rust implementation:** Wrap `std::collections::VecDeque<T>`. A plain `Vec<T>` cannot be used efficiently because removing from the front is O(n). `VecDeque` uses a ring buffer and gives O(1) operations at both ends.

**Time complexity:**
| Operation | Complexity |
|-----------|------------|
| enqueue (push_back) | O(1) amortized |
| dequeue (pop_front) | O(1) |
| peek front          | O(1) |
| is_empty            | O(1) |

**When to use:**
- Task scheduling / job queues
- Breadth-first search (BFS)
- Print spoolers, I/O buffering
- Producer-consumer pipelines (use `std::sync::mpsc` for multi-threaded)

**Complete example:**

```rust
use std::collections::VecDeque;

struct Queue<T> {
    data: VecDeque<T>,
}

impl<T> Queue<T> {
    fn new() -> Self {
        Queue { data: VecDeque::new() }
    }

    fn enqueue(&mut self, value: T) {
        self.data.push_back(value);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    fn peek(&self) -> Option<&T> {
        self.data.front()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// BFS level-order traversal on an implicit tree stored as a Vec.
// Each node i has children at 2*i+1 and 2*i+2.
fn bfs_levels(tree: &[i32]) -> Vec<Vec<i32>> {
    if tree.is_empty() {
        return vec![];
    }
    let mut result = Vec::new();
    let mut queue = Queue::new();
    queue.enqueue(0usize); // store indices

    while !queue.is_empty() {
        let level_size = queue.data.len();
        let mut level = Vec::new();
        for _ in 0..level_size {
            let idx = queue.dequeue().unwrap();
            level.push(tree[idx]);
            let left = 2 * idx + 1;
            let right = 2 * idx + 2;
            if left < tree.len() {
                queue.enqueue(left);
            }
            if right < tree.len() {
                queue.enqueue(right);
            }
        }
        result.push(level);
    }
    result
}

fn main() {
    // Tree: 1 -> [2, 3] -> [4, 5, 6, 7]
    let tree = vec![1, 2, 3, 4, 5, 6, 7];
    let levels = bfs_levels(&tree);
    for (i, level) in levels.iter().enumerate() {
        println!("Level {}: {:?}", i, level);
    }
}
```

**Common pitfalls:**
- Using `Vec::remove(0)` as a dequeue is O(n) — always prefer `VecDeque`.
- For multi-threaded queues, use `std::sync::mpsc::channel()` or `crossbeam::channel`.

---

## Double-Ended Queue (Deque)

**Definition:** A deque (pronounced "deck") supports O(1) insertion and removal at both the front and the back. It generalises both stacks and queues.

**Rust implementation:** Use `std::collections::VecDeque<T>` directly — it is the standard library's ring-buffer deque.

**Time complexity:**
| Operation  | Complexity |
|------------|------------|
| push_front | O(1) amortized |
| push_back  | O(1) amortized |
| pop_front  | O(1) |
| pop_back   | O(1) |
| indexing   | O(1) |

**When to use:**
- Sliding window algorithms (add to back, remove from front or back)
- Palindrome checking (compare front and back)
- Work-stealing schedulers (steal from the front of another thread's deque)
- Implementing both stacks and queues with one type

**Complete example:**

```rust
use std::collections::VecDeque;

// Sliding window maximum: for each window of size k, find the maximum.
fn sliding_window_max(nums: &[i32], k: usize) -> Vec<i32> {
    let mut result = Vec::new();
    // Deque stores indices; front always holds the index of the current window max.
    let mut deque: VecDeque<usize> = VecDeque::new();

    for i in 0..nums.len() {
        // Remove indices outside the current window.
        while deque.front().map_or(false, |&front| front + k <= i) {
            deque.pop_front();
        }
        // Remove from the back any indices whose values are less than nums[i].
        while deque.back().map_or(false, |&back| nums[back] <= nums[i]) {
            deque.pop_back();
        }
        deque.push_back(i);

        // Start adding results once the first full window is complete.
        if i + 1 >= k {
            result.push(nums[*deque.front().unwrap()]);
        }
    }
    result
}

fn main() {
    let nums = vec![1, 3, -1, -3, 5, 3, 6, 7];
    let k = 3;
    println!("{:?}", sliding_window_max(&nums, k)); // [3, 3, 5, 5, 6, 7]
}
```

**Common pitfalls:**
- `VecDeque` supports indexing with `[]`, but it is not contiguous in memory — do not assume it is a `&[T]` slice without calling `.make_contiguous()` first.
- `as_slices()` returns two slices (the ring buffer may be split); use it when you need slice-based APIs.

---

## Singly Linked List

**Definition:** A linked list is a sequence of nodes where each node holds a value and a pointer to the next node. There is no contiguous memory; traversal is always sequential.

**Rust implementation:** Use a recursive enum with `Box<T>` for heap allocation. `Box` provides single ownership, which is exactly what a singly-linked list node needs.

```rust
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}
```

**Time complexity:**
| Operation        | Complexity |
|------------------|------------|
| prepend          | O(1) |
| append (to tail) | O(n) |
| access by index  | O(n) |
| search           | O(n) |
| delete head      | O(1) |

**When to use:**
- Prepend-heavy workloads where O(1) head insertion matters
- Functional-style list processing (fold, map, recursive algorithms)
- When ownership semantics map naturally to a list (each node owns the next)
- Teaching recursion and Box<T>

> **Note:** `std::collections::LinkedList` exists in the standard library but is rarely the right choice in practice. `Vec<T>` is almost always faster due to cache locality and avoids the per-node heap allocation overhead.

**Complete example:**

```rust
#[derive(Debug)]
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T: std::fmt::Debug> List<T> {
    fn new() -> Self {
        List::Nil
    }

    // Prepend a value — O(1).
    fn prepend(self, value: T) -> Self {
        List::Cons(value, Box::new(self))
    }

    fn len(&self) -> usize {
        match self {
            List::Nil => 0,
            List::Cons(_, tail) => 1 + tail.len(),
        }
    }

    fn head(&self) -> Option<&T> {
        match self {
            List::Nil => None,
            List::Cons(value, _) => Some(value),
        }
    }
}

// Reverse a list by building a new one via prepend.
fn reverse<T: std::fmt::Debug>(list: List<T>) -> List<T> {
    let mut current = list;
    let mut reversed = List::Nil;
    loop {
        match current {
            List::Nil => break reversed,
            List::Cons(value, tail) => {
                reversed = reversed.prepend(value);
                current = *tail;
            }
        }
    }
}

fn main() {
    let list = List::new()
        .prepend(3)
        .prepend(2)
        .prepend(1);

    println!("Length: {}", list.len()); // 3
    println!("Head: {:?}", list.head()); // Some(1)

    let rev = reverse(list);
    println!("Reversed head: {:?}", rev.head()); // Some(3)
}
```

**Common pitfalls:**
- Deep lists can cause a stack overflow on drop because Rust's default `Drop` is recursive. For very long lists, implement a custom iterative `Drop`.
- `Box<T>` means single ownership — you cannot have two pointers to the same node without `Rc`.
- Appending to the tail requires traversing the whole list (O(n)); if you need fast appends, use `Vec`.

---

## Doubly Linked List

**Definition:** Like a singly linked list but each node also has a pointer to the previous node. This allows O(1) insertion and deletion at any known position, including the tail.

**Rust implementation:** Safe Rust requires `Option<Rc<RefCell<Node<T>>>>` for shared mutable references to prev/next. This is verbose but correct. For performance-critical code, raw pointers (Day 27) are used instead.

**Time complexity:**
| Operation               | Complexity |
|-------------------------|------------|
| insert at known node    | O(1) |
| delete at known node    | O(1) |
| search                  | O(n) |
| access by index         | O(n) |

**When to use:**
- LRU cache (move recently used node to front in O(1))
- O(1) deletion when you already have a reference to the node
- Browser history (forward and backward navigation)

**Complete example — node structure and basic operations:**

```rust
use std::rc::Rc;
use std::cell::RefCell;

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

#[derive(Debug)]
struct Node<T> {
    value: T,
    prev: Link<T>,
    next: Link<T>,
}

impl<T> Node<T> {
    fn new(value: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            value,
            prev: None,
            next: None,
        }))
    }
}

struct DoublyLinkedList<T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
}

impl<T: std::fmt::Debug> DoublyLinkedList<T> {
    fn new() -> Self {
        DoublyLinkedList { head: None, tail: None, len: 0 }
    }

    fn push_back(&mut self, value: T) {
        let node = Node::new(value);
        match self.tail.take() {
            None => {
                self.head = Some(Rc::clone(&node));
                self.tail = Some(node);
            }
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(Rc::clone(&node));
                node.borrow_mut().prev = Some(old_tail);
                self.tail = Some(node);
            }
        }
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            match old_head.borrow_mut().next.take() {
                None => {
                    self.tail.take();
                }
                Some(new_head) => {
                    new_head.borrow_mut().prev.take();
                    self.head = Some(new_head);
                }
            }
            self.len -= 1;
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().value
        })
    }

    fn len(&self) -> usize {
        self.len
    }
}

fn main() {
    let mut list: DoublyLinkedList<i32> = DoublyLinkedList::new();
    list.push_back(10);
    list.push_back(20);
    list.push_back(30);
    println!("Length: {}", list.len()); // 3
    println!("Pop front: {:?}", list.pop_front()); // Some(10)
    println!("Pop front: {:?}", list.pop_front()); // Some(20)
    println!("Length: {}", list.len()); // 1
}
```

**Common pitfalls:**
- `Rc<RefCell<T>>` can create reference cycles — if `prev` and `next` both hold strong references you get a memory leak. One common fix: make `prev` a `Weak<RefCell<Node<T>>>`.
- `Rc` is not thread-safe. For multi-threaded use, replace with `Arc<Mutex<Node<T>>>`.
- `borrow_mut()` panics at runtime if there is already an active borrow. This replaces the compile-time borrow checker guarantees with runtime checks.
- The full LRU cache implementation using this structure is covered in Day 16.

---

## Binary Search Tree (BST)

**Definition:** A BST is a binary tree where every node satisfies the invariant: all values in the left subtree are less than the node's value, and all values in the right subtree are greater. This enables efficient ordered operations.

**Rust implementation:** Use a recursive enum with `Box<BST<T>>` for owned child nodes. The enum variants model the two cases: an empty tree (Leaf) and a tree node (Node).

**Time complexity:**
| Operation | Average | Worst case (unbalanced) |
|-----------|---------|------------------------|
| insert    | O(log n) | O(n) |
| search    | O(log n) | O(n) |
| delete    | O(log n) | O(n) |
| in-order  | O(n)    | O(n) |

**When to use:**
- Ordered data with frequent insertion and search
- When you need in-order traversal (yields sorted output)
- As a building block for understanding balanced trees

> **Note:** For production use, prefer `std::collections::BTreeMap<K, V>` — it is a self-balancing B-tree guaranteed to be O(log n) in all cases. A hand-rolled BST will degrade to O(n) on sorted input.

**Complete example:**

```rust
#[derive(Debug)]
enum BST<T: Ord> {
    Leaf,
    Node {
        value: T,
        left: Box<BST<T>>,
        right: Box<BST<T>>,
    },
}

impl<T: Ord + std::fmt::Debug> BST<T> {
    fn new() -> Self {
        BST::Leaf
    }

    fn insert(self, new_value: T) -> Self {
        match self {
            BST::Leaf => BST::Node {
                value: new_value,
                left: Box::new(BST::Leaf),
                right: Box::new(BST::Leaf),
            },
            BST::Node { value, left, right } => {
                if new_value < value {
                    BST::Node { value, left: Box::new(left.insert(new_value)), right }
                } else if new_value > value {
                    BST::Node { value, left, right: Box::new(right.insert(new_value)) }
                } else {
                    // Duplicate: ignore (or replace — policy choice).
                    BST::Node { value, left, right }
                }
            }
        }
    }

    fn contains(&self, target: &T) -> bool {
        match self {
            BST::Leaf => false,
            BST::Node { value, left, right } => {
                if target == value {
                    true
                } else if target < value {
                    left.contains(target)
                } else {
                    right.contains(target)
                }
            }
        }
    }

    // In-order traversal yields sorted output.
    fn in_order(&self) -> Vec<&T> {
        match self {
            BST::Leaf => vec![],
            BST::Node { value, left, right } => {
                let mut result = left.in_order();
                result.push(value);
                result.extend(right.in_order());
                result
            }
        }
    }
}

fn main() {
    let tree = BST::new()
        .insert(5)
        .insert(3)
        .insert(7)
        .insert(1)
        .insert(4)
        .insert(6)
        .insert(8);

    println!("Contains 4: {}", tree.contains(&4)); // true
    println!("Contains 9: {}", tree.contains(&9)); // false
    println!("In order: {:?}", tree.in_order());   // [1, 3, 4, 5, 6, 7, 8]
}
```

**Common pitfalls:**
- Inserting sorted data produces a degenerate tree (a linked list) — O(n) operations. Use `BTreeMap` in production.
- The recursive `insert` shown above moves and rebuilds the tree on each insert. An iterative or mutable-reference approach avoids this but is more complex with Rust's ownership rules.
- Deleting a node with two children requires finding the in-order successor, which adds complexity.

---

## Hash Map

**Definition:** A hash map (or hash table) stores key-value pairs. A hash function maps each key to a bucket index, enabling average O(1) lookup, insertion, and deletion. Keys must be unique.

**Rust implementation:** `std::collections::HashMap<K, V>` — keys must implement `Hash + Eq`. Uses the SipHash 1-3 algorithm by default (DOS-resistant). Switch to a faster hasher (`FxHashMap`, `AHashMap`) in performance-critical code.

**Time complexity:**
| Operation | Average | Worst case |
|-----------|---------|------------|
| insert    | O(1)    | O(n) (rehash) |
| get       | O(1)    | O(n) |
| remove    | O(1)    | O(n) |
| contains  | O(1)    | O(n) |
| iteration | O(n)    | O(n) |

**When to use:**
- Key-value lookup (configs, caches, symbol tables)
- Counting or grouping (frequency maps, histograms)
- Memoisation / dynamic programming caches
- De-duplicating data by key

> **Note:** Use `BTreeMap` when you need sorted key iteration or range queries. Use `indexmap::IndexMap` (external crate) when insertion order matters.

**Complete example:**

```rust
use std::collections::HashMap;

fn word_frequency(text: &str) -> HashMap<&str, usize> {
    let mut freq = HashMap::new();
    for word in text.split_whitespace() {
        *freq.entry(word).or_insert(0) += 1;
    }
    freq
}

fn group_by_length(words: &[&str]) -> HashMap<usize, Vec<&str>> {
    let mut groups: HashMap<usize, Vec<&str>> = HashMap::new();
    for &word in words {
        groups.entry(word.len()).or_default().push(word);
    }
    groups
}

fn main() {
    let text = "the quick brown fox jumps over the lazy dog the fox";
    let freq = word_frequency(text);

    // Sort for deterministic output.
    let mut pairs: Vec<_> = freq.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (word, count) in &pairs {
        println!("{}: {}", word, count);
    }

    println!();
    let words = vec!["cat", "dog", "bee", "elephant", "ant", "ox"];
    let groups = group_by_length(&words);
    let mut keys: Vec<_> = groups.keys().collect();
    keys.sort();
    for k in keys {
        println!("len {}: {:?}", k, groups[k]);
    }
}
```

**Common pitfalls:**
- Iteration order over `HashMap` is random and non-deterministic across runs. Never rely on it.
- `.entry().or_insert()` is the idiomatic way to update-or-insert. Avoid `.get()` + `.insert()` which does two lookups.
- Keys that are `String` will be cloned on insert; use `&str` as the key type when the map does not own the data.
- Floats (`f32`, `f64`) do not implement `Hash` because `NaN != NaN`, so they cannot be used as `HashMap` keys.

---

## Hash Set

**Definition:** A hash set stores unique values with O(1) membership testing. It is a `HashMap` with no associated values — only keys. It also provides standard set operations: union, intersection, difference.

**Rust implementation:** `std::collections::HashSet<T>` — `T` must implement `Hash + Eq`.

**Time complexity:**
| Operation      | Average | Worst case |
|----------------|---------|------------|
| insert         | O(1)    | O(n) |
| contains       | O(1)    | O(n) |
| remove         | O(1)    | O(n) |
| union          | O(n)    | O(n) |
| intersection   | O(min(m,n)) | O(n) |

**When to use:**
- Membership testing ("have we seen this value before?")
- De-duplication of a collection
- Set algebra: union, intersection, difference, symmetric difference
- Visited tracking in graph traversal

**Complete example:**

```rust
use std::collections::HashSet;

fn find_duplicates(nums: &[i32]) -> Vec<i32> {
    let mut seen = HashSet::new();
    let mut duplicates = HashSet::new();
    for &n in nums {
        if !seen.insert(n) {
            duplicates.insert(n);
        }
    }
    let mut result: Vec<i32> = duplicates.into_iter().collect();
    result.sort(); // for deterministic output
    result
}

fn main() {
    let nums = vec![1, 2, 3, 2, 4, 3, 5];
    println!("Duplicates: {:?}", find_duplicates(&nums)); // [2, 3]

    // Set operations.
    let a: HashSet<i32> = [1, 2, 3, 4].iter().cloned().collect();
    let b: HashSet<i32> = [3, 4, 5, 6].iter().cloned().collect();

    let mut union_ab: Vec<i32> = a.union(&b).cloned().collect();
    union_ab.sort();
    println!("Union: {:?}", union_ab); // [1, 2, 3, 4, 5, 6]

    let mut intersection: Vec<i32> = a.intersection(&b).cloned().collect();
    intersection.sort();
    println!("Intersection: {:?}", intersection); // [3, 4]

    let mut difference: Vec<i32> = a.difference(&b).cloned().collect();
    difference.sort();
    println!("A - B: {:?}", difference); // [1, 2]

    println!("A is subset of union: {}", a.is_subset(&a.union(&b).cloned().collect()));
}
```

**Common pitfalls:**
- Like `HashMap`, iteration order is non-deterministic. Sort before printing if you need reproducible output.
- `insert` returns `false` if the value was already present — use this return value to detect duplicates without a second `contains` call.
- `HashSet<T>` is backed by `HashMap<T, ()>`. For sorted sets with range queries, use `BTreeSet<T>`.

---

## Priority Queue / Binary Heap

**Definition:** A priority queue always removes the highest-priority element next. A binary heap is the standard implementation: a complete binary tree stored in an array where the parent is always greater (max-heap) or smaller (min-heap) than its children.

**Rust implementation:** `std::collections::BinaryHeap<T>` — `T` must implement `Ord`. It is a max-heap: `pop()` returns the largest element. For a min-heap, wrap values with `std::cmp::Reverse<T>`.

**Time complexity:**
| Operation | Complexity |
|-----------|------------|
| push      | O(log n) |
| pop       | O(log n) |
| peek      | O(1) |
| heapify   | O(n) |

**When to use:**
- Task scheduling where tasks have priorities
- Dijkstra's shortest path algorithm (min-heap over (distance, node))
- Top-K largest or smallest elements
- Merge K sorted lists
- Event-driven simulations (next event by timestamp)

**Complete example:**

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

#[derive(Debug, Eq, PartialEq)]
struct Task {
    priority: u32,
    name: String,
}

// Higher priority number = runs first. Implement Ord to make BinaryHeap sort by priority.
impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut scheduler: BinaryHeap<Task> = BinaryHeap::new();

    scheduler.push(Task { priority: 2, name: "Send email".to_string() });
    scheduler.push(Task { priority: 5, name: "Fix critical bug".to_string() });
    scheduler.push(Task { priority: 1, name: "Read docs".to_string() });
    scheduler.push(Task { priority: 3, name: "Code review".to_string() });

    println!("Processing tasks by priority:");
    while let Some(task) = scheduler.pop() {
        println!("  [{}] {}", task.priority, task.name);
    }

    // Min-heap example: top-3 smallest numbers.
    let numbers = vec![5, 1, 9, 3, 7, 2, 8, 4, 6];
    let mut min_heap: BinaryHeap<Reverse<i32>> = numbers.iter().map(|&n| Reverse(n)).collect();
    let mut top3 = Vec::new();
    for _ in 0..3 {
        if let Some(Reverse(n)) = min_heap.pop() {
            top3.push(n);
        }
    }
    println!("3 smallest: {:?}", top3); // [1, 2, 3]
}
```

**Common pitfalls:**
- `BinaryHeap` is a max-heap. For min-heap behaviour, use `Reverse<T>` — it flips the ordering.
- `BinaryHeap` does not support updating the priority of an element already in the heap. A common workaround is to insert a new entry and lazily discard stale entries on pop.
- Peeking gives the max element; there is no way to peek at the minimum without popping everything.

---

## Graph

**Definition:** A graph is a collection of nodes (vertices) connected by edges. Edges can be directed or undirected, weighted or unweighted. Graphs model networks, dependency trees, maps, social connections, and more.

**Rust implementation:**
- **Simple adjacency list (safe):** `HashMap<String, Vec<String>>` — best for most use cases.
- **Weighted edges:** `HashMap<String, Vec<(String, u32)>>` — store (neighbour, weight) pairs.
- **Shared mutable nodes (Day 16):** `Vec<Rc<RefCell<GraphNode>>>` — when nodes need to be mutated during traversal.
- **Arena-based (Day 27):** index into a `Vec<Node>` — avoids `Rc` overhead with good performance.

**Time complexity (adjacency list):**
| Operation         | Complexity |
|-------------------|------------|
| add vertex        | O(1) |
| add edge          | O(1) |
| BFS / DFS         | O(V + E) |
| check edge exists | O(degree) |

**When to use:**
- Modelling networks (social graphs, computer networks)
- Dependency resolution (build systems, package managers)
- Pathfinding (BFS for shortest path, Dijkstra for weighted)
- Cycle detection, topological sort

**Complete example — BFS shortest path in an unweighted graph:**

```rust
use std::collections::{HashMap, VecDeque, HashSet};

struct Graph {
    adjacency: HashMap<String, Vec<String>>,
}

impl Graph {
    fn new() -> Self {
        Graph { adjacency: HashMap::new() }
    }

    fn add_edge(&mut self, from: &str, to: &str) {
        self.adjacency
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
        // For undirected graph, add the reverse edge too:
        self.adjacency
            .entry(to.to_string())
            .or_default()
            .push(from.to_string());
    }

    // BFS shortest path — returns the path as a Vec of node names, or None.
    fn shortest_path(&self, start: &str, goal: &str) -> Option<Vec<String>> {
        if start == goal {
            return Some(vec![start.to_string()]);
        }
        let mut visited: HashSet<String> = HashSet::new();
        // Queue stores (current_node, path_so_far).
        let mut queue: VecDeque<(String, Vec<String>)> = VecDeque::new();
        queue.push_back((start.to_string(), vec![start.to_string()]));
        visited.insert(start.to_string());

        while let Some((node, path)) = queue.pop_front() {
            if let Some(neighbours) = self.adjacency.get(&node) {
                for neighbour in neighbours {
                    if neighbour == goal {
                        let mut full_path = path.clone();
                        full_path.push(neighbour.clone());
                        return Some(full_path);
                    }
                    if visited.insert(neighbour.clone()) {
                        let mut new_path = path.clone();
                        new_path.push(neighbour.clone());
                        queue.push_back((neighbour.clone(), new_path));
                    }
                }
            }
        }
        None
    }
}

fn main() {
    let mut g = Graph::new();
    g.add_edge("A", "B");
    g.add_edge("A", "C");
    g.add_edge("B", "D");
    g.add_edge("C", "D");
    g.add_edge("D", "E");

    match g.shortest_path("A", "E") {
        Some(path) => println!("Shortest path: {:?}", path), // ["A", "B", "D", "E"] or via C
        None => println!("No path found"),
    }

    match g.shortest_path("A", "Z") {
        Some(path) => println!("Path: {:?}", path),
        None => println!("No path from A to Z"), // No path found
    }
}
```

**Common pitfalls:**
- Not tracking visited nodes causes infinite loops on cyclic graphs.
- The path-cloning BFS above is simple but memory-intensive. For large graphs, store a `parent` map instead and reconstruct the path at the end.
- `HashMap` iteration order is non-deterministic, so BFS may return different-but-equally-short paths on each run for graphs with multiple shortest paths.
- For weighted shortest paths, use Dijkstra's algorithm with a `BinaryHeap<Reverse<(u32, String)>>`.

---

## Sorted Map / B-Tree Map

**Definition:** A sorted map maintains its keys in sorted order at all times. The standard library's `BTreeMap` is backed by a B-tree, a self-balancing tree structure that keeps data sorted and guarantees O(log n) for all operations. Unlike `HashMap`, iteration always proceeds in ascending key order.

**Rust implementation:** `std::collections::BTreeMap<K, V>` — keys must implement `Ord`. No hashing required. `BTreeSet<T>` is the set equivalent.

**Time complexity:**
| Operation       | Complexity |
|-----------------|------------|
| insert          | O(log n) |
| get             | O(log n) |
| remove          | O(log n) |
| range query     | O(log n + k) where k is result size |
| iteration       | O(n) in sorted order |

**When to use:**
- You need sorted iteration over keys (e.g., printing an ordered report)
- Range queries: "give me all events between timestamp A and B"
- Deterministic / reproducible output (unlike `HashMap`)
- Leaderboards, ordered indexes, time-series data

**Complete example:**

```rust
use std::collections::BTreeMap;

fn main() {
    // Event log keyed by timestamp (seconds since epoch).
    let mut events: BTreeMap<u64, String> = BTreeMap::new();
    events.insert(1_700_000_100, "User logged in".to_string());
    events.insert(1_700_000_050, "Server started".to_string());
    events.insert(1_700_000_300, "Request received".to_string());
    events.insert(1_700_000_200, "Cache miss".to_string());
    events.insert(1_700_000_400, "User logged out".to_string());

    // Iteration is always in ascending key (timestamp) order.
    println!("Full event log:");
    for (ts, event) in &events {
        println!("  {}: {}", ts, event);
    }

    // Range query: events between two timestamps (inclusive).
    let start = 1_700_000_100u64;
    let end   = 1_700_000_350u64;
    println!("\nEvents between {} and {}:", start, end);
    for (ts, event) in events.range(start..=end) {
        println!("  {}: {}", ts, event);
    }

    // First and last events.
    if let Some((ts, event)) = events.iter().next() {
        println!("\nFirst event at {}: {}", ts, event);
    }
    if let Some((ts, event)) = events.iter().next_back() {
        println!("Last event at {}:  {}", ts, event);
    }
}
```

**Common pitfits:**
- `BTreeMap` is slower than `HashMap` for pure key-value lookup (O(log n) vs O(1)). Only switch to it when sorted order or range queries are needed.
- Range syntax uses Rust's range types: `start..end` (exclusive), `start..=end` (inclusive), `..end`, `start..`, `..`.
- `BTreeMap` has better cache performance than a naive BST because B-tree nodes hold multiple keys, reducing pointer chasing.

---

## Choosing the Right Structure

Use this table when you know what you need but are unsure which type to reach for:

| Need | Use |
|------|-----|
| LIFO (undo, call stack, DFS) | `Vec<T>` as stack, or custom `Stack<T>` |
| FIFO (task queue, BFS) | `VecDeque<T>` |
| Fast insert/remove at both ends | `VecDeque<T>` |
| Key → value lookup | `HashMap<K, V>` |
| Key → value, sorted / range queries | `BTreeMap<K, V>` |
| Membership test, deduplication | `HashSet<T>` |
| Membership test, sorted / range | `BTreeSet<T>` |
| Priority ordering (max or min) | `BinaryHeap<T>` (max); `BinaryHeap<Reverse<T>>` (min) |
| Ordered, growable sequences | `Vec<T>` |
| Recursive / tree structures | `enum` + `Box<T>` |
| Graph / network | `HashMap<String, Vec<String>>` (adjacency list) |
| Shared ownership of nodes | `Rc<RefCell<T>>` (single-threaded) |
| Shared ownership across threads | `Arc<Mutex<T>>` |
| High-performance linked list | Raw pointers with `unsafe` (Day 27) |
| Insertion-order map | `indexmap::IndexMap<K, V>` (external crate) |
| Interned strings / many duplicates | `HashMap<String, usize>` + `Vec<String>` as arena |

### Quick mental model

```
Need fast random access?         → Vec<T>
Need fast lookup by key?         → HashMap or BTreeMap
  Need sorted keys?              → BTreeMap
Need FIFO?                       → VecDeque
Need LIFO?                       → Vec (as stack)
Need priority ordering?          → BinaryHeap
Need uniqueness / set algebra?   → HashSet or BTreeSet
Building a tree or recursive structure? → enum + Box<T>
Modelling relationships?         → HashMap<K, Vec<V>> (adjacency list)
Nodes need shared references?    → Rc<RefCell<T>> or Arc<Mutex<T>>
```
