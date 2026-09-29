# Day 23: Database with sqlx and SQLite

> **Project:** Persistent Notes API — replace the in-memory HashMap with a real SQLite database using sqlx

## Learning Objectives

By the end of today you will be able to:
- Use sqlx to execute async SQL queries with compile-time verification
- Manage a connection pool and share it across axum handlers as `State`
- Write and run schema migrations using sqlx's migration system
- Map database rows to Rust structs with `#[derive(sqlx::FromRow)]`
- Use transactions to ensure atomic multi-step database operations

---

## Concepts

### 1. sqlx Overview

> **Docs:** [`sqlx`](https://docs.rs/sqlx/latest/sqlx/) · [sqlx SQLite](https://docs.rs/sqlx/latest/sqlx/sqlite/index.html) · [SQLite docs](https://www.sqlite.org/docs.html)

sqlx is an async SQL library for Rust that offers something most ORMs don't: compile-time query verification. When you use the `sqlx::query!` macro, sqlx connects to a real database at compile time, sends the query, and verifies that the SQL is valid and that the column types match the Rust types you're expecting. Typos in column names become compile errors rather than runtime panics.

This verification requires a database to be available at compile time. sqlx achieves this either by connecting to a live database (pointed to by the `DATABASE_URL` environment variable) or by using a prepared query cache in a `sqlx-data.json` file (generated with `cargo sqlx prepare`). For development, the live connection is simplest.

sqlx supports PostgreSQL, MySQL, and SQLite. Day 23 uses SQLite because it requires no server — the database is a single file on disk. The `SqlitePool` type manages a pool of connections to that file.

```rust
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite:notes.db").await?;
    println!("Connected to SQLite");

    // Simple query — returns a Vec of anonymous structs
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notes")
        .fetch_one(&pool)
        .await?;
    println!("Note count: {}", row.0);
    Ok(())
}
```

#### Exercise 1.1 — Connect and Query

**Goal:** Open a SQLite database and run a simple query.

Set `DATABASE_URL=sqlite:test.db` in your environment. Create a `main` that connects to the pool and executes `SELECT 1 AS val`. Print the result.

**Expected output:**
```
Connected to SQLite
val = 1
```

> **Hint:** `sqlx::query!("SELECT 1 AS val").fetch_one(&pool).await?` returns an anonymous record with a field `val`. Access it as `row.val`.

---

### 2. Connection Pool

> **Docs:** [`sqlx`](https://docs.rs/sqlx/latest/sqlx/) · [sqlx SQLite](https://docs.rs/sqlx/latest/sqlx/sqlite/index.html)

A connection pool keeps a collection of open database connections ready to use, instead of opening a new connection for every query. Opening a connection is expensive (file locks, initialization overhead) — pooling makes it nearly free.

`SqlitePool::connect(url)` creates a pool with sensible defaults. You can tune it with `SqlitePoolOptions`:

```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;

let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .connect("sqlite:notes.db")
    .await?;
```

The pool implements `Clone` (it's reference-counted internally), so you can pass it to multiple threads or axum handlers safely — just clone it. In axum, you register it as `State<SqlitePool>`:

```rust
let app = Router::new()
    .route("/notes", get(list_notes))
    .with_state(pool);  // SqlitePool is Clone + Send + Sync

async fn list_notes(State(pool): State<SqlitePool>) -> Json<Vec<Note>> {
    // use pool here
    todo!()
}
```

#### Exercise 2.1 — Pool in Axum State

**Goal:** Thread a `SqlitePool` through axum `State`.

Build a minimal axum app with `GET /db-check` that runs `SELECT 1` against the pool from `State` and returns `"db ok"`. Verify it responds correctly.

**Expected output:**
```
db ok
```

> **Hint:** The pool can be moved directly into `.with_state(pool)` without wrapping in `Arc` — it's already reference-counted.

---

### 3. Migrations

> **Docs:** [sqlx migrations](https://docs.rs/sqlx/latest/sqlx/migrate/index.html)

Database schema changes need to be tracked and reproducible. sqlx's migration system works by keeping SQL files in a `migrations/` directory. Each file is named with a sequential number and a description: `0001_create_notes.sql`, `0002_add_tags.sql`. sqlx tracks which migrations have been applied in a `_sqlx_migrations` table it manages itself.

You run all pending migrations at application startup with the `sqlx::migrate!()` macro. The macro embeds the migration files into the binary at compile time, so your binary is self-contained — no need to ship the SQL files separately.

```
migrations/
  0001_create_notes.sql
```

```sql
-- migrations/0001_create_notes.sql
CREATE TABLE IF NOT EXISTS notes (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    title   TEXT NOT NULL,
    body    TEXT NOT NULL
);
```

```rust
// In main, before starting the server:
sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .expect("Failed to run migrations");
```

For manual migration management, install the sqlx CLI:
```bash
cargo install sqlx-cli --no-default-features --features sqlite
sqlx migrate run --database-url sqlite:notes.db
```

#### Exercise 3.1 — Write and Run a Migration

**Goal:** Create a migrations directory, write a schema, and run it automatically on startup.

Create `migrations/0001_create_notes.sql` with the notes schema above. In `main`, call `sqlx::migrate!()` before starting the server. Verify the `notes` table is created by running `sqlite3 notes.db ".tables"`.

**Expected output:**
```
notes
_sqlx_migrations
```

> **Hint:** `sqlx::migrate!()` with no arguments defaults to `"./migrations"` relative to the crate root. The `DATABASE_URL` env var must be set for compile-time verification with `query!` macros.

---

### 4. `#[derive(sqlx::FromRow)]`

> **Docs:** [`sqlx`](https://docs.rs/sqlx/latest/sqlx/) · [`sqlx::query!`](https://docs.rs/sqlx/latest/sqlx/macro.query.html) · [`sqlx::query_as!`](https://docs.rs/sqlx/latest/sqlx/macro.query_as.html)

`sqlx::query_as!` maps query results to a named Rust struct. For this to work, the struct must implement `sqlx::FromRow`, which you get for free by deriving it. sqlx matches column names from the query result to struct field names. Fields must implement `sqlx::Decode` (all primitive types, `String`, `Option<T>`, etc. work out of the box).

```rust
use sqlx::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct Note {
    id: i64,      // SQLite INTEGER maps to i64
    title: String,
    body: String,
}

// Fetch all notes
let notes: Vec<Note> = sqlx::query_as!(Note, "SELECT id, title, body FROM notes")
    .fetch_all(&pool)
    .await?;

// Fetch one by id
let note = sqlx::query_as!(Note, "SELECT id, title, body FROM notes WHERE id = ?", id)
    .fetch_optional(&pool)
    .await?;  // Returns Option<Note>
```

Note that SQLite's `AUTOINCREMENT` columns return `i64` in Rust, not `u64`. Plan your types accordingly.

#### Exercise 4.1 — Fetch from Database

**Goal:** Insert a row directly in SQLite then retrieve it with `query_as!`.

Use `sqlite3 notes.db "INSERT INTO notes (title, body) VALUES ('Test', 'Body');"` to add a row. Write a function `fetch_all_notes(pool: &SqlitePool) -> Vec<Note>` using `query_as!`. Print each note's title.

**Expected output:**
```
Note: Test
```

> **Hint:** `fetch_all` returns `Vec<T>`, `fetch_one` panics if missing, `fetch_optional` returns `Option<T>`. Prefer `fetch_optional` for single-row lookups.

---

### 5. Transactions

> **Docs:** [`sqlx`](https://docs.rs/sqlx/latest/sqlx/)

A transaction groups multiple SQL statements into an all-or-nothing unit. Either all statements commit or none do — the database never ends up in a partial state. This is essential for operations like "create a note and update a tag count" that must stay consistent.

```rust
async fn create_note_transactionally(
    pool: &SqlitePool,
    title: &str,
    body: &str,
) -> Result<Note, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let note = sqlx::query_as!(
        Note,
        "INSERT INTO notes (title, body) VALUES (?, ?) RETURNING id, title, body",
        title,
        body
    )
    .fetch_one(&mut *tx)
    .await?;

    // If anything below fails, the INSERT is rolled back automatically
    sqlx::query!("UPDATE stats SET note_count = note_count + 1")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(note)
}
```

If `tx` is dropped without calling `.commit()`, sqlx automatically issues a `ROLLBACK`. This means `?` early returns in the middle of a transaction are safe — the drop handler cleans up.

#### Exercise 5.1 — Atomic Create

**Goal:** Wrap a note creation in a transaction and verify rollback behaviour.

Write a function that begins a transaction, inserts a note, then deliberately returns an error before committing. Verify (by querying the database after) that the insert was rolled back and the notes table remains empty.

**Expected output:**
```
Insert failed (as expected)
Note count after rollback: 0
```

> **Hint:** Drop `tx` without calling `.commit()` to trigger automatic rollback. `Err(sqlx::Error::RowNotFound)` is a convenient error to return manually for testing.

---

## Day Project: Persistent Notes API

### What You're Building

Replace the `Arc<Mutex<HashMap<u64, Note>>>` from Day 22 with a `SqlitePool`. All five REST endpoints now read from and write to a real SQLite database. The API surface stays identical — the same curl commands work — but data persists between server restarts.

### Requirements

1. Create `migrations/0001_create_notes.sql` with the notes table schema
2. Run `sqlx::migrate!()` on startup before the server starts listening
3. Share `SqlitePool` as axum `State`
4. `GET /notes` — `SELECT * FROM notes`, return `Vec<Note>`
5. `POST /notes` — `INSERT INTO notes`, return the created note with status 201
6. `GET /notes/:id` — `SELECT ... WHERE id = ?`, return 404 if `fetch_optional` returns `None`
7. `PUT /notes/:id` — `UPDATE notes SET ... WHERE id = ?`, return 404 if no rows affected
8. `DELETE /notes/:id` — `DELETE FROM notes WHERE id = ?`, return 204 or 404
9. Set `DATABASE_URL=sqlite:notes.db` before building/running

### Getting Started

```toml
[package]
name = "day-23"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "migrate"] }
```

Set the database URL before running:
```bash
export DATABASE_URL=sqlite:notes.db   # Linux/macOS
set DATABASE_URL=sqlite:notes.db      # Windows CMD
$env:DATABASE_URL="sqlite:notes.db"   # PowerShell
```

Migration file at `day-23/migrations/0001_create_notes.sql`:
```sql
CREATE TABLE IF NOT EXISTS notes (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    body  TEXT NOT NULL
);
```

Key handler pattern — adapt this for each endpoint:

```rust
use axum::{extract::{Path, State}, http::StatusCode, Json};
use sqlx::SqlitePool;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
struct Note {
    id: i64,
    title: String,
    body: String,
}

#[derive(serde::Deserialize)]
struct CreateNote {
    title: String,
    body: String,
}

async fn create_note(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateNote>,
) -> Result<(StatusCode, Json<Note>), StatusCode> {
    let note = sqlx::query_as!(
        Note,
        "INSERT INTO notes (title, body) VALUES (?, ?) RETURNING id, title, body",
        payload.title,
        payload.body
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(note)))
}

async fn get_note(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Note>, StatusCode> {
    sqlx::query_as!(Note, "SELECT id, title, body FROM notes WHERE id = ?", id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = axum::Router::new()
        .route("/notes", axum::routing::get(list_notes).post(create_note))
        .route("/notes/:id", axum::routing::get(get_note)
            .put(update_note)
            .delete(delete_note))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}
```

### Running Your Solution

```bash
export DATABASE_URL=sqlite:notes.db
cargo run -p day-23
```

Successful output:
```
Listening on http://localhost:3000
```

Stop the server, restart it, and verify that previously created notes are still present — persistence confirmed.

### Extension Challenges

- **Easy:** Add a `created_at TEXT NOT NULL DEFAULT (datetime('now'))` column to the schema in a second migration `0002_add_timestamps.sql` and include it in the `Note` struct.
- **Medium:** Add full-text search via `GET /notes?q=keyword` that uses `SELECT ... WHERE title LIKE ? OR body LIKE ?` with `format!("%{}%", q)` as the parameter.
- **Hard:** Use a transaction in `POST /notes` that also inserts into an `audit_log` table (`action TEXT, note_id INTEGER, happened_at TEXT`). If the audit insert fails, the note creation rolls back too.
