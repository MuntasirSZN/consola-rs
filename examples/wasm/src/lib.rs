//! consola-rs browser / WASM example.
//!
//! This crate is built with `wasm-pack` and loaded from a browser HTML page.
//! `BrowserReporter` detects the browser environment and emits styled console
//! output via `console.log`, `console.warn`, and `console.error` with CSS badges.
//!
//! Build:    wasm-pack build --target web --release
//! Serve:    python3 -m http.server 8080
//! Open:     http://localhost:8080/index.html

use wasm_bindgen::prelude::wasm_bindgen;
use consola::{reporters::BrowserReporter, create_core_consola, log_levels};

/// Entry point — runs automatically when the WASM module loads.
#[wasm_bindgen(start)]
pub fn main() {
    // Show panic messages in the browser console
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let consola = create_core_consola(
        Some(log_levels::VERBOSE),
        vec![Box::new(BrowserReporter::new())],
    );

    // ── All log types ──────────────────────────────────────────────────────
    consola.fatal("Critical system failure: kernel panic");
    consola.error("Unhandled exception: TypeError: Cannot read properties of undefined");
    consola.warn("Slow query detected (2.3s): SELECT * FROM orders");
    consola.log("Server listening on http://localhost:3000");
    consola.info("Connected to PostgreSQL at localhost:5432");
    consola.success("Database migration completed (42/42)");
    consola.ready("Accepting connections on port 3000");
    consola.start("Building project...");
    consola.box_("v2.0.0 released! See https://github.com/example/releases");
    consola.debug("Cache lookup for key 'user:42': hit");
    consola.trace("List of 10 items");
    consola.verbose("WebSocket ping interval: 30s");

    // ── Tagged output ──────────────────────────────────────────────────────
    let api = consola.with_tag("api");
    api.info("GET /api/v1/products 200 OK");
    api.warn("POST /api/v1/checkout 429 Too Many Requests");
    api.error("PUT /api/v1/profile 500 Internal Server Error");

    // ── Fail ───────────────────────────────────────────────────────────────
    consola.fail("Deploy failed: health check returned 503");
}
