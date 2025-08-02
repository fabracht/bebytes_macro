# Contributing to BeBytes

Thank you for your interest in contributing to BeBytes! This guide will help you get started.

## Development Setup

### Prerequisites

- Rust 1.73 or later
- Git

### Setting Up Pre-commit Hooks

We use git hooks to ensure code quality before commits reach CI. This saves time by catching issues early.

#### Quick Setup

Run the setup script from the project root:

```bash
./setup-hooks.sh
```

You'll be presented with options:
1. **Fast checks** (Recommended for development) - Runs formatting, clippy, and compilation checks
2. **Full validation** - Runs all CI checks including tests for both std and no_std
3. **Custom** - Choose which hooks to install
4. **Pre-commit framework** - Uses the Python pre-commit framework

#### Manual Setup

Alternatively, you can manually set up hooks:

```bash
# Use fast checks for development
ln -s ../../.githooks/pre-commit-fast .git/hooks/pre-commit

# Or use full validation
ln -s ../../.githooks/pre-commit .git/hooks/pre-commit
ln -s ../../.githooks/pre-push .git/hooks/pre-push

# Or configure git to use the .githooks directory
git config core.hooksPath .githooks
```

#### Available Hooks

- **pre-commit-fast**: Quick validation (formatting, clippy, compilation)
- **pre-commit**: Full CI validation (all tests, std and no_std)
- **pre-push**: Runs full validation before pushing

#### Bypassing Hooks

If you need to bypass hooks temporarily:

```bash
git commit --no-verify
git push --no-verify
```

## Code Style

- Run `cargo fmt` before committing
- Fix all `cargo clippy -- -W clippy::pedantic` warnings
- Ensure tests pass with both std and no_std features

## Testing

Run all tests:
```bash
cargo test
cargo test --no-default-features
```

Run specific test categories:
```bash
cargo test --test compile_fail  # Compile-time error tests
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Ensure all hooks pass
5. Commit your changes
6. Push to your fork
7. Open a Pull Request

## CI Pipeline

Our CI runs the following checks:
- `cargo fmt -- --check`
- `cargo clippy -- -W clippy::pedantic`
- `cargo build` (with and without default features)
- `cargo test` (with and without default features)

The pre-commit hooks run these same checks locally.