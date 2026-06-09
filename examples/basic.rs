//! ─── basic.rs ────────────────────────────────────────────────────────────────
//! Demonstrates core `consola` logging API: levels, tags, defaults, pause/resume,
//! and mock functionality.

use consola::{LogType, create_fancy_consola, log_levels};

fn main() {
    // ── Create a fancy consola with VERBOSE level ─────────────────────────────
    let consola = create_fancy_consola(Some(log_levels::VERBOSE));

    // ── Log at different levels ───────────────────────────────────────────────
    consola.info("This is an info message");
    consola.warn("This is a warning");
    consola.error("This is an error");
    consola.success("Operation completed successfully");
    consola.fail("Operation failed");
    consola.debug("Debug detail: checking connection pool");
    consola.trace("Trace: entered function do_work");
    consola.verbose("Verbose: configuration loaded");
    consola.log("Plain log message");
    consola.fatal("Fatal: unrecoverable state");
    consola.ready("Application is ready");
    consola.start("Starting background workers");
    consola.box_("Boxed notice for the user");

    // ── Raw variants (no badge / icon enrichment) ─────────────────────────────
    consola.info_raw("Raw info — no badge");
    consola.error_raw("Raw error — no enrichment");

    // ── with_tag ──────────────────────────────────────────────────────────────
    let db = consola.with_tag("db");
    db.info("Connected to database");
    db.warn("Slow query detected (>500ms)");

    let http = consola.with_tag("http");
    http.info("GET /api/users 200 OK");
    http.error("POST /api/login 500 Internal Server Error");

    // ── with_defaults ─────────────────────────────────────────────────────────
    let verbose_consola = consola.with_defaults(
        consola::LogObjectInput::new()
            .message("default prefix")
            .tag("defaults-demo"),
    );
    verbose_consola.info("This message inherits tag and default message");
    verbose_consola.warn("Tags and defaults can still be overridden per-call");

    // ── pause / resume ────────────────────────────────────────────────────────
    consola.pause_logs();
    consola.info("This message is queued while paused");
    consola.warn("This warning is also queued");
    println!("  [Logging is paused — queued 2 messages]");
    consola.resume_logs(); // the two queued messages are flushed now

    // ── mock_types (stub — replaces log methods with a test double) ───────────
    consola.mock_types(|ty: LogType, input: &consola::LogObjectInput| {
        // In a test scenario you might suppress certain types or capture them.
        // Here we let everything through.
        eprintln!(
            "  [mock] {}: {}",
            ty.as_str(),
            input.message.as_deref().unwrap_or("")
        );
        true // returning true means "allow this message through"
    });
    consola.info("This message was processed by the mock");
    consola.warn("And this one too");

    // Reset mock by calling mock_types with a passthrough — or just stop mocking
    // by not installing a mock.  There is no separate `clear_mock` method;
    // passing a transparent filter effectively resets behaviour.
    consola.mock_types(|_ty, _input| true);

    // ── log_obj for structured logging ────────────────────────────────────────
    consola.log_obj(
        &consola::LogObjectInput::new()
            .message("Structured log entry")
            .tag("api"),
    );
}
