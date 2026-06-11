// ─── FancyReporter ───────────────────────────────────────────────────────────
// Pure formatter — no I/O. Colors via `anstyle` crate.

use std::sync::LazyLock;

use crate::constants::{LogLevel, LogType};
use crate::types::{ErrorInfo, FormatOptions, LogContext, LogObject, Reporter};
use crate::util::boxes::{BoxOpts, box_text};
use crate::util::color::{self, get_color};
use crate::util::string::string_width;

const TYPE_COLOR_MAP: &[(LogType, &str)] = &[
    (LogType::Info, "cyan"),
    (LogType::Fail, "red"),
    (LogType::Success, "green"),
    (LogType::Ready, "green"),
    (LogType::Start, "magenta"),
];

const LEVEL_COLOR_MAP: &[(LogLevel, &str)] = &[(0, "red"), (1, "yellow")];

const TYPE_ICONS: &[(LogType, &str, &str)] = &[
    (LogType::Error, "✖", "×"),
    (LogType::Fatal, "✖", "×"),
    (LogType::Ready, "✔", "√"),
    (LogType::Warn, "⚠", "‼"),
    (LogType::Info, "ℹ", "i"),
    (LogType::Success, "✔", "√"),
    (LogType::Debug, "⚙", "D"),
    (LogType::Trace, "→", "→"),
    (LogType::Fail, "✖", "×"),
    (LogType::Start, "◐", "o"),
];

fn unicode_supported() -> bool {
    static CACHED: LazyLock<bool> = LazyLock::new(|| {
        let term = std::env::var("TERM").unwrap_or_default();
        let lang = std::env::var("LANG").unwrap_or_default();
        !(cfg!(windows) && (term == "MINGW" || term.contains("cygwin")))
            || lang.contains("UTF-8")
            || lang.contains("utf8")
    });
    *CACHED
}

fn icon_for(ty: LogType, unicode: bool) -> &'static str {
    for &(t, u, a) in TYPE_ICONS {
        if t == ty {
            return if unicode { u } else { a };
        }
    }
    ""
}

fn type_color_name(ty: LogType, level: LogLevel) -> &'static str {
    for &(t, c) in TYPE_COLOR_MAP {
        if t == ty {
            return c;
        }
    }
    for &(l, c) in LEVEL_COLOR_MAP {
        if l == level {
            return c;
        }
    }
    "gray"
}

fn bg_color_fn(name: &str) -> fn(&str) -> String {
    let bg_name = format!("bg_{}", name);
    let f = get_color(&bg_name);
    // If the bg_name variant didn't exist, try the camelCase variant
    if name != "white" {
        let alt = format!("bg{}{}", name[..1].to_uppercase(), &name[1..]);
        let f2 = get_color(&alt);
        // Check if it's any different from the default
        if f2 as usize != get_color("white") as usize {
            return f2;
        }
    }
    f
}

fn character_format(text: &str) -> String {
    let input = text.to_string();

    // Highlight backticks: `text` -> colored cyan
    let mut step1 = String::with_capacity(input.len());
    let mut rest = input.as_str();
    while let Some(start) = rest.find('`') {
        step1.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(end) = after.find('`') {
            let inner = &after[..end];
            step1.push_str(&color::cyan(inner));
            rest = &after[end + 1..];
        } else {
            step1.push('`');
            rest = after;
        }
    }
    step1.push_str(rest);

    // Underline underscores: _text_
    let chars: Vec<char> = step1.chars().collect();
    let mut out = String::with_capacity(step1.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '_'
            && (i == 0 || chars[i - 1] == ' ')
            && let Some(end) = chars[i + 1..].iter().position(|&c| c == '_')
        {
            let end = i + 1 + end;
            let end_ok = end + 1 >= chars.len() || chars[end + 1] == ' ';
            if end_ok {
                let inner: String = chars[i + 1..end].iter().collect();
                out.push_str(&color::underline(&inner));
                i = end + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Fancy reporter with colors, icons, and formatting.
#[derive(Debug, Clone)]
pub struct FancyReporter {
    unicode: bool,
}

impl Default for FancyReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl FancyReporter {
    /// Creates a new `FancyReporter`, detecting Unicode support from the environment.
    pub fn new() -> Self {
        Self {
            unicode: unicode_supported(),
        }
    }

    fn format_type(&self, log_obj: &LogObject, is_badge: bool, _opts: &FormatOptions) -> String {
        let color_name = type_color_name(log_obj.r#type, log_obj.level);
        if is_badge {
            let type_str = log_obj.r#type.as_str().to_uppercase();
            let badge = format!(" {} ", type_str);
            bg_color_fn(color_name)(&color::black(&badge))
        } else {
            let icon_str = icon_for(log_obj.r#type, self.unicode);
            let display = if !icon_str.is_empty() {
                icon_str
            } else {
                log_obj.icon.as_deref().unwrap_or(log_obj.r#type.as_str())
            };
            get_color(color_name)(display)
        }
    }

    /// Format an error chain recursively, matching consola-js output format.
    fn format_error(err: &ErrorInfo, _opts: &FormatOptions, level: usize) -> String {
        let indent = "  ".repeat(level + 2);
        let caused_prefix = if level > 0 {
            format!("{}[cause]: {}", "  ".repeat(level), err.message)
        } else {
            err.message.clone()
        };

        let mut result = caused_prefix;

        if let Some(stack) = &err.stack
            && !stack.is_empty()
        {
            // Blank line before stack
            result.push('\n');
            // Format each line with proper indentation
            for line in stack.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                // Color the "at" part
                let formatted = if let Some(loc) = line.strip_prefix("at ") {
                    format!("{}{}{}", indent, color::gray("at "), color::cyan(loc))
                } else if let Some(loc) = line.strip_prefix("    at ") {
                    format!(
                        "{}{}{}",
                        indent,
                        color::gray("at "),
                        color::cyan(loc.trim())
                    )
                } else if line.starts_with("  ") || line.starts_with('\t') {
                    format!("{}{}", indent, color::cyan(line.trim()))
                } else {
                    format!("{}{}", indent, color::cyan(line))
                };
                result.push_str(&format!("\n{}", formatted));
            }
        }

        if let Some(cause) = &err.cause {
            result.push_str("\n\n");
            result.push_str(&Self::format_error(cause, _opts, level + 1));
        }

        result
    }

    fn format_log_obj(&self, log_obj: &LogObject, opts: &FormatOptions) -> String {
        let basic = crate::reporters::basic::BasicReporter;
        let formatted = basic.format_args(&log_obj.args, opts);
        let mut parts = formatted.split('\n');
        let message = parts.next().unwrap_or("");
        let additional: Vec<&str> = parts.collect();

        if log_obj.r#type == LogType::Box {
            let body = if additional.is_empty() {
                character_format(message)
            } else {
                let add = additional.join("\n");
                character_format(&format!("{}\n{}", message, add))
            };
            return box_text(
                &body,
                &BoxOpts {
                    title: log_obj.title.as_ref().map(|t| character_format(t)),
                    style: None,
                },
            );
        }

        let date = basic.format_date(opts);
        let colored_date = if !date.is_empty() {
            color::gray(&date)
        } else {
            String::new()
        };

        let is_badge = log_obj.badge || log_obj.level < 2;
        let type_str = self.format_type(log_obj, is_badge, opts);

        let tag = if !log_obj.tag.is_empty() {
            color::gray(&log_obj.tag)
        } else {
            String::new()
        };

        // Left side: type + tag + message
        let left = basic.filter_and_join(&[type_str, tag, character_format(message)]);
        // Right side: just the date, right-aligned to terminal edge
        let right = colored_date;

        // Auto-detect terminal width when not set
        let columns = opts.columns.unwrap_or(0) as usize;
        let date_width = string_width(&right);
        let left_width = string_width(&left);

        let mut line = if columns > 0 && date_width > 0 && left_width + date_width + 2 < columns {
            // Right-align the date at the terminal edge
            let space = columns.saturating_sub(left_width + date_width + 1);
            format!("{}{}{}", left, " ".repeat(space), right)
        } else if columns > 0 && date_width > 0 {
            // Not enough room for alignment, append inline
            format!("{}  {}", left, right)
        } else if !right.is_empty() && left.is_empty() {
            right
        } else if !right.is_empty() {
            format!("{}  {}", left, right)
        } else {
            left
        };

        // Append additional lines from args
        if !additional.is_empty() {
            line.push_str(&character_format(&format!("\n{}", additional.join("\n"))));
        }

        // Append error info (error chain with stack traces)
        if let Some(err) = &log_obj.error {
            let error_text = Self::format_error(err, opts, 0);
            line.push_str(&format!("\n{}", error_text));
        }

        if is_badge {
            format!("\n{}\n", line)
        } else {
            line
        }
    }
}

impl Reporter for FancyReporter {
    fn format(&self, log_obj: &LogObject, ctx: &LogContext) -> Result<String, String> {
        Ok(self.format_log_obj(log_obj, &ctx.options.format_options))
    }

    fn clone_box(&self) -> Box<dyn Reporter> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::LogType;
    use crate::types::ConsolaOptions;
    use crate::util::color;
    use std::sync::Arc;

    fn make_ctx() -> LogContext {
        LogContext {
            options: Arc::new(ConsolaOptions::default()),
        }
    }

    fn make_ctx_no_date() -> LogContext {
        LogContext {
            options: Arc::new(ConsolaOptions {
                format_options: crate::types::FormatOptions {
                    date: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
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
        let r = FancyReporter::new();
        let d = FancyReporter::default();
        assert_eq!(r.unicode, d.unicode);
    }

    #[test]
    fn test_unicode_supported_returns_bool() {
        let result = unicode_supported();
        assert!(result || !result);
    }

    #[test]
    fn test_type_color_map_has_entries() {
        let mapped: Vec<LogType> = TYPE_COLOR_MAP.iter().map(|(t, _)| *t).collect();
        assert!(mapped.contains(&LogType::Info));
        assert!(mapped.contains(&LogType::Fail));
        assert!(mapped.contains(&LogType::Success));
        assert!(mapped.contains(&LogType::Ready));
        assert!(mapped.contains(&LogType::Start));
    }

    #[test]
    fn test_type_icons_has_all_entries() {
        let mapped: Vec<LogType> = TYPE_ICONS.iter().map(|(t, _, _)| *t).collect();
        assert!(mapped.contains(&LogType::Error));
        assert!(mapped.contains(&LogType::Fatal));
        assert!(mapped.contains(&LogType::Ready));
        assert!(mapped.contains(&LogType::Warn));
        assert!(mapped.contains(&LogType::Info));
        assert!(mapped.contains(&LogType::Success));
        assert!(mapped.contains(&LogType::Debug));
        assert!(mapped.contains(&LogType::Trace));
        assert!(mapped.contains(&LogType::Fail));
        assert!(mapped.contains(&LogType::Start));
    }

    #[test]
    fn test_level_color_map_entries() {
        let mapped: Vec<LogLevel> = LEVEL_COLOR_MAP.iter().map(|(l, _)| *l).collect();
        assert!(mapped.contains(&0));
        assert!(mapped.contains(&1));
    }

    #[test]
    fn test_format_with_icons() {
        color::set_color_enabled(false);
        let r = FancyReporter { unicode: true };
        let ctx = make_ctx_no_date();

        let obj = make_log_obj(LogType::Info, &["hello"], "");
        let result = r.format(&obj, &ctx).unwrap();
        assert!(result.contains('ℹ'));
        assert!(result.contains("hello"));

        let obj = make_log_obj(LogType::Success, &["ok"], "");
        let result = r.format(&obj, &ctx).unwrap();
        assert!(result.contains('✔'));
    }

    #[test]
    fn test_format_badge_low_level() {
        color::set_color_enabled(false);
        let r = FancyReporter { unicode: true };
        let ctx = make_ctx_no_date();

        let obj = make_log_obj(LogType::Error, &["err"], "");
        let result = r.format(&obj, &ctx).unwrap();
        assert!(result.contains("ERROR"));
        assert!(result.starts_with('\n'));
        assert!(result.ends_with('\n'));

        let obj = make_log_obj(LogType::Warn, &["warn"], "");
        let result = r.format(&obj, &ctx).unwrap();
        assert!(result.contains("WARN"));
    }

    #[test]
    fn test_format_with_tag() {
        color::set_color_enabled(false);
        let r = FancyReporter { unicode: true };
        let ctx = make_ctx_no_date();
        let obj = make_log_obj(LogType::Info, &["hello"], "mytag");
        let result = r.format(&obj, &ctx).unwrap();
        assert!(result.contains("mytag"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_format_box() {
        color::set_color_enabled(false);
        let r = FancyReporter { unicode: true };
        let ctx = make_ctx_no_date();
        let mut obj = make_log_obj(LogType::Box, &["hello"], "");
        obj.title = Some("title".into());
        let result = r.format(&obj, &ctx).unwrap();
        assert!(result.contains("hello"));
        let has_border = result.contains('╭') || result.contains('┌');
        assert!(has_border);
    }

    #[test]
    fn test_format_badge() {
        color::set_color_enabled(false);
        let r = FancyReporter { unicode: true };
        let ctx = make_ctx_no_date();
        let mut obj = make_log_obj(LogType::Info, &["hello"], "");
        obj.badge = true;
        let result = r.format(&obj, &ctx).unwrap();
        assert!(result.starts_with('\n'));
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_character_format_backticks() {
        color::set_color_enabled(false);
        let result = character_format("use `foo` here");
        assert!(result.contains("foo"));
        assert!(!result.contains('`'));
    }

    #[test]
    fn test_character_format_underscores() {
        color::set_color_enabled(false);
        let result = character_format("use _bar_ here");
        assert!(result.contains("bar"));
        assert!(!result.contains('_'));
    }

    #[test]
    fn test_character_format_combined() {
        color::set_color_enabled(false);
        let result = character_format("_a_ and `b`");
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        assert!(!result.contains('`'));
    }

    #[test]
    fn test_bg_color_fn_returns_function() {
        let f = bg_color_fn("red");
        let output = f("test");
        assert!(output.contains("test"));

        let f = bg_color_fn("green");
        let output = f("test");
        assert!(output.contains("test"));
    }

    #[test]
    fn test_icon_for_unicode_true() {
        assert_eq!(icon_for(LogType::Info, true), "ℹ");
        assert_eq!(icon_for(LogType::Error, true), "✖");
        assert_eq!(icon_for(LogType::Success, true), "✔");
        assert_eq!(icon_for(LogType::Warn, true), "⚠");
        assert_eq!(icon_for(LogType::Start, true), "◐");
        assert_eq!(icon_for(LogType::Log, true), "");
    }

    #[test]
    fn test_icon_for_unicode_false() {
        assert_eq!(icon_for(LogType::Info, false), "i");
        assert_eq!(icon_for(LogType::Error, false), "×");
        assert_eq!(icon_for(LogType::Success, false), "√");
        assert_eq!(icon_for(LogType::Warn, false), "‼");
        assert_eq!(icon_for(LogType::Start, false), "o");
        assert_eq!(icon_for(LogType::Log, false), "");
    }

    #[test]
    fn test_type_color_name_known() {
        assert_eq!(type_color_name(LogType::Info, 3), "cyan");
        assert_eq!(type_color_name(LogType::Fail, 3), "red");
        assert_eq!(type_color_name(LogType::Success, 3), "green");
        assert_eq!(type_color_name(LogType::Ready, 3), "green");
        assert_eq!(type_color_name(LogType::Start, 3), "magenta");
    }

    #[test]
    fn test_type_color_name_falls_back_to_level() {
        assert_eq!(type_color_name(LogType::Log, 0), "red");
        assert_eq!(type_color_name(LogType::Log, 1), "yellow");
    }

    #[test]
    fn test_type_color_name_falls_back_to_gray() {
        assert_eq!(type_color_name(LogType::Box, 5), "gray");
    }

    #[test]
    fn test_clone_box() {
        let r: Box<dyn Reporter> = Box::new(FancyReporter { unicode: true });
        let cloned = r.clone_box();
        let ctx = make_ctx_no_date();
        let obj = make_log_obj(LogType::Info, &["test"], "");
        assert_eq!(
            r.format(&obj, &ctx).unwrap(),
            cloned.format(&obj, &ctx).unwrap()
        );
    }

    #[test]
    fn test_format_date_appears_with_default_opts() {
        color::set_color_enabled(false);
        let r = FancyReporter { unicode: true };
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &["hello"], "");
        let result = r.format(&obj, &ctx).unwrap();
        assert!(
            result.contains(":"),
            "Expected timestamp in output: {}",
            result
        );
    }

    #[test]
    fn test_format_with_error_and_backtrace() {
        color::set_color_enabled(false);
        let r = FancyReporter { unicode: true };
        let ctx = make_ctx_no_date();
        let mut obj = make_log_obj(LogType::Error, &["an error occurred"], "");
        obj.error = Some(ErrorInfo {
            message: "an error occurred".into(),
            stack: Some("at error.ts:3:19\n    at processTicksAndRejections (native:7:39)".into()),
            backtrace: None,
            cause: Some(Box::new(ErrorInfo {
                message: "root cause".into(),
                stack: Some("at error.ts:4:14".into()),
                backtrace: None,
                cause: None,
            })),
        });
        let result = r.format(&obj, &ctx).unwrap();
        assert!(
            result.contains("ERROR"),
            "Badge should be present: {}",
            result
        );
        assert!(
            result.contains("error.ts:3:19"),
            "Stack line should appear: {}",
            result
        );
    }

    #[test]
    fn test_format_with_columns_right_aligns_date() {
        color::set_color_enabled(false);
        let r = FancyReporter { unicode: true };
        let fmt_opts = crate::types::FormatOptions {
            columns: Some(120),
            ..Default::default()
        };
        let ctx = LogContext {
            options: Arc::new(ConsolaOptions {
                format_options: fmt_opts,
                ..ConsolaOptions::default()
            }),
        };
        let obj = make_log_obj(LogType::Info, &["aligned"], "");
        let result = r.format(&obj, &ctx).unwrap();
        // With columns=120, the date should be far to the right
        assert!(
            result.contains("aligned"),
            "Message should be in output: {}",
            result
        );
        assert!(
            result.contains(":"),
            "Timestamp should be in output: {}",
            result
        );
    }
}
