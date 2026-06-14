//! Integration tests for the public types API.

use consola::{
    LogType, log_levels,
    types::{
        ConfirmPromptOptions, ConsolaOptions, ErrorInfo, FormatOptions, LogContext, LogObject,
        LogObjectInput, MultiSelectOptions, PromptCommonOptions, PromptOptions, Reporter,
        SelectOption, SelectPromptOptions, TextPromptOptions,
    },
};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Helper: TestReporter
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct TestReporter;

impl Reporter for TestReporter {
    fn format(
        &self,
        log_obj: &LogObject,
        _ctx: &LogContext,
    ) -> Result<String, consola::error::ConsolaError> {
        Ok(log_obj.message.clone().unwrap_or_default())
    }
    fn clone_box(&self) -> Box<dyn Reporter> {
        Box::new(self.clone())
    }
}

// ---------------------------------------------------------------------------
// FormatOptions
// ---------------------------------------------------------------------------

#[test]
fn format_options_default() {
    let opts = FormatOptions::default();
    assert!(opts.date);
    assert!(!opts.colors);
    assert!(opts.compact);
    assert_eq!(opts.error_level, 0);
}

// ---------------------------------------------------------------------------
// LogObjectInput
// ---------------------------------------------------------------------------

#[test]
fn log_object_input_type_() {
    let input = LogObjectInput::new().type_(LogType::Warn);
    assert_eq!(input.r#type, Some(LogType::Warn));
}

#[test]
fn log_object_input_tag() {
    let input = LogObjectInput::new().tag("http");
    assert_eq!(input.tag.as_deref(), Some("http"));
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
    let input = LogObjectInput::new().arg("single");
    assert_eq!(input.args, vec!["single"]);
}

#[test]
fn log_object_input_additional() {
    let input = LogObjectInput::new().additional("extra");
    assert_eq!(input.additional.as_deref(), Some("extra"));
}

#[test]
fn log_object_input_chained() {
    let input = LogObjectInput::new()
        .message("msg")
        .tag("tag")
        .additional("addl")
        .arg("a1")
        .arg("a2");
    assert_eq!(input.message.as_deref(), Some("msg"));
    assert_eq!(input.tag.as_deref(), Some("tag"));
    assert_eq!(input.additional.as_deref(), Some("addl"));
    assert_eq!(input.args, vec!["a1", "a2"]);
}

// ---------------------------------------------------------------------------
// LogObject
// ---------------------------------------------------------------------------

#[test]
fn log_object_new() {
    let obj = LogObject::new(LogType::Info);
    assert_eq!(obj.r#type, LogType::Info);
    assert_eq!(obj.level, log_levels::INFO);
    assert!(obj.timestamp_ms > 0);
}

#[test]
fn log_object_new_all_types() {
    for ty in consola::constants::LOG_TYPES.iter().copied() {
        let obj = LogObject::new(ty);
        assert_eq!(obj.r#type, ty);
        assert!(obj.timestamp_ms > 0);
    }
}

#[cfg(feature = "jiff")]
#[test]
fn log_object_timestamp_jiff() {
    let obj = LogObject::new(LogType::Info);
    let zoned = obj.timestamp_jiff();
    assert!(zoned.is_some());
}

#[cfg(feature = "chrono")]
#[test]
fn log_object_timestamp_chrono() {
    let obj = LogObject::new(LogType::Info);
    let dt = obj.timestamp_chrono();
    assert!(dt.is_some());
}

// ---------------------------------------------------------------------------
// ConsolaOptions
// ---------------------------------------------------------------------------

#[test]
fn consola_options_debug() {
    let opts = ConsolaOptions::default();
    // Debug should not panic and should contain some recognizable info.
    let debug_str = format!("{opts:?}");
    assert!(!debug_str.is_empty());
}

#[test]
fn consola_options_clone() {
    let opts = ConsolaOptions::default();
    let cloned = opts.clone();
    assert_eq!(cloned.level, log_levels::INFO);
    assert_eq!(cloned.throttle, 1000);
    assert_eq!(cloned.throttle_min, 5);
}

#[test]
fn consola_options_default() {
    let opts = ConsolaOptions::default();
    assert!(opts.reporters.is_empty());
    assert_eq!(opts.level, log_levels::INFO);
    assert_eq!(opts.throttle, 1000);
    assert_eq!(opts.throttle_min, 5);
}

// ---------------------------------------------------------------------------
// Reporter trait object safety / clone
// ---------------------------------------------------------------------------

#[test]
fn reporter_trait_object_safety() {
    let r: Box<dyn Reporter> = Box::new(TestReporter);
    let obj = LogObject::new(LogType::Info);
    let ctx = LogContext {
        options: Arc::new(ConsolaOptions::default()),
    };
    let result = r.format(&obj, &ctx);
    assert!(result.is_ok());
}

#[test]
fn reporter_clone_box() {
    let r = TestReporter;
    let cloned: Box<dyn Reporter> = r.clone_box();
    let obj = LogObject::new(LogType::Info);
    let ctx = LogContext {
        options: Arc::new(ConsolaOptions::default()),
    };
    assert_eq!(
        cloned.format(&obj, &ctx).unwrap(),
        obj.message.unwrap_or_default()
    );
}

#[test]
fn reporter_box_clone() {
    let r: Box<dyn Reporter> = Box::new(TestReporter);
    let cloned = r.clone();
    let obj = LogObject::new(LogType::Info);
    let ctx = LogContext {
        options: Arc::new(ConsolaOptions::default()),
    };
    assert_eq!(
        cloned.format(&obj, &ctx).unwrap(),
        obj.message.unwrap_or_default()
    );
}

// ---------------------------------------------------------------------------
// LogContext
// ---------------------------------------------------------------------------

#[test]
fn log_context_new() {
    let ctx = LogContext {
        options: Arc::new(ConsolaOptions::default()),
    };
    assert_eq!(ctx.options.level, log_levels::INFO);
}

#[test]
fn log_context_debug_clone() {
    let ctx = LogContext {
        options: Arc::new(ConsolaOptions::default()),
    };
    let debug_str = format!("{ctx:?}");
    assert!(!debug_str.is_empty());

    let cloned = ctx.clone();
    assert_eq!(cloned.options.level, ctx.options.level);
}

// ---------------------------------------------------------------------------
// SelectOption
// ---------------------------------------------------------------------------

#[test]
fn select_option_new() {
    let opt = SelectOption {
        label: "Yes".into(),
        value: "yes".into(),
        hint: None,
    };
    assert_eq!(opt.label, "Yes");
    assert_eq!(opt.value, "yes");
    assert!(opt.hint.is_none());
}

#[test]
fn select_option_with_hint() {
    let opt = SelectOption {
        label: "Custom".into(),
        value: "custom".into(),
        hint: Some("Enter a custom value".into()),
    };
    assert_eq!(opt.hint.as_deref(), Some("Enter a custom value"));
}

#[test]
fn select_option_debug_clone() {
    let opt = SelectOption {
        label: "A".into(),
        value: "a".into(),
        hint: None,
    };
    let debug_str = format!("{opt:?}");
    assert!(!debug_str.is_empty());

    let cloned = opt.clone();
    assert_eq!(cloned.label, "A");
    assert_eq!(cloned.value, "a");
}

// ---------------------------------------------------------------------------
// PromptCommonOptions
// ---------------------------------------------------------------------------

#[test]
fn prompt_common_options_default() {
    let opts = PromptCommonOptions { cancel: None };
    assert!(opts.cancel.is_none());
}

#[test]
fn prompt_common_options_with_cancel() {
    let opts = PromptCommonOptions {
        cancel: Some("user-abort".into()),
    };
    assert_eq!(opts.cancel.as_deref(), Some("user-abort"));
}

// ---------------------------------------------------------------------------
// TextPromptOptions
// ---------------------------------------------------------------------------

#[test]
fn text_prompt_options() {
    let opts = TextPromptOptions {
        common: PromptCommonOptions { cancel: None },
        r#type: None,
        default: Some("default".into()),
        placeholder: Some("Enter name".into()),
        initial: None,
    };
    assert_eq!(opts.default.as_deref(), Some("default"));
    assert_eq!(opts.placeholder.as_deref(), Some("Enter name"));
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
    let debug_str = format!("{opts:?}");
    assert!(!debug_str.is_empty());

    let cloned = opts.clone();
    assert!(cloned.common.cancel.is_none());
}

// ---------------------------------------------------------------------------
// ConfirmPromptOptions
// ---------------------------------------------------------------------------

#[test]
fn confirm_prompt_options() {
    let opts = ConfirmPromptOptions {
        common: PromptCommonOptions { cancel: None },
        r#type: "confirm".into(),
        initial: Some(true),
    };
    assert_eq!(opts.initial, Some(true));
    assert_eq!(opts.r#type, "confirm");
}

// ---------------------------------------------------------------------------
// SelectPromptOptions
// ---------------------------------------------------------------------------

#[test]
fn select_prompt_options() {
    let opts = SelectPromptOptions {
        common: PromptCommonOptions { cancel: None },
        r#type: "select".into(),
        initial: None,
        options: vec![
            SelectOption {
                label: "Red".into(),
                value: "red".into(),
                hint: None,
            },
            SelectOption {
                label: "Blue".into(),
                value: "blue".into(),
                hint: None,
            },
        ],
    };
    assert_eq!(opts.options.len(), 2);
}

// ---------------------------------------------------------------------------
// MultiSelectOptions
// ---------------------------------------------------------------------------

#[test]
fn multi_select_options() {
    let opts = MultiSelectOptions {
        common: PromptCommonOptions { cancel: None },
        r#type: "multiselect".into(),
        initial: None,
        options: vec![SelectOption {
            label: "X".into(),
            value: "x".into(),
            hint: None,
        }],
        required: Some(true),
    };
    assert_eq!(opts.required, Some(true));
}

// ---------------------------------------------------------------------------
// PromptOptions enum matching
// ---------------------------------------------------------------------------

#[test]
fn prompt_options_match_text() {
    let popts = PromptOptions::Text(TextPromptOptions {
        common: PromptCommonOptions { cancel: None },
        r#type: None,
        default: None,
        placeholder: None,
        initial: None,
    });

    match &popts {
        PromptOptions::Text(t) => assert!(t.initial.is_none()),
        _ => panic!("expected Text variant"),
    }
}

#[test]
fn prompt_options_match_confirm() {
    let popts = PromptOptions::Confirm(ConfirmPromptOptions {
        common: PromptCommonOptions { cancel: None },
        r#type: "confirm".into(),
        initial: Some(false),
    });

    match &popts {
        PromptOptions::Confirm(c) => assert_eq!(c.initial, Some(false)),
        _ => panic!("expected Confirm variant"),
    }
}

#[test]
fn prompt_options_match_select() {
    let popts = PromptOptions::Select(SelectPromptOptions {
        common: PromptCommonOptions { cancel: None },
        r#type: "select".into(),
        initial: None,
        options: vec![],
    });

    match &popts {
        PromptOptions::Select(s) => assert!(s.options.is_empty()),
        _ => panic!("expected Select variant"),
    }
}

#[test]
fn prompt_options_match_multiselect() {
    let popts = PromptOptions::MultiSelect(MultiSelectOptions {
        common: PromptCommonOptions { cancel: None },
        r#type: "multiselect".into(),
        initial: None,
        options: vec![],
        required: None,
    });

    match &popts {
        PromptOptions::MultiSelect(m) => assert!(m.required.is_none()),
        _ => panic!("expected MultiSelect variant"),
    }
}

#[test]
fn prompt_options_debug() {
    let popts = PromptOptions::Text(TextPromptOptions {
        common: PromptCommonOptions { cancel: None },
        r#type: None,
        default: None,
        placeholder: None,
        initial: None,
    });
    let debug_str = format!("{popts:?}");
    assert!(!debug_str.is_empty());
}

#[test]
fn prompt_options_clone() {
    let popts = PromptOptions::Select(SelectPromptOptions {
        common: PromptCommonOptions { cancel: None },
        r#type: "select".into(),
        initial: None,
        options: vec![],
    });
    let cloned = popts.clone();
    match &cloned {
        PromptOptions::Select(s) => assert_eq!(s.r#type, "select"),
        _ => panic!("expected Select variant"),
    }
}

// ---------------------------------------------------------------------------
// FormatOptions — terminal width coverage
// ---------------------------------------------------------------------------

#[test]
fn format_options_terminal_width_default() {
    let opts = FormatOptions::default();
    // columns is platform-dependent; just verify it doesn't panic and is Some or None
    match opts.columns {
        Some(w) => assert!(w > 0),
        None => { /* expected when not connected to a terminal */ }
    }
}

// ---------------------------------------------------------------------------
// ErrorInfo
// ---------------------------------------------------------------------------

#[test]
fn error_info_default() {
    let err = ErrorInfo::default();
    assert_eq!(err.message, "");
    assert!(err.stack.is_none());
    assert!(err.backtrace.is_none());
    assert!(err.cause.is_none());
}

#[test]
fn error_info_with_fields() {
    let err = ErrorInfo {
        message: "err".into(),
        stack: Some("trace".into()),
        backtrace: None,
        cause: None,
    };
    assert_eq!(err.message, "err");
    assert_eq!(err.stack.as_deref(), Some("trace"));
    assert!(err.backtrace.is_none());
    assert!(err.cause.is_none());
}

#[test]
fn error_info_nested_cause() {
    let inner = ErrorInfo {
        message: "inner error".into(),
        stack: None,
        backtrace: None,
        cause: None,
    };
    let outer = ErrorInfo {
        message: "outer error".into(),
        stack: None,
        backtrace: None,
        cause: Some(Box::new(inner)),
    };
    assert_eq!(outer.message, "outer error");
    assert!(outer.cause.is_some());
    assert_eq!(outer.cause.as_ref().unwrap().message, "inner error");
    assert!(outer.cause.as_ref().unwrap().cause.is_none());
}

// ---------------------------------------------------------------------------
// LogObject — timestamp and tag
// ---------------------------------------------------------------------------

#[test]
fn log_object_timestamp_ms() {
    let obj = LogObject::new(LogType::Info);
    assert!(
        obj.timestamp_ms > 0,
        "timestamp_ms should be positive, got {}",
        obj.timestamp_ms
    );
}

#[test]
fn log_object_tag_default_empty() {
    let obj = LogObject::new(LogType::Info);
    assert_eq!(obj.tag, "", "default tag should be empty");
}

// ---------------------------------------------------------------------------
// ConsolaOptions — level and throttle defaults
// ---------------------------------------------------------------------------

#[test]
fn consola_options_level_default() {
    assert_eq!(
        ConsolaOptions::default().level,
        log_levels::INFO,
        "default level should be INFO"
    );
}

#[test]
fn consola_options_throttle_default() {
    let opts = ConsolaOptions::default();
    assert_eq!(opts.throttle, 1000);
    assert_eq!(opts.throttle_min, 5);
}

// ---------------------------------------------------------------------------
// Reporter trait object clone
// ---------------------------------------------------------------------------

#[test]
fn reporter_trait_object_clone() {
    let r: Box<dyn Reporter> = Box::new(TestReporter);
    let cloned = r.clone();
    let obj = LogObject::new(LogType::Info);
    let ctx = LogContext {
        options: Arc::new(ConsolaOptions::default()),
    };
    let original_result = r.format(&obj, &ctx).unwrap();
    let cloned_result = cloned.format(&obj, &ctx).unwrap();
    assert_eq!(original_result, cloned_result);
}

// ---------------------------------------------------------------------------
// LogContext — options accessible
// ---------------------------------------------------------------------------

#[test]
fn log_context_options_accessible() {
    let options = Arc::new(ConsolaOptions::default());
    let ctx = LogContext { options };
    assert_eq!(ctx.options.level, log_levels::INFO);
}

// ---------------------------------------------------------------------------
// LogObjectInput — builder chains
// ---------------------------------------------------------------------------

#[test]
fn log_object_input_new_builder() {
    let input = LogObjectInput::new().message("x").tag("t").title("title");
    assert_eq!(input.message.as_deref(), Some("x"));
    assert_eq!(input.tag.as_deref(), Some("t"));
    assert_eq!(input.title.as_deref(), Some("title"));
}

#[test]
fn log_object_input_all_fields() {
    let input = LogObjectInput {
        level: Some(log_levels::INFO),
        r#type: Some(LogType::Info),
        tag: Some("tag".into()),
        message: Some("msg".into()),
        additional: Some("addl".into()),
        args: vec!["a".into(), "b".into()],
        title: Some("title".into()),
        badge: Some(true),
        icon: Some("icon".into()),
        style: Some("color:red".into()),
        error: Some(ErrorInfo {
            message: "err".into(),
            stack: None,
            backtrace: None,
            cause: None,
        }),
    };
    assert_eq!(input.level, Some(log_levels::INFO));
    assert_eq!(input.r#type, Some(LogType::Info));
    assert_eq!(input.tag.as_deref(), Some("tag"));
    assert_eq!(input.message.as_deref(), Some("msg"));
    assert_eq!(input.additional.as_deref(), Some("addl"));
    assert_eq!(input.args, vec!["a", "b"]);
    assert_eq!(input.title.as_deref(), Some("title"));
    assert_eq!(input.badge, Some(true));
    assert_eq!(input.icon.as_deref(), Some("icon"));
    assert_eq!(input.style.as_deref(), Some("color:red"));
    assert!(input.error.is_some());
    assert_eq!(input.error.as_ref().unwrap().message, "err");
    assert!(input.error.as_ref().unwrap().cause.is_none());
}

// ===================================================================
// Edge cases — now_ms, terminal_width
// ===================================================================

#[test]
fn test_now_ms_positive() {
    // now_ms() is called internally when creating LogObject — verify the timestamp
    let obj = LogObject::new(LogType::Info);
    assert!(
        obj.timestamp_ms > 1_700_000_000_000,
        "timestamp {} should be > 2023",
        obj.timestamp_ms
    );
    assert!(
        obj.timestamp_ms < 9_999_999_999_999,
        "timestamp {} should be reasonable",
        obj.timestamp_ms
    );
}

#[test]
fn test_terminal_width_api() {
    use consola::types::format::terminal_width;
    // In test environments stdout is not a terminal, so this should return None.
    let tw = terminal_width();
    match tw {
        None => {}
        Some(w) => assert!(w > 0, "terminal width should be positive, got {w}"),
    }
}

#[test]
fn test_terminal_width_integration() {
    use consola::types::format::terminal_width;
    let tw = terminal_width();
    match tw {
        None => {}
        Some(w) => assert!(w > 0, "terminal width should be positive, got {w}"),
    }
}
