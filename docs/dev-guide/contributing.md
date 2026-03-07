# Contribution Guidelines

See [CONTRIBUTING.md](../../CONTRIBUTING.md) in the project root for full guidelines.

## Quick Reference

### Setup

```bash
git clone https://github.com/YOUR_USERNAME/rimuru.git
cd rimuru
mkdir -p ui/dist && echo '<html></html>' > ui/dist/index.html
cargo build
```

### Before Submitting a PR

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
```

### Adding a New iii Function

1. Create or edit a module in `crates/rimuru-core/src/functions/`
2. Register functions in the module's `pub fn register(iii: &III, kv: &StateKV)`
3. Add HTTP route in `crates/rimuru-core/src/http.rs` if needed
4. Add CLI subcommand in `crates/rimuru-cli/src/` if needed

### Adding Dependencies

Add to `[workspace.dependencies]` in root `Cargo.toml`, then reference with `{ workspace = true }` in crate `Cargo.toml`.
