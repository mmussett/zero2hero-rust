# Day 25: Logging and Observability with tracing

> **Project:** Instrumented Notes API — add structured logging, request tracing, and environment-controlled log levels to the notes service

## Learning Objectives

By the end of today you will be able to:
- Explain the difference between `log` (flat messages) and `tracing` (spans + structured fields)
- Instrument async functions with `#[instrument]` to automatically capture spans and arguments
- Emit structured events with `tracing::info!`, `debug!`, `warn!`, and `error!`
- Configure log levels at runtime using `RUST_LOG` and `EnvFilter`
- Add `TraceLayer` to an axum router for automatic HTTP request/response logging

---

## Concepts

### 1. `tracing` vs `log`

> **Docs:** [tracing](https://docs.rs/tracing/latest/tracing/) · [log](https://docs.rs/log/latest/log/)

The `log` crate is Rust's original logging facade. It works fine for synchronous code: you call `log::info!("processing item {}", id)` and a log line appears. The problem is that in async code, a single logical operation (handling one HTTP request) may be interleaved with dozens of other operations across multiple tasks. A flat stream of log lines from `log` gives you no way to see which lines belong to which request.

`tracing` solves this with two concepts: **spans** and **events**. A span represents a unit of work with a beginning and an end — like "handling request GET /notes/42". Events (the equivalent of log lines) occur inside a span's context. The subscriber can attach the active span's fields to every event emitted inside it, so you automatically know which request generated each log line, even across `.await` points.

Structured fields are the other key difference. Instead of formatting everything into a string (`"user {} created note {}"`, user_id, note_id), tracing lets you attach key-value pairs (`user_id = user_id, note_id = note_id`). This makes logs machine-readable and enables powerful filtering and aggregation in log management systems like Datadog, Honeycomb, or Grafana Loki.

```rust
use tracing::{info, warn, instrument};

// Structured fields — NOT string interpolation
info!(user_id = 42, action = "login", "user authenticated");
warn!(note_id = 7, "note not found");

// Both Display (%) and Debug (?) formatters are available
info!(value = %some_display_type, other = ?some_debug_type, "event");
```

#### Exercise 1.1 — Compare log and tracing

**Goal:** Observe the difference in output format between `log` and `tracing`.

Add both `log = "0.4"` and `tracing = "0.1"` to `Cargo.toml`. In `main`, call `log::info!("log message: {}", 1)` and `tracing::info!(n = 1, "tracing event")`. Initialize `tracing_subscriber::fmt().init()` and run the program. Notice that `tracing_subscriber` captures both (via the `log` compatibility layer) and formats structured fields differently.

**Expected output (approximate — format varies):**
```
INFO log message: 1
INFO tracing event n=1
```

> **Hint:** `tracing_subscriber` automatically bridges `log` events via the `tracing-log` feature. You don't need a separate log subscriber.

---

### 2. Spans

> **Docs:** [tracing::Span](https://docs.rs/tracing/latest/tracing/struct.Span.html) · [#[instrument]](https://docs.rs/tracing/latest/tracing/attr.instrument.html)

A span marks the beginning and end of a logical operation. All events emitted while a span is "entered" (active on the current thread or task) are associated with it. Spans can be nested — a "request" span might contain a "database query" span.

The easiest way to create spans in async code is the `#[instrument]` attribute macro. Placed on an async function, it creates a span when the function is called and closes it when the function returns (including across `.await` points, which is the critical difference from manual span usage).

```rust
use tracing::instrument;

#[instrument]
async fn fetch_user(id: u64) -> String {
    // This event carries the span field id=<value>
    tracing::debug!("fetching user from database");
    format!("user_{}", id)
}

// Manual span creation (when #[instrument] isn't suitable):
async fn manual_example() {
    let span = tracing::info_span!("my_operation", key = "value");
    let _guard = span.enter();
    // do work
    // guard dropped here — span ends
}
```

`#[instrument]` captures all function arguments as span fields by default. Use `skip(field_name)` to exclude large or sensitive arguments:

```rust
#[instrument(skip(pool, password), fields(user_id = %user.id))]
async fn login(pool: &Pool, user: &User, password: &str) -> Result<Token> {
    // pool and password are excluded; user.id is recorded
}
```

#### Exercise 2.1 — Instrument a Function

**Goal:** Add `#[instrument]` to a function and observe the span in output.

Write an async function `process_items(items: Vec<u32>)` that sleeps 10ms per item and logs `info!(item = i, "processed")`. Add `#[instrument]` to it. Call it from `main`. Run with `RUST_LOG=debug` and observe how the span open/close and nested events appear in the output.

**Expected output (with RUST_LOG=debug):**
```
DEBUG process_items{items=[1, 2, 3]}: processed item=1
DEBUG process_items{items=[1, 2, 3]}: processed item=2
DEBUG process_items{items=[1, 2, 3]}: processed item=3
```

> **Hint:** The span name defaults to the function name. Override it with `#[instrument(name = "custom_name")]`.

---

### 3. Events

> **Docs:** [tracing](https://docs.rs/tracing/latest/tracing/)

Events are the leaf nodes of tracing's data model — a point-in-time record that something happened. They are always emitted within a span context (even if that span is the implicit root span). The macro syntax mirrors `format!` but with structured fields prepended:

```rust
tracing::trace!("very detailed, usually off");
tracing::debug!(query = %sql, rows = count, "query executed");
tracing::info!(note_id = id, "note created");
tracing::warn!(retries = n, "retrying failed request");
tracing::error!(error = %e, "handler failed");
```

Field syntax:
- `field = value` — records as a structured field (value must implement `tracing::Value`)
- `field = %value` — formats with `Display`
- `field = ?value` — formats with `Debug`

The `%` and `?` sigils mirror `format!`'s `{}` and `{:?}`. Fields without sigils work for types that implement `tracing::Value` directly (integers, booleans, strings).

```rust
use tracing::info;

async fn create_note_handler(title: &str, body: &str) {
    // Good: structured fields for machine consumption
    info!(
        note_title = %title,
        body_len = body.len(),
        "creating note"
    );

    // Avoid: everything in the message string
    // info!("creating note with title={} body_len={}", title, body.len());
}
```

#### Exercise 3.1 — Structured vs Unstructured

**Goal:** Practice emitting events with structured fields.

Write a function that creates a mock note (just a struct, no database). Emit: a `debug` event before creation with the input fields, an `info` event after with the note's id and title, and a `warn` if the title is longer than 50 characters. Use structured fields for all three.

**Expected output (RUST_LOG=debug):**
```
DEBUG creating note title="My Title" body_len=10
INFO  note created id=1 title="My Title"
```

> **Hint:** `body.len()` is a `usize`, which implements `tracing::Value` directly — no `%` or `?` needed.

---

### 4. Subscribers

> **Docs:** [tracing-subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/) · [EnvFilter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)

A subscriber is the component that receives spans and events and decides what to do with them (write to stdout, send to a remote collector, etc.). `tracing_subscriber::fmt()` is the standard pretty-printer for development.

`EnvFilter` lets operators control verbosity at runtime without recompiling. It reads `RUST_LOG` (the same variable the `env_logger` crate uses, for familiarity):

```rust
use tracing_subscriber::{fmt, EnvFilter};

fn init_tracing() {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)   // show the module path of each event
        .with_thread_ids(false)
        .compact()           // single-line format
        .init();
}
```

`RUST_LOG` supports directive syntax for fine-grained control:

```bash
RUST_LOG=debug                   # all modules at debug
RUST_LOG=info                    # all at info
RUST_LOG=day_25=debug,axum=warn  # module-specific levels
RUST_LOG=day_25::handlers=trace  # even more specific
```

#### Exercise 4.1 — Configure EnvFilter

**Goal:** Observe how RUST_LOG changes output verbosity.

Initialize tracing with `EnvFilter::from_default_env()`. Emit events at trace, debug, info, warn, and error levels from `main`. Run the program three times: `RUST_LOG=error`, `RUST_LOG=info`, and `RUST_LOG=trace`. Count how many lines appear each time.

**Expected output (RUST_LOG=info):**
```
INFO  my app starting
WARN  this is a warning
ERROR this is an error
```

> **Hint:** `EnvFilter::from_default_env()` reads `RUST_LOG`. If the variable is unset, it defaults to no filtering (all events are suppressed). Use `EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))` to default to info level.

---

### 5. axum + tracing with TraceLayer

> **Docs:** [tracing::Span](https://docs.rs/tracing/latest/tracing/struct.Span.html) · [tracing-subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/)

`tower-http`'s `TraceLayer` wraps every request in a span that records the method, path, status code, and latency. Adding it to an axum router takes one line:

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/notes", get(list_notes))
    .layer(TraceLayer::new_for_http())
    .with_state(pool);
```

Every request now produces spans like:
```
INFO  request{method=GET path=/notes}: started processing request
INFO  request{method=GET path=/notes}: finished processing request status=200 latency=1.2ms
```

Your handler spans (from `#[instrument]`) nest inside the request span, so the full call tree is visible. Combine `TraceLayer` with `#[instrument(skip(state))]` on every handler for complete observability.

```rust
use axum::extract::State;
use tracing::instrument;

#[instrument(skip(state), fields(note_count))]
async fn list_notes(State(state): State<AppState>) -> Json<Vec<Note>> {
    let notes = state.notes.lock().unwrap();
    let count = notes.len();
    tracing::Span::current().record("note_count", count);
    info!("listing all notes");
    Json(notes.values().cloned().collect())
}
```

`tracing::Span::current().record("field", value)` lets you record a field whose value you don't know at span creation time (like a result count).

#### Exercise 5.1 — Add TraceLayer

**Goal:** Observe automatic request/response logging.

Add `TraceLayer::new_for_http()` to your axum router. Run the server with `RUST_LOG=debug`. Send three different requests (list, create, get by id). Observe that each request generates a span with method, path, and status code. Notice how handler spans appear nested within the request span.

**Expected output (excerpt):**
```
DEBUG request{method=POST path=/notes version=HTTP/1.1}: tower_http::trace::on_request: started processing request
DEBUG request{method=POST path=/notes version=HTTP/1.1}: create_note: note created id=1 title="Hello"
INFO  request{method=POST path=/notes version=HTTP/1.1}: tower_http::trace::on_response: finished processing request status=201 latency=2ms
```

> **Hint:** Set `RUST_LOG=tower_http=debug,day_25=debug` to see both tower's internal events and your handler events.

---

## Day Project: Instrumented Notes API

### What You're Building

Add full observability to the notes API from Days 22–24. Every HTTP request gets a span. Every handler emits structured events. The log level is controlled by `RUST_LOG`. By the end, you can set `RUST_LOG=info` for clean production output or `RUST_LOG=debug` to see every database query and its parameters.

### Requirements

1. Initialize `tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init()` at the very start of `main`
2. Add `TraceLayer::new_for_http()` to the axum router
3. Add `#[instrument(skip(state))]` to every handler
4. In `create_note`: emit `info!(note_id = note.id, title = %note.title, "note created")`
5. In `get_note`: emit `debug!(note_id = id, "fetching note")` and `warn!(note_id = id, "note not found")` on 404
6. In `delete_note`: emit `info!(note_id = id, "note deleted")` on success
7. In `list_notes`: emit `debug!(count = notes.len(), "listing notes")`
8. Show sample output at `RUST_LOG=info` and `RUST_LOG=debug` in comments at the top of `main.rs`

### Getting Started

```toml
[package]
name = "day-25"
version = "0.1.0"
edition = "2021"

[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tower-http = { version = "0.5", features = ["trace"] }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Key setup in `main.rs`:

```rust
use tracing_subscriber::{fmt, EnvFilter};
use tower_http::trace::TraceLayer;
use tracing::instrument;

fn init_tracing() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .compact()
        .init();
}

fn build_router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/notes", axum::routing::get(list_notes).post(create_note))
        .route("/notes/:id", axum::routing::get(get_note)
            .put(update_note)
            .delete(delete_note))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[instrument(skip(state))]
async fn create_note(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::Json(payload): axum::Json<CreateNote>,
) -> (axum::http::StatusCode, axum::Json<Note>) {
    // ... create the note ...
    tracing::info!(note_id = note.id, title = %note.title, "note created");
    (axum::http::StatusCode::CREATED, axum::Json(note))
}
```

### Running Your Solution

```bash
# Clean production output
RUST_LOG=info cargo run -p day-25

# Verbose development output
RUST_LOG=debug cargo run -p day-25

# Tower HTTP internal + your handlers only
RUST_LOG=tower_http=debug,day_25=debug cargo run -p day-25
```

Sample output at `RUST_LOG=info`:
```
INFO  day_25: Notes API listening on http://localhost:3000
INFO  request{method=POST path=/notes}: tower_http::trace::on_response: finished processing request status=201 latency=1ms
INFO  create_note: note created note_id=1 title="Buy milk"
INFO  request{method=GET path=/notes}: tower_http::trace::on_response: finished processing request status=200 latency=0ms
```

Sample output at `RUST_LOG=debug` adds:
```
DEBUG request{method=GET path=/notes}: list_notes: listing notes count=1
DEBUG request{method=GET path=/notes/999}: get_note: fetching note note_id=999
WARN  request{method=GET path=/notes/999}: get_note: note not found note_id=999
```

### Extension Challenges

- **Easy:** Add `with_target(true)` to the subscriber so each event shows its module path. Observe how axum and tower events are labeled differently from your code.
- **Medium:** Add `tracing-opentelemetry` and configure an OTLP exporter pointing to a local Jaeger instance (`docker run -p 16686:16686 -p 4317:4317 jaegertracing/all-in-one`). View the request traces in the Jaeger UI.
- **Hard:** Write a custom `tracing_subscriber::Layer` that counts events by level (info, warn, error) and exposes the counts via a `GET /metrics` endpoint as plain text.
