# Contributing to consola-rs

Thanks for your interest! Early-stage MVP.

## Workflow

1. Fork & branch from `main`.
1. Ensure `cargo fmt`, `cargo clippy` pass (warnings tolerated during MVP).
1. Add tests (nextest compatible) for new behavior.
1. Submit PR referencing related task numbers from `tasks.md`.

## Feature Flags

See `Cargo.toml` for available features. Prefer minimal default features.

## MSRV

Rust 1.85 (edition 2024).

## Testing

Use `cargo test`. (Future: `cargo nextest run`).

## Code Style

Avoid `unwrap()` / `expect()` outside tests.
