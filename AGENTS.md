# AGENTS.md - Coding Agent Onboarding Guide

## Repository Overview

consola-rs is a Rust + WASM port of [unjs/consola](https://github.com/unjs/consola), providing elegant console logging for both native Rust and browser environments. This is an early-stage MVP project (version 0.0.0-alpha.0) targeting feature parity with the JavaScript version while adding Rust-native capabilities.

**Project Type**: Library crate\
**Languages**: Rust (edition 2024, MSRV 1.85)\
**Target Runtimes**: Native Rust + WebAssembly (WASM)\
**Repository Size**: ~9 source files, ~30 test files, comprehensive feature system

## Build & Validation Commands

### Prerequisites

- Rust 1.85+ (edition 2024)
- cargo-nextest automatically installed in CI via taiki-e/install-action
- `just` (optional, provides convenient shortcuts)

### Core Commands (ALWAYS run these in order)

```bash
# 1. Format check (REQUIRED - CI enforces this)
cargo fmt --all -- --check
# OR: just fmt-check

# 2. Linting (REQUIRED - CI fails on warnings)
cargo clippy --all-targets --all-features -- -D warnings
# OR: just lint

# 3. Build all targets
cargo build --all --locked
# OR: just build

# 4. Run tests (standard)
cargo test --all-features -- --nocapture

# 5. Run tests (preferred - nextest, mirrors CI)
cargo install cargo-nextest  # Only needed once
cargo nextest run --all-features --all-targets --locked
cargo test --doc --locked  # Doc tests (nextest doesn't run these)
# OR: just test (auto-detects nextest, falls back to cargo test)

# 6. Run all checks (format + lint + test)
just check

# 7. Install all development tools
just install-tools
```

**Timing**: Full CI pipeline takes ~60-90 seconds. cargo-nextest installation via `cargo install` takes ~6 minutes on first run (CI uses cached install-action).

### Feature Combinations

The project uses extensive feature gating. Test these builds when modifying dependencies or features:

```bash
# Minimal build (no features)
cargo build --no-default-features

# WASM target (adds wasm-bindgen)  
cargo build --features wasm

# All features (default for CI)
cargo build --all-features
```

**Default features**: `["color", "fancy"]`
**Available features**: `async-reporters`, `bridge-log`, `bridge-tracing`, `color`, `fancy`, `json`, `prompt-demand`, `wasm`

### Additional Tools & Workflows

The project includes several auxiliary CI workflows:

- **Commit linting** (committed.yml): Enforces conventional commit messages on PRs
- **Spell checking** (typos.yml): Catches typos in documentation and code
- **Scheduled runs**: CI runs weekly on Sundays to catch dependency issues

Use the Makefile for local development convenience:

```bash
just help           # Show all available targets
just pre-commit     # Run format + lint + test (mirrors CI)
just watch          # Auto-run tests on file changes (requires cargo-watch)
just install-tools  # Install all dev dependencies (nextest, deny, audit, tarpaulin)
```

## Project Architecture & Layout

### Source Structure (`src/`)

- **`lib.rs`**: Main facade, re-exports all modules
- **`levels.rs`**: Log level constants and type registry (global state with RwLock)
- **`record.rs`**: LogRecord struct and argument handling
- **`throttling.rs`**: Message deduplication and repetition counting
- **`reporter.rs`**: BasicReporter, FancyReporter, JsonReporter implementations
- **`format.rs`**: Formatting pipeline, segments, styling
- **`error_chain.rs`**: Error source chain extraction with cycle detection
- **`utils.rs`**: Box drawing, tree formatting, alignment helpers
- **`clock.rs`**: Clock abstraction for deterministic testing

### Test Organization (`tests/`)

- **Integration tests**: Separate files per major component
- **Snapshot tests**: Uses `insta` crate for output validation (strips ANSI)
- **Property tests**: Some use `proptest` for randomized testing
- **WASM tests**: Gated behind `wasm-bindgen-test` (not run in CI currently)

### Key Configuration Files

- **`Cargo.toml`**: Complex feature matrix, see `[features]` section
- **`.github/workflows/ci.yml`**: Multi-platform CI (Linux, macOS, Windows) with separate jobs for fmt, clippy, test, audit, coverage
- **`.github/workflows/committed.yml`**: Commit message linting (pull requests only)
- **`.github/workflows/typos.yml`**: Spell checking (pull requests only)
- **`Makefile`**: Convenient development shortcuts (fmt, lint, test, etc.)
- **`deny.toml`**: cargo-deny configuration for security/license auditing
- **`CONTRIBUTING.md`**: MVP-focused workflow, nextest preferred
- **`tasks.md`**: Comprehensive development roadmap (460+ tasks)
- **`SPEC.md`**: Technical specification and design notes

## Common Issues & Workarounds

### Build Issues

1. **Missing cargo-nextest**: Install with `cargo install cargo-nextest` (takes 5-6 minutes)
1. **Feature conflicts**: Always use `--all-features` for consistency with CI
1. **MSRV violations**: Project requires Rust 1.85+ (edition 2024)

### Test Issues

1. **Broken pipe errors**: When running with `| head` or similar, ignore these - tests still pass
1. **ANSI in snapshots**: Snapshot tests strip ANSI automatically, colored output is expected in logs
1. **Timing-dependent tests**: Use MockClock for deterministic timestamps

### Environment Variables

- **`CONSOLA_LEVEL`**: Controls log level filtering (debug, info, warn, error)
- **`NO_COLOR`**: Disables ANSI color output (handled by anstream)
- **`FORCE_COLOR`**: Forces color output

## Validation Pipeline

### CI Checks (must pass)

The CI pipeline runs as separate jobs (use Swatinem/rust-cache for faster builds):

1. **Format Job** (`fmt`):

   - `cargo fmt --all -- --check` (zero tolerance)

1. **Clippy Job** (`clippy`):

   - `cargo clippy --all-targets --all-features -- -D warnings` (zero warnings)

1. **Test Job** (`test`) - Matrix across Linux, macOS, Windows:

   - `cargo build --all --locked`
   - `cargo nextest run --all-features --all-targets --locked` (using taiki-e/install-action)
   - `cargo test --doc --locked`

1. **Audit Job** (`audit`):

   - `cargo deny check` (via EmbarkStudios/cargo-deny-action)

1. **Coverage Job** (`coverage`) - Linux only:

   - `cargo +nightly tarpaulin` with llvm engine
   - Uploads to Codecov (requires `CODECOV_TOKEN` secret)

### Manual Verification Steps

```bash
# Verify feature combinations work
cargo check --no-default-features
cargo check --features wasm
cargo check --features json,prompt-demand

# Test environment variable handling
CONSOLA_LEVEL=debug cargo test --test utils_tests
NO_COLOR=1 cargo test --test fancy_tests

# Run security/license checks locally
just deny  # Requires cargo-deny

# Run all pre-commit checks
just pre-commit
```

## Development Workflow Notes

### Code Style Requirements

- **No `unwrap()`/`expect()`** outside tests (enforced by future lint)
- **Thread-safe by default**: Uses parking_lot::RwLock for global state
- **Error handling**: Uses `anyhow` and `thiserror`, proper error chains
- **Feature gates**: Prefer minimal dependencies, most integrations are optional

### Testing Strategy

- **Unit tests**: Embedded in `src/lib.rs` and individual modules
- **Integration tests**: Comprehensive test files in `tests/`
- **Snapshot tests**: For output format validation (colored + plain text)
- **Property tests**: For randomized input validation
- **Mock infrastructure**: MockClock, TestSink for deterministic testing

### WASM Considerations

- **WASM builds work** but have limited interactive capabilities
- **Prompt features disabled** in WASM (runtime error if called)
- **Test with**: `cargo build --features wasm` (adds wasm-bindgen)

## Key Facts for Agents

1. **Trust these instructions**: They are validated and current. Only search if instructions are incomplete or incorrect.

1. **Always run format/clippy first**: CI has zero tolerance for style violations.

1. **Use nextest when available**: Mirrors CI more closely than `cargo test`. CI uses taiki-e/install-action for fast installation.

1. **Multi-platform CI**: Tests run on Linux, macOS, and Windows. Ensure compatibility across platforms.

1. **Feature combinations matter**: Test at least default features and `--no-default-features`.

1. **MVP status**: Some features are incomplete (see tasks.md), focus on working functionality.

1. **Thread safety**: Global type registry uses RwLock, design for concurrent access.

1. **Performance conscious**: Uses smallvec, blake3 for fingerprinting, optimized for logging hot paths.

1. **Extensive documentation**: README, SPEC.md, tasks.md, and CONTRIBUTING.md contain detailed context.

1. **Security/License auditing**: cargo-deny runs in CI. Test locally with `just deny` before committing.

1. **Code coverage**: Tracked via tarpaulin + Codecov (nightly Rust, Linux only).

## File Inventory (Root Level)

```
├── Cargo.lock                       # Dependency lock file
├── Cargo.toml                       # Main project configuration
├── justfile                         # Development shortcuts (just help)
├── deny.toml                        # cargo-deny security/license config
├── CONTRIBUTING.md                  # Development workflow
├── LICENSE                          # MIT license
├── README.md                        # Project overview
├── SPEC.md                          # Technical specification
├── tasks.md                         # Development roadmap (460+ tasks)
├── src/                             # Source code (9 modules)
├── tests/                           # Integration tests (5 test files + snapshots)
├── .github/workflows/
│   ├── ci.yml                       # Main CI (multi-platform)
│   ├── committed.yml                # Commit linting (PRs only)
│   └── typos.yml                    # Spell checking (PRs only)
├── .gitignore                       # Git ignore (just /target)
└── .vscode/settings.json            # VS Code configuration
```

This file provides everything needed to work efficiently with consola-rs. The project is well-structured but in active development - focus on the working features and existing test coverage.
