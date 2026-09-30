//! Finance API — a personal finance tracker REST API backed by SQLite.
//!
//! Tracks income and expense transactions with date, amount (stored as integer
//! cents), description, and category. Provides monthly summaries and CSV export.
//!
//! # Endpoints
//! - `POST   /transactions`           — record a new transaction
//! - `GET    /transactions`           — list transactions (filterable)
//! - `DELETE /transactions/:id`       — remove a transaction
//! - `GET    /summary?month=YYYY-MM`  — monthly aggregate totals
//! - `GET    /transactions/export`    — download as CSV
//! - `GET    /categories`             — list available categories
//! - `POST   /categories`             — add a new category

use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use thiserror::Error;
use tower_http::trace::TraceLayer;
use tracing::info;

// ────────────────────────────────────────────────────────────────────────────
// Domain errors
// ────────────────────────────────────────────────────────────────────────────

/// All recoverable errors within the finance API.
#[derive(Debug, Error)]
pub enum ApiError {
    /// The requested resource was not found.
    #[error("not found")]
    NotFound,

    /// The request body or query parameters contain invalid data.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// A SQLite operation failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// An error occurred while building the CSV response.
    #[error("CSV error: {0}")]
    Csv(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::InvalidInput(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            ApiError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error".to_string()),
            ApiError::Csv(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = serde_json::json!({ "error": message });
        (status, Json(body)).into_response()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Shared state
// ────────────────────────────────────────────────────────────────────────────

/// Application state shared across all handler tasks.
#[derive(Clone)]
pub struct AppState {
    /// SQLite connection pool.
    pub pool: SqlitePool,
}

// ────────────────────────────────────────────────────────────────────────────
// Domain types
// ────────────────────────────────────────────────────────────────────────────

/// Discriminates between income and expense transactions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    /// Money received (salary, freelance, etc.).
    Income,
    /// Money spent (food, rent, utilities, etc.).
    Expense,
}

impl TransactionType {
    /// Convert to the string stored in SQLite.
    fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Income => "income",
            TransactionType::Expense => "expense",
        }
    }

    /// Parse from a string stored in SQLite.
    fn from_str(s: &str) -> Result<Self, ApiError> {
        match s {
            "income" => Ok(TransactionType::Income),
            "expense" => Ok(TransactionType::Expense),
            other => Err(ApiError::InvalidInput(format!(
                "unknown tx_type '{other}'; expected 'income' or 'expense'"
            ))),
        }
    }
}

/// A persisted financial transaction.
#[derive(Debug, Serialize)]
pub struct Transaction {
    /// Auto-assigned primary key.
    pub id: i64,
    /// Transaction date in `YYYY-MM-DD` format.
    pub date: String,
    /// Amount in integer cents (always positive; sign is implicit in `tx_type`).
    pub amount_cents: i64,
    /// Human-readable description.
    pub description: String,
    /// Category name (must exist in the `categories` table).
    pub category: String,
    /// Whether this is income or an expense.
    pub tx_type: TransactionType,
    /// ISO-8601 creation timestamp from SQLite.
    pub created_at: String,
}

/// Request body for `POST /transactions`.
#[derive(Debug, Deserialize)]
pub struct NewTransaction {
    /// Transaction date in `YYYY-MM-DD` format.
    pub date: String,
    /// Amount in integer cents (must be > 0).
    pub amount_cents: i64,
    /// Human-readable description.
    pub description: String,
    /// Category name.
    pub category: String,
    /// Income or expense.
    pub tx_type: TransactionType,
}

/// Monthly aggregate summary returned by `GET /summary`.
#[derive(Debug, Serialize)]
pub struct Summary {
    /// The month queried, in `YYYY-MM` format.
    pub month: String,
    /// Sum of all income in cents for the month.
    pub total_income_cents: i64,
    /// Sum of all expenses in cents for the month.
    pub total_expense_cents: i64,
    /// Net value: income minus expenses, in cents.
    pub net_cents: i64,
    /// Per-category totals (absolute cents, sign not applied).
    pub by_category: HashMap<String, i64>,
}

// ────────────────────────────────────────────────────────────────────────────
// Query-string filter structs
// ────────────────────────────────────────────────────────────────────────────

/// Optional query parameters for `GET /transactions`.
#[derive(Debug, Deserialize, Default)]
pub struct ListParams {
    /// Lower bound date filter (inclusive), format `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Upper bound date filter (inclusive), format `YYYY-MM-DD`.
    pub to: Option<String>,
    /// Filter to a specific category.
    pub category: Option<String>,
}

/// Query parameter for `GET /summary`.
#[derive(Debug, Deserialize)]
pub struct SummaryParams {
    /// The month to summarise, in `YYYY-MM` format.
    pub month: String,
}

// ────────────────────────────────────────────────────────────────────────────
// Request body for categories
// ────────────────────────────────────────────────────────────────────────────

/// Request body for `POST /categories`.
#[derive(Debug, Deserialize)]
pub struct NewCategory {
    /// The category name to create.
    pub name: String,
}

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

/// Map a sqlx Row to a `Transaction`, surfacing column errors as `ApiError`.
fn row_to_transaction(row: &sqlx::sqlite::SqliteRow) -> Result<Transaction, ApiError> {
    let tx_type_str: String = row.try_get("tx_type")?;
    Ok(Transaction {
        id: row.try_get("id")?,
        date: row.try_get("date")?,
        amount_cents: row.try_get("amount_cents")?,
        description: row.try_get("description")?,
        category: row.try_get("category")?,
        tx_type: TransactionType::from_str(&tx_type_str)?,
        created_at: row.try_get("created_at")?,
    })
}

// ────────────────────────────────────────────────────────────────────────────
// Handlers
// ────────────────────────────────────────────────────────────────────────────

/// `POST /transactions` — create a new transaction and return it with HTTP 201.
pub async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewTransaction>,
) -> Result<(StatusCode, Json<Transaction>), ApiError> {
    // Basic validation — amounts must be positive.
    if payload.amount_cents <= 0 {
        return Err(ApiError::InvalidInput(
            "amount_cents must be a positive integer".to_string(),
        ));
    }

    // Validate date format loosely (YYYY-MM-DD) by checking length and hyphens.
    if !is_valid_date(&payload.date) {
        return Err(ApiError::InvalidInput(
            "date must be in YYYY-MM-DD format".to_string(),
        ));
    }

    let row = sqlx::query(
        "INSERT INTO transactions (date, amount_cents, description, category, tx_type)
         VALUES (?, ?, ?, ?, ?)
         RETURNING id, date, amount_cents, description, category, tx_type, created_at",
    )
    .bind(&payload.date)
    .bind(payload.amount_cents)
    .bind(&payload.description)
    .bind(&payload.category)
    .bind(payload.tx_type.as_str())
    .fetch_one(&state.pool)
    .await?;

    let tx = row_to_transaction(&row)?;
    Ok((StatusCode::CREATED, Json(tx)))
}

/// `GET /transactions` — list transactions with optional date and category filters.
pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Transaction>>, ApiError> {
    // Build the WHERE clause dynamically based on which filters were supplied.
    // All parameters are bound to avoid SQL injection.
    let mut conditions: Vec<&'static str> = Vec::new();
    if params.from.is_some() {
        conditions.push("date >= ?");
    }
    if params.to.is_some() {
        conditions.push("date <= ?");
    }
    if params.category.is_some() {
        conditions.push("category = ?");
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT id, date, amount_cents, description, category, tx_type, created_at \
         FROM transactions {where_clause} ORDER BY date DESC, id DESC"
    );

    // Bind parameters in the same order they appear in `conditions`.
    let mut query = sqlx::query(&sql);
    if let Some(ref from) = params.from {
        query = query.bind(from);
    }
    if let Some(ref to) = params.to {
        query = query.bind(to);
    }
    if let Some(ref cat) = params.category {
        query = query.bind(cat);
    }

    let rows = query.fetch_all(&state.pool).await?;

    let transactions: Result<Vec<Transaction>, ApiError> =
        rows.iter().map(row_to_transaction).collect();

    Ok(Json(transactions?))
}

/// `DELETE /transactions/:id` — remove a transaction by id; 404 if missing.
pub async fn delete_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM transactions WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// `GET /summary?month=YYYY-MM` — aggregate totals for a given calendar month.
pub async fn monthly_summary(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SummaryParams>,
) -> Result<Json<Summary>, ApiError> {
    // Validate month format.
    if params.month.len() != 7 || params.month.chars().nth(4) != Some('-') {
        return Err(ApiError::InvalidInput(
            "month must be in YYYY-MM format".to_string(),
        ));
    }

    // Use SQLite's substr to match on the YYYY-MM prefix of the `date` column.
    let prefix = format!("{}%", params.month);

    // Fetch all transactions in the month so we can compute both totals and
    // the per-category breakdown in one pass.
    let rows = sqlx::query(
        "SELECT amount_cents, tx_type, category FROM transactions WHERE date LIKE ?",
    )
    .bind(&prefix)
    .fetch_all(&state.pool)
    .await?;

    let mut total_income: i64 = 0;
    let mut total_expense: i64 = 0;
    let mut by_category: HashMap<String, i64> = HashMap::new();

    for row in &rows {
        let amount: i64 = row.try_get("amount_cents")?;
        let tx_type_str: String = row.try_get("tx_type")?;
        let category: String = row.try_get("category")?;

        match tx_type_str.as_str() {
            "income" => total_income += amount,
            "expense" => total_expense += amount,
            _ => {} // Guarded by DB CHECK constraint; ignore unknown values here.
        }

        // Accumulate absolute amounts per category.
        *by_category.entry(category).or_insert(0) += amount;
    }

    Ok(Json(Summary {
        month: params.month.clone(),
        total_income_cents: total_income,
        total_expense_cents: total_expense,
        net_cents: total_income - total_expense,
        by_category,
    }))
}

/// `GET /transactions/export` — stream all transactions as a CSV file download.
pub async fn export_csv(
    State(state): State<Arc<AppState>>,
) -> Result<Response, ApiError> {
    let rows = sqlx::query(
        "SELECT id, date, amount_cents, description, category, tx_type, created_at \
         FROM transactions ORDER BY date DESC, id DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    // Build CSV in memory — no external crate needed for this volume.
    let mut csv = String::from("id,date,amount_cents,description,category,type,created_at\n");

    for row in &rows {
        let id: i64 = row.try_get("id")?;
        let date: String = row.try_get("date")?;
        let amount: i64 = row.try_get("amount_cents")?;
        let description: String = row.try_get("description")?;
        let category: String = row.try_get("category")?;
        let tx_type: String = row.try_get("tx_type")?;
        let created_at: String = row.try_get("created_at")?;

        // Escape description and category fields that may contain commas or quotes.
        let description_escaped = csv_escape(&description);
        let category_escaped = csv_escape(&category);

        csv.push_str(&format!(
            "{id},{date},{amount},{description_escaped},{category_escaped},{tx_type},{created_at}\n"
        ));
    }

    // Construct the response with correct Content-Type and Content-Disposition.
    let response = (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"transactions.csv\"",
            ),
        ],
        csv,
    )
        .into_response();

    Ok(response)
}

/// Escape a CSV field by wrapping it in double-quotes if it contains commas,
/// double-quotes, or newlines, doubling any embedded double-quotes.
fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// `GET /categories` — return the list of all category names.
pub async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<String>>, ApiError> {
    let rows = sqlx::query("SELECT name FROM categories ORDER BY name")
        .fetch_all(&state.pool)
        .await?;

    let names: Result<Vec<String>, sqlx::Error> =
        rows.iter().map(|r| r.try_get("name")).collect();

    Ok(Json(names?))
}

/// `POST /categories` — add a new category; returns 201 with the name on success.
pub async fn create_category(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewCategory>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if payload.name.trim().is_empty() {
        return Err(ApiError::InvalidInput("category name must not be empty".to_string()));
    }

    sqlx::query("INSERT INTO categories (name) VALUES (?)")
        .bind(payload.name.trim())
        .execute(&state.pool)
        .await
        .map_err(|e| {
            // Distinguish UNIQUE constraint violations from other DB errors.
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.kind() == sqlx::error::ErrorKind::UniqueViolation {
                    return ApiError::InvalidInput(format!(
                        "category '{}' already exists",
                        payload.name
                    ));
                }
            }
            ApiError::Database(e)
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "name": payload.name.trim() })),
    ))
}

// ────────────────────────────────────────────────────────────────────────────
// Validation helpers
// ────────────────────────────────────────────────────────────────────────────

/// Return `true` if `s` looks like a valid `YYYY-MM-DD` date string.
///
/// This is a lightweight format check, not a full calendar validation.
fn is_valid_date(s: &str) -> bool {
    // Must be exactly 10 chars: 4 digits, hyphen, 2 digits, hyphen, 2 digits.
    let bytes = s.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

// ────────────────────────────────────────────────────────────────────────────
// Database initialisation
// ────────────────────────────────────────────────────────────────────────────

/// Create tables and seed built-in categories if they don't exist.
async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transactions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            date         TEXT NOT NULL,
            amount_cents INTEGER NOT NULL,
            description  TEXT NOT NULL,
            category     TEXT NOT NULL,
            tx_type      TEXT NOT NULL CHECK(tx_type IN ('income','expense')),
            created_at   TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await
    .context("failed to create transactions table")?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS categories (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )",
    )
    .execute(pool)
    .await
    .context("failed to create categories table")?;

    // Seed built-in categories; ignore conflicts so repeated restarts are safe.
    for cat in &["salary", "food", "rent", "utilities", "entertainment", "other"] {
        sqlx::query("INSERT OR IGNORE INTO categories (name) VALUES (?)")
            .bind(cat)
            .execute(pool)
            .await
            .context("failed to seed category")?;
    }

    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Entry point
// ────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "finance_api=debug,tower_http=debug".into()),
        )
        .init();

    let pool = SqlitePool::connect("sqlite:finance.db?mode=rwc")
        .await
        .context("failed to open SQLite database")?;

    run_migrations(&pool).await?;

    let state = Arc::new(AppState { pool });

    // Note: `/transactions/export` must be registered before `/:id` so that
    // Axum does not try to parse "export" as an integer id.
    let app = Router::new()
        .route("/transactions/export", get(export_csv))
        .route("/transactions", post(create_transaction).get(list_transactions))
        .route("/transactions/:id", delete(delete_transaction))
        .route("/summary", get(monthly_summary))
        .route("/categories", get(list_categories).post(create_category))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .context("failed to bind TCP listener on port 3001")?;

    info!("Finance API listening on http://0.0.0.0:3001");

    axum::serve(listener, app)
        .await
        .context("server error")?;

    Ok(())
}
