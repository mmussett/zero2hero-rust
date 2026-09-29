# Zero to Hero: Rust

[![Deploy to GitHub Pages](https://github.com/mmussett/zero2hero-rust/actions/workflows/deploy-pages.yml/badge.svg)](https://github.com/mmussett/zero2hero-rust/actions/workflows/deploy-pages.yml)

A self-paced, project-based 30-day curriculum that takes you from absolute beginner to confident, production-minded Rust developer.

**Read online:** [mmussett.github.io/zero2hero-rust](https://mmussett.github.io/zero2hero-rust/)

---

## What This Is

Each day introduces a focused set of concepts paired with a hands-on project you build from scratch. Exercises reinforce every concept as it is introduced — you never read about something without immediately writing it. Later days revisit earlier projects, iterating toward production quality rather than throwing them away.

By the end you will have written a concurrent TCP server, an async HTTP API backed by a database, a WebAssembly module running in the browser, a published crate with full documentation, and a complete capstone application of your own design.

## Prerequisites

- No prior Rust experience required
- Comfort with at least one other programming language (variables, loops, functions)
- A terminal and a text editor or IDE

Install the Rust toolchain before Day 01:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify:

```bash
rustc --version
cargo --version
```

## Curriculum

| Week | Days | Theme |
|------|------|-------|
| 1 | 01–07 | **Foundations** — toolchain, ownership, borrowing, enums, modules |
| 2 | 08–14 | **Core Language** — collections, traits, generics, lifetimes, iterators, testing |
| 3 | 15–21 | **Intermediate Patterns** — smart pointers, concurrency, async, I/O, CLI, WebAssembly |
| 4 | 22–30 | **Production** — HTTP APIs, databases, error handling, observability, macros, publishing |

See [CURRICULUM.md](CURRICULUM.md) for the full day-by-day plan.

### Week 1 — Foundations

| Day | Concept | Project |
|-----|---------|---------|
| 01 | Toolchain, Cargo, safe vs unsafe, ownership preview | Hello, Cargo |
| 02 | Variables, types, control flow, Copy vs Move | CLI calculator |
| 03 | Functions, structs, `Result<T, E>` | Contact card |
| 04 | Ownership and move semantics | Memory visualizer |
| 05 | Borrowing, references, lifetime introduction | String statistics tool |
| 06 | Enums and pattern matching | Traffic light state machine |
| 07 | Modules and packages | Multi-module library crate |

### Week 2 — Core Language

| Day | Concept | Project |
|-----|---------|---------|
| 08 | `std::collections` — Vec, HashMap, BTreeMap, VecDeque, BinaryHeap | Word frequency counter |
| 09 | `Option` & `Result`, `?` operator, combinators | CSV row parser |
| 10 | Traits and default implementations | Shape library |
| 11 | Generics and data structures (Stack, Queue, LinkedList, Graph) | Data structures library |
| 12 | Lifetimes — elision, structs, `'static`, HRTB | Longest-string finder |
| 13 | Closures and iterators | Data pipeline |
| 14 | Testing — unit, integration, doc tests | Full test suite |

### Week 3 — Intermediate Patterns

| Day | Concept | Project |
|-----|---------|---------|
| 15 | Trait objects and dynamic dispatch | Plugin renderer |
| 16 | Smart pointers — `Box`, `Rc`, `RefCell` | Binary tree |
| 17 | Concurrency — threads, `Arc`, `Mutex`, channels, atomics, TCP, UDP | Parallel file hasher; threaded echo server |
| 18 | Async/Await, Tokio, `select!`, async channels, async TCP & UDP | Async URL downloader; async echo server |
| 19 | `std::io` streams, file I/O, serde | Interactive calculator; config manager |
| 20 | CLI applications with `clap` | `todo` CLI tool |
| 21 | WebAssembly — `wasm-pack`, `wasm-bindgen` | Image filter in the browser |

### Week 4 — Production

| Day | Concept | Project |
|-----|---------|---------|
| 22 | HTTP server with `axum` | REST API for a notes app |
| 23 | Database with `sqlx` + SQLite | Persist notes to a database |
| 24 | Error handling at scale — `thiserror`, `anyhow` | Clean error hierarchy for the notes app |
| 25 | Logging and observability — `tracing` | Structured logs and spans on the API |
| 26 | Benchmarking and performance — `criterion` | Profile and improve the word-frequency counter |
| 27 | Unsafe Rust — raw pointers, FFI, unsafe traits | Raw-pointer linked list |
| 28 | Macros — `macro_rules!` and proc macro intro | Custom `Describe` derive macro |
| 29 | Publishing and packaging | Publish the shape library to crates.io |
| 30 | Capstone | End-to-end CLI + API + database app |

## Running the Code

This repository is a Cargo workspace. Each day is an independent crate.

```bash
# Check everything compiles
cargo check --workspace

# Run a specific day
cargo run -p day-01

# Test a specific day
cargo test -p day-14

# Day 21 (WebAssembly) uses wasm-pack instead
wasm-pack build day-21
```

## Reference Material

- [CURRICULUM.md](CURRICULUM.md) — full 30-day plan with concepts and projects
- [PRIMITIVES.md](PRIMITIVES.md) — every Rust primitive type: representation, ranges, methods, examples
- [DATA_STRUCTURES.md](DATA_STRUCTURES.md) — 11 data structures with complexity tables and Rust implementations

## License

MIT
