# Contributing to consola-rs

Thanks for your interest in contributing to consola-rs! This is an early-stage MVP project aiming for feature parity with the JavaScript [@unjs/consola](https://github.com/unjs/consola) library.

## Quick Start

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/consola-rs.git
   cd consola-rs
   ```
3. Create a feature branch:
   ```bash
   git checkout -b feature/my-contribution
   ```

## Development Workflow

### Before You Start

1. Read `tasks.md` to understand project status and priorities
2. Read `AGENTS.md` for development guidelines
3. Read `ARCHITECTURE.md` to understand the system design
4. Check existing issues and PRs to avoid duplicate work

### Making Changes

1. **Format your code** (required):
   ```bash
   cargo fmt --all
   ```

2. **Lint your code** (required):
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Build your code**:
   ```bash
   cargo build --all --locked
   ```

4. **Run tests**:
   ```bash
   # Standard cargo test
   cargo test --all-features --all-targets --locked
   
   # Or with nextest (preferred, mirrors CI)
   cargo nextest run --all-features --all-targets --locked
   cargo test --doc --locked  # Doc tests
   ```

5. **Run all checks** (shortcut):
   ```bash
   make check
   ```

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add custom reporter support
fix: correct throttling window expiry
docs: update REPORTERS.md with examples
test: add property tests for level filtering
chore: update dependencies
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

### Pull Requests

1. **Title**: Use conventional commit format
2. **Description**: 
   - Explain what and why
   - Reference related task numbers from `tasks.md`
   - Include before/after examples if relevant
3. **Tests**: Add tests for new behavior
4. **Documentation**: Update docs if behavior changes

Example PR description:
```markdown
## What

Implements Task 111: Snapshot tests for basic/fancy/box outputs

## Why

Ensures output formatting remains consistent across changes

## Changes

- Added snapshot tests for raw logging
- Added snapshot tests for multiple log types
- Updated test documentation

## Testing

All tests pass:
- `cargo test --all-features`
- Snapshot tests validate against baseline
```

## Feature Flags

consola-rs uses extensive feature gating. Test your changes with different feature combinations:

```bash
# Minimal build (no features)
cargo build --no-default-features

# WASM target
cargo build --features wasm

# JSON output
cargo build --features json

# All features (default for CI)
cargo build --all-features
```

See `FEATURE-FLAGS.md` for complete feature matrix.

## MSRV (Minimum Supported Rust Version)

**Rust 1.85** (edition 2024)

We use the latest stable Rust edition for modern language features. This is enforced in CI.

## Testing

### Test Organization

- **Unit tests**: Embedded in source files (`src/`)
- **Integration tests**: Separate test files (`tests/`)
- **Snapshot tests**: Using `insta` crate
- **Property tests**: Using `proptest` crate
- **Benchmarks**: In `benches/` directory

### Running Tests

```bash
# All tests
cargo test --all-features

# Specific test
cargo test --test snapshot_tests

# With nextest (faster, parallel)
cargo nextest run --all-features

# Doc tests (nextest doesn't run these)
cargo test --doc

# Update snapshots
INSTA_UPDATE=always cargo test --test snapshot_tests
```

### Writing Tests

Follow existing patterns:

```rust
#[test]
fn test_my_feature() {
    let mut logger = Logger::new(MemoryReporter::new());
    logger.log("info", None, ["test message"]);
    
    let captured = logger.reporter().get_records();
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].message.as_ref().unwrap(), "test message");
}
```

For snapshot tests:

```rust
#[test]
fn snapshot_my_output() {
    let output = generate_output();
    insta::assert_snapshot!("my_output", output);
}
```

## Code Style

### Guidelines

1. **No `unwrap()` / `expect()` outside tests**
   - Use `?` operator or proper error handling
   - Tests may use these for conciseness

2. **Avoid `unsafe` code**
   - Only for legitimate system calls (e.g., terminal size detection)
   - Document safety invariants

3. **Thread safety**
   - Use `parking_lot::RwLock` for shared state
   - Design for concurrent access

4. **Error handling**
   - Use `anyhow` for application errors
   - Use `thiserror` for library errors

5. **Performance**
   - Avoid allocations in hot paths
   - Use `SmallVec` for small collections
   - Profile before optimizing

### Formatting

We use `rustfmt` with default settings. CI will fail if code is not formatted.

### Linting

We use `clippy` with `-D warnings` (all warnings are errors). Fix all clippy warnings before submitting.

## Documentation

### Code Documentation

- Add doc comments to public APIs
- Include examples in doc comments
- Run `cargo doc --all-features` to verify

### Guides

When adding major features, update or create guides:
- `README.md`: User-facing overview
- `REPORTERS.md`: Custom reporter guide
- `PROMPTS.md`: Interactive prompts
- `ARCHITECTURE.md`: System design
- `BENCHMARKS.md`: Performance notes

## Continuous Integration

Our CI runs on:
- **Platforms**: Linux, macOS, Windows
- **Checks**: Format, clippy, build, test, audit, coverage

CI must pass before merging. Check the Actions tab for results.

## Development Tools

### Useful Commands

```bash
# Format check
make fmt-check

# Lint
make lint

# Build
make build

# Test
make test

# All checks (fmt + lint + test)
make check

# Install dev tools
make install-tools

# Watch tests (requires cargo-watch)
make watch
```

### Recommended Tools

- `cargo-nextest`: Faster test runner
- `cargo-watch`: Auto-run tests on changes
- `cargo-deny`: Security and license auditing
- `cargo-tarpaulin`: Code coverage
- `cargo-insta`: Snapshot testing

Install all at once:
```bash
make install-tools
```

## Security

If you discover a security vulnerability, please email security@example.com (TODO: add actual email) instead of opening a public issue.

## Questions?

- Check `tasks.md` for project status
- Read `AGENTS.md` for development guidelines
- Open an issue for discussion
- Join our Discord (TODO: add link if available)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to consola-rs! 🎉
