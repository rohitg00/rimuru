# Building from Source

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.83+ | Compiler and cargo |
| iii engine | latest | Runtime engine ([iii-hq/iii](https://github.com/iii-hq/iii)) |
| Node.js | 18+ | Web UI build (optional) |

No database required. Rimuru uses in-memory KV state.

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

### Install iii Engine

```bash
# Auto-detect platform and install
curl -fsSL https://raw.githubusercontent.com/rohitg00/rimuru/main/install.sh | bash
```

Or download manually from [iii-hq/iii releases](https://github.com/iii-hq/iii/releases).

## Clone and Build

```bash
git clone https://github.com/rohitg00/rimuru.git
cd rimuru
```

### Create UI Dist Stub

The worker embeds the Web UI via `include_str!()`. For compilation without building the UI:

```bash
mkdir -p ui/dist && echo '<html></html>' > ui/dist/index.html
```

### Debug Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

Binaries: `target/release/rimuru-worker`, `target/release/rimuru`, `target/release/rimuru-tui`

### Build Specific Crate

```bash
cargo build -p rimuru-core      # Worker
cargo build -p rimuru-cli       # CLI
cargo build -p rimuru-tui       # TUI
cargo build -p rimuru-desktop   # Desktop (requires Tauri deps)
```

## Building the Web UI

```bash
cd ui
npm install
npm run build
```

Output goes to `ui/dist/` and gets embedded in the worker binary on next `cargo build`.

## Building the Desktop App

### System Dependencies

**Linux:**
```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

**macOS:** Xcode Command Line Tools (`xcode-select --install`)

### Build

```bash
cd crates/rimuru-desktop
cargo tauri build
```

Outputs: `.dmg` (macOS), `.AppImage` (Linux), `.msi` (Windows)

## Running Tests

```bash
cargo test --all
```

## Development Workflow

```bash
# Format
cargo fmt --all

# Lint
cargo clippy --all-targets -- -D warnings

# Pre-commit check
cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test --all
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RIMURU_ENGINE_URL` | iii engine WebSocket URL | `ws://127.0.0.1:49134` |
| `RIMURU_PORT` | HTTP server port | `3100` |
| `RUST_LOG` | Log level | `info` |
