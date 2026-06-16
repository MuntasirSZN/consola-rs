# Contributing to consola-rs

## Getting Started

**Prerequisites**: Rust 1.88+ (edition 2024), `just` command runner.

```bash
git clone https://github.com/<your-username>/consola-rs
cd consola-rs
just --list           # see all available recipes
just fmt-check        # verify formatting
just build            # build workspace
just test             # run tests
```

## Development

The project uses `just` as a command runner. All common tasks map to just recipes:

| Recipe            | Description                                         |
|-------------------|-----------------------------------------------------|
| `just fmt`        | Format with rustfmt                                 |
| `just fmt-check`  | Check formatting                                    |
| `just lint`       | Clippy (all features)                               |
| `just lint-features`| Clippy per-feature (matches CI)                   |
| `just lint-fix`   | Clippy autofix                                      |
| `just build`      | Build workspace (`--locked`)                        |
| `just build-release`| Build release                                     |
| `just test`       | Nextest + doc tests                                 |
| `just check-features`| Check per-feature compilation (matches CI)       |
| `just docs-ci`    | Build docs for all feature combos (CI)              |
| `just coverage-ci`| Coverage: nextest + doctest → lcov (CI)             |
| `just deny`       | cargo-deny audit                                    |
| `just audit`      | cargo-audit                                         |
| `just bench`      | Divan benchmarks                                    |
| `just bench-ci`   | Run CodSpeed benchmarks                             |
| `just bench-build`| Build CodSpeed benchmark targets                    |
| `just publish`    | Publish to crates.io                                |
| `just ci-check`   | Full CI check: fmt, lint-features, build, test, docs, deny |
| `just check`      | Pre-submit check: fmt, lint, test                   |
| `just pre-commit` | fmt + lint + test + deny                            |
| `just watch`      | Watch files and re-run clippy + nextest             |

### WASM tests

```bash
cargo install wasm-pack
wasm-pack test --headless --chrome --features browser
wasm-pack test --node --features browser
```

## CI

Pull request CI runs these jobs (defined in `.github/workflows/ci.yml`):

- Format (`just fmt-check`)
- Clippy on 3 OS (`just lint-features`)
- Tests on 3 OS (`just build` + `just test`)
- WASM tests (Chrome + Node)
- Per-feature compilation on 3 OS (`just check-features`)
- Documentation (`just docs-ci`)
- Code coverage (`just coverage-ci`)
- Security audit via cargo-deny
- CodeQL, spell check, commit lint, PR title validation, workflow security
- Benchmarks via CodSpeed

## Code Style

- `#![deny(unsafe_code)]` — no `unsafe` in the core library
- All public items must have doc comments (`#![warn(missing_docs)]`)
- Follow standard Rust formatting (`just fmt`)
- Use `Result` types, avoid panics
- Keep dependencies minimal; use optional features over required deps

## Pull Requests

1. Branch from `main`
2. Keep commits small and descriptive
3. Run `just check` and `just deny` before submitting
4. PR title must follow Conventional Commits: `type(scope): description`
   - Types: `feat`, `fix`, `chore`, `docs`, `test`, `refactor`, `style`
   - Scope optional (e.g. `consola`, `reporters`, `util`, `prompt`)
5. CI must pass

## Testing

- Unit tests alongside the code under test
- WASM tests via `wasm-pack test`
- Benchmarks use `codspeed-divan-compat` (`#[divan::bench]`). Don't worry about CodSpeed, only for CI. Will work locally.
- Test utility: `CaptureReporter` in `src/consola.rs` records formatted output

## Architecture

- **`Consola`** — main thread-safe logger (`parking_lot::Mutex`). Methods: log types, reporters, tags, pause/resume, instance derivation.
- **`Reporter` trait** — `format(&self, &LogObject, &LogContext) -> Result<String, String>` + `clone_box()`. Built-in: `FancyReporter` (colored), `BasicReporter` (plain).
- **`LogObject`** — fully resolved log entry passed to reporters. Fields: level, type, tag, message, args, timestamp, title, badge, icon, style, error.
- **`LogObjectInput`** — builder for partial log input. Builder methods: `type_()`, `tag()`, `message()`, `args()`, `arg()`, `additional()`, `title()`.
- **`ConsolaOptions`** — instance config: reporters list, log level, defaults, throttle settings, format options.
- **`FormatOptions`** — output formatting: columns, date, colors, compact, error_level.
- Prompt types: `TextPromptOptions`, `ConfirmPromptOptions`, `SelectPromptOptions`, `MultiSelectOptions`, `SelectOption`.
- Log/tracing integration: `log::Log` and `tracing::Subscriber` impls under feature flags.
