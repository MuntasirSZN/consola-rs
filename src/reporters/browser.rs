//! Reporter for browser console environments.
//!
//! Uses runtime detection: when the `browser` feature is enabled and
//! `web_sys::window()` returns `Some` (i.e. running in a browser),
//! messages are printed with styled badges matching consola-js output.
//! Otherwise, falls back to plain text formatting.

use crate::constants::{LogLevel, LogType};
use crate::types::{FormatOptions, LogContext, LogObject, Reporter};

/// Runtime browser detection: returns `true` when the current environment is
/// a browser with a DOM `window` object. This works on `wasm32` targets
/// compiled with the `browser` feature, and gracefully returns `false` on
/// native targets or non-browser WASM environments (e.g. Node.js WASM).
fn is_browser() -> bool {
    #[cfg(all(target_arch = "wasm32", feature = "browser"))]
    {
        // web_sys::window() returns None when not in a browser (e.g. Node.js WASM)
        web_sys::window().is_some()
    }
    #[cfg(not(all(target_arch = "wasm32", feature = "browser")))]
    {
        false
    }
}

/// Reporter for browser console environments.
///
/// When `browser` feature is enabled and the environment is detected as a
/// browser, uses `console.log`/`console.warn`/`console.error` with styled
/// badges. Otherwise produces plain text formatted output.
#[derive(Debug, Clone)]
pub struct BrowserReporter {
    /// Default CSS color used when no level or type color matches.
    pub default_color: String,
    /// Per-log-level CSS colors used when no type-specific color is found.
    /// Keyed by log level number (0=error, 1=warn, 3=info).
    pub level_colors: [(LogLevel, String); 3],
    /// Per-type CSS colors, checked before falling back to `level_colors`.
    pub type_colors: Vec<(String, String)>,
    /// Whether the environment was detected as a browser.
    pub browser: bool,
}

impl Default for BrowserReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserReporter {
    /// Creates a new `BrowserReporter` with default color settings.
    pub fn new() -> Self {
        Self {
            default_color: "#7f8c8d".into(),
            level_colors: [
                (0, "#c0392b".into()),
                (1, "#f39c12".into()),
                (3, "#00BCD4".into()),
            ],
            type_colors: vec![("success".into(), "#2ecc71".into())],
            browser: is_browser(),
        }
    }

    /// Find the CSS color for a given log object, matching JS logic:
    /// 1. typeColorMap[logObj.type] (type-specific)
    /// 2. levelColorMap[logObj.level] (level-based)
    /// 3. defaultColor
    #[allow(dead_code)]
    fn color_for(&self, log_obj: &LogObject) -> &str {
        let type_name = log_obj.r#type.as_str();
        for (name, color) in &self.type_colors {
            if name == type_name {
                return color;
            }
        }
        for (level, color) in &self.level_colors {
            if *level == log_obj.level {
                return color;
            }
        }
        &self.default_color
    }

    /// Get the appropriate console function for the log level.
    #[cfg(all(target_arch = "wasm32", feature = "browser"))]
    fn console_fn_with_style(level: LogLevel, formatted: &str, style: &str) {
        let args = wasm_bindgen::JsValue::from_str(formatted);
        let style_val = wasm_bindgen::JsValue::from_str(style);
        let empty = wasm_bindgen::JsValue::from_str("");
        if level < 1 {
            web_sys::console::error_3(&args, &style_val, &empty);
        } else if level == 1 {
            web_sys::console::warn_3(&args, &style_val, &empty);
        } else {
            web_sys::console::log_3(&args, &style_val, &empty);
        }
    }

    /// Get the appropriate console function for plain (unstyled) output.
    #[cfg(all(target_arch = "wasm32", feature = "browser"))]
    fn console_fn_plain(level: LogLevel, msg: &str) {
        let args = wasm_bindgen::JsValue::from_str(msg);
        if level < 1 {
            web_sys::console::error_1(&args);
        } else if level == 1 {
            web_sys::console::warn_1(&args);
        } else {
            web_sys::console::log_1(&args);
        }
    }

    /// Emit a styled badge + message to the browser console.
    #[cfg(all(target_arch = "wasm32", feature = "browser"))]
    fn emit_browser_styled(&self, log_obj: &LogObject) {
        let badge_text = self.badge_text(log_obj);
        let msg = log_obj.args.join(" ");

        if badge_text.is_empty() {
            Self::console_fn_plain(log_obj.level, &msg);
        } else {
            Self::console_fn_with_style(
                log_obj.level,
                &format!("%c[{}]%c {}", badge_text, msg),
                &format!(
                    "background: {}; border-radius: 0.5em; color: white; font-weight: bold; padding: 2px 0.5em;",
                    self.color_for(log_obj),
                ),
            );
        }
    }

    /// Build the badge text (tag:type) matching consola-js.
    fn badge_text(&self, log_obj: &LogObject) -> String {
        let type_str = if log_obj.r#type == LogType::Log {
            String::new()
        } else {
            log_obj.r#type.as_str().to_string()
        };
        let tag = log_obj.tag.clone();

        [tag, type_str]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(":")
    }

    fn fmt_log(&self, log_obj: &LogObject) -> (String, String) {
        let badge = self.badge_text(log_obj);
        let msg = log_obj.args.join(" ");
        (badge, msg)
    }

    fn format_log_obj(&self, log_obj: &LogObject, _opts: &FormatOptions) -> String {
        let (badge, msg) = self.fmt_log(log_obj);
        if badge.is_empty() {
            msg
        } else {
            format!("{} {}", badge, msg)
        }
    }
}

impl Reporter for BrowserReporter {
    fn format(&self, log_obj: &LogObject, ctx: &LogContext) -> Result<String, String> {
        // In browser: emit styled output, return empty string (already emitted)
        #[cfg(all(target_arch = "wasm32", feature = "browser"))]
        if self.browser {
            self.emit_browser_styled(log_obj);
            return Ok(String::new());
        }

        // Fallback: text formatting (native or non-browser wasm)
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
    use std::sync::Arc;

    fn make_ctx() -> LogContext {
        LogContext {
            options: Arc::new(ConsolaOptions::default()),
        }
    }

    fn make_log_obj(ty: LogType, args: &[&str], tag: &str, level: i32) -> LogObject {
        LogObject {
            level,
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
        let r = BrowserReporter::new();
        let d = BrowserReporter::default();
        assert_eq!(r.default_color, d.default_color);
        assert_eq!(r.level_colors.len(), 3);
        assert_eq!(r.type_colors.len(), 1);
    }

    #[test]
    fn test_format_plain_message() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Log, &["hello", "world"], "", 2);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_format_with_type_tag_badge() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &["hello"], "mytag", 3);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "mytag:info hello");
    }

    #[test]
    fn test_format_with_type_only() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &["hello"], "", 3);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "info hello");
    }

    #[test]
    fn test_format_with_tag_only() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Log, &["hello"], "mytag", 2);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "mytag hello");
    }

    #[test]
    fn test_format_error_type() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Error, &["fail"], "", 0);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "error fail");
    }

    #[test]
    fn test_format_empty_args() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &[], "", 3);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "info ");
    }

    #[test]
    fn test_clone_box() {
        let r: Box<dyn Reporter> = Box::new(BrowserReporter::new());
        let cloned = r.clone_box();
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &["test"], "", 3);
        assert_eq!(
            r.format(&obj, &ctx).unwrap(),
            cloned.format(&obj, &ctx).unwrap()
        );
    }

    #[test]
    fn test_default_colors() {
        let r = BrowserReporter::new();
        assert_eq!(r.default_color, "#7f8c8d");
        assert_eq!(r.level_colors[0], (0, "#c0392b".to_string()));
        assert_eq!(r.level_colors[1], (1, "#f39c12".to_string()));
        assert_eq!(r.level_colors[2], (3, "#00BCD4".to_string()));
        assert_eq!(
            r.type_colors[0],
            ("success".to_string(), "#2ecc71".to_string())
        );
    }

    #[test]
    fn test_color_for_type_specific() {
        let r = BrowserReporter::new();
        let mut obj = make_log_obj(LogType::Success, &["test"], "", 3);
        assert_eq!(r.color_for(&obj), "#2ecc71");
        obj.r#type = LogType::Info;
        assert_eq!(r.color_for(&obj), "#00BCD4");
    }

    #[test]
    fn test_color_for_level_based() {
        let r = BrowserReporter::new();
        let obj = make_log_obj(LogType::Log, &["test"], "", 0);
        assert_eq!(r.color_for(&obj), "#c0392b");

        let obj = make_log_obj(LogType::Log, &["test"], "", 1);
        assert_eq!(r.color_for(&obj), "#f39c12");
    }

    #[test]
    fn test_color_for_default() {
        let r = BrowserReporter::new();
        let obj = make_log_obj(LogType::Log, &["test"], "", 999);
        assert_eq!(r.color_for(&obj), "#7f8c8d");
    }

    #[test]
    fn test_badge_text() {
        let r = BrowserReporter::new();
        let obj = make_log_obj(LogType::Info, &["msg"], "tag", 3);
        assert_eq!(r.badge_text(&obj), "tag:info");

        let obj = make_log_obj(LogType::Info, &["msg"], "", 3);
        assert_eq!(r.badge_text(&obj), "info");

        let obj = make_log_obj(LogType::Log, &["msg"], "", 2);
        assert_eq!(r.badge_text(&obj), "");

        let obj = make_log_obj(LogType::Log, &["msg"], "tag", 2);
        assert_eq!(r.badge_text(&obj), "tag");
    }

    #[test]
    fn test_is_browser_on_native() {
        // Always false on native targets
        #[cfg(not(all(target_arch = "wasm32", feature = "browser")))]
        assert!(!is_browser());
    }
}
