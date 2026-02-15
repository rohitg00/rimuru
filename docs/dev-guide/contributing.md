---
type: reference
title: Contribution Guidelines
created: 2026-02-05
tags:
  - contributing
  - development
  - developer-guide
related:
  - "[[building]]"
  - "[[architecture]]"
---

# Contribution Guidelines

Thank you for your interest in contributing to Rimuru! This guide covers how to contribute code, documentation, and bug reports.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/rimuru.git
   cd rimuru
   ```
3. **Set up the development environment** (see [[building]])
4. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Types of Contributions

### Bug Reports

Open an issue with:
- Clear title describing the bug
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, etc.)
- Relevant logs or error messages

### Feature Requests

Open an issue with:
- Clear description of the feature
- Use case / motivation
- Proposed implementation (optional)

### Code Contributions

1. Check existing issues for related work
2. Open an issue to discuss larger changes before starting
3. Write tests for new functionality
4. Follow the code style guidelines below
5. Submit a pull request

### Documentation

- Fix typos or clarify existing docs
- Add examples or tutorials
- Improve API documentation
- Translate documentation

## Code Style

### Rust

Follow the official Rust style guide. Key points:

```rust
// Use snake_case for functions and variables
fn calculate_total_cost() -> f64 { }
let total_tokens = 1000;

// Use CamelCase for types and traits
struct SessionMetrics { }
trait CostTracker { }

// Use SCREAMING_SNAKE_CASE for constants
const MAX_RETRY_ATTEMPTS: u32 = 3;

// Document public APIs
/// Calculates the cost for the given token usage.
///
/// # Arguments
///
/// * `input_tokens` - Number of input tokens
/// * `output_tokens` - Number of output tokens
/// * `model_name` - The model used
///
/// # Returns
///
/// The calculated cost in USD.
///
/// # Errors
///
/// Returns `RimuruError::ModelNotFound` if the model is unknown.
pub async fn calculate_cost(
    input_tokens: i64,
    output_tokens: i64,
    model_name: &str,
) -> RimuruResult<f64> { }
```

### Error Handling

Use the project's error types:

```rust
use crate::error::{RimuruError, RimuruResult};

// Return RimuruResult for fallible operations
pub async fn get_session(&self, id: Uuid) -> RimuruResult<Session> {
    self.repo.find_by_id(id).await?
        .ok_or_else(|| RimuruError::NotFound("Session".to_string(), id.to_string()))
}

// Use ? for error propagation
let config = Config::load()?;
let db = Database::connect(&config.database_url).await?;
```

### Async Code

Use async/await consistently:

```rust
use async_trait::async_trait;

#[async_trait]
impl AgentAdapter for MyAdapter {
    async fn connect(&mut self) -> RimuruResult<()> {
        // Use .await for async operations
        self.client.connect().await?;
        Ok(())
    }
}
```

### Testing

Write tests for all new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        let cost = calculate_cost_simple(1000, 500, 0.01, 0.03);
        assert!((cost - 0.025).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_async_operation() {
        let adapter = MockAdapter::new();
        let result = adapter.get_sessions().await;
        assert!(result.is_ok());
    }
}
```

## Pull Request Process

### Before Submitting

1. **Format code**:
   ```bash
   cargo fmt
   ```

2. **Run lints**:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

3. **Run tests**:
   ```bash
   cargo test --all
   ```

4. **Update documentation** if needed

### PR Guidelines

- Use a descriptive title (e.g., "Add support for Cursor agent adapter")
- Reference related issues (e.g., "Fixes #123")
- Describe what changed and why
- Include screenshots for UI changes
- Keep PRs focused - one feature/fix per PR

### PR Template

```markdown
## Description
Brief description of changes.

## Related Issue
Fixes #123

## Changes
- Added X
- Modified Y
- Removed Z

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing performed

## Screenshots (if applicable)
```

### Review Process

1. A maintainer will review your PR
2. Address any requested changes
3. Once approved, a maintainer will merge

## Commit Messages

Follow conventional commits:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, no code change
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks

Examples:
```
feat(adapters): add Cursor agent adapter

Implements the Cursor agent adapter with session tracking
and cost calculation support.

Closes #42
```

```
fix(tui): resolve rendering issue on narrow terminals

The dashboard view now properly handles terminals
narrower than 80 columns.
```

## Development Tips

### Running a Single Test

```bash
cargo test test_name -- --nocapture
```

### Viewing Logs in Tests

```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Database Migrations

Add new migrations in `migrations/`:
```bash
# Create migration file
touch migrations/YYYYMMDDHHMMSS_description.sql
```

### Adding Dependencies

1. Add to `[workspace.dependencies]` in root `Cargo.toml`
2. Reference with `{ workspace = true }` in crate's `Cargo.toml`

Example:
```toml
# Root Cargo.toml
[workspace.dependencies]
new-crate = "1.0"

# Crate Cargo.toml
[dependencies]
new-crate = { workspace = true }
```

## Getting Help

- Open a discussion on GitHub
- Ask in PR/issue comments
- Review existing documentation and code

## Recognition

Contributors are recognized in:
- GitHub contributors list
- README.md for major features

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.

## See Also

- [[building]] - Build from source
- [[architecture]] - System architecture
- [[creating-adapters]] - How to add new agent adapters
- [[creating-plugins]] - Plugin development guide
