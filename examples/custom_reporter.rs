//! ─── custom_reporter.rs ──────────────────────────────────────────────────────
//! Demonstrates implementing the `Reporter` trait and using it with `Consola`.

use consola::{
    create_consola, log_levels,
    types::{LogContext, LogObject, Reporter},
};

// ── Custom reporter ───────────────────────────────────────────────────────────

/// A reporter that prepends a timestamp and formats messages as
/// `[LEVEL] message`.
#[derive(Debug, Clone)]
struct TimestampReporter {
    prefix: String,
}

impl TimestampReporter {
    fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }
}

impl Reporter for TimestampReporter {
    fn format(&self, log_obj: &LogObject, _ctx: &LogContext) -> Result<String, String> {
        // Format: "{prefix} [{level}] message"
        let args_str = if log_obj.args.is_empty() {
            String::new()
        } else {
            log_obj.args.join(" ")
        };

        let tag_part = if log_obj.tag.is_empty() {
            String::new()
        } else {
            format!(" <{}>", log_obj.tag)
        };

        Ok(format!(
            "{} [{}]{}{}{}",
            self.prefix,
            log_obj.r#type.as_str(),
            tag_part,
            if args_str.is_empty() { "" } else { ": " },
            args_str,
        ))
    }

    fn clone_box(&self) -> Box<dyn Reporter> {
        Box::new(self.clone())
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    // Create a consola instance with our custom reporter.
    let consola = create_consola(
        Some(log_levels::VERBOSE),
        vec![Box::new(TimestampReporter::new("[my-app]")) as Box<dyn Reporter>],
    );

    consola.info("Application started");
    consola.warn("Disk space is low (12% remaining)");
    consola.error("Failed to connect to upstream service");

    // Tags work naturally with custom reporters.
    let db_consola = consola.with_tag("db");
    db_consola.info("Connected to PostgreSQL");
    db_consola.error("Connection pool exhausted");

    // Multiple reporters can be combined.
    let multi_consola = create_consola(
        Some(log_levels::VERBOSE),
        vec![
            Box::new(TimestampReporter::new("[stderr]")) as Box<dyn Reporter>,
            Box::new(TimestampReporter::new("[archive]")) as Box<dyn Reporter>,
        ],
    );
    multi_consola.info("This message goes through two reporters");
}
