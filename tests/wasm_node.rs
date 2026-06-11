#![cfg(target_arch = "wasm32")]

//! Node.js WASM integration tests — verify text fallback formatting.
//!
//! Run:  wasm-pack test --node
//!
//! In Node.js, `web_sys::window()` returns `None`, so `BrowserReporter`
//! falls back to plain `[type] message` text formatting (no CSS styling,
//! no console.* calls). These tests verify the fallback output is correct.

use consola::reporters::BrowserReporter;
use consola::types::{ConsolaOptions, LogContext, LogObject, Reporter};
use consola::{LogType, create_core_consola, log_levels};
use std::sync::Arc;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!();

#[wasm_bindgen_test]
fn browser_not_detected_in_node() {
    let r = BrowserReporter::new();
    assert!(!r.browser, "Browser should NOT be detected in Node.js");
}

#[wasm_bindgen_test]
fn browser_format_fallback_text() {
    let r = BrowserReporter::new();
    let log_obj = LogObject::new(LogType::Error);
    let ctx = LogContext {
        options: Arc::new(ConsolaOptions::default()),
    };
    let result = r.format(&log_obj, &ctx).unwrap();
    // Non-browser mode: format() returns the text
    assert_eq!(result, "[error] ");
}

#[wasm_bindgen_test]
fn browser_format_with_tag_and_type() {
    let r = BrowserReporter::new();
    let log_obj = LogObject {
        level: 2,
        r#type: LogType::Info,
        tag: "test".into(),
        message: None,
        additional: None,
        args: vec!["hello world".into()],
        timestamp_ms: 0,
        title: None,
        badge: false,
        icon: None,
        style: None,
        error: None,
    };
    let ctx = LogContext {
        options: Arc::new(ConsolaOptions::default()),
    };
    let result = r.format(&log_obj, &ctx).unwrap();
    assert_eq!(result, "[test:info] hello world");
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
