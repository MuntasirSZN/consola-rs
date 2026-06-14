//! ─── json.rs ─────────────────────────────────────────────────────────────────
//! Demonstrates a JSON-logging reporter, mirroring the consola-js JSON example:
//!
//! ```ts
//! import { createConsola } from "consola";
//! const consola = createConsola({
//!   reporters: [{ log: (logObj) => { console.log(JSON.stringify(logObj)); } }],
//! });
//! consola.log("foo bar");
//! ```

use consola::{
    create_consola, log_levels,
    types::{ErrorInfo, LogContext, LogObject, Reporter},
};

// ── JSON Reporter ─────────────────────────────────────────────────────────────

/// A reporter that serializes every log object as a single-line JSON entry.
///
/// Compatible with log aggregators (ELK, Datadog, etc.) that consume
/// newline-delimited JSON (NDJSON).
#[derive(Debug, Clone)]
struct JsonReporter;

impl Reporter for JsonReporter {
    fn format(
        &self,
        log_obj: &LogObject,
        _ctx: &LogContext,
    ) -> Result<String, consola::error::ConsolaError> {
        let obj = serde_json::json!({
            "level": log_obj.level,
            "type": log_obj.r#type.as_str(),
            "tag": log_obj.tag,
            "message": log_obj.message,
            "additional": log_obj.additional,
            "args": log_obj.args,
            "timestamp_ms": log_obj.timestamp_ms,
            "title": log_obj.title,
            "badge": log_obj.badge,
            "icon": log_obj.icon,
            "style": log_obj.style,
            "error": log_obj.error.as_ref().map(error_to_json),
        });
        serde_json::to_string(&obj)
            .map_err(|e| consola::error::ConsolaError::Reporter(e.to_string()))
    }

    fn clone_box(&self) -> Box<dyn Reporter> {
        Box::new(self.clone())
    }
}

/// Recursively convert an `ErrorInfo` into a JSON value.
fn error_to_json(err: &ErrorInfo) -> serde_json::Value {
    serde_json::json!({
        "message": err.message,
        "stack": err.stack,
        "backtrace": err.backtrace,
        "cause": err.cause.as_ref().map(|c| error_to_json(c)),
    })
}

// ── Truncated JSON Reporter ───────────────────────────────────────────────────

/// A reporter that omits large or internal fields — useful when you only need
/// the essential log fields in JSON.
#[derive(Debug, Clone)]
struct CompactJsonReporter;

impl Reporter for CompactJsonReporter {
    fn format(
        &self,
        log_obj: &LogObject,
        _ctx: &LogContext,
    ) -> Result<String, consola::error::ConsolaError> {
        let obj = serde_json::json!({
            "type": log_obj.r#type.as_str(),
            "tag": log_obj.tag,
            "message": log_obj.message,
            "args": log_obj.args,
            "timestamp_ms": log_obj.timestamp_ms,
        });
        serde_json::to_string(&obj)
            .map_err(|e| consola::error::ConsolaError::Reporter(e.to_string()))
    }

    fn clone_box(&self) -> Box<dyn Reporter> {
        Box::new(self.clone())
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    // Single JSON reporter – closest to the consola-js example.
    let consola = create_consola(
        Some(log_levels::VERBOSE),
        vec![Box::new(JsonReporter) as Box<dyn Reporter>],
    );

    consola.log("foo bar");
    consola.info("Server listening on port 3000");
    consola.warn("Disk usage at 87%");
    consola.error("Connection refused: timeout after 30s");
    consola.info_raw("raw message bypasses formatting but still yields JSON");

    // With tags.
    let http = consola.with_tag("http");
    http.info("GET /api/users 200 OK");
    http.warn("POST /api/login 429 Too Many Requests");

    // With structured log input.
    consola.log_obj(
        &consola::LogObjectInput::new()
            .type_(consola::LogType::Info)
            .tag("db")
            .message("Query complete")
            .arg("duration_ms: 3.2"),
    );

    // An error-level log with automatic backtrace capture (requires default
    // features — `backtrace` feature enabled).
    consola.fatal("Segmentation fault: attempted to access null pointer");

    println!();
    println!("── Compact JSON reporter ──");

    // Compact reporter — fewer fields, easier to read.
    let compact = create_consola(
        Some(log_levels::VERBOSE),
        vec![Box::new(CompactJsonReporter) as Box<dyn Reporter>],
    );

    compact.info("started");
    compact.log_obj(
        &consola::LogObjectInput::new()
            .type_(consola::LogType::Info)
            .tag("http")
            .message("GET /api/health")
            .arg("status: 200"),
    );
}
