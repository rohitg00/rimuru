# Architecture

Rimuru is built on [iii-engine](https://github.com/iii-hq/iii) (Worker/Function/Trigger primitives) with in-memory KV state. No database required.

## High-Level Architecture

```
                iii Engine (WS :49134)
                        |
         +--------------+--------------+
         |              |              |
   rimuru-worker   rimuru-cli    rimuru-desktop
   (core + HTTP)   (iii client)  (Tauri v2 + embedded worker)
         |
   +-----------+
   |           |
  Web UI    rimuru-tui
  (:3100)   (HTTP client)
```

## Crates

| Crate | Binary | Description |
|-------|--------|-------------|
| `rimuru-core` | `rimuru-worker` | iii Worker with 40+ functions, axum HTTP server, embedded Web UI |
| `rimuru-cli` | `rimuru` | CLI connecting to iii engine, calls functions, prints output |
| `rimuru-tui` | `rimuru-tui` | Ratatui terminal UI with 10 tabs and 15 themes |
| `rimuru-desktop` | Desktop app | Tauri v2 native app with embedded worker and 46 IPC commands |

## Core Modules

### Functions (`rimuru-core/src/functions/`)

Each module registers iii functions via `register(iii, kv)`:

| Module | Functions | Purpose |
|--------|-----------|---------|
| `agents` | `rimuru.agents.*` | Agent discovery, sync, CRUD |
| `sessions` | `rimuru.sessions.*` | Session listing, details |
| `costs` | `rimuru.costs.*` | Cost tracking, summaries, daily rollups |
| `models` | `rimuru.models.*` | Model pricing catalog (8 models) |
| `metrics` | `rimuru.metrics.*` | System metrics (CPU, memory) |
| `hooks` | `rimuru.hooks.*` | Event hook registration and dispatch |
| `plugins` | `rimuru.plugins.*` | Plugin install/uninstall/list |
| `mcp` | `rimuru.mcp.*` | MCP server management |
| `skillkit` | `rimuru.skillkit.*` | SkillKit marketplace bridge |
| `health` | `rimuru.health.*` | Health checks |
| `config` | `rimuru.config.*` | Configuration management |
| `hardware` | `rimuru.hardware.*`, `rimuru.advisor.*` | Hardware detection, model advisor |
| `sysutil` | `rimuru.sysutil.*` | System utilities |

### State (`rimuru-core/src/state.rs`)

In-memory KV store wrapping iii-sdk's state API:

```rust
pub struct StateKV {
    iii: III,
}

impl StateKV {
    pub async fn set(&self, scope: &str, key: &str, value: Value);
    pub async fn get(&self, scope: &str, key: &str) -> Option<Value>;
    pub async fn list(&self, scope: &str) -> Vec<(String, Value)>;
    pub async fn delete(&self, scope: &str, key: &str);
}
```

Scopes: `agents`, `sessions`, `costs`, `models`, `metrics`, `hooks`, `plugins`, `mcp`, `hardware`, `advisor`, `config`, `cost_agent`, `cost_model`, `cost_daily`

### HTTP Server (`rimuru-core/src/http.rs`)

Axum server at `:3100` with ~40 routes. Each handler calls an iii function and returns JSON. Also serves the embedded Web UI (single-file React app).

### Triggers (`rimuru-core/src/triggers/`)

- **API triggers** (`api.rs`): SSE event streams for real-time updates
- **Schedule triggers** (`schedules.rs`): Periodic agent sync, metrics collection, cost rollups

### Adapters (`rimuru-core/src/adapters/`)

Agent discovery reads config directories on disk:

| Agent | Config Path |
|-------|-------------|
| Claude Code | `~/.claude/` |
| Cursor | `~/Library/Application Support/Cursor/` |
| Copilot | VS Code extension storage |
| Codex | `~/.config/codex/` |
| Goose | `~/.config/goose/` |
| OpenCode | `~/.opencode/` |

### Worker (`rimuru-core/src/worker.rs`)

Startup sequence:
1. Connect to iii engine
2. Register all functions
3. Detect hardware
4. Discover agents
5. Start HTTP server
6. Begin scheduled triggers (agent sync, metrics, cost rollups)

## Data Flow

```
Agent Config Files --> Adapter.discover() --> KV[agents] --> HTTP API --> UI/CLI/TUI
                                                  |
                                          Session Sync --> KV[sessions] --> Cost Calculation --> KV[costs]
```

## Web UI

Single-file React app built with Vite, embedded via `include_str!()` in the worker binary. 13 pages including a pixel-art "Tempest City" view where agents walk around as anime characters.
