# IronDB

IronDB is a lightweight Redis-inspired in-memory database server written in Rust.

It supports TCP clients, simple key-value commands, concurrent connections, TTL expiration, and snapshot persistence. The project is designed as a long-term Rust backend/infrastructure project, starting small but gradually expanding toward more database internals such as append-only logs, eviction policies, metrics, and replication experiments.

The goal is to explore Rust for async networking, command parsing, shared state, error handling, persistence, and backend systems design.

## Why I built this

I wanted a hands-on Rust project that goes beyond a typical CRUD API. IronDB focuses on backend systems skills: async TCP networking, concurrent shared state, command parsing, TTL expiration, and snapshot persistence.

## Features

- TCP server built with Tokio
- Simple line-based command protocol
- Concurrent client connections (one task per connection)
- In-memory key-value storage
- TTL expiration with lazy removal on access
- Background expired-key cleanup every 5 seconds
- JSON snapshot persistence (`SAVE` / `LOAD`)
- Basic server information via `INFO`
- Unit and integration tests
- GitHub Actions CI for formatting, linting, and tests

## Demo

Start the server:

```bash
cargo run -- --port 6379
```

Connect with netcat:

```bash
nc 127.0.0.1 6379
```

Example session:

```text
PING
PONG

SET name Junan
OK

GET name
Junan

EXISTS name
1

EXPIRE name 30
OK

TTL name
28

DEL name
1

GET name
(nil)
```

Or run the demo script:

```bash
nc 127.0.0.1 6379 < examples/demo.txt
```

## Commands

| Command | Example | Response |
|---|---|---|
| `PING` | `PING` | `PONG` |
| `SET` | `SET name Junan` | `OK` |
| `GET` | `GET name` | value or `(nil)` |
| `DEL` | `DEL name` | `1` or `0` |
| `EXISTS` | `EXISTS name` | `1` or `0` |
| `EXPIRE` | `EXPIRE key 60` | `OK` or `0` |
| `TTL` | `TTL key` | `-2`, `-1`, or seconds |
| `SETEX` | `SETEX key 60 value` | `OK` |
| `SAVE` | `SAVE` | `OK` |
| `LOAD` | `LOAD` | `OK` |
| `INFO` | `INFO` | server stats |

**TTL semantics:**

- `-2` — key does not exist (or expired)
- `-1` — key exists with no expiry
- `N` — seconds remaining

## Architecture

```text
Client (nc)
    │
    ▼
TcpListener (server.rs)
    │ tokio::spawn per connection
    ▼
connection.rs ── read line ──► command.rs (parse)
                    │
                    ▼
              protocol.rs (execute + format)
                    │
                    ▼
         Arc<RwLock<Store>> (store.rs)
                    │
                    ▼
         persistence.rs (JSON snapshot)
```

- **`store.rs`** — in-memory `HashMap` with optional `expires_at` per key
- **`command.rs`** — parses line-based commands into a `Command` enum
- **`protocol.rs`** — executes commands and formats `Response` values
- **`server.rs`** — binds TCP, spawns clients, runs background cleanup
- **`connection.rs`** — per-client read/parse/execute/write loop
- **`stats.rs`** — tracks uptime, clients, commands, expired keys
- **`persistence.rs`** — JSON snapshot save/load

Shared state uses `Arc<tokio::sync::RwLock<Store>>` so many client tasks can read/write safely.

## Running locally

```bash
cargo run -- --port 6379
```

With snapshot file:

```bash
cargo run -- --port 6379 --snapshot ./irondb.snapshot.json
```

## Tests

```bash
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

## Docker

```bash
docker build -t irondb .
docker run -p 6379:6379 irondb
```

## Roadmap

- Append-only log persistence
- CLI client
- Benchmark command
- Redis RESP protocol support
- LRU eviction
- Metrics output
- Pub/Sub experiment
- Additional data types
- Replication experiment

## Limitations

IronDB is currently an experimental Redis-inspired project and is not intended for production use.

- No Redis RESP protocol support yet
- No authentication
- No clustering or replication
- Snapshot persistence only; append-only log is planned later
- String values only
- No memory limit or eviction policy yet

## License

MIT — see [LICENSE](LICENSE).
