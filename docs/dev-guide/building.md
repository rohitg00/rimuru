---
type: reference
title: Building from Source
created: 2026-02-05
tags:
  - building
  - development
  - developer-guide
related:
  - "[[architecture]]"
  - "[[contributing]]"
---

# Building from Source

This guide covers how to build Rimuru from source on Linux, macOS, and Windows.

## Prerequisites

### Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Compiler and cargo |
| PostgreSQL | 14+ | Database |
| Node.js | 18+ | Desktop app frontend |
| pnpm | 8+ | Desktop app package manager |

### Installing Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup update
```

### Installing PostgreSQL

**macOS (Homebrew):**
```bash
brew install postgresql@16
brew services start postgresql@16
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Windows:**
Download from https://www.postgresql.org/download/windows/

### Installing Node.js and pnpm

```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Install pnpm
corepack enable
corepack prepare pnpm@latest --activate
```

## Clone the Repository

```bash
git clone https://github.com/rohitg00/rimuru.git
cd rimuru
```

## Building Core Crates

### Debug Build

```bash
cargo build
```

This builds all workspace members in debug mode.

### Release Build

```bash
cargo build --release
```

Release binaries are placed in `target/release/`.

### Build Specific Crate

```bash
# Core library only
cargo build -p rimuru-core

# CLI only
cargo build -p rimuru-cli

# TUI only
cargo build -p rimuru-tui
```

## Running Tests

### All Tests

```bash
cargo test --all
```

### Specific Crate Tests

```bash
cargo test -p rimuru-core
cargo test -p rimuru-cli
cargo test -p rimuru-tui
```

### Integration Tests

Integration tests require a PostgreSQL database:

```bash
# Start test database with Docker
docker compose -f tests/docker-compose.yml up -d

# Run integration tests
cargo test --test '*' -- --ignored
```

### Test with Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Building the TUI

The TUI uses Ratatui for terminal rendering:

```bash
cargo build -p rimuru-tui --release
./target/release/rimuru-tui
```

## Building the Desktop App

The desktop app uses Tauri with a React frontend.

### Prerequisites

**Linux:**
```bash
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**macOS:**
Xcode Command Line Tools are required:
```bash
xcode-select --install
```

**Windows:**
- Visual Studio Build Tools with C++ support
- WebView2 runtime

### Build Desktop App

```bash
cd rimuru-desktop

# Install frontend dependencies
pnpm install

# Development mode
pnpm tauri dev

# Production build
pnpm tauri build
```

Build outputs:
- macOS: `src-tauri/target/release/bundle/dmg/`
- Linux: `src-tauri/target/release/bundle/appimage/`
- Windows: `src-tauri/target/release/bundle/msi/`

## Building Documentation

### Rust API Documentation

```bash
cargo doc --no-deps --document-private-items
open target/doc/rimuru_core/index.html
```

### User Documentation

User docs are in `docs/user-guide/` as Markdown files.

## Development Workflow

### Code Formatting

```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt
```

### Linting

```bash
# Check for issues
cargo clippy --all-targets

# With warnings as errors
cargo clippy --all-targets -- -D warnings
```

### Pre-commit Checks

Run before committing:

```bash
# Format, lint, test
cargo fmt && cargo clippy --all-targets && cargo test --all
```

## Environment Variables

Required for building/running:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection URL | Required |
| `RUST_LOG` | Log level (debug, info, warn, error) | `warn` |

Optional:

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | For Anthropic model sync |
| `OPENAI_API_KEY` | For OpenAI model sync |
| `GOOGLE_AI_API_KEY` | For Google model sync |

## Build Profiles

### Debug Profile (default)

```toml
[profile.dev]
opt-level = 0
debug = true
```

### Release Profile

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### Bench Profile

```toml
[profile.bench]
opt-level = 3
debug = true
```

## Troubleshooting

### OpenSSL Issues (Linux)

```bash
# Ubuntu/Debian
sudo apt install pkg-config libssl-dev

# Fedora
sudo dnf install openssl-devel
```

### SQLx Compile-Time Verification

SQLx verifies queries at compile time. Set `DATABASE_URL` or use offline mode:

```bash
# Generate query cache
cargo sqlx prepare --workspace

# Build without database connection
SQLX_OFFLINE=true cargo build
```

### Tauri Build Fails

Ensure all system dependencies are installed. Check the Tauri docs:
https://tauri.app/v1/guides/getting-started/prerequisites

### Memory Issues During Build

For large projects, limit parallel jobs:

```bash
cargo build -j 4  # Limit to 4 parallel jobs
```

## Cross-Compilation

### Linux → Windows

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --target x86_64-pc-windows-gnu --release
```

### Linux → macOS

Requires osxcross:
```bash
rustup target add x86_64-apple-darwin
cargo build --target x86_64-apple-darwin --release
```

## See Also

- [[architecture]] - System architecture overview
- [[contributing]] - Contribution guidelines
- [[api-reference]] - API documentation
