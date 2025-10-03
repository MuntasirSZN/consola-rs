# justfile for changelogen-rs development

tools := "cargo-nextest cargo-deny cargo-audit cargo-llvm-cov cargo-watch cargo-codspeed cargo-insta"

# Commands

format := "cargo fmt --all"
clippy := "cargo clippy --all-targets --all-features"
coverage := "cargo llvm-cov --all-features --workspace"
build := "cargo build --all --locked"
nextest := "cargo nextest run --all-features --locked"

# Default recipe (shows help)
_default:
    @just --list

# Format code with rustfmt
fmt:
    {{ format }}

# Check code formatting
fmt-check:
    {{ format }} -- --check

# Run clippy linter
lint:
    {{ clippy }} -- -D warnings

# Run clippy linter and autofix
lint-fix:
    {{ clippy }} --fix -- -D warnings

# Build the project
build:
    {{ build }}

# Build release version
build-release:
    {{ build }} --release

# Run tests with nextest
test:
    {{ nextest }}
    cargo test --doc --locked

# Run tests without doc tests
test-fast:
    {{ nextest }}

# Run benchmarks (uses CodSpeed)
bench:
    cargo codspeed build
    cargo codspeed run

# Generate documentation
doc:
    cargo doc --all --no-deps --open

# Run all checks (format, lint, test)
check: fmt-check lint test

# Run pre-commit checks
pre-commit: fmt lint test deny
    @echo "{{ GREEN + BOLD }}✅ All pre-commit checks passed!{{ NORMAL }}"

# Run cargo-deny checks
deny:
    cargo deny check

# Run security audit
audit:
    cargo audit

# Install development tools using cargo-binstall
install-tools:
    @echo "{{ BLUE + BOLD }}Installing development tools...{{ NORMAL }}"
    cargo binstall -y {{ tools }} || cargo install cargo-binstall && cargo binstall -y {{ tools }}
    @echo "{{ GREEN + BOLD }}✅ Tools installed!{{ NORMAL }}"

# Install git pre-commit hook (run with just pre-commit)
install-hook:
    @echo -e "#!/bin/sh\njust pre-commit" > .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    @echo "{{ GREEN + BOLD }}✅ Pre-commit hook installed!{{ NORMAL }}"

# Clean build artifacts
clean:
    cargo clean

# Generate coverage report (text summary)
coverage:
    {{ coverage }}

# Generate HTML coverage report and open in browser
coverage-html:
    {{ coverage }} --html --open

# Generate lcov.info for Codecov (matches CI workflow)
coverage-lcov:
    {{ coverage }} --lcov --output-path lcov.info
    @echo "{{ GREEN + BOLD }}✅ Coverage report saved to lcov.info{{ NORMAL }}"

# Clean coverage data
coverage-clean:
    cargo llvm-cov clean

# Watch for changes and run tests
watch:
    cargo watch -x 'clippy -- -D warnings' -x 'nextest run'

# Create a release build and run the binary
release:
    cargo build --release
    @echo "{{ BLUE + BOLD }}Release binary:{{ NORMAL }} {{ UNDERLINE + CYAN }}target/release/changelogen{{ NORMAL }}"
