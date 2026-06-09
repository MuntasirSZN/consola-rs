// ─── Common types ─────────────────────────────────────────────────────────────

use std::sync::Arc;

use crate::constants::{LogLevel, LogType, log_levels};

// ─── Format options ───────────────────────────────────────────────────────────

/// Controls formatting behavior of log output.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Override the terminal column width for output wrapping.
    pub columns: Option<u16>,
    /// Whether to include a timestamp prefix in log output.
    pub date: bool,
    /// Whether to use ANSI color codes in formatted output.
    pub colors: bool,
    /// Whether to use compact formatting (single-line output).
    pub compact: bool,
    /// Maximum error level to display in stack traces.
    pub error_level: u32,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            columns: None,
            date: true,
            colors: false,
            compact: true,
            error_level: 0,
        }
    }
}

// ─── Log object input (partial, for defaults / user-provided) ─────────────────

/// Partial log input used to construct a fully resolved `LogObject`.
///
/// All fields are optional; defaults and overrides are merged with `ConsolaOptions.defaults`.
#[derive(Debug, Clone, Default)]
pub struct LogObjectInput {
    /// Optional log level override.
    pub level: Option<LogLevel>,
    /// Optional log type override.
    pub r#type: Option<LogType>,
    /// Optional tag to categorize the log entry.
    pub tag: Option<String>,
    /// Optional primary log message.
    pub message: Option<String>,
    /// Optional secondary text displayed alongside the message.
    pub additional: Option<String>,
    /// Additional positional arguments for richer formatting.
    pub args: Vec<String>,

    // Extra fields used by reporters
    /// Optional title displayed prominently by some reporters.
    pub title: Option<String>,
    /// Whether to render a badge indicator.
    pub badge: Option<bool>,
    /// Optional icon name or emoji string.
    pub icon: Option<String>,
    /// Optional CSS-like style string (reporter-specific).
    pub style: Option<String>,
}

impl LogObjectInput {
    /// Create an empty `LogObjectInput` with all fields in their default state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the message, returning the builder for chaining.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Set the tag, returning the builder for chaining.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Set all positional args at once, returning the builder for chaining.
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Append a single positional arg, returning the builder for chaining.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set the title, returning the builder for chaining.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the type, returning the builder for chaining.
    pub fn type_(mut self, ty: LogType) -> Self {
        self.r#type = Some(ty);
        self
    }

    /// Set the additional text, returning the builder for chaining.
    pub fn additional(mut self, addl: impl Into<String>) -> Self {
        self.additional = Some(addl.into());
        self
    }
}

// ─── Log object (fully resolved, passed to reporters) ─────────────────────────

/// A fully resolved log entry passed to reporters for formatting.
///
/// Constructed from `LogObjectInput` merged with the consola instance defaults.
#[derive(Debug, Clone)]
pub struct LogObject {
    /// Resolved log level.
    pub level: LogLevel,
    /// Resolved log type (e.g. Info, Warn, Error).
    pub r#type: LogType,
    /// Resolved tag string, empty if none was set.
    pub tag: String,
    /// Primary message text, if any.
    pub message: Option<String>,
    /// Secondary/additional text, if any.
    pub additional: Option<String>,
    /// Positional arguments for richer formatting.
    pub args: Vec<String>,
    /// Milliseconds since Unix epoch.
    pub timestamp_ms: i64,

    // Extra fields
    /// Optional title displayed prominently by some reporters.
    pub title: Option<String>,
    /// Whether to render a badge indicator.
    pub badge: bool,
    /// Optional icon name or emoji string.
    pub icon: Option<String>,
    /// Optional CSS-like style string (reporter-specific).
    pub style: Option<String>,
}

impl LogObject {
    /// Create a new `LogObject` from a `LogType`.
    ///
    /// The level is derived from the type, the timestamp is set to now, and all
    /// other fields are left in their default/empty state.
    pub fn new(ty: LogType) -> Self {
        let level = ty.level();
        Self {
            level,
            r#type: ty,
            tag: String::new(),
            message: None,
            additional: None,
            args: Vec::new(),
            timestamp_ms: now_ms(),
            title: None,
            badge: false,
            icon: None,
            style: None,
        }
    }

    /// Return the timestamp as a jiff Zoned (feature = "jiff", default).
    /// Returns `None` if the timestamp is invalid.
    #[cfg(feature = "jiff")]
    pub fn timestamp_jiff(&self) -> Option<jiff::Zoned> {
        jiff::Timestamp::from_millisecond(self.timestamp_ms)
            .ok()
            .map(|ts| ts.to_zoned(jiff::tz::TimeZone::system()))
    }

    /// Return the timestamp as a chrono DateTime<Utc> (feature = "chrono").
    /// Returns `None` if the timestamp is invalid.
    #[cfg(feature = "chrono")]
    pub fn timestamp_chrono(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::from_timestamp_millis(self.timestamp_ms)
    }
}

/// Get current time in milliseconds since epoch.
#[cfg(feature = "jiff")]
pub(crate) fn now_ms() -> i64 {
    jiff::Zoned::now().timestamp().as_millisecond()
}

#[cfg(all(feature = "chrono", not(feature = "jiff")))]
pub(crate) fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(not(any(feature = "jiff", feature = "chrono")))]
pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ─── Reporter trait (pure format, no I/O) ────────────────────────────────────

/// Context passed to reporters alongside the log object.
#[derive(Debug, Clone)]
pub struct LogContext {
    /// Shared consola options available during formatting.
    pub options: Arc<ConsolaOptions>,
}

/// A reporter formats a LogObject into a display string.
/// The Consola handles actual emission via log/tracing.
pub trait Reporter: std::fmt::Debug + Send + Sync {
    /// Format a log object. Returns `Err(reason)` if this reporter cannot handle the
    /// current environment (e.g. BasicReporter on WASM). `Ok(text)` to emit.
    fn format(&self, log_obj: &LogObject, ctx: &LogContext) -> Result<String, String>;
    /// Clone the reporter into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Reporter>;
}

impl Clone for Box<dyn Reporter> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// ─── Prompt-related types ─────────────────────────────────────────────────────

/// A single selectable option in a prompt menu.
#[derive(Debug, Clone)]
pub struct SelectOption {
    /// Display label shown to the user.
    pub label: String,
    /// Value returned when this option is selected.
    pub value: String,
    /// Optional hint text displayed alongside the label.
    pub hint: Option<String>,
}

/// Common options shared across all prompt types.
#[derive(Debug, Clone)]
pub struct PromptCommonOptions {
    /// Optional cancellation token; set to abort the prompt.
    pub cancel: Option<String>,
}

/// Options for a text input prompt.
#[derive(Debug, Clone)]
pub struct TextPromptOptions {
    /// Shared prompt options.
    pub common: PromptCommonOptions,
    /// Optional type override for the input (e.g. "password").
    pub r#type: Option<String>,
    /// Default value returned if the user provides no input.
    pub default: Option<String>,
    /// Placeholder text displayed inside the input field.
    pub placeholder: Option<String>,
    /// Initial pre-filled value.
    pub initial: Option<String>,
}

/// Options for a yes/no confirmation prompt.
#[derive(Debug, Clone)]
pub struct ConfirmPromptOptions {
    /// Shared prompt options.
    pub common: PromptCommonOptions,
    /// Type identifier for the confirm prompt.
    pub r#type: String,
    /// Default boolean state.
    pub initial: Option<bool>,
}

/// Options for a single-select prompt.
#[derive(Debug, Clone)]
pub struct SelectPromptOptions {
    /// Shared prompt options.
    pub common: PromptCommonOptions,
    /// Type identifier for the select prompt.
    pub r#type: String,
    /// Value of the initially selected option.
    pub initial: Option<String>,
    /// Available options to choose from.
    pub options: Vec<SelectOption>,
}

/// Options for a multi-select prompt.
#[derive(Debug, Clone)]
pub struct MultiSelectOptions {
    /// Shared prompt options.
    pub common: PromptCommonOptions,
    /// Type identifier for the multi-select prompt.
    pub r#type: String,
    /// Values of the initially selected options.
    pub initial: Option<Vec<String>>,
    /// Available options to choose from.
    pub options: Vec<SelectOption>,
    /// Whether at least one selection is required.
    pub required: Option<bool>,
}

/// Union of all supported prompt option types.
#[derive(Debug, Clone)]
pub enum PromptOptions {
    /// Free-form text input.
    Text(TextPromptOptions),
    /// Yes/no confirmation.
    Confirm(ConfirmPromptOptions),
    /// Single selection from a list.
    Select(SelectPromptOptions),
    /// Multiple selection from a list.
    MultiSelect(MultiSelectOptions),
}

// ─── Consola options ──────────────────────────────────────────────────────────

/// Configuration options for a `Consola` instance.
#[derive(Debug)]
pub struct ConsolaOptions {
    /// List of reporters that format and output log entries.
    pub reporters: Vec<Box<dyn Reporter>>,
    /// Minimum log level that will be displayed.
    pub level: LogLevel,
    /// Default field values applied to every log entry.
    pub defaults: LogObjectInput,
    /// Minimum interval (ms) between duplicate log messages.
    pub throttle: u64,
    /// Minimum number of occurrences before throttling activates.
    pub throttle_min: u32,
    /// Formatting options for reporters.
    pub format_options: FormatOptions,
}

impl Clone for ConsolaOptions {
    fn clone(&self) -> Self {
        Self {
            reporters: self.reporters.clone(),
            level: self.level,
            defaults: self.defaults.clone(),
            throttle: self.throttle,
            throttle_min: self.throttle_min,
            format_options: self.format_options.clone(),
        }
    }
}

impl Default for ConsolaOptions {
    fn default() -> Self {
        Self {
            reporters: Vec::new(),
            level: log_levels::INFO,
            defaults: LogObjectInput::default(),
            throttle: 1000,
            throttle_min: 5,
            format_options: FormatOptions::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::constants::{LogType, log_levels};
    use crate::types::*;

    // ─── Helper Reporter for trait tests ────────────────────────────────────

    #[derive(Debug, Clone)]
    struct TestReporter;

    impl Reporter for TestReporter {
        fn format(&self, log_obj: &LogObject, _ctx: &LogContext) -> Result<String, String> {
            Ok(log_obj.message.clone().unwrap_or_default())
        }

        fn clone_box(&self) -> Box<dyn Reporter> {
            Box::new(self.clone())
        }
    }

    // ─── 1. FormatOptions default ───────────────────────────────────────────

    #[test]
    fn format_options_default() {
        let opts = FormatOptions::default();
        assert!(opts.columns.is_none());
        assert!(opts.date);
        assert!(!opts.colors);
        assert!(opts.compact);
        assert_eq!(opts.error_level, 0);
    }

    // ─── 2. LogObjectInput builder methods ──────────────────────────────────

    #[test]
    fn log_object_input_type_() {
        let input = LogObjectInput::new().type_(LogType::Error);
        assert_eq!(input.r#type, Some(LogType::Error));
    }

    #[test]
    fn log_object_input_tag() {
        let input = LogObjectInput::new().tag("my-tag");
        assert_eq!(input.tag.as_deref(), Some("my-tag"));
    }

    #[test]
    fn log_object_input_message() {
        let input = LogObjectInput::new().message("hello");
        assert_eq!(input.message.as_deref(), Some("hello"));
    }

    #[test]
    fn log_object_input_args() {
        let input = LogObjectInput::new().args(vec!["a".into(), "b".into()]);
        assert_eq!(input.args, vec!["a", "b"]);
    }

    #[test]
    fn log_object_input_arg() {
        let input = LogObjectInput::new().arg("x");
        assert_eq!(input.args, vec!["x"]);
    }

    #[test]
    fn log_object_input_additional() {
        let input = LogObjectInput::new().additional("extra");
        assert_eq!(input.additional.as_deref(), Some("extra"));
    }

    #[test]
    fn log_object_input_chained() {
        let input = LogObjectInput::new()
            .type_(LogType::Warn)
            .tag("net")
            .message("msg")
            .additional("addl")
            .args(vec!["x".into()]);
        assert_eq!(input.r#type, Some(LogType::Warn));
        assert_eq!(input.tag.as_deref(), Some("net"));
        assert_eq!(input.message.as_deref(), Some("msg"));
        assert_eq!(input.additional.as_deref(), Some("addl"));
        assert_eq!(input.args, vec!["x"]);
    }

    // ─── 3. LogObject construction ──────────────────────────────────────────

    #[test]
    fn log_object_new() {
        let obj = LogObject::new(LogType::Info);
        assert_eq!(obj.r#type, LogType::Info);
        assert_eq!(obj.level, log_levels::INFO);
        assert_eq!(obj.tag, "");
        assert!(obj.message.is_none());
        assert!(obj.additional.is_none());
        assert!(obj.args.is_empty());
        assert!(!obj.badge);
        assert!(obj.title.is_none());
        assert!(obj.icon.is_none());
        assert!(obj.style.is_none());
        // timestamp_ms should be a reasonable value near now
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let diff = (obj.timestamp_ms - now).abs();
        assert!(diff < 5000, "timestamp_ms should be within 5s of now");
    }

    #[test]
    fn log_object_new_all_types() {
        for ty in crate::constants::LOG_TYPES {
            let obj = LogObject::new(*ty);
            assert_eq!(obj.r#type, *ty);
        }
    }

    // ─── 4. LogObject helper methods ────────────────────────────────────────

    #[cfg(feature = "jiff")]
    #[test]
    fn log_object_timestamp_jiff() {
        let obj = LogObject::new(LogType::Info);
        let ts = obj.timestamp_jiff();
        // With default feature "jiff", this should return Some
        assert!(
            ts.is_some(),
            "timestamp_jiff() should return Some with jiff feature"
        );
        let zoned = ts.unwrap();
        // Verify the zoned timestamp is reasonably close to now
        let now_ms = crate::types::now_ms();
        let diff = (zoned.timestamp().as_millisecond() - now_ms).abs();
        assert!(diff < 5000, "jiff timestamp should be within 5s of now");
    }

    #[test]
    fn log_object_timestamp_chrono() {
        // chrono is not a default feature — test both paths
        #[cfg(feature = "chrono")]
        {
            let obj = LogObject::new(LogType::Info);
            let ts = obj.timestamp_chrono();
            assert!(
                ts.is_some(),
                "timestamp_chrono() should return Some with chrono feature"
            );
        }
        #[cfg(not(feature = "chrono"))]
        {
            // timestamp_chrono not available; verify the method does not exist
            // (compile-time check — if it compiled the cfg is right)
        }
    }

    // ─── 5. ConsolaOptions — Debug and Clone ────────────────────────────────

    #[test]
    fn consola_options_debug() {
        let opts = ConsolaOptions::default();
        let debug_str = format!("{:?}", opts);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn consola_options_clone() {
        let opts = ConsolaOptions::default();
        let cloned = opts.clone();
        assert_eq!(cloned.level, opts.level);
        assert_eq!(cloned.throttle, opts.throttle);
        assert_eq!(cloned.throttle_min, opts.throttle_min);
        // Vec<Box<dyn Reporter>> clone works via Clone for Box<dyn Reporter>
        assert!(cloned.reporters.is_empty());
    }

    #[test]
    fn consola_options_default() {
        let opts = ConsolaOptions::default();
        assert_eq!(opts.level, log_levels::INFO);
        assert!(opts.reporters.is_empty());
        assert_eq!(opts.throttle, 1000);
        assert_eq!(opts.throttle_min, 5);
    }

    // ─── 6. Reporter trait ──────────────────────────────────────────────────

    #[test]
    fn reporter_trait_object_safety() {
        let reporter: Box<dyn Reporter> = Box::new(TestReporter);
        let ctx = LogContext {
            options: Arc::new(ConsolaOptions::default()),
        };
        let log_obj = LogObject::new(LogType::Info);
        let result = reporter.format(&log_obj, &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn reporter_clone_box() {
        let reporter = TestReporter;
        let cloned_box = reporter.clone_box();
        let ctx = LogContext {
            options: Arc::new(ConsolaOptions::default()),
        };
        let log_obj = LogObject::new(LogType::Error);
        let result = cloned_box.format(&log_obj, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn reporter_box_clone() {
        let original: Box<dyn Reporter> = Box::new(TestReporter);
        let cloned: Box<dyn Reporter> = original.clone();
        // Both should format independently
        let ctx = LogContext {
            options: Arc::new(ConsolaOptions::default()),
        };
        let log_obj = LogObject::new(LogType::Info);
        assert_eq!(
            cloned.format(&log_obj, &ctx).ok(),
            original.format(&log_obj, &ctx).ok()
        );
    }

    // ─── 7. LogContext ──────────────────────────────────────────────────────

    #[test]
    fn log_context_new() {
        let opts = ConsolaOptions::default();
        let ctx = LogContext {
            options: Arc::new(opts),
        };
        assert_eq!(ctx.options.level, log_levels::INFO);
    }

    #[test]
    fn log_context_debug_clone() {
        let opts = ConsolaOptions::default();
        let ctx = LogContext {
            options: Arc::new(opts),
        };
        let cloned = ctx.clone();
        assert_eq!(cloned.options.level, log_levels::INFO);
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty());
    }

    // ─── 8. SelectOption ────────────────────────────────────────────────────

    #[test]
    fn select_option_new() {
        let opt = SelectOption {
            label: "Yes".into(),
            value: "y".into(),
            hint: None,
        };
        assert_eq!(opt.label, "Yes");
        assert_eq!(opt.value, "y");
        assert!(opt.hint.is_none());
    }

    #[test]
    fn select_option_with_hint() {
        let opt = SelectOption {
            label: "Cancel".into(),
            value: "c".into(),
            hint: Some("aborts the operation".into()),
        };
        assert_eq!(opt.label, "Cancel");
        assert_eq!(opt.value, "c");
        assert_eq!(opt.hint.as_deref(), Some("aborts the operation"));
    }

    #[test]
    fn select_option_debug_clone() {
        let opt = SelectOption {
            label: "test".into(),
            value: "t".into(),
            hint: None,
        };
        let cloned = opt.clone();
        assert_eq!(cloned.label, "test");
        let debug_str = format!("{:?}", opt);
        assert!(!debug_str.is_empty());
    }

    // ─── 9. Prompt types ────────────────────────────────────────────────────

    #[test]
    fn prompt_common_options_default() {
        let pco = PromptCommonOptions { cancel: None };
        assert!(pco.cancel.is_none());
    }

    #[test]
    fn prompt_common_options_with_cancel() {
        let pco = PromptCommonOptions {
            cancel: Some("abort".into()),
        };
        assert_eq!(pco.cancel.as_deref(), Some("abort"));
    }

    #[test]
    fn text_prompt_options() {
        let opts = TextPromptOptions {
            common: PromptCommonOptions { cancel: None },
            r#type: Some("password".into()),
            default: Some("admin".into()),
            placeholder: Some("Enter password".into()),
            initial: None,
        };
        assert_eq!(opts.r#type.as_deref(), Some("password"));
        assert_eq!(opts.default.as_deref(), Some("admin"));
        assert_eq!(opts.placeholder.as_deref(), Some("Enter password"));
        assert!(opts.initial.is_none());
    }

    #[test]
    fn text_prompt_options_debug_clone() {
        let opts = TextPromptOptions {
            common: PromptCommonOptions { cancel: None },
            r#type: None,
            default: None,
            placeholder: None,
            initial: None,
        };
        let cloned = opts.clone();
        assert!(cloned.r#type.is_none());
        let debug_str = format!("{:?}", opts);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn confirm_prompt_options() {
        let opts = ConfirmPromptOptions {
            common: PromptCommonOptions {
                cancel: Some("esc".into()),
            },
            r#type: "confirm".into(),
            initial: Some(true),
        };
        assert_eq!(opts.r#type, "confirm");
        assert_eq!(opts.initial, Some(true));
        assert_eq!(opts.common.cancel.as_deref(), Some("esc"));
    }

    #[test]
    fn select_prompt_options() {
        let opts = SelectPromptOptions {
            common: PromptCommonOptions { cancel: None },
            r#type: "select".into(),
            initial: Some("b".into()),
            options: vec![
                SelectOption {
                    label: "A".into(),
                    value: "a".into(),
                    hint: None,
                },
                SelectOption {
                    label: "B".into(),
                    value: "b".into(),
                    hint: Some("best".into()),
                },
            ],
        };
        assert_eq!(opts.r#type, "select");
        assert_eq!(opts.initial.as_deref(), Some("b"));
        assert_eq!(opts.options.len(), 2);
        assert_eq!(opts.options[0].label, "A");
        assert_eq!(opts.options[1].hint.as_deref(), Some("best"));
    }

    #[test]
    fn multi_select_options() {
        let opts = MultiSelectOptions {
            common: PromptCommonOptions { cancel: None },
            r#type: "multiselect".into(),
            initial: Some(vec!["a".into(), "c".into()]),
            options: vec![
                SelectOption {
                    label: "A".into(),
                    value: "a".into(),
                    hint: None,
                },
                SelectOption {
                    label: "B".into(),
                    value: "b".into(),
                    hint: None,
                },
            ],
            required: Some(true),
        };
        assert_eq!(opts.r#type, "multiselect");
        let initial = opts.initial.clone().unwrap();
        assert_eq!(initial, vec!["a", "c"]);
        assert_eq!(opts.options.len(), 2);
        assert_eq!(opts.required, Some(true));
    }

    // ─── 10. PromptOptions enum ──────────────────────────────────────────────

    #[test]
    fn prompt_options_match_text() {
        let opts = PromptOptions::Text(TextPromptOptions {
            common: PromptCommonOptions { cancel: None },
            r#type: None,
            default: None,
            placeholder: None,
            initial: None,
        });
        match &opts {
            PromptOptions::Text(t) => assert!(t.r#type.is_none()),
            _ => panic!("expected Text variant"),
        }
    }

    #[test]
    fn prompt_options_match_confirm() {
        let opts = PromptOptions::Confirm(ConfirmPromptOptions {
            common: PromptCommonOptions { cancel: None },
            r#type: "confirm".into(),
            initial: Some(false),
        });
        match &opts {
            PromptOptions::Confirm(c) => assert_eq!(c.initial, Some(false)),
            _ => panic!("expected Confirm variant"),
        }
    }

    #[test]
    fn prompt_options_match_select() {
        let opts = PromptOptions::Select(SelectPromptOptions {
            common: PromptCommonOptions { cancel: None },
            r#type: "select".into(),
            initial: None,
            options: vec![],
        });
        match &opts {
            PromptOptions::Select(s) => assert!(s.options.is_empty()),
            _ => panic!("expected Select variant"),
        }
    }

    #[test]
    fn prompt_options_match_multiselect() {
        let opts = PromptOptions::MultiSelect(MultiSelectOptions {
            common: PromptCommonOptions { cancel: None },
            r#type: "multiselect".into(),
            initial: None,
            options: vec![],
            required: None,
        });
        match &opts {
            PromptOptions::MultiSelect(m) => assert!(m.options.is_empty()),
            _ => panic!("expected MultiSelect variant"),
        }
    }

    #[test]
    fn prompt_options_debug() {
        let opts = PromptOptions::Text(TextPromptOptions {
            common: PromptCommonOptions { cancel: None },
            r#type: None,
            default: None,
            placeholder: None,
            initial: None,
        });
        let debug_str = format!("{:?}", opts);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn prompt_options_clone() {
        let opts = PromptOptions::Confirm(ConfirmPromptOptions {
            common: PromptCommonOptions { cancel: None },
            r#type: "x".into(),
            initial: None,
        });
        let cloned = opts.clone();
        match &cloned {
            PromptOptions::Confirm(c) => assert_eq!(c.r#type, "x"),
            _ => panic!("expected Confirm"),
        }
    }
}
