//! Chat Server — a real-time multi-room WebSocket chat server with persistent
//! message history in SQLite.
//!
//! Each WebSocket connection is handled in its own Tokio task. Rooms are
//! created lazily; each room owns a `tokio::sync::broadcast` channel shared
//! by all connected clients. Incoming messages are persisted to SQLite so
//! that new joiners receive the last 50 messages as history.
//!
//! # Endpoints
//! - `GET /ws`    — upgrade to WebSocket; JSON message protocol described below
//! - `GET /rooms` — list active rooms and approximate user count

use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use thiserror::Error;
use tokio::sync::broadcast;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};

// ────────────────────────────────────────────────────────────────────────────
// Error types
// ────────────────────────────────────────────────────────────────────────────

/// All recoverable errors within the chat server.
#[derive(Debug, Error)]
pub enum ChatError {
    /// A SQLite operation failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// JSON serialisation or deserialisation failed.
    #[error("serialisation error: {0}")]
    Serialisation(#[from] serde_json::Error),
}

impl IntoResponse for ChatError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.to_string() });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Shared application state
// ────────────────────────────────────────────────────────────────────────────

/// Global state shared by all connected WebSocket handlers.
pub struct AppState {
    /// SQLite connection pool for message persistence.
    pub pool: SqlitePool,
    /// Map from room name to that room's broadcast sender.
    /// Each room channel has a capacity of 128 messages.
    pub rooms: DashMap<String, broadcast::Sender<ChatMessage>>,
}

// ────────────────────────────────────────────────────────────────────────────
// Message protocol — server → client
// ────────────────────────────────────────────────────────────────────────────

/// A chat message as stored in memory and persisted to the database.
///
/// This type is cloned into the broadcast channel so all subscribers in a room
/// receive it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The room the message belongs to.
    pub room: String,
    /// The author's username.
    pub username: String,
    /// Message text body.
    pub body: String,
    /// SQLite-formatted UTC timestamp (`datetime('now')`).
    pub ts: String,
}

/// Every frame sent from the server to a client is one of these events.
#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ServerEvent {
    /// Confirms a successful room join; includes current member list.
    Joined {
        /// The room the client joined.
        room: String,
    },
    /// A chat message from any member of the room.
    Message {
        /// Room the message belongs to.
        room: String,
        /// Author's username.
        username: String,
        /// Message text.
        body: String,
        /// Timestamp string.
        ts: String,
    },
    /// A new user connected to the room.
    UserJoined {
        /// The room.
        room: String,
        /// The user who just joined.
        username: String,
    },
    /// A user disconnected from the room.
    UserLeft {
        /// The room.
        room: String,
        /// The user who left.
        username: String,
    },
    /// Sent immediately after join; contains recent message history.
    History {
        /// Up to 50 recent messages in chronological order.
        messages: Vec<ChatMessage>,
    },
    /// An error message for the client (e.g. protocol violation).
    Error {
        /// Human-readable error description.
        message: String,
    },
}

// ────────────────────────────────────────────────────────────────────────────
// Message protocol — client → server
// ────────────────────────────────────────────────────────────────────────────

/// Every WebSocket frame from a client is one of these actions.
#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClientAction {
    /// Join a room with a chosen username.
    Join {
        /// Target room name.
        room: String,
        /// Desired display name.
        username: String,
    },
    /// Send a chat message to the current room.
    Message {
        /// Message text.
        body: String,
    },
    /// Voluntarily leave the current room.
    Leave,
}

// ────────────────────────────────────────────────────────────────────────────
// REST handlers
// ────────────────────────────────────────────────────────────────────────────

/// `GET /rooms` — list active rooms and their approximate connected-user count
/// (measured by the broadcast subscriber count).
pub async fn list_rooms(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<serde_json::Value>> {
    let rooms: Vec<serde_json::Value> = state
        .rooms
        .iter()
        .map(|entry| {
            serde_json::json!({
                "room": entry.key().clone(),
                "user_count": entry.value().receiver_count(),
            })
        })
        .collect();

    Json(rooms)
}

// ────────────────────────────────────────────────────────────────────────────
// WebSocket upgrade handler
// ────────────────────────────────────────────────────────────────────────────

/// `GET /ws` — upgrade the HTTP connection to WebSocket and hand off to `handle_socket`.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

// ────────────────────────────────────────────────────────────────────────────
// Per-connection handler
// ────────────────────────────────────────────────────────────────────────────

/// Drive a single WebSocket connection for its entire lifetime.
///
/// The function first waits for a `join` action, then enters a select loop
/// that multiplexes incoming client frames against broadcast messages from the
/// room channel until the connection closes.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // ── Phase 1: wait for the join action ──────────────────────────────────
    let (room_name, username) = loop {
        let frame = match receiver.next().await {
            Some(Ok(f)) => f,
            // Client disconnected before joining.
            _ => return,
        };

        let text = match frame {
            Message::Text(t) => t,
            Message::Close(_) => return,
            // Ignore binary frames and pings during the handshake phase.
            _ => continue,
        };

        match serde_json::from_str::<ClientAction>(&text) {
            Ok(ClientAction::Join { room, username }) => {
                break (room, username);
            }
            Ok(_) => {
                // Client sent a message or leave before joining — send an error.
                let err = ServerEvent::Error {
                    message: "send a 'join' action first".to_string(),
                };
                if let Ok(json) = serde_json::to_string(&err) {
                    let _ = sender.send(Message::Text(json.into())).await;
                }
            }
            Err(e) => {
                let err = ServerEvent::Error {
                    message: format!("invalid JSON: {e}"),
                };
                if let Ok(json) = serde_json::to_string(&err) {
                    let _ = sender.send(Message::Text(json.into())).await;
                }
            }
        }
    };

    // ── Phase 2: subscribe to the room broadcast channel ───────────────────
    // Use DashMap's entry API to create the channel lazily if this is the
    // first client to join the room.
    let tx = state
        .rooms
        .entry(room_name.clone())
        .or_insert_with(|| {
            // capacity 128: old messages are dropped for slow receivers
            let (tx, _) = broadcast::channel(128);
            tx
        })
        .clone();

    // Subscribe *before* sending history so we don't miss concurrent messages.
    let mut rx = tx.subscribe();

    // ── Phase 3: send history ──────────────────────────────────────────────
    match fetch_history(&state.pool, &room_name).await {
        Ok(messages) => {
            let history = ServerEvent::History { messages };
            if let Ok(json) = serde_json::to_string(&history) {
                let _ = sender.send(Message::Text(json.into())).await;
            }
        }
        Err(e) => {
            error!("failed to fetch history for room {}: {e}", room_name);
        }
    }

    // ── Phase 4: notify others that this user joined ───────────────────────
    let joined_event = ServerEvent::Joined {
        room: room_name.clone(),
    };
    if let Ok(json) = serde_json::to_string(&joined_event) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    // Broadcast user_joined to all *other* subscribers in the room.
    let _ = tx.send(ChatMessage {
        room: room_name.clone(),
        username: format!("__system__user_joined__{}", username),
        body: String::new(),
        ts: current_timestamp(),
    });

    // ── Phase 5: message loop ──────────────────────────────────────────────
    // We need to send outbound frames, which requires exclusive access to the
    // sink. We merge two async streams using tokio::select!:
    //   • `receiver` — frames arriving from this client
    //   • `rx`       — broadcast messages from other clients in the room
    loop {
        tokio::select! {
            // Incoming frame from this client.
            maybe_frame = receiver.next() => {
                let frame = match maybe_frame {
                    Some(Ok(f)) => f,
                    // Error or clean close — exit the loop.
                    _ => break,
                };

                match frame {
                    Message::Text(text) => {
                        match serde_json::from_str::<ClientAction>(&text) {
                            Ok(ClientAction::Message { body }) => {
                                let ts = current_timestamp();
                                let msg = ChatMessage {
                                    room: room_name.clone(),
                                    username: username.clone(),
                                    body,
                                    ts,
                                };

                                // Persist to DB (non-fatal if it fails).
                                if let Err(e) = persist_message(&state.pool, &msg).await {
                                    error!("failed to persist message: {e}");
                                }

                                // Broadcast to all subscribers (including self).
                                let _ = tx.send(msg);
                            }
                            Ok(ClientAction::Leave) => break,
                            Ok(ClientAction::Join { .. }) => {
                                // Already joined; ignore re-join attempts.
                            }
                            Err(e) => {
                                let err = ServerEvent::Error {
                                    message: format!("invalid JSON: {e}"),
                                };
                                if let Ok(json) = serde_json::to_string(&err) {
                                    let _ = sender.send(Message::Text(json.into())).await;
                                }
                            }
                        }
                    }
                    Message::Close(_) => break,
                    // Respond to pings automatically handled by Axum; skip other frames.
                    _ => {}
                }
            }

            // Broadcast message from another (or this) client.
            maybe_msg = rx.recv() => {
                match maybe_msg {
                    Ok(msg) => {
                        // Check for synthetic system events encoded in the username field.
                        let event = if let Some(joined_user) =
                            msg.username.strip_prefix("__system__user_joined__")
                        {
                            // Don't echo the user_joined event back to the joining user.
                            if joined_user == username {
                                continue;
                            }
                            ServerEvent::UserJoined {
                                room: msg.room.clone(),
                                username: joined_user.to_string(),
                            }
                        } else if let Some(left_user) =
                            msg.username.strip_prefix("__system__user_left__")
                        {
                            ServerEvent::UserLeft {
                                room: msg.room.clone(),
                                username: left_user.to_string(),
                            }
                        } else {
                            ServerEvent::Message {
                                room: msg.room.clone(),
                                username: msg.username.clone(),
                                body: msg.body.clone(),
                                ts: msg.ts.clone(),
                            }
                        };

                        if let Ok(json) = serde_json::to_string(&event) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                // Client disconnected while we were sending.
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        // This subscriber fell too far behind; log and continue.
                        warn!("subscriber lagged by {n} messages in room '{}'", room_name);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // The room channel was dropped (no senders left).
                        break;
                    }
                }
            }
        }
    }

    // ── Phase 6: clean up ──────────────────────────────────────────────────
    // Notify remaining subscribers that this user left.
    let _ = tx.send(ChatMessage {
        room: room_name.clone(),
        username: format!("__system__user_left__{}", username),
        body: String::new(),
        ts: current_timestamp(),
    });

    // Remove the room entry if no other subscribers remain (receiver_count == 0
    // means only our now-unsubscribed rx was keeping it alive).
    if tx.receiver_count() == 0 {
        state.rooms.remove(&room_name);
    }

    info!("user '{}' left room '{}'", username, room_name);
}

// ────────────────────────────────────────────────────────────────────────────
// Database helpers
// ────────────────────────────────────────────────────────────────────────────

/// Persist a chat message to the `messages` table.
async fn persist_message(pool: &SqlitePool, msg: &ChatMessage) -> Result<(), ChatError> {
    sqlx::query(
        "INSERT INTO messages (room, username, body) VALUES (?, ?, ?)",
    )
    .bind(&msg.room)
    .bind(&msg.username)
    .bind(&msg.body)
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetch the last 50 messages for `room` in chronological order.
async fn fetch_history(pool: &SqlitePool, room: &str) -> Result<Vec<ChatMessage>, ChatError> {
    // Subquery selects the 50 most recent rows; the outer query re-orders them
    // chronologically (oldest-first) so clients see conversation history in order.
    // We include `id` in the subquery so the outer ORDER BY can reference it.
    let rows = sqlx::query(
        "SELECT room, username, body, created_at AS ts \
         FROM (
           SELECT id, room, username, body, created_at
           FROM messages
           WHERE room = ?
           ORDER BY id DESC
           LIMIT 50
         )
         ORDER BY id ASC",
    )
    .bind(room)
    .fetch_all(pool)
    .await?;

    let mut messages = Vec::with_capacity(rows.len());
    for row in &rows {
        messages.push(ChatMessage {
            room: row.try_get("room")?,
            username: row.try_get("username")?,
            body: row.try_get("body")?,
            ts: row.try_get("ts")?,
        });
    }

    Ok(messages)
}

// ────────────────────────────────────────────────────────────────────────────
// Utility
// ────────────────────────────────────────────────────────────────────────────

/// Return the current UTC time as a SQLite-compatible datetime string.
///
/// Format: `YYYY-MM-DD HH:MM:SS` (SQLite's `datetime('now')` format).
fn current_timestamp() -> String {
    // Use std time rather than pulling in a dependency for this simple formatting.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Convert unix seconds to a simple UTC datetime string.
    let (y, mo, d, h, mi, s) = unix_to_datetime(secs);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
}

/// Convert a Unix timestamp (seconds since epoch) to `(year, month, day, hour, min, sec)`.
///
/// This is a minimal implementation for UTC only; it handles leap years and
/// variable-length months correctly but does not support leap seconds.
fn unix_to_datetime(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let mins = secs / 60;
    let mi = mins % 60;
    let hours = mins / 60;
    let h = hours % 24;
    let days = hours / 24;

    // Days since 1970-01-01, compute year.
    let mut remaining = days;
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    // Compute month within the year.
    let leap = is_leap(year);
    let month_lengths = [31u64, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for &len in &month_lengths {
        if remaining < len {
            break;
        }
        remaining -= len;
        month += 1;
    }

    let day = remaining + 1;
    (year, month, day, h, mi, s)
}

/// Return `true` if `year` is a Gregorian leap year.
fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

// ────────────────────────────────────────────────────────────────────────────
// Database initialisation
// ────────────────────────────────────────────────────────────────────────────

/// Create the `messages` table and index if they do not already exist.
async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            room       TEXT NOT NULL,
            username   TEXT NOT NULL,
            body       TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await
    .context("failed to create messages table")?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_messages_room ON messages(room)",
    )
    .execute(pool)
    .await
    .context("failed to create messages index")?;

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
                .unwrap_or_else(|_| "chat_server=debug,tower_http=debug".into()),
        )
        .init();

    let pool = SqlitePool::connect("sqlite:chat.db?mode=rwc")
        .await
        .context("failed to open SQLite database")?;

    run_migrations(&pool).await?;

    let state = Arc::new(AppState {
        pool,
        rooms: DashMap::new(),
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/rooms", get(list_rooms))
        .with_state(state)
        // Permissive CORS so a browser-based client can connect.
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .context("failed to bind TCP listener on port 3002")?;

    info!("Chat server listening on http://0.0.0.0:3002");
    info!("Connect via WebSocket at ws://localhost:3002/ws");

    axum::serve(listener, app)
        .await
        .context("server error")?;

    Ok(())
}
