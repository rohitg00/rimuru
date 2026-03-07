# Getting Started

Rimuru monitors and optimizes costs across multiple AI coding agents. No database required.

## Installation

### One-Line Install

```bash
curl -fsSL https://raw.githubusercontent.com/rohitg00/rimuru/main/install.sh | bash
```

This installs the iii engine and rimuru binaries (`rimuru-worker`, `rimuru`, `rimuru-tui`).

### From Source

```bash
git clone https://github.com/rohitg00/rimuru.git
cd rimuru
mkdir -p ui/dist && echo '<html></html>' > ui/dist/index.html
cargo build --release
```

## Quick Start

### 1. Start the iii Engine

```bash
iii
```

### 2. Start the Worker

```bash
rimuru-worker
```

The worker starts at `http://localhost:3100` with the Web UI.

### 3. Check Health

```bash
rimuru health
```

### 4. View Agents

Rimuru auto-discovers installed AI agents:

```bash
rimuru agents list
```

Supported agents: Claude Code, Cursor, GitHub Copilot, Codex, Goose, OpenCode.

### 5. Check Costs

```bash
rimuru costs summary
rimuru costs daily
```

## Interfaces

### Web UI

Open `http://localhost:3100` in your browser. 13 pages including Dashboard, Agents, Sessions, Costs, Models, Metrics, Plugins, Hooks, MCP, SkillKit, Config, Activity, and Tempest City.

### TUI

```bash
rimuru-tui
```

10 tabs with 15 Tensura-themed color schemes. Press `t` to cycle themes.

### CLI

```bash
rimuru agents list
rimuru sessions list
rimuru costs summary
rimuru models list
rimuru metrics current
rimuru plugins list
rimuru hooks list
rimuru mcp list
rimuru config get
rimuru health
```

### Desktop App

Native app using Tauri v2 with an embedded worker:

```bash
# Download from GitHub Releases, or build from source
cd crates/rimuru-desktop
cargo tauri build
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RIMURU_ENGINE_URL` | iii engine WebSocket URL | `ws://127.0.0.1:49134` |
| `RIMURU_PORT` | Worker HTTP port | `3100` |
| `RUST_LOG` | Log level | `info` |

## Next Steps

- [CLI Reference](cli-reference.md)
- [Keyboard Shortcuts](keyboard-shortcuts.md)
- [Themes](themes.md)
- [Costs](costs.md)
- [Agents](agents.md)
