# Day 17: Concurrency — Threads, Arc, and Mutex

> **Project:** Parallel File Hasher — compute checksums of 8 "file contents" both sequentially and in parallel, comparing wall-clock time using `std::time::Instant`.

## Learning Objectives

By the end of today you will be able to:
- Spawn OS threads with `thread::spawn` and join them with `JoinHandle::join`.
- Explain why thread closures must be `'static + Send` and use `move` to satisfy this.
- Share data across threads using `Arc<T>` (atomic reference counting).
- Guard mutable shared state with `Mutex<T>` and understand mutex poisoning.
- Use `Arc<Mutex<T>>` as the canonical pattern for shared mutable state across threads.
- Compare `RwLock<T>` with `Mutex<T>` and choose appropriately.

---

## Concepts

### 1. `thread::spawn` and `JoinHandle`

> **Docs:** [Threads](https://doc.rust-lang.org/book/ch16-01-threads.html) · [thread module](https://doc.rust-lang.org/std/thread/) · [thread::spawn](https://doc.rust-lang.org/std/thread/fn.spawn.html)

Rust's `std::thread::spawn` creates a new OS thread that runs a closure concurrently with the calling thread. The function returns a `JoinHandle<T>` where `T` is the return type of the closure. Calling `.join()` on the handle blocks the caller until that thread finishes and gives back `Result<T, Box<dyn Any + Send>>` — the `Err` case occurs when the thread panicked.

Every spawned thread is an independent execution unit scheduled by the OS. They are heavyweight compared to async tasks (each gets its own stack, typically 2–8 MB by default) but they are the right tool for CPU-bound work that should exploit multiple cores.

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        // This runs in a new OS thread
        let sum: u64 = (1..=1_000_000).sum();
        sum
    });

    // The main thread can do other work here while the child runs
    println!("Main thread: working...");

    // Block until the child finishes, then get its return value
    let result = handle.join().expect("child thread panicked");
    println!("Sum from child thread: {}", result);
}
```

**Expected output:**
```
Main thread: working...
Sum from child thread: 500000500000
```

#### Exercise 1.1 — Hello from Threads

**Goal:** Spawn five threads, each printing its index, and join them all.

```rust
use std::thread;

fn main() {
    let handles: Vec<_> = (0..5)
        .map(|i| thread::spawn(move || println!("Hello from thread {i}")))
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
```

**Expected output (order may vary):**
```
Hello from thread 0
Hello from thread 2
Hello from thread 1
Hello from thread 4
Hello from thread 3
```

> **Hint:** The `move` keyword is required — without it, `i` is borrowed, but the thread may outlive the loop iteration.

---

### 2. Moving Data into Threads

> **Docs:** [Send trait](https://doc.rust-lang.org/std/marker/trait.Send.html) · [Sync trait](https://doc.rust-lang.org/std/marker/trait.Sync.html) · [Threads](https://doc.rust-lang.org/book/ch16-01-threads.html)

A closure passed to `thread::spawn` must be `'static + Send`. `'static` means the closure must not borrow any non-static references — the spawned thread may outlive the calling scope, so borrowed data might be freed before the thread finishes. `Send` means the data is safe to transfer to another thread (most types are `Send`; `Rc<T>` is a notable exception).

The solution is almost always a `move` closure: capture all needed data by value so the thread owns it. For large data you want to share without copying, use `Arc` (Section 3).

```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3, 4, 5];

    // Without move, this would fail to compile:
    // the closure may outlive the borrowed `data`
    let handle = thread::spawn(move || {
        // data is moved into this closure; we own it
        let sum: i32 = data.iter().sum();
        println!("Sum: {sum}");
        sum
    });

    // data is no longer accessible here — it was moved
    // println!("{:?}", data); // would be a compile error

    let result = handle.join().unwrap();
    println!("Got back: {result}");
}
```

**Expected output:**
```
Sum: 15
Got back: 15
```

#### Exercise 2.1 — String Processing

**Goal:** Spawn a thread that takes ownership of a `String`, processes it, and returns the result.

Spawn a thread that receives a `String` via `move`, reverses it, and returns the reversed string. In `main`, print both the original (before the move) and the result from `join`.

**Expected output:**
```
Original: Hello, Rust!
Reversed: !tsuR ,olleH
```

> **Hint:** Store the original in a second variable before the `move` closure, or clone it before spawning.

---

### 3. `Arc<T>` — Atomic Reference Counting

> **Docs:** [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) · [Book — Shared state](https://doc.rust-lang.org/book/ch16-03-shared-state.html)

`Arc<T>` (Atomically Reference Counted) is the thread-safe sibling of `Rc<T>`. It tracks the reference count using CPU atomic operations, which makes it safe to clone and send across thread boundaries. `Arc::clone` is still cheap (an atomic increment) but slightly more expensive than `Rc::clone` because atomic operations imply memory barriers.

The rule of thumb: use `Rc<T>` within a single thread, use `Arc<T>` when sharing across threads. Like `Rc<T>`, `Arc<T>` only provides shared references — for mutation you still need `Mutex<T>` or `RwLock<T>`.

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(vec![10, 20, 30, 40, 50]);

    let mut handles = Vec::new();

    for i in 0..3 {
        // Clone the Arc — increments the reference count atomically
        let data_clone = Arc::clone(&data);

        let handle = thread::spawn(move || {
            // data_clone gives us read access to the shared Vec
            let sum: i32 = data_clone.iter().sum();
            println!("Thread {i}: sum = {sum}");
            sum
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Arc strong count after join: {}", Arc::strong_count(&data));
}
```

**Expected output (thread order may vary):**
```
Thread 0: sum = 150
Thread 1: sum = 150
Thread 2: sum = 150
Arc strong count after join: 1
```

#### Exercise 3.1 — Shared Config

**Goal:** Share a read-only `HashMap<String, String>` config across 4 threads using `Arc`.

Create a `HashMap` with 3 entries. Wrap it in `Arc::new`. Spawn 4 threads, each looking up a key (hardcode which key each thread checks) and printing the value. Join all threads.

**Expected output (order may vary):**
```
Thread 0: host = localhost
Thread 1: port = 8080
Thread 2: host = localhost
Thread 3: port = 8080
```

> **Hint:** `Arc<HashMap<String, String>>` can be cloned with `Arc::clone` and sent across threads. The `HashMap` is immutable, so no `Mutex` is needed.

---

### 4. `Mutex<T>` — Mutual Exclusion

> **Docs:** [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html) · [Book — Shared state](https://doc.rust-lang.org/book/ch16-03-shared-state.html)

A `Mutex<T>` (mutual exclusion lock) wraps data and ensures that only one thread accesses it at a time. `mutex.lock()` blocks the calling thread until it acquires the lock and returns a `MutexGuard<T>`. The guard dereferences to `T` so you can read or mutate the data. When the guard is dropped (end of scope or explicitly with `drop`), the lock is released automatically.

*Poisoning:* if a thread panics while holding the lock, the `Mutex` is marked "poisoned." Subsequent calls to `lock()` return `Err`. You can recover by calling `.into_inner()` on the error, but usually a poisoned mutex indicates a serious bug.

```rust
use std::sync::Mutex;
use std::thread;

fn main() {
    let counter = Mutex::new(0_u32);

    // Single-threaded usage
    {
        let mut guard = counter.lock().unwrap();
        *guard += 1;
    } // guard dropped, lock released

    println!("Counter: {}", *counter.lock().unwrap());

    // Demonstrate poisoning (academic — usually avoid this)
    let m = Mutex::new(42_i32);
    let result = thread::spawn(move || {
        let _guard = m.lock().unwrap();
        panic!("oops"); // poisons the mutex
    }).join();

    println!("Thread result: {}", result.is_err()); // true — it panicked
}
```

**Expected output:**
```
Counter: 1
Thread result: true
```

#### Exercise 4.1 — Threaded Counter

**Goal:** Increment a shared counter from 10 threads, each incrementing it 100 times. Verify the final value is 1000.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0_u32));
    let mut handles = Vec::new();

    for _ in 0..10 {
        let c = Arc::clone(&counter);
        let h = thread::spawn(move || {
            for _ in 0..100 {
                *c.lock().unwrap() += 1;
            }
        });
        handles.push(h);
    }

    for h in handles { h.join().unwrap(); }

    println!("Final count: {}", *counter.lock().unwrap());
}
```

**Expected output:**
```
Final count: 1000
```

> **Hint:** This is the canonical `Arc<Mutex<T>>` pattern — clone the `Arc` before moving into each thread.

---

### 5. `Arc<Mutex<T>>` — Canonical Shared Mutable State

> **Docs:** [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) · [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html) · [Book — Shared state](https://doc.rust-lang.org/book/ch16-03-shared-state.html)

`Arc<Mutex<T>>` is the idiomatic Rust pattern for any data that multiple threads must read and write. `Arc` handles the shared ownership; `Mutex` handles the exclusive access. You clone the `Arc` before moving it into each thread, so all threads share the same underlying `Mutex<T>`.

A common mistake is holding the `MutexGuard` across an `.await` point in async code or across thread-blocking work — this can cause deadlocks. Keep the lock scope as short as possible.

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;

fn main() {
    let results: Arc<Mutex<HashMap<usize, u64>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let handles: Vec<_> = (0..5).map(|i| {
        let results = Arc::clone(&results);
        thread::spawn(move || {
            let value: u64 = (1..=(i as u64 + 1) * 1000).sum();
            results.lock().unwrap().insert(i, value);
        })
    }).collect();

    for h in handles { h.join().unwrap(); }

    let map = results.lock().unwrap();
    let mut keys: Vec<usize> = map.keys().cloned().collect();
    keys.sort();
    for k in keys {
        println!("Thread {k}: {}", map[&k]);
    }
}
```

**Expected output:**
```
Thread 0: 500500
Thread 1: 2001000
Thread 2: 4501500
Thread 3: 8002000
Thread 4: 12502500
```

---

### 6. `RwLock<T>` vs `Mutex<T>`

> **Docs:** [RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html) · [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html)

A `RwLock<T>` (read-write lock) allows either many concurrent readers *or* one writer, but never both simultaneously. This is more efficient than `Mutex` when reads are far more frequent than writes — all readers can proceed in parallel rather than serialising on the lock.

Use `rwlock.read()` for shared read access and `rwlock.write()` for exclusive write access. Both block until the lock can be acquired. The same poisoning behaviour as `Mutex` applies.

```rust
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let cache: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));

    // One writer thread
    {
        let cache = Arc::clone(&cache);
        thread::spawn(move || {
            let mut w = cache.write().unwrap();
            w.push("entry-1".to_string());
            w.push("entry-2".to_string());
        }).join().unwrap();
    }

    // Multiple reader threads
    let handles: Vec<_> = (0..3).map(|i| {
        let cache = Arc::clone(&cache);
        thread::spawn(move || {
            let r = cache.read().unwrap();
            println!("Reader {i}: {:?}", *r);
        })
    }).collect();

    for h in handles { h.join().unwrap(); }
}
```

**Expected output (reader order may vary):**
```
Reader 0: ["entry-1", "entry-2"]
Reader 1: ["entry-1", "entry-2"]
Reader 2: ["entry-1", "entry-2"]
```

#### Exercise 6.1 — Choose Your Lock

**Goal:** Practice choosing between `Mutex` and `RwLock`.

You have a `HashMap<String, u64>` that is initialised once at startup and then read by 8 threads concurrently with no further writes. Which type should you use: `Arc<Mutex<HashMap>>` or `Arc<RwLock<HashMap>>`? Write the solution using the appropriate type. Spawn 8 reader threads that each look up the key `"score"` and print it.

**Expected output (order may vary):**
```
Thread 0: score = 9999
Thread 1: score = 9999
...
Thread 7: score = 9999
```

> **Hint:** When there are no concurrent writers, `RwLock` lets all 8 readers proceed simultaneously — `Mutex` would serialise them one at a time.

---

### 7. `std::sync::mpsc` — Message Passing Channels

> **Docs:** [mpsc](https://doc.rust-lang.org/std/sync/mpsc/) · [Book — Message passing](https://doc.rust-lang.org/book/ch16-02-message-passing.html)

`mpsc` stands for **multiple producer, single consumer**. The `std::sync::mpsc::channel()` function returns a `(Sender<T>, Receiver<T>)` pair. `Sender<T>` can be cloned freely — each clone is an independent producer. `Receiver<T>` cannot be cloned; there is exactly one consumer per channel.

Key methods:
- `sender.send(value)` — returns `Result`; fails (with `SendError`) if the `Receiver` has been dropped.
- `receiver.recv()` — blocks until a message arrives; returns `Err` when all `Sender`s have been dropped (the channel is "hung up"). This is the natural loop-termination signal.
- `receiver.try_recv()` — non-blocking; returns `Err(TryRecvError::Empty)` if no message is ready.

This pattern embodies the Go proverb: *"Do not communicate by sharing memory; share memory by communicating."* Channels often replace `Arc<Mutex<T>>` when data naturally flows in one direction between threads.

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel::<String>();

    // Spawn 3 producers, each with a cloned Sender
    for i in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(format!("hello from thread {i}")).unwrap();
        });
    }

    // Drop the original tx so the channel closes when all clones are gone
    drop(tx);

    // recv() returns Err once every Sender has been dropped
    for msg in rx {
        println!("Received: {msg}");
    }
    println!("Channel closed — all senders dropped.");
}
```

**Expected output (order may vary):**
```
Received: hello from thread 0
Received: hello from thread 2
Received: hello from thread 1
Channel closed — all senders dropped.
```

#### Exercise 7.1 — Pipeline

**Goal:** Use two channels to build a two-stage pipeline. Stage-1 threads send integers to stage-2 threads; stage-2 threads square each integer and forward the result to main.

Spawn 3 stage-1 threads sending the numbers 1, 2, 3. Spawn 3 stage-2 threads, each receiving one number, squaring it, and sending to the final channel. Main collects and prints all squared results.

**Expected output (order may vary):**
```
1 squared = 1
4 squared = 4
9 squared = 9
```

> **Hint:** Clone the `Sender` before moving it into each thread. Remember to `drop` the original sender at each stage so the downstream receiver sees the channel close properly.

---

### 8. Atomic Types — Lock-Free Shared State

> **Docs:** [Atomics](https://doc.rust-lang.org/std/sync/atomic/) · [Ordering](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)

The `std::sync::atomic` module provides types like `AtomicUsize`, `AtomicBool`, `AtomicI32`, and others. They support fine-grained, lock-free operations directly on shared values — no `Mutex` needed.

Core operations:
- `load(ordering)` — read the current value.
- `store(val, ordering)` — write a new value.
- `fetch_add(val, ordering)` / `fetch_sub` — atomically add/subtract and return the *old* value.
- `compare_exchange(current, new, success_ord, fail_ord)` — only store `new` if the current value equals `current`; the basis of most lock-free algorithms.

**Memory orderings** control how the CPU and compiler may reorder instructions:
- `Ordering::SeqCst` — full sequential consistency; the safe default when in doubt. Every thread sees all `SeqCst` operations in the same global order.
- `Ordering::Relaxed` — no synchronisation guarantees; safe only for independent counters where you only need the final total (no happens-before relationship required).
- `Ordering::Acquire` / `Ordering::Release` — paired producer-consumer guarantee: a `Release` store is visible to a thread that performs an `Acquire` load of the same atomic.

Atomic types implement `Send + Sync` without any wrapper, making them ideal for global counters, flags, and statistics that would otherwise require `Arc<Mutex<T>>`.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

// A static atomic needs no Arc — it lives for the entire program
static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn main() {
    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                COUNTER.fetch_add(1, Ordering::SeqCst);
            })
        })
        .collect();

    for h in handles { h.join().unwrap(); }

    println!("Final counter: {}", COUNTER.load(Ordering::SeqCst));
    // Equivalent Arc<Mutex<usize>> would be:
    //   let c = Arc::new(Mutex::new(0usize));
    //   ... *c.lock().unwrap() += 1; ...
    // Atomics skip the lock entirely — no blocking, no poisoning.
}
```

**Expected output:**
```
Final counter: 10
```

#### Exercise 8.1 — Atomic Flag

**Goal:** Use an `Arc<AtomicBool>` as a cancellation flag. Spawn a worker thread that spins in a loop printing "working…" until the flag is set to `true`. After 50 ms, the main thread sets the flag and joins the worker.

**Expected output (approximately):**
```
working...
working...
working...
Cancelled — worker stopped.
```

> **Hint:** `AtomicBool::new(false)` for the initial state. In the worker loop use `flag.load(Ordering::Relaxed)` — `Relaxed` is sufficient here because the only guarantee you need is that the worker *eventually* sees the write, not that it happens before some other memory access.

---

### 9. TCP Networking — `std::net::TcpListener` and `TcpStream`

> **Docs:** [TcpListener](https://doc.rust-lang.org/std/net/struct.TcpListener.html) · [TcpStream](https://doc.rust-lang.org/std/net/struct.TcpStream.html) · [std::net](https://doc.rust-lang.org/std/net/)

TCP (Transmission Control Protocol) provides a reliable, ordered, connection-oriented byte stream. Rust's `std::net` module exposes it without any external crates.

Key API:
- `TcpListener::bind("127.0.0.1:7878")` — binds a socket to a local address and port.
- `listener.incoming()` — an iterator that yields one `TcpStream` per accepted connection; blocks until the next client connects.
- `TcpStream` implements both `Read` and `Write`. Wrap in `BufReader` for line-oriented reading; use `write_all` for guaranteed writes.
- `stream.try_clone()` — duplicates the file descriptor so you can hold separate read and write handles (same underlying socket).

The classic pattern for a multi-client server: for every stream returned by `incoming()`, spawn a new thread to handle it.

```rust
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::thread;

fn handle_client(stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut writer = stream.try_clone().expect("clone failed");
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        writer.write_all(line.as_bytes()).unwrap();
        line.clear();
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Listening on :7878 — connect with: nc 127.0.0.1 7878");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| handle_client(stream));
    }
}
```

To test without a separate client binary, run the server and in another terminal:
```
nc 127.0.0.1 7878
```

You can also write a minimal client:

```rust
use std::net::TcpStream;
use std::io::{Write, BufRead, BufReader};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
    stream.write_all(b"Hello, server!\n").unwrap();
    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();
    println!("Server echoed: {}", response.trim());
}
```

Run the server binary in one terminal, then the client binary in another. The client connects, sends one line, and prints the echoed response.

#### Exercise 9.1 — Uppercase Echo Server

**Goal:** Modify the echo server so that every line received is converted to uppercase before being sent back.

**Expected output (client side):**
```
Server echoed: HELLO, SERVER!
```

> **Hint:** `line.to_uppercase()` returns a new `String`. Call `write_all(uppercased.as_bytes())` instead of echoing the original `line`.

---

### 10. UDP Sockets — `std::net::UdpSocket`

> **Docs:** [UdpSocket](https://doc.rust-lang.org/std/net/struct.UdpSocket.html) · [std::net](https://doc.rust-lang.org/std/net/)

UDP (User Datagram Protocol) is connectionless and unreliable — datagrams may be lost, duplicated, or arrive out of order — but its lower overhead makes it the right choice for latency-sensitive applications like DNS, video streaming, VoIP, and games.

Key differences from TCP:
- No `accept()` loop — you `bind` once and immediately start reading/writing datagrams.
- `recv_from(&mut buf)` returns `(bytes_read, src_addr)`. Every receive is independent.
- `send_to(&buf, dest_addr)` sends a single datagram to the specified address.
- Datagram boundaries are preserved (unlike TCP's stream, where multiple writes may coalesce into one read).

```rust
// Server
use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:8080").unwrap();
    println!("UDP server listening on :8080");
    let mut buf = [0u8; 1024];
    loop {
        let (len, src) = socket.recv_from(&mut buf).unwrap();
        let msg = std::str::from_utf8(&buf[..len]).unwrap();
        println!("Received from {}: {}", src, msg);
        socket.send_to(&buf[..len], src).unwrap(); // echo back
    }
}
```

```rust
// Client
use std::net::UdpSocket;

fn main() {
    // Binding to "0.0.0.0:0" lets the OS choose any available port
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.send_to(b"ping", "127.0.0.1:8080").unwrap();
    let mut buf = [0u8; 1024];
    let (len, src) = socket.recv_from(&mut buf).unwrap();
    println!("Got from {}: {}", src, std::str::from_utf8(&buf[..len]).unwrap());
}
```

The `0.0.0.0:0` binding pattern is idiomatic for clients: port `0` tells the OS to assign any free ephemeral port. There is no handshake and no connection state — once you have a socket, you can send and receive immediately.

#### Exercise 10.1 — UDP Ping Pong

**Goal:** Write a UDP server that responds to the message `"ping"` with `"pong"` and to any other message with `"unknown command"`. Write a client that sends `"ping"` and prints the server's response.

**Expected output (client):**
```
pong
```

> **Hint:** After `recv_from`, compare `msg.trim()` against the string `"ping"` to handle any trailing newline. Use `socket.send_to(b"pong", src)` to reply.

---

## Day Project: Parallel File Hasher

### What You're Building

You will simulate hashing 8 "files" (represented as hardcoded strings) using a simple checksum (sum of bytes as `u64`). You will run this twice: once sequentially in a loop, and once in parallel with one thread per file. Results from the parallel run are collected into a shared `Arc<Mutex<HashMap<String, u64>>>`. You will time both approaches with `std::time::Instant` to measure the speedup — even though the work is trivial, the pattern generalises directly to real I/O-bound hashing.

### Requirements

1. Define `const FILE_CONTENTS: [(&str, &str); 8]` — an array of `(filename, content)` pairs. Use `include_str!`-style long strings to make the content non-trivial (or just repeat a pattern string many times with `.repeat(N)`).
2. Implement `fn checksum(content: &str) -> u64` that sums all bytes.
3. Sequential approach: iterate over `FILE_CONTENTS`, compute the checksum, store in a local `HashMap`. Time with `Instant::now()` and `.elapsed()`.
4. Parallel approach: for each file, clone the `Arc<Mutex<HashMap<String, u64>>>` and spawn a thread that computes the checksum and inserts it into the shared map. Collect all `JoinHandle`s and join them. Time the whole thing.
5. Print both result maps (sorted by filename) and both elapsed times.
6. Assert that both maps contain the same keys and values.

### Getting Started

```bash
cargo new day-17
cd day-17
```

No external dependencies needed — everything is in `std`.

```toml
[package]
name = "day-17"
version = "0.1.0"
edition = "2021"
```

### Running Your Solution

```bash
cargo run -p day-17
```

Successful output:

```
=== Sequential ===
file1.txt: 4821934
file2.txt: 5093821
file3.txt: 4712830
file4.txt: 5214091
file5.txt: 4998321
file6.txt: 5112043
file7.txt: 4823910
file8.txt: 5048291
Time: 312µs

=== Parallel ===
file1.txt: 4821934
file2.txt: 5093821
file3.txt: 4712830
file4.txt: 5214091
file5.txt: 4998321
file6.txt: 5112043
file7.txt: 4823910
file8.txt: 5048291
Time: 89µs

Results match: true
```

### Starter Code Sketch

Below is enough scaffolding to get oriented without spoiling the solution:

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

// Each entry is (filename, content).
// Make the content long enough to do non-trivial work.
const FILE_CONTENTS: [(&str, &str); 8] = [
    ("file1.txt", "Lorem ipsum dolor sit amet, ..."),
    ("file2.txt", "Rust is a systems programming language ..."),
    // ... six more entries
    ("file8.txt", "The quick brown fox ..."),
];

fn checksum(content: &str) -> u64 {
    content.bytes().map(|b| b as u64).sum()
}

fn sequential() -> HashMap<String, u64> {
    // TODO: iterate, compute, store
    todo!()
}

fn parallel() -> HashMap<String, u64> {
    let results: Arc<Mutex<HashMap<String, u64>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // TODO: spawn one thread per file, join, return the inner HashMap
    todo!()
}

fn main() {
    let t = Instant::now();
    let seq = sequential();
    let seq_time = t.elapsed();

    let t = Instant::now();
    let par = parallel();
    let par_time = t.elapsed();

    // TODO: print sorted results and times
    // TODO: assert_eq!(seq, par);
}
```

### Extension Challenges

- **Easy:** Add a third timing run using `std::thread::scope` (stable since Rust 1.63) instead of `Arc<Mutex<>>` — scoped threads can borrow from the parent scope, so you do not need `Arc` at all.
- **Medium:** Use a real hash function (add the `sha2` crate) and hash actual byte slices. Compare timing with and without threads for large inputs (generate content with `.repeat(100_000)`).
- **Hard:** Use a `Mutex<Vec<JoinHandle<(String, u64)>>>` pattern instead of a shared map — each thread returns its result through the `JoinHandle`, and the main thread collects without any mutex on the result side.

---

## Quick Reference

| Concept | Type | Use When |
|---|---|---|
| Shared ownership, single thread | `Rc<T>` | Single-threaded graphs / trees |
| Shared ownership, multi-thread | `Arc<T>` | Sharing read-only data across threads |
| Exclusive mutable access | `Mutex<T>` | Any shared mutable state |
| Many readers, few writers | `RwLock<T>` | Read-heavy shared data |
| Shared mutable across threads | `Arc<Mutex<T>>` | The canonical pattern |

## Common Pitfalls

**Deadlock:** Acquiring two locks in different orders in different threads. Always acquire locks in a consistent order, or use a single lock that covers both resources.

**Lock contention:** Holding a `MutexGuard` for longer than necessary. Compute results locally, then take the lock only to store them.

**Forgetting to join:** If you drop a `JoinHandle` without calling `.join()`, the thread is *detached* — it keeps running but you lose the ability to wait for it or get its result. Always join unless you genuinely want fire-and-forget behaviour.

**`Rc` instead of `Arc`:** `Rc<T>` is not `Send`, so the compiler will reject it in a `thread::spawn` closure. The fix is always to swap `Rc` for `Arc`.

**Mutex poisoning:** If your program panics with "poisoned" in the message, a thread panicked while holding the lock. Fix the root cause rather than calling `.into_inner()` to recover silently.

## Further Reading

- [The Rust Book — Chapter 16: Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [`std::sync` module documentation](https://doc.rust-lang.org/std/sync/index.html)
- [`std::thread` module documentation](https://doc.rust-lang.org/std/thread/index.html)
- [Rust Atomics and Locks (free online book)](https://marabos.nl/atomics/) — deep dive into the memory model
