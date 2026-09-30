//! Persistent key-value store with an append-only write-ahead log and a TCP server.
//!
//! # Architecture
//! - **Storage engine** (`KvStore`): pure `std`, append-only JSON-lines log with an
//!   in-memory index mapping each key to the byte offset of its latest log entry.
//! - **TCP server**: async Tokio server; each connection runs in its own task and
//!   shares the store via `Arc<tokio::sync::Mutex<KvStore>>`.
//!
//! # Wire Protocol (one command per line, UTF-8)
//! ```text
//! SET key value  →  OK  or  ERR <message>
//! GET key        →  VALUE <value>  or  NIL  or  ERR <message>
//! DEL key        →  OK  or  NIL (key absent)
//! LIST prefix    →  <key>\n … END
//! COMPACT        →  OK
//! QUIT           →  BYE  (server closes connection)
//! ```
//!
//! # Usage
//! ```text
//! kvdb serve [--addr 127.0.0.1] [--port 6379] [--data ./kvdb.log]
//! ```

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

// ── Error type ────────────────────────────────────────────────────────────────

/// Domain errors for the key-value store engine.
#[derive(Debug, Error)]
pub enum KvError {
    /// An underlying I/O error (file open, read, write, seek).
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// A JSON serialisation or deserialisation failure.
    #[error("Serialisation error: {0}")]
    Serialize(#[from] serde_json::Error),
    /// The requested key does not exist in the store.
    #[error("Key not found")]
    KeyNotFound,
    /// A log entry could not be deserialised; the log may be corrupted.
    #[error("Corrupt log: {0}")]
    CorruptLog(String),
}

// ── Log entry ─────────────────────────────────────────────────────────────────

/// A single record appended to the write-ahead log file.
///
/// Each entry is serialised as a JSON object on its own line (`\n`-terminated).
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op")]
enum LogEntry {
    /// A set (insert or update) operation.
    Set {
        /// The key being set.
        key: String,
        /// The associated value.
        value: String,
    },
    /// A delete operation.
    Delete {
        /// The key being removed.
        key: String,
    },
}

// ── Storage engine ────────────────────────────────────────────────────────────

/// An append-only key-value store backed by a JSON-lines log file.
///
/// On startup, all log entries are replayed in order to rebuild an in-memory
/// index.  Reads seek directly to the relevant log offset; writes append a new
/// entry and update the index.
pub struct KvStore {
    /// Canonical path to the log file (used for seeking and compaction).
    log_path: PathBuf,
    /// Maps each live key to the byte offset of its latest `Set` log entry.
    index: HashMap<String, u64>,
    /// Append-only writer; flushed after every mutating operation.
    log: BufWriter<File>,
    /// Total bytes written so far; used to compute offsets for new entries.
    log_size: u64,
}

impl KvStore {
    /// Open (or create) the log file at `path` and replay it to rebuild the index.
    ///
    /// Returns a ready-to-use `KvStore` with the in-memory index fully populated.
    pub fn open(path: &Path) -> Result<Self, KvError> {
        // Create the file if it does not yet exist, then close this handle.
        OpenOptions::new().create(true).append(true).open(path)?;

        // Replay existing entries to build the in-memory index.
        // We use a separate read-only handle so the append handle (opened below)
        // is never used for seeking, keeping it clean for future writes.
        let mut index: HashMap<String, u64> = HashMap::new();
        let mut log_size: u64 = 0;
        {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            for line_result in reader.lines() {
                let line = line_result?;
                if line.is_empty() {
                    // Empty lines are unlikely but not invalid; count the newline byte.
                    log_size += 1;
                    continue;
                }
                // writeln! always emits \n, so each entry occupies line.len() + 1 bytes.
                let entry_offset = log_size;
                let entry: LogEntry = serde_json::from_str(&line).map_err(|e| {
                    KvError::CorruptLog(format!("at byte offset {}: {}", entry_offset, e))
                })?;
                match entry {
                    LogEntry::Set { key, .. } => {
                        index.insert(key, entry_offset);
                    }
                    LogEntry::Delete { key } => {
                        index.remove(&key);
                    }
                }
                log_size += line.len() as u64 + 1;
            }
        }

        // Open a dedicated append handle for future writes.
        let file = OpenOptions::new().append(true).open(path)?;
        let log = BufWriter::new(file);

        Ok(KvStore { log_path: path.to_path_buf(), index, log, log_size })
    }

    /// Insert or overwrite `key` with `value`.
    ///
    /// Appends a `Set` entry to the log and updates the in-memory index.
    pub fn set(&mut self, key: String, value: String) -> Result<(), KvError> {
        let entry = LogEntry::Set { key: key.clone(), value };
        let json = serde_json::to_string(&entry)?;
        let offset = self.log_size;
        // Write entry as a single JSON line; writeln! guarantees a trailing \n.
        writeln!(self.log, "{}", json)?;
        self.log.flush()?;
        self.log_size += json.len() as u64 + 1;
        self.index.insert(key, offset);
        Ok(())
    }

    /// Retrieve the value for `key`, or `None` if the key does not exist.
    ///
    /// Seeks to the stored offset in the log file and deserialises the entry.
    pub fn get(&self, key: &str) -> Result<Option<String>, KvError> {
        let &offset = match self.index.get(key) {
            Some(o) => o,
            None => return Ok(None),
        };
        // Open a fresh read handle so we can seek without disturbing the write handle.
        let mut file = File::open(&self.log_path)?;
        file.seek(SeekFrom::Start(offset))?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let entry: LogEntry = serde_json::from_str(line.trim()).map_err(|e| {
            KvError::CorruptLog(format!("at byte offset {}: {}", offset, e))
        })?;
        match entry {
            LogEntry::Set { value, .. } => Ok(Some(value)),
            // A Delete entry at a Set offset would indicate log corruption.
            LogEntry::Delete { key: k } => Err(KvError::CorruptLog(format!(
                "expected Set at offset {} for key '{}', found Delete of '{}'",
                offset, key, k
            ))),
        }
    }

    /// Look up `key` and return its value; returns `KeyNotFound` if absent.
    ///
    /// Convenience wrapper around [`get`] for callers that require the key to exist.
    pub fn get_required(&self, key: &str) -> Result<String, KvError> {
        self.get(key)?.ok_or(KvError::KeyNotFound)
    }

    /// Delete `key` from the store.
    ///
    /// Returns `true` if the key existed (and was deleted), `false` otherwise.
    pub fn delete(&mut self, key: &str) -> Result<bool, KvError> {
        if !self.index.contains_key(key) {
            return Ok(false);
        }
        let entry = LogEntry::Delete { key: key.to_string() };
        let json = serde_json::to_string(&entry)?;
        writeln!(self.log, "{}", json)?;
        self.log.flush()?;
        self.log_size += json.len() as u64 + 1;
        self.index.remove(key);
        Ok(true)
    }

    /// Return all keys whose names start with `prefix`, sorted lexicographically.
    pub fn list(&self, prefix: &str) -> Vec<String> {
        let mut keys: Vec<String> = self
            .index
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        keys.sort();
        keys
    }

    /// Compact the log by rewriting it with only the latest value for each live key.
    ///
    /// Uses a write-to-temp-then-rename approach.  On Windows, the old log handle
    /// is closed before the rename (via `mem::replace`) since the OS does not allow
    /// renaming over an open file in the default sharing mode.
    pub fn compact(&mut self) -> Result<(), KvError> {
        // Flush any buffered writes before reading the log.
        self.log.flush()?;

        // Collect all live key-value pairs by reading from the current log.
        let keys: Vec<String> = self.index.keys().cloned().collect();
        let mut live_pairs: Vec<(String, String)> = Vec::with_capacity(keys.len());
        for key in &keys {
            if let Some(value) = self.get(key)? {
                live_pairs.push((key.clone(), value));
            }
        }

        // Build the compacted content in memory; simultaneously build the new index.
        let mut buf: Vec<u8> = Vec::new();
        let mut new_index: HashMap<String, u64> = HashMap::with_capacity(live_pairs.len());
        let mut new_offset: u64 = 0;
        for (key, value) in &live_pairs {
            let entry = LogEntry::Set { key: key.clone(), value: value.clone() };
            let json = serde_json::to_string(&entry)?;
            new_index.insert(key.clone(), new_offset);
            let line = format!("{}\n", json);
            new_offset += line.len() as u64;
            buf.extend_from_slice(line.as_bytes());
        }

        // Write the compacted content to a temporary file alongside the log.
        let tmp_path = self.log_path.with_extension("log.tmp");
        fs::write(&tmp_path, &buf)?;

        // Close the old log file handle.  We replace self.log with a handle to the
        // temp file (which has already been fully written), then drop the old handle.
        // This ensures the original log_path is not held open when we rename on Windows.
        {
            let tmp_append = OpenOptions::new().append(true).open(&tmp_path)?;
            let old_log = mem::replace(&mut self.log, BufWriter::new(tmp_append));
            drop(old_log); // Closes the old file handle to log_path
        }

        // On Windows, rename fails if the destination exists; remove it first.
        if self.log_path.exists() {
            fs::remove_file(&self.log_path)?;
        }
        fs::rename(&tmp_path, &self.log_path)?;

        // Reopen the log at its canonical path for subsequent appends.
        let final_file = OpenOptions::new().append(true).open(&self.log_path)?;
        self.log = BufWriter::new(final_file);

        self.index = new_index;
        self.log_size = new_offset;
        Ok(())
    }
}

// ── TCP server ────────────────────────────────────────────────────────────────

/// Accept connections on `listener`, spawning a task for each one.
async fn serve_loop(listener: TcpListener, store: Arc<Mutex<KvStore>>) {
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                debug!("accepted connection from {}", peer);
                let store = Arc::clone(&store);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, store).await {
                        error!("connection error: {}", e);
                    }
                });
            }
            Err(e) => error!("accept error: {}", e),
        }
    }
}

/// Drive a single client connection: read lines, dispatch commands, write responses.
async fn handle_connection(stream: TcpStream, store: Arc<Mutex<KvStore>>) -> Result<()> {
    let (read_half, write_half) = stream.into_split();
    let mut reader = tokio::io::BufReader::new(read_half);
    let mut writer = tokio::io::BufWriter::new(write_half);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await.context("reading from client")?;
        if n == 0 {
            break; // EOF — client disconnected gracefully
        }

        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

        let quit = dispatch(&trimmed, &store, &mut writer).await?;
        writer.flush().await.context("flushing response")?;
        if quit {
            break;
        }
    }
    Ok(())
}

/// Parse and execute one command, writing the response to `writer`.
///
/// Returns `true` if the connection should be closed (`QUIT` command).
///
/// # Blocking note
/// The underlying `KvStore` operations (especially `get` which seeks a file) are
/// blocking.  In a production system these would be wrapped in
/// `tokio::task::block_in_place`; for this learning project we accept the tradeoff.
async fn dispatch(
    cmd: &str,
    store: &Arc<Mutex<KvStore>>,
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::OwnedWriteHalf>,
) -> Result<bool> {
    // Split the first whitespace-delimited token as the command verb.
    let (verb, rest) = cmd.split_once(' ').unwrap_or((cmd, ""));

    match verb.to_uppercase().as_str() {
        "SET" => {
            // SET <key> <value>  — value may contain spaces
            match rest.split_once(' ') {
                Some((key, value)) => {
                    let res = {
                        let mut guard = store.lock().await;
                        guard.set(key.to_string(), value.to_string())
                    };
                    match res {
                        Ok(()) => writer.write_all(b"OK\n").await?,
                        Err(e) => writer.write_all(format!("ERR {}\n", e).as_bytes()).await?,
                    }
                }
                None => writer.write_all(b"ERR SET requires key and value\n").await?,
            }
            Ok(false)
        }

        "GET" => {
            let key = rest.trim();
            let res = {
                let guard = store.lock().await;
                guard.get(key)
            };
            match res {
                Ok(Some(v)) => writer.write_all(format!("VALUE {}\n", v).as_bytes()).await?,
                Ok(None) => writer.write_all(b"NIL\n").await?,
                Err(e) => writer.write_all(format!("ERR {}\n", e).as_bytes()).await?,
            }
            Ok(false)
        }

        "DEL" => {
            let key = rest.trim();
            let res = {
                let mut guard = store.lock().await;
                guard.delete(key)
            };
            match res {
                Ok(true) => writer.write_all(b"OK\n").await?,
                Ok(false) => writer.write_all(b"NIL\n").await?,
                Err(e) => writer.write_all(format!("ERR {}\n", e).as_bytes()).await?,
            }
            Ok(false)
        }

        "LIST" => {
            let prefix = rest.trim();
            let keys = {
                let guard = store.lock().await;
                guard.list(prefix)
            };
            for k in &keys {
                writer.write_all(format!("{}\n", k).as_bytes()).await?;
            }
            writer.write_all(b"END\n").await?;
            Ok(false)
        }

        "COMPACT" => {
            let res = {
                let mut guard = store.lock().await;
                guard.compact()
            };
            match res {
                Ok(()) => writer.write_all(b"OK\n").await?,
                Err(e) => writer.write_all(format!("ERR {}\n", e).as_bytes()).await?,
            }
            Ok(false)
        }

        "QUIT" => {
            writer.write_all(b"BYE\n").await?;
            Ok(true) // Signal caller to close the connection
        }

        _ => {
            writer.write_all(format!("ERR unknown command '{}'\n", verb).as_bytes()).await?;
            Ok(false)
        }
    }
}

// ── CLI ───────────────────────────────────────────────────────────────────────

/// A persistent key-value store server with an append-only log.
#[derive(Debug, Parser)]
#[command(name = "kvdb", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Start the TCP key-value server.
    Serve(ServeArgs),
}

/// Arguments for the `serve` subcommand.
#[derive(Debug, clap::Args)]
struct ServeArgs {
    /// IP address to bind the TCP listener to.
    #[arg(long, default_value = "127.0.0.1")]
    addr: String,
    /// TCP port to listen on.
    #[arg(long, default_value_t = 6379)]
    port: u16,
    /// Path to the persistent log file.
    #[arg(long, default_value = "kvdb.log")]
    data: PathBuf,
}

// ── Entry point ────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // Initialise structured logging; control verbosity with RUST_LOG=kvdb=debug.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Serve(args) => {
            let addr: std::net::SocketAddr = format!("{}:{}", args.addr, args.port)
                .parse()
                .with_context(|| format!("parsing address '{}:{}'", args.addr, args.port))?;

            let store = KvStore::open(&args.data)
                .with_context(|| format!("opening store at '{}'", args.data.display()))?;
            let store = Arc::new(Mutex::new(store));

            let listener = TcpListener::bind(addr)
                .await
                .with_context(|| format!("binding to {}", addr))?;
            info!("kvdb listening on {}", addr);

            serve_loop(listener, store).await;
        }
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a temporary log path unique to the calling test.
    fn tmp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("kvdb_test_{}.log", name));
        // Remove leftovers from a previous run so tests start clean.
        let _ = fs::remove_file(&p);
        let _ = fs::remove_file(p.with_extension("log.tmp"));
        p
    }

    /// Open a fresh store at a temporary path.
    fn open_store(name: &str) -> (KvStore, PathBuf) {
        let path = tmp_path(name);
        let store = KvStore::open(&path).expect("open KvStore");
        (store, path)
    }

    /// `set` followed by `get` returns the stored value.
    #[test]
    fn test_set_and_get() {
        let (mut store, _path) = open_store("set_get");
        store.set("hello".into(), "world".into()).expect("set");
        let val = store.get("hello").expect("get");
        assert_eq!(val, Some("world".to_string()));
    }

    /// `get` on a missing key returns `None`.
    #[test]
    fn test_get_missing() {
        let (store, _path) = open_store("get_missing");
        let val = store.get("nonexistent").expect("get");
        assert_eq!(val, None);
    }

    /// `set` twice for the same key returns the latest value.
    #[test]
    fn test_overwrite() {
        let (mut store, _path) = open_store("overwrite");
        store.set("k".into(), "v1".into()).expect("set 1");
        store.set("k".into(), "v2".into()).expect("set 2");
        let val = store.get("k").expect("get");
        assert_eq!(val, Some("v2".to_string()));
    }

    /// `delete` removes the key; subsequent `get` returns `None`.
    #[test]
    fn test_delete() {
        let (mut store, _path) = open_store("delete");
        store.set("foo".into(), "bar".into()).expect("set");
        let existed = store.delete("foo").expect("delete");
        assert!(existed, "delete should report key existed");
        let val = store.get("foo").expect("get after delete");
        assert_eq!(val, None);
    }

    /// `delete` on a missing key returns `false`.
    #[test]
    fn test_delete_missing() {
        let (mut store, _path) = open_store("delete_missing");
        let existed = store.delete("ghost").expect("delete missing");
        assert!(!existed);
    }

    /// After reopening the store the index is rebuilt correctly from the log.
    #[test]
    fn test_persistence() {
        let path = tmp_path("persistence");
        {
            let mut store = KvStore::open(&path).expect("open 1");
            store.set("a".into(), "1".into()).expect("set a");
            store.set("b".into(), "2".into()).expect("set b");
            store.delete("a").expect("delete a");
        }
        // Reopen — index must be rebuilt from the log
        let store = KvStore::open(&path).expect("open 2");
        assert_eq!(store.get("a").expect("get a"), None);
        assert_eq!(store.get("b").expect("get b"), Some("2".to_string()));
    }

    /// `compact` reduces the log to only live entries; values remain accessible.
    #[test]
    fn test_compact() {
        let (mut store, path) = open_store("compact");
        store.set("x".into(), "1".into()).expect("set x=1");
        store.set("x".into(), "2".into()).expect("set x=2 (overwrites)");
        store.set("y".into(), "hello".into()).expect("set y");
        store.set("z".into(), "delete me".into()).expect("set z");
        store.delete("z").expect("delete z");

        let before_size = fs::metadata(&path).expect("stat before").len();
        store.compact().expect("compact");
        let after_size = fs::metadata(&path).expect("stat after").len();

        // The compacted log must be smaller (fewer entries).
        assert!(
            after_size < before_size,
            "compact should shrink the log ({} → {})",
            before_size,
            after_size
        );

        // Values must be correct after compaction.
        assert_eq!(store.get("x").expect("get x"), Some("2".to_string()));
        assert_eq!(store.get("y").expect("get y"), Some("hello".to_string()));
        assert_eq!(store.get("z").expect("get z"), None);
    }

    /// `list` returns all keys matching the prefix, sorted.
    #[test]
    fn test_list_prefix() {
        let (mut store, _path) = open_store("list_prefix");
        store.set("user:1".into(), "Alice".into()).expect("set user:1");
        store.set("user:2".into(), "Bob".into()).expect("set user:2");
        store.set("config:theme".into(), "dark".into()).expect("set config");
        let users = store.list("user:");
        assert_eq!(users, vec!["user:1", "user:2"]);
        let all = store.list("");
        assert_eq!(all.len(), 3);
    }
}
