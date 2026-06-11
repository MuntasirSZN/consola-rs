#![cfg(all(target_arch = "wasm32", feature = "browser"))]

//! Browser WASM integration tests — verify styled browser console output.
//!
//! Run:  wasm-pack test --chrome --features browser  (or --firefox, --safari, --headless)
//!
//! In browser mode, `BrowserReporter::format()` returns an empty string
//! because output is emitted directly via `console.log`/`warn`/`error`.
//! These tests verify that the reporter detects the browser environment
//! and never panics.

use consola::reporters::BrowserReporter;
use consola::types::{ConsolaOptions, LogContext, LogObject, Reporter};
use consola::{LogType, create_core_consola, log_levels};
use std::sync::Arc;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn browser_env_detected() {
    let r = BrowserReporter::new();
    assert!(
        r.browser,
        "BrowserReporter should detect browser environment"
    );
}

#[wasm_bindgen_test]
fn browser_format_returns_empty_in_browser() {
    let r = BrowserReporter::new();
    let mut log_obj = LogObject::new(LogType::Error);
    log_obj.args = vec!["something".into()];
    let ctx = LogContext {
        options: Arc::new(ConsolaOptions::default()),
    };
    let result = r.format(&log_obj, &ctx).unwrap();
    // In browser mode, format() returns empty — output goes via console.*
    assert!(
        result.is_empty(),
        "Browser mode format should return empty string"
    );
}

#[wasm_bindgen_test]
fn consola_error_does_not_panic() {
    let consola = create_core_consola(
        Some(log_levels::VERBOSE),
        vec![Box::new(BrowserReporter::new())],
    );
    consola.error("test error");
    consola.warn("test warn");
    consola.log("test log");
    consola.info("test info");
    consola.success("test success");
    consola.fatal("test fatal");
}

#[wasm_bindgen_test]
fn consola_tagged_output() {
    let consola = create_core_consola(
        Some(log_levels::VERBOSE),
        vec![Box::new(BrowserReporter::new())],
    );
    let tagged = consola.with_tag("api");
    tagged.info("GET /api/test 200 OK");
    tagged.error("POST /api/test 500");
}
