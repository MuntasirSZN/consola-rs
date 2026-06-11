//! ─── error.rs ────────────────────────────────────────────────────────────────
//! Demonstrates error logging with automatic backtrace capture — matching the
//! consola-js `error.ts` example.
//!
//! When the `backtrace` feature is enabled (default), every `consola.error()` and
//! `consola.fatal()` call captures a real Rust stack backtrace at the call site.
//! The backtrace is printed below the message, showing file, line, and function.
//!
//! Run with:  cargo run --example error

use consola::{Consola, create_fancy_consola, log_levels};

/// Logs at error level — backtrace will point to the origin of failure inside
/// this function, not just to the caller in main.
fn load_config(consola: &Consola, path: &str) {
    if path.is_empty() {
        consola.error("Configuration error: empty path provided");
    } else if !path.ends_with(".toml") {
        consola.error(&format!(
            "Configuration error: unsupported extension for '{}': expected .toml",
            path,
        ));
    } else if let Err(msg) = read_file(path) {
        // Backtrace goes through read_file → load_config → main
        consola.fatal(&msg);
    } else {
        consola.success("Config loaded successfully");
    }
}

/// A deeper function — backtrace will include this frame when calling error().
fn read_file(path: &str) -> Result<(), String> {
    Err(format!("no such file or directory: '{}'", path))
}

fn main() {
    let consola = create_fancy_consola(Some(log_levels::VERBOSE));

    // ── Simple error — backtrace shows main at the call site ─────────────────
    consola.error("This is an example error. Everything is fine!");

    // ── Error from within a deep function ────────────────────────────────────
    // Backtrace shows: read_file → load_config → main (the actual failure path).
    load_config(&consola, "config.toml");

    // ── Multiple call types ──────────────────────────────────────────────────
    load_config(&consola, "config.yaml");
    load_config(&consola, "");

    // ── Error in a tagged scope ──────────────────────────────────────────────
    consola
        .with_tag("http")
        .error("POST /api/checkout 500: Internal Server Error");

    // ── Fatal + fail sequence ────────────────────────────────────────────────
    consola.fatal("Fatal: database connection lost");
    consola.fail("Deploy failed: health check returned 503");
}
