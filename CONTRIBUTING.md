# Contributing to consola-rs

Thanks for your interest in contributing to consola-rs! This is an early-stage MVP project aiming for feature parity with the JavaScript [@unjs/consola](https://github.com/unjs/consola) library.

## Quick Start

1. Fork & branch from `main`.
1. Ensure `cargo fmt`, `cargo clippy` pass (warnings tolerated during MVP).
1. Add tests (nextest compatible) for new behavior.
1. Submit PR referencing related task numbers from `tasks.md`.

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

1. **Avoid `unsafe` code**

   - Only for legitimate system calls (e.g., terminal size detection)
   - Document safety invariants

1. **Thread safety**

   - Use `parking_lot::RwLock` for shared state
   - Design for concurrent access

1. **Error handling**

   - Use `anyhow` for application errors
   - Use `thiserror` for library errors

1. **Performance**

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
just fmt-check

# Lint
just lint

# Build
just build

# Test
just test

# All checks (fmt + lint + test)
just check

# Install dev tools
just install-tools

# Watch tests (requires cargo-watch)
just watch
```

### Recommended Tools

- `cargo-nextest`: Faster test runner
- `cargo-watch`: Auto-run tests on changes
- `cargo-deny`: Security and license auditing
- `cargo-llvm-cov`: Code coverage
- `cargo-insta`: Snapshot testing

Install all at once:

```bash
just install-tools
```

## Security

If you discover a security vulnerability, please email <muntasir.joypurhat@gmail.com> instead of opening a public issue. See [SECURITY.md](./SECURITY.md) file for more info.

## Questions?

- Check `tasks.md` for project status
- Read `AGENTS.md` for development guidelines
- Open an issue for discussion
- Join our Discord (TODO: add link if available)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

______________________________________________________________________

Thank you for contributing to consola-rs! 🎉
