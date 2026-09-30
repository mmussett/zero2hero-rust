# URL Shortener

A self-hosted URL shortener REST API backed by SQLite, built with Axum and sqlx.

## Build & Run

```bash
cd capstone/url-shortener
cargo run
```

The server starts on **http://localhost:3000**. SQLite creates `urls.db` in the
working directory on first run.

Set `RUST_LOG=debug` for verbose tracing output.

## API Endpoints

### Create a short link

```bash
curl -X POST http://localhost:3000/shorten \
  -H 'Content-Type: application/json' \
  -d '{"url": "https://www.rust-lang.org/learn"}'
```

**Response 201**
```json
{ "short": "http://localhost:3000/aB3xYz", "code": "aB3xYz" }
```

### Redirect (follow the short link)

```bash
curl -L http://localhost:3000/aB3xYz
```

Returns HTTP 301 → original URL. The click counter is incremented on each visit.

### View statistics

```bash
curl http://localhost:3000/stats/aB3xYz
```

**Response 200**
```json
{
  "code": "aB3xYz",
  "original": "https://www.rust-lang.org/learn",
  "created_at": "2024-01-15 10:30:00",
  "click_count": 3
}
```

### Delete a short link

```bash
curl -X DELETE http://localhost:3000/aB3xYz
```

Returns **204 No Content** on success, **404** if the code does not exist.

### HTML Dashboard

Open <http://localhost:3000/> in a browser to see the 10 most recently created
short links in a simple table.
