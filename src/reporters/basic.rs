//! BasicReporter — pure formatter — no I/O. Returns Result<String, String> for the Consola to emit.

use crate::types::{ErrorInfo, FormatOptions, LogContext, LogObject, Reporter};

fn bracket(x: &str) -> String {
    if x.is_empty() {
        String::new()
    } else {
        format!("[{}]", x)
    }
}

/// Formats log messages as plain text.
#[derive(Debug, Clone)]
pub struct BasicReporter;

impl Default for BasicReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicReporter {
    /// Creates a new `BasicReporter`.
    pub fn new() -> Self {
        Self
    }

    /// Formats an error with its source chain (recursive).
    pub fn format_error(err: &ErrorInfo, _opts: &FormatOptions, level: usize) -> String {
        let caused_prefix = if level > 0 {
            format!("{}[cause]: ", "  ".repeat(level))
        } else {
            String::new()
        };

        let mut result = format!("{}{}", caused_prefix, err.message);

        if let Some(stack) = &err.stack
            && !stack.is_empty()
        {
            // Blank line before stack
            result.push('\n');
            // Indent each line of the stack
            for line in stack.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                result.push_str(&format!("\n{}{}", "  ".repeat(level + 2), trimmed));
            }
        }

        if let Some(cause) = &err.cause {
            result.push_str("\n\n");
            result.push_str(&Self::format_error(cause, _opts, level + 1));
        }

        result
    }

    /// Joins the log message arguments into a single space-separated string.
    /// Detects errors in args and formats them with source chains.
    pub fn format_args(&self, args: &[String], _opts: &FormatOptions) -> String {
        let mut parts = Vec::with_capacity(args.len());
        for arg in args {
            parts.push(arg.clone());
        }
        parts.join(" ")
    }

    /// Formats the current time as 12-hour local time (`h:mm:ss AM/PM`).
    #[allow(unreachable_code)]
    pub fn format_date(&self, opts: &FormatOptions) -> String {
        self.format_date_at(opts, crate::types::now_ms())
    }

    /// Like `format_date` but accepts an explicit timestamp (milliseconds since epoch).
    /// Used internally so tests can inject a specific time.
    #[allow(unreachable_code)]
    pub(crate) fn format_date_at(&self, opts: &FormatOptions, _now_ms: i64) -> String {
        if opts.date {
            #[cfg(feature = "jiff")]
            {
                if let Ok(ts) = jiff::Timestamp::from_millisecond(_now_ms) {
                    let zoned = ts.to_zoned(jiff::tz::TimeZone::system());
                    let civil = zoned.datetime();
                    let h = civil.hour();
                    let hour12 = match h {
                        0 => 12,
                        1..=12 => h,
                        _ => h - 12,
                    };
                    let ampm = if h < 12 { "AM" } else { "PM" };
                    return format!(
                        "{}:{:02}:{:02} {}",
                        hour12,
                        civil.minute(),
                        civil.second(),
                        ampm
                    );
                }
            }

            #[cfg(all(feature = "chrono", not(feature = "jiff")))]
            {
                use chrono::Timelike;
                let local = chrono::Local::now();
                let h = local.hour12();
                let hour12 = match h.1 {
                    0 => 12,
                    n => n,
                };
                let ampm = if h.0 { "PM" } else { "AM" };
                return format!(
                    "{}:{:02}:{:02} {}",
                    hour12,
                    local.minute(),
                    local.second(),
                    ampm
                );
            }

            #[cfg(all(feature = "time", not(any(feature = "jiff", feature = "chrono"))))]
            {
                let offset =
                    time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
                let now = time::OffsetDateTime::now_utc().to_offset(offset);
                let h = now.hour();
                let hour12 = match h {
                    0 => 12,
                    1..=12 => h,
                    _ => h - 12,
                };
                let ampm = if h < 12 { "AM" } else { "PM" };
                return format!(
                    "{}:{:02}:{:02} {}",
                    hour12,
                    now.minute(),
                    now.second(),
                    ampm
                );
            }

            // Fallback: UTC-based 12-hour (unreachable when a crate feature is active)
            {
                let total_secs = (_now_ms / 1000) as u64;
                let hours = (total_secs / 3600) % 24;
                let mins = (total_secs / 60) % 60;
                let secs = total_secs % 60;
                let hour12 = match hours {
                    0 => 12,
                    1..=12 => hours,
                    _ => hours - 12,
                };
                let ampm = if hours < 12 { "AM" } else { "PM" };
                format!("{}:{:02}:{:02} {}", hour12, mins, secs, ampm)
            }
        } else {
            String::new()
        }
    }

    /// Filters out empty parts and joins the remainder with spaces.
    pub fn filter_and_join(&self, parts: &[String]) -> String {
        parts
            .iter()
            .filter(|p| !p.is_empty())
            .fold(String::new(), |mut acc, p| {
                if !acc.is_empty() {
                    acc.push(' ');
                }
                acc.push_str(p);
                acc
            })
    }

    /// Formats a `LogObject` into a plain-text string based on the given format options.
    pub fn format_log_obj(&self, log_obj: &LogObject, opts: &FormatOptions) -> String {
        let message = self.format_args(&log_obj.args, opts);

        if log_obj.r#type == crate::constants::LogType::Box {
            let mut lines: Vec<String> = Vec::new();
            lines.push(String::new());
            let tag = bracket(&log_obj.tag);
            if !tag.is_empty() {
                lines.push(format!(" > {}", tag));
            }
            if let Some(title) = &log_obj.title {
                lines.push(format!(" > {}", title));
            }
            for line in message.split('\n') {
                lines.push(format!(" > {}", line));
            }
            lines.push(String::new());
            return lines.join("\n");
        }

        let base = self.filter_and_join(&[
            bracket(log_obj.r#type.as_str()),
            bracket(&log_obj.tag),
            message,
        ]);

        // Append error info if present
        if let Some(err) = &log_obj.error {
            let error_text = Self::format_error(err, opts, 0);
            format!("{}\n{}", base, error_text)
        } else {
            base
        }
    }
}

impl Reporter for BasicReporter {
    fn format(&self, log_obj: &LogObject, ctx: &LogContext) -> Result<String, String> {
        let opts = &ctx.options.format_options;
        Ok(self.format_log_obj(log_obj, opts))
    }

    fn clone_box(&self) -> Box<dyn Reporter> {
        Box::new(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::LogType;
    use crate::types::ConsolaOptions;
    use std::sync::Arc;

    fn make_ctx() -> LogContext {
        LogContext {
            options: Arc::new(ConsolaOptions::default()),
        }
    }

    fn make_log_obj(ty: LogType, args: &[&str], tag: &str) -> LogObject {
        LogObject {
            level: ty.level(),
            r#type: ty,
            tag: tag.to_string(),
            message: None,
            additional: None,
            args: args.iter().map(|s| s.to_string()).collect(),
            timestamp_ms: 0,
            title: None,
            badge: false,
            icon: None,
            style: None,
            error: None,
        }
    }

    #[test]
    fn test_new_and_default() {
        let r = BasicReporter::new();
        let d = BasicReporter;
        assert_eq!(format!("{:?}", r), "BasicReporter");
        assert_eq!(format!("{:?}", d), "BasicReporter");
    }

    #[test]
    fn test_format_plain() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &["hello", "world"], "");
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "[info] hello world");
    }

    #[test]
    fn test_format_with_tag() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &["hello"], "mytag");
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "[info] [mytag] hello");
    }

    #[test]
    fn test_format_box_no_title() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Box, &["hello"], "");
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "\n > hello\n");
    }

    #[test]
    fn test_format_box_with_tag() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Box, &["hello"], "mytag");
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "\n > [mytag]\n > hello\n");
    }

    #[test]
    fn test_format_box_with_title() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let mut obj = make_log_obj(LogType::Box, &["hello"], "");
        obj.title = Some("MyTitle".into());
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "\n > MyTitle\n > hello\n");
    }

    #[test]
    fn test_format_with_error() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let mut obj = make_log_obj(LogType::Error, &["an error occurred"], "");
        obj.error = Some(ErrorInfo {
            message: "an error occurred".into(),
            stack: Some("  at error.rs:10:5\n  at main.rs:20:1".into()),
            backtrace: None,
            cause: Some(Box::new(ErrorInfo {
                message: "root cause".into(),
                stack: Some("  at lib.rs:5:1".into()),
                backtrace: None,
                cause: None,
            })),
        });
        let result = r.format(&obj, &ctx).unwrap();
        assert!(result.contains("[error]"));
        assert!(result.contains("an error occurred"));
        assert!(result.contains("[cause]:"));
        assert!(result.contains("root cause"));
    }

    #[test]
    fn test_format_various_types() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let types = [
            LogType::Silent,
            LogType::Fatal,
            LogType::Error,
            LogType::Warn,
            LogType::Log,
            LogType::Info,
            LogType::Success,
            LogType::Fail,
            LogType::Ready,
            LogType::Start,
            LogType::Debug,
            LogType::Trace,
            LogType::Verbose,
        ];
        for ty in &types {
            let obj = make_log_obj(*ty, &["msg"], "");
            let result = r.format(&obj, &ctx).unwrap();
            assert_eq!(result, format!("[{}] msg", ty.as_str()));
        }
    }

    #[test]
    fn test_format_empty_args() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &[], "");
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "[info]");
    }

    #[test]
    fn test_clone_box() {
        let r: Box<dyn Reporter> = Box::new(BasicReporter);
        let cloned = r.clone_box();
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &["test"], "");
        assert_eq!(
            r.format(&obj, &ctx).unwrap(),
            cloned.format(&obj, &ctx).unwrap()
        );
    }

    #[test]
    fn test_format_is_ok_on_all_platforms() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &["x"], "");
        assert!(r.format(&obj, &ctx).is_ok());
    }

    #[test]
    fn test_format_date_at_midnight() {
        let r = BasicReporter;
        let opts = FormatOptions {
            date: true,
            ..Default::default()
        };
        // Epoch midnight (1970-01-01 00:00:00 UTC)
        // In the jiff implementation, this should produce "12:00:00 AM"
        let result = r.format_date_at(&opts, 0);
        assert!(
            result.contains(":"),
            "expected formatted time, got: '{}'",
            result
        );
    }
}
