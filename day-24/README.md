# Day 24: Error Handling at Scale — thiserror and anyhow

> **Project:** Refactored Notes API — replace panics and opaque errors with structured, context-rich error types

## Learning Objectives

By the end of today you will be able to:
- Explain why `Box<dyn Error>` is insufficient for production code
- Define domain error types using `thiserror` that callers can programmatically match on
- Add rich context to propagated errors using `anyhow`
- Layer thiserror (domain) and anyhow (application) appropriately
- Eliminate every `.unwrap()` from application code

---

## Concepts

### 1. The Problem with `Box<dyn Error>`

> **Docs:** [Error handling overview](https://doc.rust-lang.org/book/ch09-00-error-handling.html) · [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html)

`Box<dyn Error>` is convenient — `?` can convert any error into it, and it works as a return type for quick scripts. But it has a critical flaw: the error is **opaque**. Once you've boxed it, the only thing you can do is display it or debug-print it. You cannot match on `Box<dyn Error>` to handle a "not found" case differently from a "network timeout" case.

Consider a handler that calls a database function returning `Box<dyn Error>`. The handler has no idea whether the error means the resource is missing (return 404), the query was bad (return 400), or the database is down (return 503). All three collapse into the same `500 Internal Server Error`.

```rust
// This compiles but loses information:
async fn get_note(id: u64) -> Result<Note, Box<dyn std::error::Error>> {
    let note = db_fetch(id).await?;
    Ok(note)
}

// The handler can't distinguish these cases:
match get_note(id).await {
    Ok(note) => Json(note).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(), // always 500!
}
```

The solution is to use concrete, matchable error types for domain logic and a context-adding wrapper for application-level propagation.

#### Exercise 1.1 — Feel the Pain

**Goal:** Understand what information is lost when using `Box<dyn Error>`.

Write a function `fn parse_id(s: &str) -> Result<u64, Box<dyn std::error::Error>>` that parses a string to `u64`. Call it with `"abc"`, catch the error, and try to match it using `error.downcast_ref::<std::num::ParseIntError>()`. Print whether the downcast succeeded. This works — but explain in a comment why it's fragile in a larger codebase.

**Expected output:**
```
Downcast succeeded: ParseIntError { kind: InvalidDigit }
```

> **Hint:** `Box<dyn Error>` does support `downcast_ref` via `Any`, but callers must know the concrete type ahead of time, creating invisible coupling.

---

### 2. `thiserror` — Errors Callers Can Handle

> **Docs:** [`thiserror`](https://docs.rs/thiserror/latest/thiserror/) · [`From` trait](https://doc.rust-lang.org/std/convert/trait.From.html) · [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html)

`thiserror` is a derive macro that generates `std::error::Error` and `std::fmt::Display` implementations for your error enum. You write the enum; thiserror writes the boilerplate. This gives you:

- `#[error("message with {field} interpolation")]` for Display
- `#[from]` for automatic `impl From<OtherError> for YourError`
- `#[source]` to expose the underlying cause

Use thiserror for **library and domain layer errors** — any error that a caller might want to inspect and respond to differently.

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NoteError {
    #[error("note {0} not found")]
    NotFound(u64),

    #[error("title cannot be empty")]
    InvalidTitle,

    #[error("database error")]
    Database(#[from] sqlx::Error),
}

// Automatic conversion: sqlx::Error -> NoteError via ?
async fn fetch_note(pool: &SqlitePool, id: u64) -> Result<Note, NoteError> {
    sqlx::query_as!(Note, "SELECT id, title, body FROM notes WHERE id = ?", id)
        .fetch_optional(pool)
        .await?  // sqlx::Error converts via #[from]
        .ok_or(NoteError::NotFound(id))
}
```

Now the caller can match on `NoteError::NotFound` and return 404, while `NoteError::Database` becomes 500. The distinction is preserved all the way up.

#### Exercise 2.1 — Define a Domain Error

**Goal:** Write a `NoteError` enum with thiserror and use it in a function.

Define `NoteError` with `NotFound(u64)`, `InvalidTitle(String)`, and `ParseError(#[from] std::num::ParseIntError)`. Write a function `fn parse_note_id(s: &str) -> Result<u64, NoteError>` that parses the string to `u64`. Call it with both a valid ID and `"bad"`, and print the error messages.

**Expected output:**
```
Parsed: 42
Error: invalid digit found in string
```

> **Hint:** `s.parse::<u64>()` returns `Result<u64, ParseIntError>`. With `#[from] std::num::ParseIntError` on the variant, `?` automatically converts.

---

### 3. `anyhow` — Context-Rich Error Propagation

> **Docs:** [`anyhow`](https://docs.rs/anyhow/latest/anyhow/) · [`anyhow::Context`](https://docs.rs/anyhow/latest/anyhow/trait.Context.html) · [`anyhow::Error`](https://docs.rs/anyhow/latest/anyhow/struct.Error.html)

`anyhow` is for the **application layer** — the code that wires things together and ultimately reports errors to operators or users. It doesn't care about distinguishing error types; it cares about giving you enough context to diagnose what went wrong.

`anyhow::Result<T>` is `Result<T, anyhow::Error>`. Any error that implements `std::error::Error` can be converted into `anyhow::Error` via `?`. The critical feature is `.context("message")` and `.with_context(|| format!("..."))`: these prepend a human-readable description to the error chain.

```rust
use anyhow::{Context, Result};

async fn handle_request(id_str: &str) -> Result<Note> {
    let id: u64 = id_str
        .parse()
        .with_context(|| format!("failed to parse note ID from '{}'", id_str))?;

    let note = fetch_note(&pool, id)
        .await
        .with_context(|| format!("fetching note id={}", id))?;

    Ok(note)
}

// Error chain when id_str = "abc":
// failed to parse note ID from 'abc'
// caused by: invalid digit found in string
```

Do not use `anyhow` in library code — it forces `anyhow::Error` on your library's users. Use it in binaries and integration layers where you're reporting errors rather than propagating them for programmatic handling.

#### Exercise 3.1 — Add Context to Errors

**Goal:** Practice the `.with_context()` pattern.

Write a function `fn read_config(path: &str) -> anyhow::Result<String>` that reads a file with `std::fs::read_to_string`. Use `.with_context(|| format!("reading config from {}", path))`. Call it with a path that doesn't exist and print the full error with `{:#}` (which prints the full error chain).

**Expected output:**
```
Error: reading config from /nonexistent/config.toml
caused by: No such file or directory (os error 2)
```

> **Hint:** `println!("{:#}", err)` prints the full chain. `{:?}` also works but is more verbose. `{}` prints only the top-level message.

---

### 4. Error Hierarchies — Layering thiserror and anyhow

> **Docs:** [`thiserror`](https://docs.rs/thiserror/latest/thiserror/) · [`anyhow`](https://docs.rs/anyhow/latest/anyhow/)

Real applications have layers: a domain layer that defines business rules and errors, and an application layer (web handlers, CLI commands) that orchestrates and reports. The right tool for each layer differs.

```
Domain layer    →  thiserror  (NoteError, AuthError, etc.)
Application     →  anyhow     (adds context, converts for display)
HTTP handlers   →  IntoResponse impl on NoteError (maps to status codes)
```

The key insight: `NoteError` implements `IntoResponse` so axum can convert it directly. The handler uses `anyhow::Result` internally but converts at the boundary:

```rust
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;

impl IntoResponse for NoteError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            NoteError::NotFound(id) => {
                (StatusCode::NOT_FOUND, format!("note {} not found", id))
            }
            NoteError::InvalidTitle(t) => {
                (StatusCode::UNPROCESSABLE_ENTITY, format!("invalid title: {}", t))
            }
            NoteError::Database(e) => {
                eprintln!("database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string())
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

// Handler uses NoteError directly — axum calls into_response on Err
async fn get_note_handler(
    Path(id): Path<u64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Note>, NoteError> {
    let note = fetch_note(&pool, id).await?;  // NoteError propagates directly
    Ok(Json(note))
}
```

#### Exercise 4.1 — Implement IntoResponse

**Goal:** Make your `NoteError` type usable as an axum response.

Implement `IntoResponse` for `NoteError` as shown above. Write a handler that returns `Result<Json<Note>, NoteError>` and always returns `NoteError::NotFound(99)`. Hit it with curl and verify the response is a proper 404 JSON body.

**Expected output (curl):**
```json
{"error": "note 99 not found"}
```

> **Hint:** The handler's return type `Result<Json<Note>, NoteError>` tells axum to call `into_response()` on the `NoteError` when you return `Err(...)`.

---

### 5. Removing `.unwrap()`

> **Docs:** [? operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator) · [Error handling overview](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

Every `.unwrap()` in production code is a potential crash. A `None` value or an `Err` during a request handler will panic the thread (or task), and in async code that may silently drop a connection rather than returning a clean error to the client.

The systematic approach to eliminating `.unwrap()`:

1. **`?`** — propagate the error to the caller (most common)
2. **`unwrap_or_else(|_| default)`** — provide a fallback value
3. **`if let Some(x) = ...`** — handle the `None` case explicitly
4. **`ok_or(MyError::Missing)?`** — convert `Option` to `Result` then propagate
5. **`expect("reason")`** — acceptable only in tests or for invariants that truly cannot fail; document why

```rust
// Before: panics if the mutex is poisoned
let notes = state.notes.lock().unwrap();

// After: propagate with context
let notes = state.notes.lock()
    .map_err(|_| NoteError::Internal("mutex poisoned".to_string()))?;

// Before: panics if env var is missing
let db_url = std::env::var("DATABASE_URL").unwrap();

// After: meaningful error at startup
let db_url = std::env::var("DATABASE_URL")
    .context("DATABASE_URL must be set")?;
```

#### Exercise 5.1 — Unwrap Audit

**Goal:** Find and remove all `.unwrap()` calls from the Day 22 code.

Copy `day-22/src/main.rs` into a new file. Use `grep -n '\.unwrap()' src/main.rs` to list every occurrence. Replace each one using the techniques above. Re-run `cargo test` to verify nothing broke.

**Expected output:**
```
grep -n '.unwrap()' src/main.rs
(no output — all unwraps removed)
```

> **Hint:** The mutex lock `.unwrap()` in handlers can become `.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?` since the handler already returns `Result<_, StatusCode>`.

---

## Day Project: Refactored Notes API

### What You're Building

Take the Day 22–23 notes app and harden its error handling. The API surface stays the same; the internals improve significantly. By the end, every failure mode returns a meaningful HTTP response (never a panic), every database call has context for debugging, and the code is maintainable.

### Requirements

1. Define `enum NoteError { NotFound(u64), Database(#[from] sqlx::Error), InvalidTitle(String) }` with `thiserror`
2. Implement `IntoResponse` for `NoteError`: `NotFound` → 404, `InvalidTitle` → 422, `Database` → 500
3. In database functions (`fetch_note`, `create_note`, etc.), use `NoteError` as the return type
4. In the `main` startup path, use `anyhow::Result` and add `.context(...)` to each step (connect to DB, run migrations)
5. Validate that note titles are non-empty in `create_note` and `update_note`, returning `NoteError::InvalidTitle`
6. Remove every `.unwrap()` from the codebase — confirm with `grep -n unwrap src/main.rs` returning nothing
7. Keep all five REST endpoints functional with the same behaviour as Day 22

### Getting Started

```toml
[package]
name = "day-24"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "1"
anyhow = "1"
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "migrate"] }
```

Structure your `main.rs` around these types and the two-layer approach:

```rust
use thiserror::Error;
use anyhow::{Context, Result};
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};

#[derive(Debug, Error)]
pub enum NoteError {
    #[error("note {0} not found")]
    NotFound(u64),

    #[error("invalid title: {0}")]
    InvalidTitle(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for NoteError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            NoteError::NotFound(id) => (StatusCode::NOT_FOUND, format!("note {} not found", id)),
            NoteError::InvalidTitle(t) => (StatusCode::UNPROCESSABLE_ENTITY, format!("invalid title: {}", t)),
            NoteError::Database(e) => {
                eprintln!("db error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            }
        };
        (status, Json(serde_json::json!({ "error": msg }))).into_response()
    }
}

// Application startup uses anyhow for context-rich error reporting
#[tokio::main]
async fn main() -> Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL environment variable must be set")?;

    let pool = sqlx::SqlitePool::connect(&db_url)
        .await
        .with_context(|| format!("connecting to database at {}", db_url))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("running database migrations")?;

    // ... build and start router
    Ok(())
}
```

### Running Your Solution

```bash
export DATABASE_URL=sqlite:notes.db
cargo run -p day-24
```

Test that validation works:
```bash
curl -X POST http://localhost:3000/notes \
  -H 'Content-Type: application/json' \
  -d '{"title":"","body":"some body"}'
# {"error":"invalid title: title cannot be empty"}
# HTTP 422

curl http://localhost:3000/notes/999
# {"error":"note 999 not found"}
# HTTP 404
```

### Extension Challenges

- **Easy:** Add a `NoteError::TitleTooLong(usize)` variant with message `"title exceeds maximum length of 256 characters (got {0})"`. Enforce it in validation.
- **Medium:** Add a middleware layer that catches any handler panic (using `tower::ServiceBuilder` and a custom `HandleError` layer) and converts it to a 500 JSON response instead of dropping the connection.
- **Hard:** Create a separate `notes-domain` crate (library) that contains `NoteError` and the database functions. The `day-24` binary depends on it. This enforces the architectural boundary between domain and application layers at the crate level.
