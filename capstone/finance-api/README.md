# Finance API

A personal finance tracker REST API backed by SQLite, built with Axum and sqlx.
Tracks income and expense transactions with date filtering, monthly summaries,
and CSV export.

## Build & Run

```bash
cd capstone/finance-api
cargo run
```

The server starts on **http://localhost:3001**. SQLite creates `finance.db` in
the working directory on first run. Six built-in categories are seeded:
`salary`, `food`, `rent`, `utilities`, `entertainment`, `other`.

## API Endpoints

### Create a transaction

```bash
curl -X POST http://localhost:3001/transactions \
  -H 'Content-Type: application/json' \
  -d '{
    "date": "2024-01-15",
    "amount_cents": 350000,
    "description": "January salary",
    "category": "salary",
    "tx_type": "income"
  }'
```

**Response 201** — returns the full `Transaction` object with assigned `id`.

### List transactions

```bash
# All transactions
curl http://localhost:3001/transactions

# Filter by date range
curl "http://localhost:3001/transactions?from=2024-01-01&to=2024-01-31"

# Filter by category
curl "http://localhost:3001/transactions?category=food"

# Combine filters
curl "http://localhost:3001/transactions?from=2024-01-01&category=salary"
```

### Delete a transaction

```bash
curl -X DELETE http://localhost:3001/transactions/1
```

Returns **204 No Content** on success, **404** if the id does not exist.

### Monthly summary

```bash
curl "http://localhost:3001/summary?month=2024-01"
```

**Response 200**
```json
{
  "month": "2024-01",
  "total_income_cents": 350000,
  "total_expense_cents": 120000,
  "net_cents": 230000,
  "by_category": {
    "salary": 350000,
    "food": 45000,
    "rent": 75000
  }
}
```

### Export CSV

```bash
curl -o transactions.csv http://localhost:3001/transactions/export
```

Downloads `transactions.csv` with columns:
`id,date,amount_cents,description,category,type,created_at`

### List categories

```bash
curl http://localhost:3001/categories
```

### Add a category

```bash
curl -X POST http://localhost:3001/categories \
  -H 'Content-Type: application/json' \
  -d '{"name": "travel"}'
```

Returns **201 Created** with `{"name": "travel"}`.
