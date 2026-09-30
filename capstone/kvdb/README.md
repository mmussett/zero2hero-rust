# kvdb — Persistent Key-Value Store

A TCP key-value server backed by an append-only write-ahead log.  Every write
is durable: the log is flushed after each `SET` or `DEL`.  The in-memory index
maps each live key to the byte offset of its latest log entry, so reads are a
single seek + deserialise.

## Build

```bash
cargo build -p kvdb
# or run directly:
cargo run -p kvdb -- serve
```

## Starting the server

```bash
# Default: bind 127.0.0.1:6379, log file kvdb.log
cargo run -p kvdb -- serve

# Custom options
cargo run -p kvdb -- serve --addr 0.0.0.0 --port 7000 --data /var/lib/kvdb.log
```

Environment variable `RUST_LOG=kvdb=debug` enables verbose connection logging.

## Wire protocol

One UTF-8 command per line (`\n`-terminated).  The server replies on the same
TCP connection.

| Command | Response |
|---------|----------|
| `SET key value` | `OK` or `ERR <message>` |
| `GET key` | `VALUE <value>`, `NIL`, or `ERR <message>` |
| `DEL key` | `OK` (deleted) or `NIL` (key absent) |
| `LIST prefix` | One key per line, then `END` |
| `COMPACT` | `OK` |
| `QUIT` | `BYE` (server closes the connection) |

The `value` in `SET key value` may contain spaces; everything after the first
space following the key is treated as the value.

## Example session (netcat)

```text
$ nc 127.0.0.1 6379
SET greeting hello world
OK
GET greeting
VALUE hello world
SET counter 0
OK
SET counter 1
OK
GET counter
VALUE 1
LIST counter
counter
END
DEL greeting
OK
GET greeting
NIL
COMPACT
OK
QUIT
BYE
```

## Compaction

As the log grows, old superseded entries accumulate.  `COMPACT` rewrites the
log retaining only the latest value for each live key, then atomically replaces
the old log.  This is safe to run while the server is live.

## Running tests

```bash
cargo test -p kvdb
```

Tests exercise `set`, `get`, `delete`, `list`, persistence across reopen, and
compaction correctness.  Each test uses an isolated temporary log file.
