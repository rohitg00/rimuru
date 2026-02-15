# Contributing to Rimuru

Thank you for your interest in contributing to Rimuru! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Code Style](#code-style)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Review Process](#review-process)

## Getting Started

### Types of Contributions

We welcome several types of contributions:

- **Bug Reports**: Found a bug? Open an issue with details
- **Feature Requests**: Have an idea? Open an issue to discuss
- **Code Contributions**: Fix bugs or implement features
- **Documentation**: Improve docs, fix typos, add examples
- **Testing**: Add tests, improve coverage, report edge cases

### Good First Issues

Look for issues labeled [`good first issue`](https://github.com/rohitg00/rimuru/labels/good%20first%20issue) - these are great for newcomers.

## Development Setup

### Prerequisites

- **Rust**: 1.75 or later
- **PostgreSQL**: 14 or later
- **Node.js**: 18+ (for desktop app)

### Setup Steps

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/rimuru.git
cd rimuru

# Add upstream remote
git remote add upstream https://github.com/rohitg00/rimuru.git

# Create database
createdb rimuru_dev

# Configure environment
cp .env.example .env
echo "DATABASE_URL=postgres://localhost/rimuru_dev" > .env

# Build and verify
cargo build
cargo test
```

### Running Locally

```bash
# CLI
cargo run --bin rimuru -- status

# TUI
cargo run --bin rimuru-tui

# Desktop (requires npm install first)
cd rimuru-desktop
npm install
npm run tauri dev
```

## Making Changes

### Branching Strategy

```bash
# Sync with upstream
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/bug-description
```

### Branch Naming

- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/changes
- `chore/` - Maintenance tasks

## Code Style

### Rust

We follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check lints
cargo clippy --all-targets -- -D warnings
```

**Guidelines:**
- Use `thiserror` for error types
- Prefer `async` for I/O operations
- Write doc comments for public APIs
- Use meaningful variable names
- Keep functions focused and small

### TypeScript (Desktop)

```bash
cd rimuru-desktop
npm run lint
npm run format
```

**Guidelines:**
- Use TypeScript strict mode
- Prefer functional components with hooks
- Use proper type annotations
- Follow React best practices

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting (no code change)
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

**Examples:**
```
feat(cli): add session export command
fix(tui): correct scroll behavior in agent list
docs(readme): update installation instructions
test(core): add adapter trait tests
```

## Testing

### Running Tests

```bash
# All tests
cargo test --all

# Specific crate
cargo test -p rimuru-core

# With output
cargo test -- --nocapture

# Integration tests (requires database)
cargo test --test '*'
```

### Writing Tests

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("test", AgentType::Claude);
        assert_eq!(agent.name, "test");
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = fetch_data().await;
        assert!(result.is_ok());
    }
}
```

**Integration Tests:**
Place in `tests/integration/` directory.

### Coverage

Aim for 70%+ coverage on new code. Run coverage locally:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --all-features
```

## Submitting Changes

### Before Submitting

1. **Sync with upstream:**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks:**
   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test --all
   ```

3. **Update documentation** if needed

### Creating a Pull Request

1. Push your branch to your fork
2. Open a PR against `rohitg00/rimuru:main`
3. Fill out the PR template completely
4. Link related issues

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How was this tested?

## Checklist
- [ ] Code follows project style
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CI passes
```

## Review Process

### What to Expect

1. **Automated Checks**: CI runs on all PRs
2. **Code Review**: Maintainer reviews within 2-3 days
3. **Feedback**: Address any requested changes
4. **Merge**: Once approved and CI passes

### Review Criteria

- Code quality and style
- Test coverage
- Documentation completeness
- Breaking change considerations
- Security implications

### After Merge

- Delete your feature branch
- Sync your fork with upstream

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Ratatui Docs](https://ratatui.rs/)
- [Tauri Docs](https://tauri.app/v1/guides/)

## Questions?

- Open a [Discussion](https://github.com/rohitg00/rimuru/discussions)
- Check existing [Issues](https://github.com/rohitg00/rimuru/issues)

Thank you for contributing to Rimuru!
