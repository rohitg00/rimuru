---
type: reference
title: Getting Started with Rimuru
created: 2026-02-05
tags:
  - installation
  - quickstart
  - setup
related:
  - "[[agents]]"
  - "[[cli-reference]]"
  - "[[sessions]]"
---

# Getting Started with Rimuru

Rimuru is a unified AI agent orchestration and cost tracking platform. This guide walks you through installation, initial setup, and basic usage.

## Prerequisites

Before installing Rimuru, ensure you have:

- **Rust**: Version 1.75 or later
- **PostgreSQL**: Version 14 or later

## Installation

### From Source (Recommended)

```bash
git clone https://github.com/rohitg00/rimuru.git
cd rimuru
cargo build --release
```

The binary will be available at `./target/release/rimuru`.

### Using Cargo

```bash
cargo install rimuru
```

## Database Setup

Rimuru requires a PostgreSQL database. Choose the setup method that matches your environment:

### macOS (Homebrew)

```bash
brew install postgresql@16
brew services start postgresql@16
createdb rimuru_dev
```

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
sudo -u postgres createdb rimuru_dev
```

### Docker

```bash
docker run --name rimuru-postgres \
  -e POSTGRES_DB=rimuru_dev \
  -e POSTGRES_USER=rimuru \
  -e POSTGRES_PASSWORD=your_password \
  -p 5432:5432 \
  -d postgres:16
```

## Configuration

### Environment File

Create a `.env` file in your working directory:

```bash
cp .env.example .env
```

At minimum, set the `DATABASE_URL`:

```ini
DATABASE_URL=postgres://localhost/rimuru_dev
```

For Docker or authenticated connections:

```ini
DATABASE_URL=postgres://rimuru:your_password@localhost/rimuru_dev
```

### Key Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection URL | Required |
| `RIMURU_LOG_LEVEL` | Log level (trace, debug, info, warn, error) | `info` |
| `RIMURU_METRICS_INTERVAL` | Metrics collection interval (seconds) | `5` |
| `RIMURU_AGENT_AUTO_DISCOVER` | Auto-discover running agents | `true` |
| `RIMURU_SYNC_ENABLED` | Enable model pricing sync | `true` |
| `RIMURU_COLOR` | Enable colored terminal output | `true` |

See [[cli-reference]] for the complete list of environment variables.

## First Run

### Initialize the Database

Initialize Rimuru's database schema:

```bash
rimuru init
```

Expected output:

```
Initializing Rimuru...

  → Database URL: postgres://localhost/rimuru_dev
  → Connecting to database...
  → Running migrations...
  → Verifying connection...

✓ Database initialized successfully!
```

### Check System Status

Verify everything is working:

```bash
rimuru status
```

This displays system metrics including:
- Database connection status
- CPU and memory usage
- Active sessions count
- Total registered agents

### View Detailed Status

```bash
rimuru status --detailed
```

Includes additional system information like CPU cores, OS version, and hostname.

### JSON Output

For scripting or integration:

```bash
rimuru status --format json
```

## Basic Usage

### List Agents

View all registered AI agents:

```bash
rimuru agents
```

### View Sessions

See active and recent sessions:

```bash
rimuru sessions
rimuru sessions --active
```

### Check Costs

View today's costs:

```bash
rimuru costs today
```

View cost summary for the last 7 days:

```bash
rimuru costs summary --range 7d
```

### Sync Model Pricing

Update model pricing from providers:

```bash
rimuru sync run
```

## Supported Agents

Rimuru supports the following AI coding agents:

| Agent | Icon | Description |
|-------|------|-------------|
| Claude Code | ⟁ | Anthropic's Claude-powered coding assistant |
| Codex | ◎ | OpenAI's Codex models |
| GitHub Copilot | ◈ | GitHub's AI pair programmer |
| Goose | ⬡ | Block's autonomous AI agent |
| OpenCode | ◇ | Open-source coding assistant |
| Cursor | ◫ | AI-first code editor |

Each agent is automatically discovered when running. See [[agents]] for detailed configuration.

## TUI Mode

Rimuru includes a terminal user interface for interactive management:

```bash
rimuru-tui
```

Navigate using:
- `1-5`: Switch between Dashboard, Agents, Sessions, Costs, and Metrics views
- `j/k` or arrow keys: Navigate lists
- `Enter`: Select item
- `q`: Quit

See [[keyboard-shortcuts]] for the complete keybinding reference.

## Desktop Application

For a graphical interface, build and run the desktop application:

```bash
cd rimuru-desktop
npm install
npm run tauri dev
```

## Next Steps

- [[agents]] - Configure and manage AI agents
- [[sessions]] - Understand session tracking
- [[costs]] - Track and analyze costs
- [[skills]] - Install skills from SkillKit
- [[plugins]] - Extend Rimuru with plugins
- [[themes]] - Customize the TUI appearance

## Troubleshooting

### Database Connection Issues

**Error: "connection refused"**
- Ensure PostgreSQL is running
- Check the port (default: 5432)

**Error: "database does not exist"**
- Create the database: `createdb rimuru_dev`

**Error: "authentication failed"**
- Verify DATABASE_URL credentials
- For local development without password: `postgres://localhost/rimuru_dev`

### Migration Issues

**Error: "migration failed"**
- Check existing tables: `psql -d rimuru_dev -c "\dt"`
- Reset and rerun: `dropdb rimuru_dev && createdb rimuru_dev && rimuru init`

### Configuration Issues

**Error: "DATABASE_URL not set"**
- Create a `.env` file with DATABASE_URL
- Or export: `export DATABASE_URL=postgres://localhost/rimuru_dev`
