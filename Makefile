# Makefile for consola-rs development
# Requires: cargo, cargo-nextest (optional)

.PHONY: help
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

.PHONY: fmt
fmt: ## Format code with rustfmt
	cargo fmt --all

.PHONY: fmt-check
fmt-check: ## Check code formatting
	cargo fmt --all -- --check

.PHONY: lint
lint: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: build
build: ## Build the project
	cargo build --all --locked

.PHONY: build-release
build-release: ## Build release version
	cargo build --all --locked --release

.PHONY: test
test: ## Run tests (uses nextest if available, falls back to cargo test)
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		cargo nextest run --all-features --all-targets --locked; \
		cargo test --doc --locked; \
	else \
		cargo test --all --locked; \
	fi

.PHONY: test-fast
test-fast: ## Run tests without doc tests
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		cargo nextest run --all-features --all-targets --locked; \
	else \
		cargo test --lib --tests --locked; \
	fi

.PHONY: bench
bench: ## Run benchmarks
	cargo bench

.PHONY: doc
doc: ## Generate documentation
	cargo doc --all --no-deps --open

.PHONY: check
check: fmt-check lint test ## Run all checks (format, lint, test)

.PHONY: pre-commit
pre-commit: fmt lint test ## Run pre-commit checks
	@echo "All pre-commit checks passed!"

.PHONY: deny
deny: ## Run cargo-deny checks
	@if command -v cargo-deny >/dev/null 2>&1; then \
		cargo deny check; \
	else \
		echo "cargo-deny not installed. Install with: cargo install cargo-deny"; \
		exit 1; \
	fi

.PHONY: audit
audit: ## Run security audit
	cargo audit

.PHONY: install-tools
install-tools: ## Install development tools
	@echo "Installing development tools..."
	cargo install cargo-nextest --locked
	cargo install cargo-deny --locked
	cargo install cargo-audit --locked
	cargo install cargo-tarpaulin --locked
	@echo "Tools installed!"

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean

.PHONY: watch
watch: ## Watch for changes and run tests (requires cargo-watch)
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x 'clippy -- -D warnings' -x 'nextest run'; \
	else \
		echo "cargo-watch not installed. Install with: cargo install cargo-watch"; \
		exit 1; \
	fi

.PHONY: release
release: ## Create a release build and run the binary
	cargo build --release
	@echo "Release binary: target/release/consola"

.DEFAULT_GOAL := help
