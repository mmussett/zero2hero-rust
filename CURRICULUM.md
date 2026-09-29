# Zero to Hero: Rust — 30-Day Curriculum

A project-based path from absolute beginner to production-minded Rust developer.
Each day pairs a core concept with a hands-on project you build from scratch.

---

## Week 1: Foundations

| Day | Concept | Project |
|-----|---------|---------|
| 01 | Toolchain, Cargo, Hello World | "Hello, Cargo" — first binary, explore `cargo new`, `run`, `build` |
| 02 | Variables, Types, Control Flow, `panic!`, `unwrap`, `expect` | CLI calculator — also introduces panics and basic error handling |
| 03 | Functions & Structs, `Result<T, E>` basics | Contact card — constructor returns `Result`, validated inputs |
| 04 | Ownership & Move Semantics | Memory visualizer — trace ownership transfers through function calls |
| 05 | Borrowing, References & Lifetime Introduction | String statistics tool — count words/chars without taking ownership; first lifetime annotations |
| 06 | Enums & Pattern Matching | Traffic light state machine — model states with `enum` and `match` |
| 07 | Modules & Packages | Refactor days 1–6 into a tidy multi-module library crate |

**Week 1 milestone:** You understand Rust's core safety model — ownership, borrowing, and your first lifetime annotations — the things that make Rust, Rust.

---

## Week 2: Core Language

| Day | Concept | Project |
|-----|---------|---------|
| 08 | `std::collections`: `Vec`, `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `VecDeque`, `BinaryHeap` | Word frequency counter; sorted index; sliding window; priority queue |
| 09 | `Option` & `Result`, the `?` operator | CSV row parser — handle missing fields and malformed input gracefully |
| 10 | Traits & Default Implementations | Shape library — `Area`, `Perimeter` traits across multiple types |
| 11 | Generics | Generic `Stack<T>` — build a type-safe stack from scratch |
| 12 | Lifetimes (mastery) | Longest-string finder — struct lifetimes, `'static`, elision rules, HRTB |
| 13 | Closures & Iterators | Data pipeline — chain `map`, `filter`, `fold` to transform a dataset |
| 14 | Testing (unit, integration, doc tests) | Fully test the week's projects; write doc-tests that double as examples |

**Week 2 milestone:** You can model any domain in idiomatic Rust and handle errors without panicking.

---

## Week 3: Intermediate Patterns

| Day | Concept | Project |
|-----|---------|---------|
| 15 | Trait Objects & Dynamic Dispatch | Plugin renderer — swap formatters (JSON/plain/markdown) at runtime |
| 16 | Smart Pointers: `Box`, `Rc`, `RefCell` | Binary tree — recursive structure using `Box`; shared nodes with `Rc` |
| 17 | Concurrency: threads, `Arc`, `Mutex`, channels, atomics, TCP, UDP | Parallel file hasher; threaded TCP echo server; UDP ping-pong |
| 18 | Async / Await, Tokio, `select!`, async channels, async TCP & UDP | Async URL downloader; async TCP echo server; async UDP time server |
| 19 | `std::io` streams, File I/O & Serialization (`serde`) | Interactive calculator (stdin/stdout); config manager — read/write TOML and JSON config files |
| 20 | CLI applications (`clap`) | `todo` CLI — add, list, complete, and delete tasks with subcommands |
| 21 | WebAssembly (`wasm-pack`, `wasm-bindgen`) | Image filter — compile Rust to WASM, call it from JavaScript in the browser |

**Week 3 milestone:** You can write concurrent programs (threads, async, channels, atomics), build TCP/UDP networked applications, build real CLI tools, and compile Rust to run in the browser.

---

## Week 4: Production

| Day | Concept | Project |
|-----|---------|---------|
| 22 | HTTP Server (`axum`) | REST API — CRUD endpoints for a notes app with JSON in/out; use `reqwest` to integration-test the live server |
| 23 | Database (`sqlx` + SQLite) | Persist the notes app to a database; run migrations |
| 24 | Error handling at scale (`thiserror`, `anyhow`) | Refactor all error types across the notes app into a clean error hierarchy |
| 25 | Logging & Observability (`tracing`) | Instrument the notes API with structured, levelled logs and spans |
| 26 | Benchmarking & Performance (`criterion`) | Profile the word-frequency counter (day 08); measure and improve throughput |
| 27 | Unsafe Rust | Implement a raw-pointer linked list; learn when `unsafe` is justified |
| 28 | Macros (declarative + proc macro intro) | `debug_assert_eq!` variant; derive a custom `Describe` proc macro |
| 29 | Publishing & Packaging | Polish and publish the shape library (day 10) to crates.io with full docs |
| 30 | Capstone | End-to-end CLI + API + database app — design, build, document, and ship it |

**Week 4 milestone:** You can design, build, instrument, and publish a production Rust service.

---

## Recurring Principles

- **Build first, read second.** Every day starts with a project goal; theory follows from necessity.
- **Errors are teachers.** Compiler errors are part of the curriculum — read them, don't just fix them.
- **Iterate, don't rewrite.** Later days revisit earlier projects (day 07 refactors 01–06; day 24 refactors 22–23).
- **Standard library first.** Reach for a crate only when the stdlib genuinely can't do it.
