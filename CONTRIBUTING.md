# Contributing to Rimuru

Thank you for your interest in contributing to Rimuru!

## Development Setup

### Prerequisites

- **Rust**: 1.83+
- **iii engine**: [iii-hq/iii](https://github.com/iii-hq/iii) (installed automatically by `install.sh`)
- **Node.js**: 18+ (for Web UI development)

No database required. Rimuru uses iii-engine's in-memory KV state.

### Setup

```bash
git clone https://github.com/YOUR_USERNAME/rimuru.git
cd rimuru

# Install iii engine if not already installed
curl -fsSL https://raw.githubusercontent.com/rohitg00/rimuru/main/install.sh | bash

# Create UI dist stub (needed for compilation)
mkdir -p ui/dist && echo '<html></html>' > ui/dist/index.html

# Build
cargo build

# Run checks
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
```

### Running Locally

```bash
# Start iii engine
iii

# Start the worker (in another terminal)
cargo run -p rimuru-core --release

# CLI
cargo run -p rimuru-cli -- health

# TUI
cargo run -p rimuru-tui --release

# Desktop (Tauri v2)
cd crates/rimuru-desktop
cargo tauri dev
```

## Making Changes

### Branch Naming

```bash
git checkout -b feature/your-feature-name
git checkout -b fix/bug-description
git checkout -b docs/documentation-changes
```

### Code Style

```bash
cargo fmt --all          # Format
cargo clippy --all-targets -- -D warnings  # Lint
```

### Before Submitting

1. Run `cargo fmt --all -- --check`
2. Run `cargo clippy --all-targets -- -D warnings`
3. Run `cargo test --all`
4. Update documentation if needed

### Pull Request

1. Push your branch to your fork
2. Open a PR against `rohitg00/rimuru:main`
3. Fill out the PR description
4. Link related issues

## Project Structure

```
crates/
  rimuru-core/     # iii Worker: functions, triggers, HTTP server, Web UI
  rimuru-cli/      # CLI client (connects to iii engine via WebSocket)
  rimuru-tui/      # Terminal UI (Ratatui, connects via HTTP)
  rimuru-desktop/  # Desktop app (Tauri v2, embeds worker)
ui/                # Web UI (React + Vite)
```

### Key Patterns

- **Functions**: Register via `iii.register_function()` in `crates/rimuru-core/src/functions/`
- **State**: In-memory KV via `StateKV` (no database)
- **HTTP**: Axum handlers call iii functions in `crates/rimuru-core/src/http.rs`
- **Triggers**: API and schedule triggers in `crates/rimuru-core/src/triggers/`
- **Adapters**: Agent discovery in `crates/rimuru-core/src/adapters/`

## Questions?

- Open a [Discussion](https://github.com/rohitg00/rimuru/discussions)
- Check existing [Issues](https://github.com/rohitg00/rimuru/issues)
