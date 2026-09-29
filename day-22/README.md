# Day 22: HTTP Server with axum

> **Project:** Notes REST API — a fully-functional CRUD API for notes backed by an in-memory store

## Learning Objectives

By the end of today you will be able to:
- Build an HTTP server using axum's router and handler system
- Extract data from requests using axum's extractor types (`Json`, `Path`, `Query`, `State`)
- Share application state across handlers with `Arc<Mutex<T>>`
- Return structured JSON responses with appropriate HTTP status codes
- Write integration tests that make real HTTP requests against a live server

---

## Concepts

### 1. axum Overview

> **Docs:** [`axum`](https://docs.rs/axum/latest/axum/) · [`axum::Router`](https://docs.rs/axum/latest/axum/struct.Router.html) · [`tokio`](https://docs.rs/tokio/latest/tokio/)

axum is an ergonomic web framework built on top of `hyper` (a fast HTTP library) and `tower` (a middleware ecosystem). Its central insight is that HTTP handlers are just async functions, and axum figures out what each handler needs by looking at its argument types. You write plain async functions; axum handles the plumbing.

The entry point is `Router`. You attach routes with `.route(path, method_handler)`, where `method_handler` is one of `get(handler_fn)`, `post(handler_fn)`, `put(handler_fn)`, `delete(handler_fn)`, and so on. Handlers can be composed with tower middleware layers, giving you logging, compression, authentication, and rate limiting essentially for free.

axum's design deliberately avoids a separate "request object" that you thread through your code. Instead, you declare precisely what you need as function arguments and axum extracts it for you. This keeps handler signatures self-documenting and makes unit testing straightforward.

```rust
use axum::{Router, routing::get};
use tokio::net::TcpListener;

async fn hello() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
```

#### Exercise 1.1 — Hello Handler

**Goal:** Start a minimal axum server and verify it responds.

Create a `main.rs` with the router above. Run the server and use `curl http://localhost:3000` (or your browser) to verify you receive "Hello, world!". Then add a second route `GET /health` that returns `"ok"`.

**Expected output:**
```
Listening on http://localhost:3000
```

> **Hint:** Each call to `.route()` is chained on the `Router`. `Router::new().route("/", get(hello)).route("/health", get(health_handler))` registers both routes.

---

### 2. Handlers and Extractors

> **Docs:** [`axum::extract`](https://docs.rs/axum/latest/axum/extract/index.html) · [`axum`](https://docs.rs/axum/latest/axum/) · [HTTP status codes](https://docs.rs/axum/latest/axum/http/index.html)

Extractors are types that implement axum's `FromRequest` or `FromRequestParts` trait. axum calls `from_request` behind the scenes for each function argument before your handler runs. If extraction fails (e.g., the JSON body is malformed), axum automatically returns a 400 Bad Request response — your handler never runs.

The most common extractors are:

- `Json<T>`: deserialises the request body from JSON into `T`. `T` must implement `serde::Deserialize`.
- `Path<T>`: extracts named path segments. A route declared as `/notes/:id` with `Path<u64>` extracts the `id` segment as a `u64`.
- `Query<T>`: deserialises query parameters (`?key=value`) into `T`.
- `State<T>`: injects shared application state (covered in the next section).

Returning data is symmetric: types that implement `IntoResponse` can be returned directly. `Json<T>` (where `T: Serialize`) serialises the value to JSON and sets the `Content-Type` header. `StatusCode` alone returns an empty body with that status. A tuple `(StatusCode, Json<T>)` returns both.

```rust
use axum::{
    extract::{Path, Query},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

#[derive(Serialize)]
struct Item {
    id: u64,
    name: String,
}

// Path extracts :id, Query extracts ?q=
async fn get_item(
    Path(id): Path<u64>,
    Query(params): Query<SearchParams>,
) -> (StatusCode, Json<Item>) {
    let name = params.q.unwrap_or_else(|| format!("Item {}", id));
    (StatusCode::OK, Json(Item { id, name }))
}
```

#### Exercise 2.1 — Echo Handler

**Goal:** Practice using `Json` as both an extractor and a response type.

Write a `POST /echo` handler that accepts `Json<serde_json::Value>` and echoes the exact same JSON back with status 200.

**Expected output (with `curl -X POST http://localhost:3000/echo -H 'Content-Type: application/json' -d '{"msg":"hi"}'`):**
```json
{"msg":"hi"}
```

> **Hint:** `serde_json::Value` accepts any valid JSON, so you don't need to define a struct.

---

### 3. Shared State with `Arc`

> **Docs:** [`axum`](https://docs.rs/axum/latest/axum/) · [`tokio`](https://docs.rs/tokio/latest/tokio/)

Handlers are called from multiple threads simultaneously, so any state they share must be thread-safe. The standard pattern is to wrap your data in `Arc<Mutex<YourState>>`, derive or implement `Clone`, and register it with axum using `.with_state(state)`.

Inside a handler you add `State(state): State<AppState>` as a parameter. axum clones the `Arc` for each request — cheap because cloning an `Arc` only increments a reference count.

The state type must be `Clone + Send + Sync + 'static`. `Arc<Mutex<T>>` satisfies all of these as long as `T: Send`.

```rust
use axum::{extract::State, Router, routing::get};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<u64>>,
}

async fn increment(State(state): State<AppState>) -> String {
    let mut count = state.counter.lock().unwrap();
    *count += 1;
    format!("Count: {}", *count)
}

#[tokio::main]
async fn main() {
    let state = AppState {
        counter: Arc::new(Mutex::new(0)),
    };
    let app = Router::new()
        .route("/increment", get(increment))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

#### Exercise 3.1 — Shared Counter

**Goal:** See how state is shared and mutated across requests.

Run the counter example. Hit `GET /increment` three times. Verify the count increases on each request. Then explain in a comment why using `tokio::sync::Mutex` might be preferable to `std::sync::Mutex` in async code when the critical section does async work.

**Expected output (three requests):**
```
Count: 1
Count: 2
Count: 3
```

> **Hint:** `std::sync::Mutex::lock()` blocks the OS thread. `tokio::sync::Mutex::lock().await` yields to the async runtime instead. For short critical sections like incrementing a counter, `std::sync::Mutex` is fine.

---

### 4. Response Types

> **Docs:** [`axum::response`](https://docs.rs/axum/latest/axum/response/index.html) · [HTTP status codes](https://docs.rs/axum/latest/axum/http/index.html) · [`serde_json`](https://docs.rs/serde_json/latest/serde_json/)

axum's `IntoResponse` trait is the key to composing responses. Any type that implements it can be returned from a handler. The standard toolkit covers almost all cases:

- `&'static str` or `String`: plain text response
- `StatusCode`: status-only, no body
- `Json<T>`: JSON body with `Content-Type: application/json`
- `(StatusCode, Json<T>)`: status + JSON body
- `(StatusCode, HeaderMap, Body)`: full control

For custom error types, implement `IntoResponse` directly. This lets you define a domain error enum and return it from handlers without any `map_err` boilerplate:

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

enum AppError {
    NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

async fn risky_handler() -> Result<Json<serde_json::Value>, AppError> {
    // Returning Err converts via IntoResponse automatically
    Err(AppError::NotFound("item 42 does not exist".to_string()))
}
```

#### Exercise 4.1 — Custom Error Response

**Goal:** Implement `IntoResponse` for a simple error type.

Define an `ApiError` enum with `NotFound` and `BadRequest(String)` variants. Implement `IntoResponse` so both variants return JSON `{"error": "..."}` with the appropriate status code. Write a handler that uses `Result<Json<serde_json::Value>, ApiError>`.

**Expected output (for the not-found case):**
```json
{"error": "resource not found"}
```

> **Hint:** `(StatusCode::NOT_FOUND, Json(json!({ "error": "..." }))).into_response()` is all you need inside the `impl`.

---

### 5. Integration Testing with reqwest

> **Docs:** [`reqwest`](https://docs.rs/reqwest/latest/reqwest/) · [`tokio`](https://docs.rs/tokio/latest/tokio/)

Testing an HTTP API by starting a real server and making real HTTP calls is far more reliable than mocking. axum makes this easy: bind to port 0 (the OS assigns a free port), extract the bound address, then point your HTTP client at it.

`reqwest` is the standard async HTTP client for Rust. Its `.json::<T>()` method deserialises a response body. In a `#[tokio::test]` test, you can use `.await` normally.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use tokio::net::TcpListener;

    async fn start_test_server() -> String {
        let app = build_router(); // your router construction function
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{}", addr)
    }

    #[tokio::test]
    async fn test_hello() {
        let base = start_test_server().await;
        let resp = reqwest::get(format!("{}/", base)).await.unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.text().await.unwrap(), "Hello, world!");
    }
}
```

#### Exercise 5.1 — Write a Passing Integration Test

**Goal:** Stand up the hello server and assert on the response in a test.

Write a `#[tokio::test]` that starts the server on a random port, sends `GET /`, and asserts the body equals "Hello, world!".

**Expected output (`cargo test`):**
```
test tests::test_hello ... ok
```

> **Hint:** Add `reqwest = { version = "0.12", features = ["json"] }` to `[dev-dependencies]`, not `[dependencies]`.

---

## Day Project: Notes REST API

### What You're Building

A fully functional in-memory Notes API with five REST endpoints. Notes are stored in a `HashMap` protected by a `Mutex` inside an `Arc`, shared across all handlers as axum `State`. You will also write integration tests using `reqwest` that exercise every endpoint, giving you confidence the API works end-to-end.

This forms the foundation that Days 23–25 build on by adding database persistence, error handling, and observability.

### Requirements

1. Define `struct Note { id: u64, title: String, body: String }` (Serialize + Deserialize)
2. Define `struct CreateNote { title: String, body: String }` (Deserialize)
3. Define `struct UpdateNote { title: String, body: String }` (Deserialize)
4. Application state: `Arc<Mutex<HashMap<u64, Note>>>` with an atomic counter for IDs
5. `GET /notes` — returns `Json<Vec<Note>>` with all notes, status 200
6. `POST /notes` — creates a note, returns `(StatusCode::CREATED, Json<Note>)`
7. `GET /notes/:id` — returns `Json<Note>` or `StatusCode::NOT_FOUND`
8. `PUT /notes/:id` — replaces a note, returns `Json<Note>` or 404
9. `DELETE /notes/:id` — removes a note, returns `StatusCode::NO_CONTENT` or 404
10. Integration tests for all 5 endpoints in `#[cfg(test)]`

### Getting Started

```toml
[package]
name = "day-22"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
reqwest = { version = "0.12", features = ["json"] }
```

Scaffold your `main.rs`:

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Note {
    id: u64,
    title: String,
    body: String,
}

#[derive(Debug, Deserialize)]
struct CreateNote {
    title: String,
    body: String,
}

#[derive(Debug, Deserialize)]
struct UpdateNote {
    title: String,
    body: String,
}

#[derive(Clone)]
struct AppState {
    notes: Arc<Mutex<HashMap<u64, Note>>>,
    next_id: Arc<Mutex<u64>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            notes: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/notes", get(list_notes).post(create_note))
        .route(
            "/notes/:id",
            get(get_note).put(update_note).delete(delete_note),
        )
        .with_state(state)
}

async fn list_notes(State(state): State<AppState>) -> Json<Vec<Note>> {
    let notes = state.notes.lock().unwrap();
    Json(notes.values().cloned().collect())
}

async fn create_note(
    State(state): State<AppState>,
    Json(payload): Json<CreateNote>,
) -> (StatusCode, Json<Note>) {
    let mut id_lock = state.next_id.lock().unwrap();
    let id = *id_lock;
    *id_lock += 1;
    drop(id_lock);

    let note = Note { id, title: payload.title, body: payload.body };
    state.notes.lock().unwrap().insert(id, note.clone());
    (StatusCode::CREATED, Json(note))
}

async fn get_note(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<Note>, StatusCode> {
    let notes = state.notes.lock().unwrap();
    notes
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_note(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateNote>,
) -> Result<Json<Note>, StatusCode> {
    let mut notes = state.notes.lock().unwrap();
    if let Some(note) = notes.get_mut(&id) {
        note.title = payload.title;
        note.body = payload.body;
        Ok(Json(note.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_note(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> StatusCode {
    let mut notes = state.notes.lock().unwrap();
    if notes.remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[tokio::main]
async fn main() {
    let state = AppState::new();
    let app = build_router(state);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Notes API listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn spawn_server() -> String {
        let state = AppState::new();
        let app = build_router(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        format!("http://{}", addr)
    }

    #[tokio::test]
    async fn test_create_and_list() {
        let base = spawn_server().await;
        let client = reqwest::Client::new();

        // Create a note
        let resp = client
            .post(format!("{}/notes", base))
            .json(&serde_json::json!({ "title": "Hello", "body": "World" }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 201);
        let note: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(note["title"], "Hello");

        // List notes
        let resp = client.get(format!("{}/notes", base)).send().await.unwrap();
        assert_eq!(resp.status(), 200);
        let notes: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert_eq!(notes.len(), 1);
    }
    // Add tests for GET/:id, PUT/:id, DELETE/:id ...
}
```

### Running Your Solution

```bash
cargo run -p day-22
```

Successful output:
```
Notes API listening on http://localhost:3000
```

Test with curl:
```bash
curl -X POST http://localhost:3000/notes \
  -H 'Content-Type: application/json' \
  -d '{"title":"First Note","body":"Hello axum!"}'
# {"id":1,"title":"First Note","body":"Hello axum!"}

curl http://localhost:3000/notes
# [{"id":1,"title":"First Note","body":"Hello axum!"}]
```

Run tests:
```bash
cargo test -p day-22
```

### Extension Challenges

- **Easy:** Add a `GET /notes?search=keyword` query parameter that filters notes by title substring.
- **Medium:** Add a `PATCH /notes/:id` endpoint that accepts `UpdateNote` with both fields optional (`Option<String>`) and only updates the provided fields.
- **Hard:** Add request validation — reject `CreateNote` with an empty title by returning `(StatusCode::UNPROCESSABLE_ENTITY, Json({"error": "title cannot be empty"}))`. Extract this into a custom `ValidatedJson<T>` extractor that implements `FromRequest`.
