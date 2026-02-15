<p align="center">
  <img src="docs/assets/rimuru-banner.svg" alt="Rimuru - AI Agent Orchestration Platform" width="800"/>
</p>

<p align="center">
  <strong>Unified AI Agent Orchestration & Cost Tracking Platform</strong><br/>
  <em>Like Rimuru Tempest absorbs skills, Rimuru absorbs your AI agents into one place.</em>
</p>

<p align="center">
  <a href="https://github.com/rohitg00/rimuru/actions/workflows/ci.yml"><img src="https://github.com/rohitg00/rimuru/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://opensource.org/licenses/Apache-2.0"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.75%2B-orange.svg" alt="Rust"></a>
  <a href="https://github.com/rohitg00/rimuru/releases"><img src="https://img.shields.io/github/v/release/rohitg00/rimuru" alt="Release"></a>
</p>

<p align="center">
  <a href="#features">Features</a> &middot;
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="#installation">Installation</a> &middot;
  <a href="#supported-agents">Agents</a> &middot;
  <a href="#themes">Themes</a> &middot;
  <a href="#contributing">Contributing</a>
</p>

---

## Overview

Rimuru is a central hub for managing, monitoring, and analyzing costs across multiple AI coding agents. Whether you're using Claude Code, GitHub Copilot, Cursor, or other AI assistants, Rimuru provides unified visibility and control.

**Why Rimuru?**
- **One Dashboard** &mdash; See all your AI agents in one place
- **Cost Control** &mdash; Track spending across agents, sessions, and models
- **Real-time Monitoring** &mdash; Watch sessions, metrics, and usage as they happen
- **Extensible** &mdash; Add custom plugins, hooks, and integrations
- **Beautiful** &mdash; Ships with the Rimuru Slime theme (translucent cyan-blues with golden accents, inspired by Rimuru Tempest)

## Features

### Multi-Agent Support

Manage 6 popular AI coding agents from a single interface:

| Agent | Icon | Description |
|-------|------|-------------|
| **Claude Code** | ⟁ | Anthropic's Claude-powered coding assistant |
| **Codex** | ◎ | OpenAI Codex CLI for code generation |
| **GitHub Copilot** | ◈ | AI pair programmer integrated with VS Code |
| **Goose** | ⬡ | Block's autonomous coding agent |
| **OpenCode** | ◇ | Open-source AI coding assistant |
| **Cursor** | ◫ | AI-first code editor |

### Cost Tracking
- Real-time cost monitoring with detailed breakdowns
- Track costs by agent, session, model, and time period
- Automatic model pricing sync from provider APIs
- Budget alerts and spending reports

### Three Interfaces

| Interface | Description |
|-----------|-------------|
| **CLI** | Fast, scriptable command-line access |
| **TUI** | Rich terminal UI with keyboard navigation |
| **Desktop** | Cross-platform GUI (macOS, Linux, Windows) |

### Plugin System
Extend Rimuru with plugins for:
- **Exporters**: CSV, JSON, Webhook
- **Notifiers**: Slack, Discord
- **Custom Agents**: Add your own AI agent adapters
- **Views**: Create custom dashboards

## Themes

Rimuru ships with 15 themes. The **Rimuru Slime** theme is the default, featuring azure blue tones inspired by Rimuru Tempest's color palette.

| Theme | Colors |
|-------|--------|
| **Rimuru Slime** (default) | Deep ocean `#0d1b2a` &middot; Cyan accent `#5cc6d0` &middot; Blue `#93B9E8` |
| Tokyo Night | Purple-blues `#7aa2f7` `#c0caf5` |
| Catppuccin | Pastel `#89b4fa` `#cdd6f4` |
| Dracula | Purple-pink `#bd93f9` `#ff79c6` |
| Nord | Arctic blue `#88c0d0` `#eceff4` |
| Light | Clean white `#2563eb` |
| + 9 more | Monokai, Gruvbox, Solarized, Ayu, etc. |

## Quick Start

```bash
git clone https://github.com/rohitg00/rimuru.git
cd rimuru
cargo build --release

createdb rimuru_dev
echo "DATABASE_URL=postgres://localhost/rimuru_dev" > .env

./target/release/rimuru init
./target/release/rimuru status
```

## Installation

### From Source

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/rohitg00/rimuru.git
cd rimuru
cargo build --release
```

### Using Cargo

```bash
cargo install rimuru
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/rohitg00/rimuru/releases):

| Platform | Download |
|----------|----------|
| Linux (x64) | `rimuru-linux-x86_64.tar.gz` |
| macOS (x64) | `rimuru-darwin-x86_64.tar.gz` |
| macOS (ARM) | `rimuru-darwin-aarch64.tar.gz` |
| Windows | `rimuru-windows-x86_64.zip` |

### Desktop App

Download installers from [GitHub Releases](https://github.com/rohitg00/rimuru/releases):
- **macOS**: `rimuru-desktop.dmg`
- **Linux**: `rimuru-desktop.AppImage`
- **Windows**: `rimuru-desktop.msi`

## Prerequisites

- **Rust**: 1.75 or later
- **PostgreSQL**: 14 or later
- **Node.js**: 18+ (for desktop app development)

## CLI Commands

```bash
rimuru init              # Initialize database
rimuru status            # Show system metrics
rimuru agents list       # List all agents
rimuru agents discover   # Auto-discover agents
rimuru sessions list     # List active sessions
rimuru costs today       # Today's costs
rimuru costs summary     # Cost summary
rimuru skills search     # Search marketplace
rimuru plugins list      # List plugins
```

## Supported Agents

| Agent | Type ID | Discovery |
|-------|---------|-----------|
| Claude Code | `claude` | `~/.claude/` |
| Codex | `codex` | `codex` CLI |
| GitHub Copilot | `copilot` | VS Code extension |
| Goose | `goose` | `~/.goose/` |
| OpenCode | `opencode` | `opencode` CLI |
| Cursor | `cursor` | Cursor app |

## Project Structure

```
rimuru/
├── rimuru-core/       # Core library (adapters, models, services)
├── rimuru-cli/        # Command-line interface
├── rimuru-tui/        # Terminal user interface
├── rimuru-desktop/    # Desktop application (Tauri + React)
├── rimuru-plugin-sdk/ # Plugin development SDK
├── docs/              # Documentation
└── config/            # Configuration files
```

## Development

```bash
cargo test --all
RUST_LOG=debug cargo run --bin rimuru -- status
cargo clippy --all-targets -- -D warnings

cd rimuru-desktop
npm install
npm run tauri build
```

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md).

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes
4. Push and open a Pull Request

## Security

For security issues, please see our [Security Policy](SECURITY.md) and report vulnerabilities via GitHub's private vulnerability reporting.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

- [Tauri](https://tauri.app/) for the desktop framework
- [Ratatui](https://ratatui.rs/) for the TUI framework
- [SQLx](https://github.com/launchbadge/sqlx) for async database access
- [SkillKit](https://agenstskills.com) for skill marketplace integration
- **Rimuru Tempest** for the color inspiration

---

<p align="center">
  Made with ♥ by <a href="https://github.com/rohitg00">Rohit Ghumare</a>
</p>
