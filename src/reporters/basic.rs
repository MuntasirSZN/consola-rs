// ─── BasicReporter ────────────────────────────────────────────────────────────
// Pure formatter — no I/O. Returns Result<String, String> for the Consola to emit.

use crate::constants::LogType;
use crate::types::{FormatOptions, LogContext, LogObject, Reporter};

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

    /// Joins the log message arguments into a single space-separated string.
    pub fn format_args(&self, args: &[String], _opts: &FormatOptions) -> String {
        args.join(" ")
    }

    /// Formats the current time as `HH:MM:SS` if `opts.date` is set; returns an empty string otherwise.
    pub fn format_date(&self, opts: &FormatOptions) -> String {
        if opts.date {
            // Format as HH:MM:SS using the timestamp (millis since epoch)
            let total_secs = (crate::types::now_ms() / 1000) as u64;
            let hours = (total_secs / 3600) % 24;
            let mins = (total_secs / 60) % 60;
            let secs = total_secs % 60;
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
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

        if log_obj.r#type == LogType::Box {
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

        self.filter_and_join(&[
            bracket(log_obj.r#type.as_str()),
            bracket(&log_obj.tag),
            message,
        ])
    }
}

impl Reporter for BasicReporter {
    fn format(&self, log_obj: &LogObject, ctx: &LogContext) -> Result<String, String> {
        #[cfg(target_arch = "wasm32")]
        {
            return Err(
                "BasicReporter unavailable on WASM. Use `wasm` feature with BrowserReporter, or enable the `log` feature with a WASM-compatible logger.".into(),
            );
        }

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
    fn test_format_message_field_as_arg() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &[], "");
        let result = r.format(&obj, &ctx).unwrap();
        // args are empty, so message is empty
        assert_eq!(result, "[info]");
    }

    #[test]
    fn test_format_badge() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let mut obj = make_log_obj(LogType::Info, &["hello"], "");
        obj.badge = true;
        // BasicReporter ignores badge field visually — same output
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "[info] hello");
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
    fn test_format_args_joined_with_spaces() {
        let r = BasicReporter;
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &["a", "b", "c"], "");
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "[info] a b c");
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
    fn test_format_is_ok_on_native() {
        // On non-WASM, format always returns Ok
        let r = BasicReporter;
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &["x"], "");
        assert!(r.format(&obj, &ctx).is_ok());
    }
}
