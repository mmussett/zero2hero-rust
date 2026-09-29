# Day 30: Capstone

> **No scaffold today.** This day belongs entirely to you. Use everything you have built over the past four weeks to design and ship something of your own.

---

## 30-Day Skills Inventory

You started this curriculum as a Rust beginner. Look at what you can do now.

### Week 1 — Foundations
- [ ] **Ownership and borrowing** — you understand why Rust has no garbage collector and no use-after-free bugs (Days 1–3)
- [ ] **Structs and enums** — defining your own types with methods and associated functions (Day 4)
- [ ] **Pattern matching** — exhaustive `match`, `if let`, `while let`, destructuring (Day 4)
- [ ] **Error handling with Result** — `?` operator, `map_err`, `ok_or` (Day 5)
- [ ] **Traits** — defining shared behaviour, default implementations, trait bounds (Day 6)
- [ ] **Generics** — writing code that works over many types without runtime cost (Day 6)
- [ ] **Lifetimes** — annotating references so the borrow checker can verify validity (Day 7)
- [ ] **Closures and iterators** — `map`, `filter`, `fold`, `collect`, lazy evaluation (Day 8)
- [ ] **Testing** — `#[test]`, `assert_eq!`, integration tests, `should_panic` (Day 9)
- [ ] **Modules and crates** — `pub`, `use`, `mod`, workspace organisation (Days 9–10)

### Week 2 — Intermediate
- [ ] **Trait objects** — `Box<dyn Trait>`, dynamic dispatch, object safety (Day 11)
- [ ] **Smart pointers** — `Box<T>`, `Rc<T>`, `Arc<T>`, `RefCell<T>`, `Cow<T>` (Day 12)
- [ ] **Concurrency** — `std::thread`, `Mutex`, `RwLock`, message passing with channels (Day 13)
- [ ] **String handling** — `String` vs `&str`, UTF-8, `from_utf8`, `OsStr` (Day 14)
- [ ] **File I/O** — `std::fs`, `BufReader`, `BufWriter`, directory traversal (Day 15)
- [ ] **serde** — `#[derive(Serialize, Deserialize)]`, JSON, TOML, custom (de)serialisers (Day 16)
- [ ] **CLI tools** — argument parsing with `clap`, subcommands, `structopt`-style derives (Day 17)
- [ ] **Collections deep dive** — `HashMap`, `BTreeMap`, `BinaryHeap`, `VecDeque`, entry API (Day 18)
- [ ] **Iterators deep dive** — custom `Iterator` implementations, `chain`, `zip`, `flat_map` (Day 19)
- [ ] **WASM basics** — compiling to `wasm32-unknown-unknown`, `wasm-pack`, `#[wasm_bindgen]` (Day 20)
- [ ] **Async/await** — `async fn`, `.await`, `tokio` runtime, `Future` trait (Day 21)

### Week 4 — Production
- [ ] **HTTP servers** — axum, handlers, extractors, `State<T>`, `IntoResponse` (Day 22)
- [ ] **Databases** — sqlx, `SqlitePool`, `sqlx::query!`, `#[derive(FromRow)]`, migrations (Day 23)
- [ ] **Structured error handling** — `thiserror` for domain errors, `anyhow` for application errors (Day 24)
- [ ] **Observability** — `tracing`, `#[instrument]`, structured fields, `EnvFilter`, `TraceLayer` (Day 25)
- [ ] **Benchmarking** — criterion, `black_box`, `BenchmarkGroup`, reading statistical output (Day 26)
- [ ] **Unsafe Rust** — raw pointers, safe wrappers, `SAFETY:` comments, `Send`/`Sync` (Day 27)
- [ ] **Macros** — `macro_rules!`, hygiene, procedural macros, `syn` + `quote` (Day 28)
- [ ] **Publishing** — Cargo.toml metadata, `///` doc comments, doc tests, feature flags, `cargo publish` (Day 29)

---

## Three Capstone Project Ideas

Choose the scope that fits your time and ambition. All three produce real, useful software. None of them are toy examples.

---

### Small (2–3 days): Fully-Featured CLI Tool

**Idea A — `rgrep`: A Local grep Replacement**

Build a `grep`-like tool in Rust that searches file contents recursively, with coloured output, regex support, and context lines.

Key features:
- Recursive directory search with `walkdir`
- Regex pattern matching with the `regex` crate
- Coloured output with `colored` or `owo-colors` (matching text highlighted)
- `--context N` flag showing N lines before and after each match
- `--ignore-case` / `-i` flag
- Respects `.gitignore` with `ignore` crate (optional)
- Line number display
- File count and match count summary

You will use: `clap`, `walkdir`, `regex`, closures and iterators (Day 8), file I/O (Day 15), error handling (Day 24).

**Idea B — `passgen`: Command-Line Password Manager**

A local, encrypted password vault that stores credentials in an encrypted file.

Key features:
- `passgen add <service>` — prompts for username and password, stores encrypted
- `passgen get <service>` — decrypts and copies password to clipboard (or prints)
- `passgen list` — shows stored service names (never passwords)
- `passgen delete <service>` — removes an entry
- Master password used to derive an encryption key (argon2 + AES-256-GCM via `ring` or `aes-gcm`)
- Vault stored as an encrypted JSON file

You will use: `clap`, serde (Day 16), file I/O (Day 15), error handling (Day 24), `thiserror`.

**Idea C — `md`: A Markdown Renderer**

Render Markdown to styled terminal output using ANSI escape codes.

Key features:
- Headings rendered with colour and size variations
- Bold and italic text
- Code blocks with syntax highlighting (`syntect` crate)
- Blockquotes with a coloured left border
- Lists with proper indentation
- Hyperlinks printed as `text [url]`
- Reads from a file or stdin (`md file.md` or `cat file.md | md`)

You will use: `clap`, string processing (Day 14), `pulldown-cmark` for parsing, `colored`/`crossterm` for output.

---

### Medium (1–2 weeks): Web Service

**Idea A — URL Shortener with SQLite**

A self-hosted URL shortener with a REST API and a simple web frontend.

Key features:
- `POST /shorten` — accepts a long URL, returns a short code (`{"short": "abc123", "url": "http://localhost:3000/abc123"}`)
- `GET /:code` — redirects to the original URL (301 Permanent Redirect)
- `GET /stats/:code` — returns click count and creation timestamp
- `DELETE /:code` — removes a short URL (requires an API key header)
- Codes are random 6-character base62 strings
- Click counts tracked in the database
- Optional: rate limiting per IP with a `HashMap<IpAddr, RateLimitState>` in state
- Optional: a minimal HTML page at `GET /` listing recent URLs

You will use: axum (Day 22), sqlx + SQLite (Day 23), error handling (Day 24), tracing (Day 25), `rand` crate.

**Idea B — Personal Finance Tracker API**

A REST API for tracking income and expenses, with category-based summaries and CSV export.

Key features:
- Transactions: `{ id, date, amount_cents, description, category, type: income|expense }`
- `POST /transactions`, `GET /transactions`, `DELETE /transactions/:id`
- `GET /summary?month=2024-01` — total income, total expenses, net, breakdown by category
- `GET /transactions/export` — returns a CSV file download
- Date range filtering: `GET /transactions?from=2024-01-01&to=2024-01-31`
- Category management: `GET /categories`, `POST /categories`
- Data persisted in SQLite with proper migrations

You will use: axum (Day 22), sqlx (Day 23), thiserror + anyhow (Day 24), tracing (Day 25), serde (Day 16).

**Idea C — Chat Server with WebSockets**

A real-time multi-room chat server using WebSocket connections.

Key features:
- `GET /ws` — WebSocket upgrade endpoint
- Multiple named rooms; clients send `{"action":"join","room":"general"}` to join
- Broadcast messages to all clients in a room
- User list: list connected users in a room
- Persistent chat history (last 50 messages per room) in SQLite
- `GET /rooms` — list active rooms and their user counts
- Graceful disconnect handling

You will use: axum (Day 22) with `axum::extract::ws`, tokio broadcast channels (Day 13 concurrency concepts), sqlx (Day 23), tracing (Day 25), `tokio::sync::broadcast`.

---

### Large (2–4 weeks): Substantial Project

**Idea A — Static Site Generator**

A command-line tool that converts a directory of Markdown files into a complete static website.

Key features:
- Reads Markdown files with YAML front matter (`title`, `date`, `tags`, `draft`)
- Generates HTML using a template engine (`minijinja` or `tera`)
- Supports layouts: `post.html`, `index.html`, `tag.html` templates
- Tag pages aggregating posts by tag
- RSS feed generation
- Asset pipeline: copies static files (CSS, images) to output directory
- `ssg serve` subcommand that starts a local dev server and rebuilds on file changes (using `notify` crate for filesystem watching)
- Syntax highlighting for code blocks (`syntect`)
- Incremental builds: only regenerate changed pages

You will use: `clap`, file I/O, serde, `pulldown-cmark`, `minijinja`/`tera`, axum (for dev server), tracing, criterion (for benchmarking build times).

**Idea B — Minimal Key-Value Database**

A simple persistent key-value store inspired by bitcask, with a write-ahead log.

Key features:
- `SET key value`, `GET key`, `DELETE key`, `LIST prefix*` operations
- Data stored in append-only log files on disk (like bitcask)
- In-memory index: `HashMap<String, FileOffset>` rebuilt on startup from the log
- Compaction: rewrite the log to remove deleted and overwritten keys
- TCP server with a simple text protocol (one command per line)
- Client library crate that connects and sends commands
- Benchmarks comparing different storage strategies

You will use: file I/O (Day 15), unsafe Rust for performance-critical paths (Day 27), criterion (Day 26), thiserror (Day 24), tracing (Day 25), tokio for the TCP server (Day 21).

**Idea C — WASM-Powered Image Editor**

A browser-based image editor where the image processing logic is compiled to WASM and the UI is plain HTML/JavaScript.

Key features:
- Load a PNG/JPEG from the local filesystem via a file input
- Operations: grayscale, brightness/contrast, blur (Gaussian), sharpen, flip, rotate 90°
- Each operation implemented in Rust, compiled to WASM
- Real-time preview: operations applied and displayed as the user adjusts sliders
- Export: download the processed image as PNG
- Operation history: undo/redo stack
- Optional: implement a custom image format for saving/loading the edit history

You will use: WASM (Day 20), `image` crate for pixel manipulation, `wasm-bindgen`, `js-sys`, `web-sys` for DOM access.

---

## Project Planning Checklist

Before writing a line of code, work through this checklist. The time you spend here is repaid tenfold when you're deep in implementation.

### Define the Domain Model
- [ ] What are the core entities? (e.g., User, Note, Transaction, Message)
- [ ] What are the relationships between them?
- [ ] Which fields are mandatory? Which are optional?
- [ ] Draw a simple diagram (even on paper) of how they relate

### Choose Your Crates
- [ ] HTTP framework: `axum` (async, tower-based) or `actix-web` (actor-based)
- [ ] Database: `sqlx` (async, compile-time checked) or `diesel` (sync, strongly typed)
- [ ] Error handling: `thiserror` (domain) + `anyhow` (application)
- [ ] CLI arguments: `clap` with derive macros
- [ ] Serialisation: `serde` + `serde_json` for JSON, `toml` for config
- [ ] Async runtime: `tokio` (default choice)
- [ ] Check crates.io and lib.rs for recent download counts — prefer actively maintained crates

### Plan Your Error Types
- [ ] What domain errors can occur? (e.g., NotFound, InvalidInput, Unauthorized)
- [ ] Define an enum with `thiserror` for each domain area
- [ ] Decide which errors are user-facing (return in API response) vs operator-facing (log only)
- [ ] Implement `IntoResponse` (for axum) or `ExitCode` (for CLI) on your error types

### Write Tests First for Core Logic
- [ ] Write tests for your domain logic before implementing it (TDD, or at least test-adjacent)
- [ ] Unit tests for pure functions (area calculations, parsing, validation)
- [ ] Integration tests for database layer (use a test database or in-memory SQLite)
- [ ] HTTP integration tests with a test server on port 0 (Day 22 pattern)

### Add Logging Before You Need It
- [ ] Initialize `tracing_subscriber` on the first day of the project
- [ ] Add `#[instrument]` to every function that touches the database or external services
- [ ] Emit structured events at decision points (created, deleted, not found, error)
- [ ] Set `RUST_LOG=debug` during development; `RUST_LOG=info` for the demo

### Database Schema (if applicable)
- [ ] Write your first migration before writing any query code
- [ ] Use `INTEGER PRIMARY KEY AUTOINCREMENT` (SQLite) or `SERIAL PRIMARY KEY` (Postgres)
- [ ] Add `created_at` and `updated_at` timestamps to every table from the start
- [ ] Test migrations by running `sqlx::migrate!()` in your test setup

### API Design (if applicable)
- [ ] Define request/response types as Rust structs with `serde::Deserialize`/`Serialize`
- [ ] Decide on error response format (e.g., `{"error": "message"}`) and use it consistently
- [ ] Document your API in a README table: method, path, request body, response

### Before You Call It Done
- [ ] `cargo clippy -- -D warnings` — fix all clippy warnings
- [ ] `cargo fmt` — consistent formatting
- [ ] `grep -r unwrap src/` — replace every `.unwrap()` with proper error handling
- [ ] `RUST_LOG=debug cargo run` and manually test every feature
- [ ] Write a README with installation instructions and usage examples

---

## What's Next

> **Docs:** [Standard library](https://doc.rust-lang.org/std/) · [Cargo Book](https://doc.rust-lang.org/cargo/) · [Async Book](https://rust-lang.github.io/async-book/) · [Rust Reference](https://doc.rust-lang.org/reference/) · [crates.io](https://crates.io/) · [Rust Playground](https://play.rust-lang.org/)

You have completed a rigorous 30-day curriculum. Here is where to go deeper.

### Essential Reading
- **The Rust Book** — [book.rust-lang.org](https://doc.rust-lang.org/book/) — The authoritative introduction. You've covered most of it; revisit chapters on lifetimes and async for deeper understanding.
- **Rust by Example** — [doc.rust-lang.org/rust-by-example](https://doc.rust-lang.org/rust-by-example/) — Short, focused code examples for every language feature.
- **The Rustonomicon** — [doc.rust-lang.org/nomicon](https://doc.rust-lang.org/nomicon/) — The dark arts of unsafe Rust. Now that you have Day 27 under your belt, you're ready.
- **Programming Rust** (O'Reilly, Blandy & Orendorff) — The best book for going deep on ownership, types, and concurrency. Worth every page.

### Hands-On Practice
- **Rustlings** — [github.com/rust-lang/rustlings](https://github.com/rust-lang/rustlings) — Small exercises that fix compile errors. Great for solidifying syntax.
- **Exercism Rust Track** — [exercism.org/tracks/rust](https://exercism.org/tracks/rust) — 100+ exercises with community mentor feedback. Free.
- **Advent of Code** — [adventofcode.com](https://adventofcode.com) — Annual programming puzzles, December each year. The Rust subreddit has solution threads.
- **LeetCode in Rust** — Solving algorithmic problems forces you to use iterators, collections, and ownership under constraint.

### Video Learning
- **Jon Gjengset — "Crust of Rust"** — [youtube.com/@jonhoo](https://www.youtube.com/c/jongjengset) — Deep dives into intermediate/advanced Rust topics: smart pointers, channels, async, macros. One of the best Rust educators alive.
- **Ryan Levick — Rust Streams** — [youtube.com/@RyanLevick](https://www.youtube.com/@RyanLevickDotCom) — Practical Rust development streamed live.
- **Logan Smith — Rust Videos** — Excellent explanations of ownership, lifetimes, and type system concepts.

### Staying Current
- **This Week in Rust** — [this-week-in-rust.org](https://this-week-in-rust.org) — Weekly newsletter: crate releases, blog posts, RFCs, and community news.
- **The Rust Blog** — [blog.rust-lang.org](https://blog.rust-lang.org) — Official announcements, edition guides, and language design posts.
- **Inside Rust Blog** — [blog.rust-lang.org/inside-rust](https://blog.rust-lang.org/inside-rust/) — Compiler internals, working group updates.

### Community
- **The Rust Programming Language Forum** — [users.rust-lang.org](https://users.rust-lang.org) — Friendly, high-quality Q&A. Great for asking "is this idiomatic?"
- **Rust Subreddit** — [reddit.com/r/rust](https://reddit.com/r/rust) — News, project showcases, discussions.
- **Official Rust Discord** — [discord.gg/rust-lang](https://discord.gg/rust-lang) — Real-time help. `#beginners` is welcoming; `#async` and `#unsafe` have experts.
- **RustConf Talks** — Recorded talks from the annual Rust conference. Available on YouTube.

### Advanced Topics to Explore Next
- **Async internals** — Read "Async Rust" (O'Reilly) or Jon Gjengset's "Implementing Futures from Scratch" video
- **WASM in production** — `leptos` (full-stack Rust web) or `dioxus` (React-like UI in Rust)
- **Embedded Rust** — `embedded-hal`, `no_std`, writing firmware without an OS
- **Compiler plugins and tooling** — Writing `cargo` subcommands, custom lints with Clippy plugins
- **Performance engineering** — `perf`, `flamegraph`, `heaptrack`, SIMD intrinsics via `std::arch`

---

## Congratulations

Thirty days ago you wrote your first `fn main()`. Today you can build HTTP servers, persist data to databases, handle errors gracefully, observe your system in production, write and ship library crates, reason about unsafe memory, and generate code with macros.

That is not a small thing. Most programmers who start learning Rust give up in the first week, stopped by the borrow checker and unfamiliar patterns. You pushed through. The things that felt arbitrary and frustrating early on — lifetimes, ownership, the `?` operator — are now tools you reach for instinctively.

Here is what the Rust community has known for a long time: the difficulty is front-loaded. The borrow checker fights you at the beginning precisely because it is saving you from bugs you haven't written yet. Every time the compiler rejects your code, it is rejecting a data race, a use-after-free, or an unchecked error that would have cost you hours of debugging at 2am in production.

The code you write in Rust is not just faster than most languages — it is more honest. It tells the truth about who owns what, when things can fail, and where concurrency happens. That honesty has a cost in upfront learning time. It pays dividends for the rest of the program's lifetime.

Now build something real. Pick one of the capstone ideas, or design your own. The Rust ecosystem has everything you need. You have everything you need.

Welcome to the other side.

```
    _
   | |
   | |_ _   _ _ __ _ __
   | __| | | | '__| '__|
   | |_| |_| | |  | |
    \__|\__,_|_|  |_|  🦀
```
