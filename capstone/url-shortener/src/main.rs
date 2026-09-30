//! URL Shortener — a self-hosted REST API that maps short alphanumeric codes to
//! long URLs. Backed by SQLite via sqlx, served by Axum on port 3000.
//!
//! # Endpoints
//! - `POST /shorten`      — create a short link
//! - `GET  /:code`        — redirect to original URL (301)
//! - `GET  /stats/:code`  — JSON stats for a code
//! - `DELETE /:code`      — remove a short link
//! - `GET  /`             — HTML dashboard listing the 10 most recent links

use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use thiserror::Error;
use tower_http::trace::TraceLayer;
use tracing::info;

// ────────────────────────────────────────────────────────────────────────────
// Domain errors
// ────────────────────────────────────────────────────────────────────────────

/// All recoverable errors that can occur within the URL-shortener service.
#[derive(Debug, Error)]
pub enum AppError {
    /// The requested short code was not found in the database.
    #[error("short code not found")]
    NotFound,

    /// The supplied URL is syntactically invalid or uses an unsupported scheme.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    /// A SQLite operation failed unexpectedly.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Map each variant to an appropriate HTTP status and a JSON body.
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::InvalidUrl(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal database error".to_string(),
            ),
        };

        let body = serde_json::json!({ "error": message });
        (status, Json(body)).into_response()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Shared application state
// ────────────────────────────────────────────────────────────────────────────

/// Application state shared across all Axum handler tasks.
#[derive(Clone)]
pub struct AppState {
    /// Connection pool to the SQLite database.
    pub pool: SqlitePool,
}

// ────────────────────────────────────────────────────────────────────────────
// Request / response types
// ────────────────────────────────────────────────────────────────────────────

/// JSON body for `POST /shorten`.
#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    /// The long URL to be shortened.
    pub url: String,
}

/// JSON response returned after successfully creating a short link.
#[derive(Debug, Serialize)]
pub struct ShortenResponse {
    /// The fully-qualified short URL (e.g. `http://localhost:3000/abc123`).
    pub short: String,
    /// The 6-character code component of the short URL.
    pub code: String,
}

/// One row from the `urls` table, returned from `GET /stats/:code`.
#[derive(Debug, Serialize)]
pub struct UrlStats {
    /// The unique 6-character code.
    pub code: String,
    /// The original (long) URL.
    pub original: String,
    /// ISO-8601 creation timestamp from SQLite.
    pub created_at: String,
    /// How many times the redirect endpoint was hit for this code.
    pub click_count: i64,
}

// ────────────────────────────────────────────────────────────────────────────
// Code generation
// ────────────────────────────────────────────────────────────────────────────

/// Generate a random 6-character alphanumeric code.
///
/// Uses `rand::thread_rng` seeded from the OS entropy source. The result is
/// safe to use as a URL path component without percent-encoding.
fn generate_code() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

// ────────────────────────────────────────────────────────────────────────────
// Handlers
// ────────────────────────────────────────────────────────────────────────────

/// `POST /shorten` — validate the URL, generate a unique code, persist it, and
/// return the short link.
pub async fn shorten(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ShortenRequest>,
) -> Result<(StatusCode, Json<ShortenResponse>), AppError> {
    // Only accept http and https URLs to avoid javascript: or data: injection.
    if !payload.url.starts_with("http://") && !payload.url.starts_with("https://") {
        return Err(AppError::InvalidUrl(
            "URL must begin with http:// or https://".to_string(),
        ));
    }

    // Retry up to 5 times in the unlikely event of a code collision.
    let code = 'retry: {
        for _ in 0..5 {
            let candidate = generate_code();
            // Attempt insert; UNIQUE constraint on `code` causes an error on collision.
            let result = sqlx::query(
                "INSERT INTO urls (code, original) VALUES (?, ?)",
            )
            .bind(&candidate)
            .bind(&payload.url)
            .execute(&state.pool)
            .await;

            match result {
                Ok(_) => break 'retry candidate,
                // Detect UNIQUE constraint violations via sqlx's portable ErrorKind.
                Err(sqlx::Error::Database(ref db_err))
                    if db_err.kind() == sqlx::error::ErrorKind::UniqueViolation =>
                {
                    // Code collision — try again.
                    continue;
                }
                Err(e) => return Err(AppError::Database(e)),
            }
        }
        // All 5 attempts collided (astronomically unlikely with a 62^6 space).
        return Err(AppError::InvalidUrl(
            "failed to generate a unique code; please try again".to_string(),
        ));
    };

    let short = format!("http://localhost:3000/{code}");
    Ok((StatusCode::CREATED, Json(ShortenResponse { short, code })))
}

/// `GET /:code` — look up the code, increment the click counter, and issue a
/// permanent 301 redirect to the original URL.
pub async fn redirect(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Response, AppError> {
    // Fetch the original URL in one query.
    let row = sqlx::query("SELECT original FROM urls WHERE code = ?")
        .bind(&code)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let original: String = row.try_get("original")?;

    // Fire-and-forget click increment; ignore errors so redirects are never blocked.
    let pool = state.pool.clone();
    let code_clone = code.clone();
    tokio::spawn(async move {
        let _ = sqlx::query("UPDATE urls SET click_count = click_count + 1 WHERE code = ?")
            .bind(&code_clone)
            .execute(&pool)
            .await;
    });

    // Build a 301 response with the Location header set.
    let mut headers = HeaderMap::new();
    headers.insert(
        header::LOCATION,
        original
            .parse()
            .map_err(|_| AppError::InvalidUrl("stored URL is not a valid header value".to_string()))?,
    );

    Ok((StatusCode::MOVED_PERMANENTLY, headers).into_response())
}

/// `GET /stats/:code` — return click-count and metadata for a given code.
pub async fn stats(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<UrlStats>, AppError> {
    let row = sqlx::query(
        "SELECT code, original, created_at, click_count FROM urls WHERE code = ?",
    )
    .bind(&code)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(UrlStats {
        code: row.try_get("code")?,
        original: row.try_get("original")?,
        created_at: row.try_get("created_at")?,
        click_count: row.try_get("click_count")?,
    }))
}

/// `DELETE /:code` — remove a short link; returns 204 on success, 404 if not found.
pub async fn remove(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM urls WHERE code = ?")
        .bind(&code)
        .execute(&state.pool)
        .await?;

    // `rows_affected` == 0 means the code was not in the database.
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// `GET /` — HTML dashboard showing the 10 most recently created short URLs.
pub async fn dashboard(
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, AppError> {
    let rows = sqlx::query(
        "SELECT code, original, created_at, click_count \
         FROM urls ORDER BY created_at DESC LIMIT 10",
    )
    .fetch_all(&state.pool)
    .await?;

    // Build an HTML table row for each URL record.
    let mut table_rows = String::new();
    for row in &rows {
        let code: String = row.try_get("code")?;
        let original: String = row.try_get("original")?;
        let created_at: String = row.try_get("created_at")?;
        let click_count: i64 = row.try_get("click_count")?;
        let short = format!("http://localhost:3000/{code}");

        table_rows.push_str(&format!(
            "<tr>\
               <td><a href=\"{short}\">{code}</a></td>\
               <td><a href=\"{original}\">{original}</a></td>\
               <td>{click_count}</td>\
               <td>{created_at}</td>\
             </tr>"
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>URL Shortener</title>
  <style>
    body {{ font-family: sans-serif; max-width: 900px; margin: 2rem auto; }}
    table {{ border-collapse: collapse; width: 100%; }}
    th, td {{ border: 1px solid #ccc; padding: 0.5rem; text-align: left; }}
    th {{ background: #f5f5f5; }}
  </style>
</head>
<body>
  <h1>URL Shortener — Recent Links</h1>
  <table>
    <thead>
      <tr><th>Code</th><th>Original URL</th><th>Clicks</th><th>Created</th></tr>
    </thead>
    <tbody>
      {table_rows}
    </tbody>
  </table>
</body>
</html>"#
    );

    Ok(Html(html))
}

// ────────────────────────────────────────────────────────────────────────────
// Database initialisation
// ────────────────────────────────────────────────────────────────────────────

/// Create the `urls` table if it does not already exist.
async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS urls (
            code        TEXT PRIMARY KEY,
            original    TEXT NOT NULL,
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            click_count INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await
    .context("failed to create urls table")?;

    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Entry point
// ────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise structured logging; RUST_LOG controls the filter level.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "url_shortener=debug,tower_http=debug".into()),
        )
        .init();

    // Open (or create) the SQLite database file.
    let pool = SqlitePool::connect("sqlite:urls.db?mode=rwc")
        .await
        .context("failed to open SQLite database")?;

    run_migrations(&pool).await?;

    let state = Arc::new(AppState { pool });

    // Note: route order matters in Axum — more specific paths must come before
    // wildcard paths to avoid shadowing. `/stats/:code` and `/` are registered
    // before `/:code` so they are matched first.
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/shorten", post(shorten))
        .route("/stats/:code", get(stats))
        .route("/:code", get(redirect).delete(remove))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .context("failed to bind TCP listener on port 3000")?;

    info!("URL shortener listening on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .context("server error")?;

    Ok(())
}
