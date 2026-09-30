# Chat Server

A real-time multi-room WebSocket chat server with SQLite message history, built
with Axum, Tokio, and sqlx. Rooms are created lazily on first join. The last
50 messages per room are persisted in SQLite and replayed to newly joined users.

## Build & Run

```bash
cd capstone/chat-server
cargo run
```

The server starts on **http://localhost:3002**. SQLite creates `chat.db` in
the working directory on first run.

## Connecting

### Using websocat

```bash
# Install: cargo install websocat
websocat ws://localhost:3002/ws
```

### Using wscat

```bash
# Install: npm install -g wscat
wscat -c ws://localhost:3002/ws
```

## JSON Message Protocol

All frames are JSON objects. The `action` / `event` field acts as a discriminant.

### Client → Server

**Join a room** (must be the first message sent):
```json
{"action": "join", "room": "general", "username": "alice"}
```

**Send a message** (only valid after joining):
```json
{"action": "message", "body": "Hello, world!"}
```

**Leave the room** (closes the connection cleanly):
```json
{"action": "leave"}
```

### Server → Client

**Joined confirmation:**
```json
{"event": "joined", "room": "general"}
```

**Recent message history** (sent immediately after joining):
```json
{
  "event": "history",
  "messages": [
    {"room": "general", "username": "alice", "body": "Hey!", "ts": "2024-01-15 10:00:00"}
  ]
}
```

**Incoming chat message:**
```json
{"event": "message", "room": "general", "username": "alice", "body": "Hello!", "ts": "2024-01-15 10:01:00"}
```

**User joined notification:**
```json
{"event": "user_joined", "room": "general", "username": "bob"}
```

**User left notification:**
```json
{"event": "user_left", "room": "general", "username": "alice"}
```

**Error:**
```json
{"event": "error", "message": "send a 'join' action first"}
```

## REST Endpoints

### List active rooms

```bash
curl http://localhost:3002/rooms
```

**Response:**
```json
[
  {"room": "general", "user_count": 3},
  {"room": "rust-help", "user_count": 1}
]
```

## Example Session (two terminals)

**Terminal 1 — Alice:**
```
wscat -c ws://localhost:3002/ws
> {"action":"join","room":"general","username":"alice"}
< {"event":"joined","room":"general"}
< {"event":"history","messages":[]}
> {"action":"message","body":"Hi there!"}
< {"event":"message","room":"general","username":"alice","body":"Hi there!","ts":"..."}
< {"event":"user_joined","room":"general","username":"bob"}
```

**Terminal 2 — Bob:**
```
wscat -c ws://localhost:3002/ws
> {"action":"join","room":"general","username":"bob"}
< {"event":"joined","room":"general"}
< {"event":"history","messages":[{"room":"general","username":"alice","body":"Hi there!","ts":"..."}]}
> {"action":"message","body":"Hey Alice!"}
```
