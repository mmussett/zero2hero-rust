# Day 18: Async/Await and Tokio

> **Project:** Async URL Downloader — simulate fetching 6 URLs with variable latency, comparing sequential vs concurrent execution times to see that concurrent takes ~max(latencies) rather than ~sum.

## Learning Objectives

By the end of today you will be able to:
- Explain why async exists and when to use it instead of threads.
- Write `async fn` functions and call them with `.await`.
- Set up a Tokio runtime with `#[tokio::main]` and spawn tasks with `tokio::spawn`.
- Use `tokio::join!` to run multiple futures concurrently and collect results.
- Distinguish between sequential `.await` chains and concurrent `join!`/`spawn` patterns.
- Measure the wall-clock difference between sequential and concurrent async code.

---

## Concepts

### 1. What Async Is and Why It Exists

> **Docs:** [async/await](https://doc.rust-lang.org/book/ch17-00-async-await.html) · [Async book](https://rust-lang.github.io/async-book/) · [Tokio docs](https://docs.rs/tokio/latest/tokio/)

Threads are the OS-level abstraction for concurrent work. Each OS thread has its own stack (commonly 2–8 MB) and is scheduled by the OS kernel. Spawning thousands of threads is expensive — the memory alone is prohibitive.

Async is a *cooperative concurrency* model. An async task is a lightweight state machine (usually a few hundred bytes) managed by a user-space *runtime* rather than the OS. When a task needs to wait for I/O, it *yields* control back to the runtime, which can immediately run another ready task on the same OS thread. This makes async ideal for programs that have thousands of concurrent I/O operations (network servers, crawlers, API clients) but spend most of their time waiting — not computing.

Use threads for CPU-bound work (image encoding, compression). Use async for I/O-bound work (HTTP, database queries, file reads). Mixing both is fine: `tokio::task::spawn_blocking` runs a blocking closure on a dedicated thread pool from within async code.

#### Exercise 1.1 — Async vs Threads Decision

**Goal:** Sharpen your intuition about when to use each model.

For each scenario below, decide whether you'd use threads or async and write a one-line justification as a `// comment` in the code:

```rust
// 1. Compress 1000 PNG images using a CPU-intensive algorithm
// Answer: ___

// 2. Fetch data from 500 REST APIs concurrently
// Answer: ___

// 3. Run a web server handling thousands of simultaneous HTTP connections
// Answer: ___

// 4. Compute prime numbers up to 10^9 across 8 CPU cores
// Answer: ___
```

**Expected output:** (this is a written exercise — no runnable output)
```
No runtime output — answers in comments.
```

> **Hint:** CPU-bound = threads; I/O-bound = async.

---

### 2. `async fn` and `.await`

> **Docs:** [async/await](https://doc.rust-lang.org/book/ch17-00-async-await.html) · [Future trait](https://doc.rust-lang.org/std/future/trait.Future.html)

Marking a function with `async` changes its return type. `async fn greet() -> String` actually returns `impl Future<Output = String>`. A `Future` is a value that represents a computation which may not be complete yet. Calling `greet()` does *not* run the function body — it returns a `Future` that, when polled by the runtime, will run the body.

`.await` is the operator that suspends the current async task until a `Future` completes. Under the hood, the compiler transforms an `async fn` into a state machine so that `.await` points become states — the function can be paused and resumed without allocating a separate stack.

```rust
use tokio::time::{sleep, Duration};

async fn fetch_data(id: u32) -> String {
    // Simulate I/O delay — yields to the runtime for this duration
    sleep(Duration::from_millis(100)).await;
    format!("Data for id={id}")
}

#[tokio::main]
async fn main() {
    let result = fetch_data(42).await;
    println!("{result}");
}
```

**Expected output:**
```
Data for id=42
```

#### Exercise 2.1 — Async Greeter

**Goal:** Write an `async fn` that takes a name and a delay, sleeps for the delay, then returns a greeting string.

```rust
use tokio::time::{sleep, Duration};

async fn greet(name: &str, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("Hello, {}!", name)
}

#[tokio::main]
async fn main() {
    let msg = greet("Rust", 50).await;
    println!("{msg}");
}
```

**Expected output:**
```
Hello, Rust!
```

> **Hint:** `&str` is fine in `async fn` when the future does not outlive the reference.

---

### 3. Tokio Runtime and `tokio::spawn`

> **Docs:** [Tokio docs](https://docs.rs/tokio/latest/tokio/) · [tokio::spawn](https://docs.rs/tokio/latest/tokio/fn.spawn.html)

`#[tokio::main]` is a procedural macro that wraps your `async fn main()` in a synchronous entry point. It creates a Tokio runtime — which manages a thread pool and an event loop — runs your async main function to completion, then shuts down.

`tokio::spawn(future)` submits a future to the runtime to be run *concurrently* with the current task. It returns a `JoinHandle<T>` that you can `.await` to retrieve the result. Unlike `thread::spawn`, spawned tasks are very cheap (no OS thread per task).

```rust
use tokio::time::{sleep, Duration};

async fn task(id: u32) -> u32 {
    sleep(Duration::from_millis(50)).await;
    println!("Task {id} complete");
    id * id
}

#[tokio::main]
async fn main() {
    let h1 = tokio::spawn(task(1));
    let h2 = tokio::spawn(task(2));
    let h3 = tokio::spawn(task(3));

    // All three tasks run concurrently; we await each handle
    let r1 = h1.await.unwrap();
    let r2 = h2.await.unwrap();
    let r3 = h3.await.unwrap();

    println!("Results: {r1}, {r2}, {r3}");
}
```

**Expected output (task order may vary):**
```
Task 1 complete
Task 2 complete
Task 3 complete
Results: 1, 4, 9
```

#### Exercise 3.1 — Spawned Batch

**Goal:** Spawn 5 tasks that each return their index squared. Collect all handles into a `Vec`, await them all, and print the results in order.

```rust
#[tokio::main]
async fn main() {
    let handles: Vec<_> = (1u32..=5)
        .map(|i| tokio::spawn(async move { i * i }))
        .collect();

    for (i, h) in handles.into_iter().enumerate() {
        let result = h.await.unwrap();
        println!("Task {}: {result}", i + 1);
    }
}
```

**Expected output:**
```
Task 1: 1
Task 2: 4
Task 3: 9
Task 4: 16
Task 5: 25
```

> **Hint:** Awaiting the handles in index order gives you the results in deterministic order even if the tasks complete out of order.

---

### 4. `tokio::join!` — Concurrent Await

> **Docs:** [tokio::join!](https://docs.rs/tokio/latest/tokio/macro.join.html)

`tokio::join!(f1, f2, f3)` runs all three futures concurrently and waits for all of them. It returns a tuple of results. Unlike `spawn`, the futures passed to `join!` do not need to be `'static` — they can borrow from the current scope. `join!` drives all the futures on the current task without creating new tasks.

The critical difference from sequential `.await` chains: if `f1` takes 200ms and `f2` takes 300ms, sequential awaiting takes ~500ms. `tokio::join!(f1, f2)` takes ~300ms (the maximum).

```rust
use tokio::time::{sleep, Duration, Instant};

async fn step(name: &str, ms: u64) -> String {
    sleep(Duration::from_millis(ms)).await;
    format!("{name} done in {ms}ms")
}

#[tokio::main]
async fn main() {
    let start = Instant::now();

    // Sequential
    let a = step("A", 200).await;
    let b = step("B", 300).await;
    println!("Sequential: {}, {} | total: {}ms", a, b, start.elapsed().as_millis());

    let start = Instant::now();

    // Concurrent with join!
    let (a, b) = tokio::join!(step("A", 200), step("B", 300));
    println!("Concurrent: {}, {} | total: {}ms", a, b, start.elapsed().as_millis());
}
```

**Expected output (approximate):**
```
Sequential: A done in 200ms, B done in 300ms | total: 500ms
Concurrent: A done in 200ms, B done in 300ms | total: 301ms
```

#### Exercise 4.1 — Three-Way Join

**Goal:** Use `tokio::join!` to run three simulated API calls concurrently. The calls take 150ms, 250ms, and 100ms. Print all three results and the total elapsed time (should be ~250ms, not ~500ms).

```rust
use tokio::time::{sleep, Duration, Instant};

async fn api_call(endpoint: &str, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("response from {endpoint}")
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let (r1, r2, r3) = tokio::join!(
        api_call("/users", 150),
        api_call("/orders", 250),
        api_call("/inventory", 100),
    );
    println!("{r1}");
    println!("{r2}");
    println!("{r3}");
    println!("Total: {}ms", start.elapsed().as_millis());
}
```

**Expected output (approximate):**
```
response from /users
response from /orders
response from /inventory
Total: 252ms
```

> **Hint:** `tokio::join!` is a macro that takes futures separated by commas. All three start simultaneously and the macro resolves when the last one finishes.

---

### 5. `async` Blocks and Inline Futures

> **Docs:** [Future trait](https://doc.rust-lang.org/std/future/trait.Future.html) · [Async book](https://rust-lang.github.io/async-book/)

You can create a `Future` inline without defining a named function using an `async { ... }` block. The block captures its environment like a closure (you need `move` if you want to capture by value). This is useful when you need a throwaway future in a `join!` or `spawn` call.

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let name = String::from("Rustacean");

    let greeting = async move {
        sleep(Duration::from_millis(10)).await;
        format!("Hello, {name}!")
    };

    let result = greeting.await;
    println!("{result}");

    // Inline in join!
    let (x, y) = tokio::join!(
        async { 1 + 1 },
        async { 2 + 2 },
    );
    println!("{x} + {y} = {}", x + y);
}
```

**Expected output:**
```
Hello, Rustacean!
2 + 4 = 6
```

#### Exercise 5.1 — Async Transform

**Goal:** Use an `async move` block to capture a `Vec<u32>`, square each element inside the block, and return the result. Await it in `main`.

**Expected output:**
```
Squared: [1, 4, 9, 16, 25]
```

> **Hint:** `async move { data.iter().map(|x| x * x).collect::<Vec<_>>() }` captures `data` and returns a `Vec<u32>`.

---

### 6. Sequential vs Concurrent — Seeing the Difference

> **Docs:** [async/await](https://doc.rust-lang.org/book/ch17-00-async-await.html) · [tokio::join!](https://docs.rs/tokio/latest/tokio/macro.join.html)

The most important async concept to internalise: `.await` *yields* the current task while it waits, but it does not run other things in parallel. Two sequential `.await` calls run one after the other. `join!` or `spawn` creates true concurrency.

```rust
use tokio::time::{sleep, Duration, Instant};

async fn work(ms: u64) -> u64 { sleep(Duration::from_millis(ms)).await; ms }

#[tokio::main]
async fn main() {
    // Sequential: 100 + 200 + 300 = ~600ms
    let t = Instant::now();
    let a = work(100).await;
    let b = work(200).await;
    let c = work(300).await;
    println!("Sequential: {a}+{b}+{c} = {}ms", t.elapsed().as_millis());

    // Concurrent: max(100, 200, 300) = ~300ms
    let t = Instant::now();
    let (a, b, c) = tokio::join!(work(100), work(200), work(300));
    println!("Concurrent: {a}+{b}+{c} = {}ms", t.elapsed().as_millis());
}
```

**Expected output (approximate):**
```
Sequential: 100+200+300 = 601ms
Concurrent: 100+200+300 = 301ms
```

---

### 7. Error Handling in Async Code

> **Docs:** [async/await](https://doc.rust-lang.org/book/ch17-00-async-await.html) · [Tokio docs](https://docs.rs/tokio/latest/tokio/)

`async fn` can return `Result<T, E>` just like synchronous functions. You use `?` inside async functions exactly as you would in sync code — Rust desugars it to early-return on `Err`. When using `tokio::spawn`, the `JoinHandle` wraps the return type in another `Result` to account for task panics, so you will often see `.await.unwrap()?` or `.await??`.

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum FetchError {
    Timeout,
    NotFound(String),
}

async fn fetch(url: &str, fail: bool) -> Result<String, FetchError> {
    sleep(Duration::from_millis(50)).await;
    if fail {
        Err(FetchError::NotFound(url.to_string()))
    } else {
        Ok(format!("body of {url}"))
    }
}

#[tokio::main]
async fn main() {
    match fetch("https://example.com", false).await {
        Ok(body) => println!("Success: {body}"),
        Err(e) => println!("Error: {e:?}"),
    }

    match fetch("https://missing.example.com", true).await {
        Ok(body) => println!("Success: {body}"),
        Err(e) => println!("Error: {e:?}"),
    }
}
```

**Expected output:**
```
Success: body of https://example.com
Error: NotFound("https://missing.example.com")
```

#### Exercise 7.1 — Fallible Batch

**Goal:** Spawn 4 tasks, two of which succeed and two of which fail. Collect all results, printing successes and errors separately.

```rust
#[tokio::main]
async fn main() {
    let tasks: Vec<_> = (0..4)
        .map(|i| tokio::spawn(async move {
            if i % 2 == 0 {
                Ok::<String, String>(format!("Task {i} ok"))
            } else {
                Err(format!("Task {i} failed"))
            }
        }))
        .collect();

    for h in tasks {
        match h.await.unwrap() {
            Ok(msg) => println!("OK: {msg}"),
            Err(e) => println!("ERR: {e}"),
        }
    }
}
```

**Expected output:**
```
OK: Task 0 ok
ERR: Task 1 failed
OK: Task 2 ok
ERR: Task 3 failed
```

> **Hint:** `h.await` returns `Result<Result<String, String>, JoinError>`. The outer `unwrap()` handles the panic case; the inner `match` handles the task's own `Result`.

---

### 8. `tokio::select!` — Racing Futures

> **Docs:** [tokio::select!](https://docs.rs/tokio/latest/tokio/macro.select.html)

`tokio::select!` runs multiple futures concurrently and returns as soon as **one** of them completes, cancelling (dropping) the others. This makes it ideal for timeout patterns, cancellation tokens, and "first response wins" scenarios.

```rust
tokio::select! {
    result = future_a => { /* handle result */ }
    result = future_b => { /* handle result */ }
}
```

The cancelled branch's future is dropped at the point it last yielded — so make sure your futures are *cancellation-safe*. Avoid holding async mutex guards or partially-written state across `.await` points in a `select!` branch. The `biased;` modifier forces top-to-bottom branch evaluation order, which is useful in tests where you need deterministic selection.

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let slow_task  = async { sleep(Duration::from_millis(200)).await; "slow" };
    let fast_task  = async { sleep(Duration::from_millis(50)).await;  "fast" };

    let winner = tokio::select! {
        v = slow_task => v,
        v = fast_task => v,
    };
    println!("Winner: {}", winner); // "fast"

    // Timeout pattern — cancel an operation if it takes too long
    let result = tokio::select! {
        _ = sleep(Duration::from_millis(100)) => Err("timed out"),
        v = async { sleep(Duration::from_millis(500)).await; "done" } => Ok(v),
    };
    println!("{:?}", result); // Err("timed out")
}
```

**Expected output:**
```
Winner: fast
Err("timed out")
```

#### Exercise 8.1 — First Response Wins

**Goal:** Use `tokio::select!` to simulate querying two "databases" (async sleep functions with different delays). Print whichever responds first.

Cargo deps: `tokio = { version = "1", features = ["full"] }`

> **Hint:** Wrap each simulated DB call in an async block that sleeps then returns a string. Put both async blocks as branches in `tokio::select!`.

---

### 9. Tokio Channels — `mpsc`, `oneshot`, and `broadcast`

> **Docs:** [tokio::sync](https://docs.rs/tokio/latest/tokio/sync/index.html) · [mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html) · [oneshot](https://docs.rs/tokio/latest/tokio/sync/oneshot/index.html) · [broadcast](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html)

Tokio provides async equivalents of the standard library's sync channels. The key difference: `send` and `recv` are async — they `.await` instead of blocking a thread.

Three channel types:

1. **`tokio::sync::mpsc`** — multiple producer, single consumer (unbounded or bounded). `channel(capacity)` for bounded. `Sender::send(val).await`, `Receiver::recv().await` returns `Option<T>` (`None` when all senders drop).
2. **`tokio::sync::oneshot`** — sends exactly one value from sender to receiver. `channel()` returns `(Sender<T>, Receiver<T>)`. `send(val)` is not async (infallible on the send side). `receiver.await` waits for the value.
3. **`tokio::sync::broadcast`** — one sender, many receivers. Every receiver gets every message. `channel(capacity)`. Call `subscribe()` to add a receiver.

```rust
use tokio::sync::{mpsc, oneshot};

async fn worker(id: u32, tx: mpsc::Sender<String>, done: oneshot::Sender<()>) {
    for i in 0..3 {
        tx.send(format!("worker {id} message {i}")).await.unwrap();
    }
    done.send(()).unwrap(); // signal completion
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(32);
    let (done_tx, done_rx) = oneshot::channel::<()>();

    tokio::spawn(worker(1, tx, done_tx));

    // Wait for done signal OR first message — whichever arrives first
    tokio::select! {
        _ = done_rx => println!("worker signalled done"),
        msg = rx.recv() => println!("first message: {:?}", msg),
    }

    // Drain remaining messages
    while let Some(msg) = rx.recv().await {
        println!("{}", msg);
    }
}
```

**Expected output (order may vary):**
```
first message: Some("worker 1 message 0")
worker 1 message 1
worker 1 message 2
```

#### Exercise 9.1 — Work Queue

**Goal:** Create a bounded `mpsc` channel with capacity 4. Spawn 3 producer tasks, each sending 2 strings. One consumer task receives all 6 messages and prints them. Use `tokio::spawn` for all producers and the consumer.

> **Hint:** The consumer loop ends when all senders have dropped — `while let Some(msg) = rx.recv().await`. Drop the original `tx` in `main` after cloning it for the producers so the consumer can detect the end.

---

### 10. Async TCP — `tokio::net::TcpListener`

> **Docs:** [tokio::net](https://docs.rs/tokio/latest/tokio/net/index.html) · [tokio::net::TcpListener](https://docs.rs/tokio/latest/tokio/net/struct.TcpListener.html)

`tokio::net::TcpListener` is the async equivalent of `std::net::TcpListener`. Call `bind("addr").await` to bind, then `accept().await` to receive a `(TcpStream, SocketAddr)`. `tokio::net::TcpStream` implements `AsyncRead` and `AsyncWrite` — use `tokio::io::BufReader`, `AsyncBufReadExt::read_line`, and `AsyncWriteExt::write_all` for line-oriented I/O.

Instead of spawning an OS thread per connection, spawn a `tokio::task` — thousands of concurrent connections with negligible overhead compared to the megabytes-per-thread cost of OS threads.

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

async fn handle(stream: TcpStream) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
        writer.write_all(line.as_bytes()).await.unwrap();
        line.clear();
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7879").await.unwrap();
    println!("Async TCP echo server on :7879");
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("Connection from {}", addr);
        tokio::spawn(handle(stream));
    }
}
```

Cargo deps: `tokio = { version = "1", features = ["full"] }`

Compare with the sync version from Day 17: each `tokio::spawn` here creates a lightweight task (not a thread). Thousands of concurrent connections cost microseconds of overhead each rather than megabytes of thread stack.

#### Exercise 10.1 — Connection Counter

**Goal:** Extend the async TCP server to count total accepted connections using `Arc<AtomicUsize>` (from `std::sync::atomic`). Print the connection count each time a new client connects.

> **Hint:** Clone the `Arc` before moving it into each `tokio::spawn` closure. Use `counter.fetch_add(1, Ordering::SeqCst)` inside the spawn.

---

### 11. Async UDP — `tokio::net::UdpSocket`

> **Docs:** [tokio::net](https://docs.rs/tokio/latest/tokio/net/index.html)

`tokio::net::UdpSocket` mirrors the sync API: `bind().await`, `recv_from(&mut buf).await`, `send_to(&buf, addr).await`. Because UDP has no persistent connections, there are no per-connection tasks — one socket can serve all peers from a single loop.

```rust
// Async UDP echo server
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    let socket = UdpSocket::bind("127.0.0.1:9090").await.unwrap();
    println!("Async UDP echo server on :9090");
    let mut buf = [0u8; 1024];
    loop {
        let (len, src) = socket.recv_from(&mut buf).await.unwrap();
        println!("From {}: {}", src, std::str::from_utf8(&buf[..len]).unwrap().trim());
        socket.send_to(&buf[..len], src).await.unwrap();
    }
}
```

```rust
// Async UDP client
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    socket.send_to(b"hello async udp\n", "127.0.0.1:9090").await.unwrap();
    let mut buf = [0u8; 1024];
    let (len, _) = socket.recv_from(&mut buf).await.unwrap();
    println!("Echo: {}", std::str::from_utf8(&buf[..len]).unwrap().trim());
}
```

Contrast with sync UDP from Day 17: the socket operations `.await` instead of blocking, freeing the Tokio thread to handle other tasks while waiting for datagrams. In a server handling thousands of UDP flows, this avoids spinning up thousands of OS threads.

#### Exercise 11.1 — Async UDP Time Server

**Goal:** Write an async UDP server that receives any datagram and responds with the current time as a string (`format!("{:?}", std::time::SystemTime::now())`). Write a client that sends a single byte and prints the time it receives back.

> **Hint:** `socket.recv_from(&mut buf).await` gives you the source address — you don't need to read the datagram contents, just the `src` address to reply to.

---

## Day Project: Async URL Downloader

### What You're Building

You will simulate fetching 6 URLs, each with a different hardcoded latency (using `tokio::time::sleep`). Each "download" returns a simulated response string. You will time two strategies: (1) sequential with `.await` in a loop, and (2) concurrent with `tokio::spawn` collecting `JoinHandle`s. The difference in elapsed time demonstrates the core value proposition of async I/O.

### Requirements

1. Define an array of 6 `(url: &str, delay_ms: u64)` pairs with delays like `[120, 340, 80, 270, 150, 210]`.
2. Implement `async fn download(url: &str, delay_ms: u64) -> String` that sleeps for `delay_ms` and returns `format!("Downloaded: {url} ({delay_ms}ms)")`.
3. Sequential approach: loop over the array, `.await` each download, collect results, time with `Instant::now()`.
4. Concurrent approach: spawn one `tokio::spawn` task per URL, collect handles, `.await` each handle, time the whole thing. Use `String::from` or `.to_string()` to give each task owned data.
5. Print all results for both approaches, then print both elapsed times side by side.
6. Print the theoretical sequential time (sum of delays) and theoretical concurrent time (max of delays) as a sanity check.

### Getting Started

```bash
cargo new day-18
cd day-18
```

Add to `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### Running Your Solution

```bash
cargo run -p day-18
```

Successful output (approximate — actual ms will vary by system):

```
=== Sequential ===
Downloaded: https://api.example.com/users (120ms)
Downloaded: https://api.example.com/posts (340ms)
Downloaded: https://api.example.com/comments (80ms)
Downloaded: https://api.example.com/todos (270ms)
Downloaded: https://api.example.com/photos (150ms)
Downloaded: https://api.example.com/albums (210ms)
Time: 1173ms

=== Concurrent ===
Downloaded: https://api.example.com/users (120ms)
Downloaded: https://api.example.com/posts (340ms)
Downloaded: https://api.example.com/comments (80ms)
Downloaded: https://api.example.com/todos (270ms)
Downloaded: https://api.example.com/photos (150ms)
Downloaded: https://api.example.com/albums (210ms)
Time: 342ms

Theoretical sequential: 1170ms
Theoretical concurrent: 340ms
Speedup: 3.4x
```

### Starter Code Sketch

Below is enough scaffolding to get oriented without spoiling the solution:

```rust
use tokio::time::{sleep, Duration, Instant};

const URLS: [(&str, u64); 6] = [
    ("https://api.example.com/users",     120),
    ("https://api.example.com/posts",     340),
    ("https://api.example.com/comments",   80),
    ("https://api.example.com/todos",     270),
    ("https://api.example.com/photos",    150),
    ("https://api.example.com/albums",    210),
];

async fn download(url: &str, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("Downloaded: {url} ({delay_ms}ms)")
}

async fn run_sequential() -> Vec<String> {
    // TODO: loop over URLS, await each download, collect results
    todo!()
}

async fn run_concurrent() -> Vec<String> {
    // TODO: spawn one task per URL, collect handles, await all
    todo!()
}

#[tokio::main]
async fn main() {
    // Sequential
    let t = Instant::now();
    let results = run_sequential().await;
    let seq_time = t.elapsed().as_millis();
    println!("=== Sequential ===");
    for r in &results { println!("{r}"); }
    println!("Time: {seq_time}ms\n");

    // Concurrent
    let t = Instant::now();
    let results = run_concurrent().await;
    let par_time = t.elapsed().as_millis();
    println!("=== Concurrent ===");
    for r in &results { println!("{r}"); }
    println!("Time: {par_time}ms\n");

    // Sanity check
    let theoretical_seq: u64 = URLS.iter().map(|(_, d)| d).sum();
    let theoretical_par: u64 = *URLS.iter().map(|(_, d)| d).max().unwrap();
    println!("Theoretical sequential: {theoretical_seq}ms");
    println!("Theoretical concurrent: {theoretical_par}ms");
    println!("Speedup: {:.1}x", seq_time as f64 / par_time as f64);
}
```

### Extension Challenges

- **Easy:** Add a `tokio::time::timeout` wrapper around each download so that any download taking longer than 500ms returns an `Err`. Handle the error gracefully by printing `"TIMEOUT: {url}"` instead of the response.
- **Medium:** Use `tokio::join!` with all 6 futures instead of `spawn`/`JoinHandle`. Compare ergonomics with the spawn approach — note that `join!` requires the number of futures to be known at compile time, while the `spawn` + `Vec<JoinHandle>` approach works with dynamic lengths.
- **Hard:** Build a semaphore-limited downloader: at most 3 concurrent downloads at any time. Use `tokio::sync::Semaphore` to implement the rate limit and test with 12 URLs.

---

## Quick Reference

| Concept | Syntax | Notes |
|---|---|---|
| Declare async function | `async fn foo() -> T` | Returns `impl Future<Output = T>` |
| Await a future | `foo().await` | Suspends task until future resolves |
| Spawn concurrent task | `tokio::spawn(future)` | Returns `JoinHandle<T>` |
| Await a join handle | `handle.await.unwrap()` | Outer `unwrap` for panic, inner for `Result` |
| Concurrent futures | `tokio::join!(f1, f2, f3)` | All start; resolves when all done |
| Inline future | `async move { ... }` | Captures environment by value |
| Sleep (non-blocking) | `tokio::time::sleep(dur).await` | Yields to runtime |
| Measure elapsed time | `tokio::time::Instant::now()` | Use in async context |

## Common Pitfalls

**Blocking in async:** Never call `std::thread::sleep` or any other blocking operation inside an async function — it blocks the entire runtime thread, preventing other tasks from running. Always use `tokio::time::sleep` for delays, and `tokio::task::spawn_blocking` for CPU-intensive or blocking code.

**Sequential `.await` when you wanted concurrent:** Two `.await` calls in sequence always run one after the other. You must use `join!` or `spawn` to get concurrency.

**Forgetting `#[tokio::main]`:** Without a runtime, calling `.await` anywhere will not compile. `#[tokio::main]` is the simplest way to get a runtime; `tokio::runtime::Runtime::new()` gives you more control.

**`!Send` futures in `tokio::spawn`:** `tokio::spawn` requires the future to be `Send` (because the runtime may run it on any thread). If you hold a `Rc`, a `MutexGuard` across an `.await`, or any other `!Send` type, the compiler will reject it. Fix: use `Arc` instead of `Rc`, and release guards before `.await` points.

**Infinite spawning:** Each `tokio::spawn` creates a new task. Spawning millions of tasks for short work items has overhead. For bulk parallelism use a bounded approach (`Semaphore`, `FuturesUnordered`, or batched `join!`).

## Further Reading

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) — official, hands-on introduction
- [Async Book](https://rust-lang.github.io/async-book/) — deep dive into futures and the async model
- [`tokio` crate documentation](https://docs.rs/tokio) — comprehensive API reference
- [tokio::sync module](https://docs.rs/tokio/latest/tokio/sync/index.html) — channels, semaphores, mutexes, and more
