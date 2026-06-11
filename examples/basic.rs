//! ─── basic.rs ────────────────────────────────────────────────────────────────
//! Demonstrates all consola log types, tags, formatting, and options — matching
//! the consola-js `basic.ts`, `sample.ts`, and `special.ts` examples.

use consola::{create_fancy_consola, log_levels};

fn main() {
    let consola = create_fancy_consola(Some(log_levels::VERBOSE));

    // ── All log types with realistic messages ─────────────────────────────────
    consola.warn("A new version of consola is available: 3.0.1");
    consola.error("This is an example error. Everything is fine!");
    consola.info("Using consola 3.0.0");
    consola.start("Building project...");
    consola.success("Project built!");
    consola.log("Starting development server...");
    consola.ready("Server listening on http://localhost:3000");
    consola.debug("GET /api/users 200 OK (42ms)");
    consola.trace("renderComponent: entering with props { id: 12 }");
    consola.verbose("Cache lookup for key 'user:12': miss");
    consola.fatal("Uncaught TypeError: Cannot read properties of undefined");
    consola.fail("Deploy failed: health check returned 503");

    // ── Tags ──────────────────────────────────────────────────────────────────
    let http = consola.with_tag("http");
    http.info("GET /api/users 200 OK");
    http.warn("POST /api/login 429 Too Many Requests");
    http.error("PUT /api/profile 500 Internal Server Error");

    let db = consola.with_tag("db");
    db.info("Connected to PostgreSQL at localhost:5432");
    db.warn("Slow query detected (2.3s): SELECT * FROM orders");
    db.error("Connection pool exhausted (max: 20)");

    let auth = consola.with_tag("auth");
    auth.info("User admin authenticated via OAuth2");
    auth.warn("Failed login attempt for user 'root' from 192.168.1.100");

    // ── Nested tags (joined with ':') ─────────────────────────────────────────
    let svc = consola.with_tag("api").with_tag("v1");
    svc.info("GET /api/v1/products 200 OK");
    svc.error("POST /api/v1/checkout 400 Bad Request");
    svc.log("Rate limit: 142/1000 requests used this minute");

    // ── Raw variants ──────────────────────────────────────────────────────────
    consola.log("consola.log.raw() bypasses formatting:");
    consola.log_raw("  { \"message\": \"raw JSON payload\" }");
    consola.error_raw("  raw error string without badge enrichment");

    // ── Backtick highlighting (`text` → cyan) ─────────────────────────────────
    consola.log("We can `monospace` keywords using grave accent characters!");
    consola.info("Update available for `consola`: `v1.0.2` → `v2.0.0`");
    consola.log("Run `npm install -g consola` to update");
    consola.warn("The `format` option is deprecated, use `formatOptions` instead");

    // ── Underline highlighting (_text_ → underlined) ──────────────────────────
    consola.log("We can also _underline_ words but not_this or");
    consola.log("this should_not_be_underlined!");
    consola.info("Please _save_ your work before _exiting_");

    // ── Structured (JSON-like) arguments ──────────────────────────────────────
    consola.log_obj(
        &consola::LogObjectInput::new()
            .type_(consola::LogType::Info)
            .message("User profile")
            .arg("{\"name\": \"Cat\", \"color\": \"#454545\"}"),
    );
    consola.log_obj(
        &consola::LogObjectInput::new()
            .type_(consola::LogType::Error)
            .message("Validation failed")
            .arg("{\"field\": \"email\", \"code\": \"INVALID_FORMAT\"}"),
    );
    consola.log_obj(
        &consola::LogObjectInput::new()
            .type_(consola::LogType::Log)
            .message("Config loaded")
            .arg("{\"host\": \"localhost\", \"port\": 3000, \"tls\": true}"),
    );

    // ── Multi-line messages ───────────────────────────────────────────────────
    consola.log("`Hello` the `JS`\n`World` and `Beyond`!");

    // ── Error with message and stack ──────────────────────────────────────────
    consola.error("CustomError: Something went wrong\n  at main.rs:42:5\n  at lib.rs:120:1\n  at processTicksAndRejections (native:7:39)");

    // ── Pause / Resume ────────────────────────────────────────────────────────
    consola.pause_logs();
    consola.info("This message is queued while paused");
    consola.warn("This warning is also queued");
    println!("  [Logging is paused — 2 messages queued]");
    consola.resume_logs(); // Flushes both queued messages

    // ── with_defaults ─────────────────────────────────────────────────────────
    let verbose = consola.with_defaults(consola::LogObjectInput::new().tag("background"));
    verbose.info("Scheduled job completed: report generation");
    verbose.warn("Job finished with 3 warnings");
}
