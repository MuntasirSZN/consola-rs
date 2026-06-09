//! Reporter for browser console environments via `web-sys` bindings.
//!
//! On WASM targets with the `wasm` feature, this reporter emits styled log
//! messages directly to the browser developer console. On other targets it
//! falls back to a string-based format for testing and display.

use crate::constants::{LogLevel, LogType};
use crate::types::{FormatOptions, LogContext, LogObject, Reporter};

/// Reporter for browser console environments via web-sys.
#[derive(Debug, Clone)]
pub struct BrowserReporter {
    /// Default CSS color used when no level or type color matches.
    pub default_color: String,
    /// Per-log-level CSS colors used when no type-specific color is found.
    pub level_colors: [(LogLevel, String); 4],
    /// Per-type CSS colors, checked before falling back to `level_colors`.
    pub type_colors: Vec<(String, String)>,
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
                (999, String::new()),
            ],
            type_colors: vec![("success".into(), "#2ecc71".into())],
        }
    }

    fn fmt_log(&self, log_obj: &LogObject) -> (String, String) {
        // Returns (badge, message_text)
        let type_str = if log_obj.r#type == LogType::Log {
            String::new()
        } else {
            log_obj.r#type.as_str().to_string()
        };
        let tag = log_obj.tag.clone();

        let badge_str = if tag.is_empty() && type_str.is_empty() {
            String::new()
        } else {
            [tag.as_str(), type_str.as_str()]
                .iter()
                .filter(|s| !s.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(":")
        };

        let msg = log_obj.args.join(" ");
        (badge_str, msg)
    }

    fn format_log_obj(&self, log_obj: &LogObject, _opts: &FormatOptions) -> String {
        let (badge, msg) = self.fmt_log(log_obj);
        if badge.is_empty() {
            msg
        } else {
            format!("[{}] {}", badge, msg)
        }
    }

    /// Emit to the browser console using web-sys bindings.
    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    fn emit_browser(&self, log_obj: &LogObject, badge: &str, msg: &str) {
        use wasm_bindgen::JsValue;
        let console = web_sys::console;

        let color = self.color_for(log_obj);
        let style = format!(
            "background: {}; border-radius: 0.5em; color: white; font-weight: bold; padding: 2px 0.5em;",
            color
        );

        let console_fn: &dyn Fn(&JsValue) = if log_obj.level < 1 {
            &|v| console::error_1(v)
        } else if log_obj.level == 1 {
            &|v| console::warn_1(v)
        } else {
            &|v| console::log_1(v)
        };

        if badge.is_empty() {
            console_fn(&JsValue::from_str(msg));
        } else {
            console_fn(&JsValue::from_str(&format!("%c{} %c{}", badge, msg)));
            // In real browser this would use console.log with multiple %c arguments,
            // but web-sys' variadic console methods are complex to wire up for multiple
            // format specifiers. We output a simplified version.
        }
    }

    /// On WASM without `wasm` feature: error at compile/runtime.
    #[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
    fn emit_browser(&self, _log_obj: &LogObject, _badge: &str, _msg: &str) -> Result<(), String> {
        Err("BrowserReporter requires the `wasm` feature on WASM targets".into())
    }
}

impl Reporter for BrowserReporter {
    fn format(&self, log_obj: &LogObject, ctx: &LogContext) -> Result<String, String> {
        #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
        {
            let (badge, msg) = self.fmt_log(log_obj);
            self.emit_browser(log_obj, &badge, &msg);
            // Return empty formatted string since we already emitted
            return Ok(String::new());
        }

        #[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
        {
            return Err("BrowserReporter requires the `wasm` feature on WASM targets".into());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(self.format_log_obj(log_obj, &ctx.options.format_options))
        }
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
        }
    }

    #[test]
    fn test_new_and_default() {
        let r = BrowserReporter::new();
        let d = BrowserReporter::default();
        assert_eq!(r.default_color, d.default_color);
        assert_eq!(r.level_colors.len(), 4);
        assert_eq!(r.type_colors.len(), 1);
    }

    #[test]
    fn test_format_plain_message() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        // LogType::Log has no type_str, no tag -> no badge
        let obj = make_log_obj(LogType::Log, &["hello", "world"], "", 2);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_format_with_type_tag_badge() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        // Info type with tag -> badge "tag:info"
        let obj = make_log_obj(LogType::Info, &["hello"], "mytag", 3);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "[mytag:info] hello");
    }

    #[test]
    fn test_format_with_type_only() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        // Info type, no tag -> badge "info"
        let obj = make_log_obj(LogType::Info, &["hello"], "", 3);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "[info] hello");
    }

    #[test]
    fn test_format_with_tag_only() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        // Log type with tag -> badge just "tag" (type_str is empty for Log)
        let obj = make_log_obj(LogType::Log, &["hello"], "mytag", 2);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "[mytag] hello");
    }

    #[test]
    fn test_format_error_type() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Error, &["fail"], "", 0);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "[error] fail");
    }

    #[test]
    fn test_format_empty_args() {
        let r = BrowserReporter::new();
        let ctx = make_ctx();
        let obj = make_log_obj(LogType::Info, &[], "", 3);
        let result = r.format(&obj, &ctx).unwrap();
        assert_eq!(result, "[info] ");
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
        assert_eq!(r.level_colors[3], (999, String::new()));
        assert_eq!(
            r.type_colors[0],
            ("success".to_string(), "#2ecc71".to_string())
        );
    }
}
