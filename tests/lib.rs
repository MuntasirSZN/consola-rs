//! Integration tests for the consola crate.
//!
//! Tests the public API: factory functions, Consola methods, reporter formatting,
//! level filtering, throttling, and prompt stubs.

use std::sync::Arc;

use ::consola::reporters::{BasicReporter, FancyReporter};
use ::consola::types::{ConsolaOptions, LogContext, LogObject, Reporter};
use ::consola::*;

// ---------------------------------------------------------------------------
// Factory function tests
// ---------------------------------------------------------------------------

#[test]
fn test_create_basic_consola() {
    let c = create_basic_consola(None);
    c.info("hello");
}

#[test]
fn test_create_fancy_consola() {
    let c = create_fancy_consola(None);
    c.info("hello");
}

#[test]
fn test_create_core_consola() {
    let c = create_core_consola(None, vec![]);
    c.info("hello");
}

#[test]
fn test_create_consola_with_multiple_reporters() {
    let reporters: Vec<Box<dyn Reporter>> = vec![
        Box::new(BasicReporter::new()),
        Box::new(FancyReporter::new()),
    ];
    let c = create_consola(None, reporters);
    c.info("hello");
}

#[test]
fn test_create_consola_default_level() {
    let c = create_consola(None, vec![]);
    assert_eq!(c.level(), log_levels::INFO);
}

#[test]
fn test_create_basic_consola_default_level() {
    let c = create_basic_consola(None);
    assert_eq!(c.level(), log_levels::INFO);
}

#[test]
fn test_create_fancy_consola_default_level() {
    let c = create_fancy_consola(None);
    assert_eq!(c.level(), log_levels::INFO);
}

// ---------------------------------------------------------------------------
// Consola instance tests
// ---------------------------------------------------------------------------

#[test]
fn test_can_set_level() {
    let c = create_basic_consola(None);
    assert_eq!(c.level(), log_levels::INFO);

    // DEBUG (4) > INFO (3) -> filtered
    assert!(!c.debug("should be filtered"));

    // INFO (3) <= INFO (3) -> passes
    assert!(c.info("should pass"));

    c.set_level(log_levels::DEBUG);
    assert_eq!(c.level(), log_levels::DEBUG);

    // DEBUG (4) <= DEBUG (4) -> passes now
    assert!(c.debug("should pass now"));

    // Set level to 0 (lowest after normalize: SILENT clamps to 0)
    c.set_level(log_levels::SILENT);
    // FATAL and ERROR are level 0, so 0 > 0 is false -> passes
    assert!(c.fatal("fatal passes"));
    assert!(c.error("error passes"));

    // WARN (1) > 0 -> filtered
    assert!(!c.warn("warn filtered"));
}

#[test]
fn test_with_tag() {
    let c = create_basic_consola(None);
    let tagged = c.with_tag("mytag");
    tagged.info("hello");
    // Returns a new Consola with tag applied; no panic
}

#[test]
fn test_pause_resume() {
    let c = create_basic_consola(None);
    c.pause_logs();
    // Message is queued while paused; still returns true
    assert!(c.info("queued message"));
    c.resume_logs();
    // After resume, queued messages are flushed
}

#[test]
fn test_log_methods_all_types() {
    let c = create_basic_consola(Some(log_levels::VERBOSE));
    assert!(c.log("log msg"));
    assert!(c.fatal("fatal msg"));
    assert!(c.error("error msg"));
    assert!(c.warn("warn msg"));
    assert!(c.info("info msg"));
    assert!(c.success("success msg"));
    assert!(c.fail("fail msg"));
    assert!(c.ready("ready msg"));
    assert!(c.start("start msg"));
    assert!(c.box_("box msg"));
    assert!(c.debug("debug msg"));
    assert!(c.trace("trace msg"));
    assert!(c.verbose("verbose msg"));
}

#[test]
fn test_raw_methods() {
    let c = create_basic_consola(Some(log_levels::VERBOSE));
    assert!(c.log_raw("raw log"));
    assert!(c.fatal_raw("raw fatal"));
    assert!(c.error_raw("raw error"));
    assert!(c.warn_raw("raw warn"));
    assert!(c.info_raw("raw info"));
    assert!(c.success_raw("raw success"));
    assert!(c.fail_raw("raw fail"));
    assert!(c.ready_raw("raw ready"));
    assert!(c.start_raw("raw start"));
    assert!(c.box_raw("raw box"));
    assert!(c.debug_raw("raw debug"));
    assert!(c.trace_raw("raw trace"));
    assert!(c.verbose_raw("raw verbose"));
}

#[test]
fn test_log_obj() {
    let c = create_basic_consola(None);
    let input = LogObjectInput::new().message("structured log").tag("mytag");
    assert!(c.log_obj(&input));
}

#[test]
fn test_library_static_works() {
    // The CONSOLA static is lazily initialized with default options
    assert!(CONSOLA.info("static consola works"));
}

#[test]
fn test_consola_static_emits() {
    // CONSOLA at INFO level, messages at or below INFO should pass
    assert!(CONSOLA.info("hello from static"));
    assert!(CONSOLA.warn("warn from static"));
    assert!(CONSOLA.error("error from static"));
    assert!(CONSOLA.fatal("fatal from static"));
    assert!(CONSOLA.log("log from static"));
}

// ---------------------------------------------------------------------------
// Level filter tests
// ---------------------------------------------------------------------------

#[test]
fn test_level_filter() {
    // Set level to WARN (1)
    let c = create_basic_consola(Some(log_levels::WARN));

    // ERROR (0) <= WARN (1) -> passes
    assert!(c.error("error passes"));

    // WARN (1) <= WARN (1) -> passes
    assert!(c.warn("warn passes"));

    // INFO (3) > WARN (1) -> filtered
    assert!(!c.info("info filtered"));

    // DEBUG (4) > WARN (1) -> filtered
    assert!(!c.debug("debug filtered"));

    // TRACE (5) > WARN (1) -> filtered
    assert!(!c.trace("trace filtered"));

    // Set level to FATAL (0)
    let c = create_basic_consola(Some(log_levels::FATAL));

    // FATAL (0) <= FATAL (0) -> passes
    assert!(c.fatal("fatal passes"));

    // ERROR (0) <= FATAL (0) -> passes
    assert!(c.error("error passes"));

    // WARN (1) > FATAL (0) -> filtered
    assert!(!c.warn("warn filtered"));
}

// ---------------------------------------------------------------------------
// Throttle tests
// ---------------------------------------------------------------------------

#[test]
fn test_throttle_basic() {
    // Create a consola with aggressive throttle settings
    let c = Consola::new(ConsolaOptions {
        throttle: 5000,  // 5 second window
        throttle_min: 1, // collapse after first repeat
        reporters: vec![Box::new(BasicReporter) as Box<dyn Reporter>],
        level: log_levels::INFO,
        ..ConsolaOptions::default()
    });

    // First call -> logged, returns true
    assert!(c.info("repeat me"));

    // Second call within throttle window -> still returns true
    assert!(c.info("repeat me"));

    // Different message -> true
    assert!(c.info("different"));

    // Third repeat of original -> true
    assert!(c.info("repeat me"));
}

// ---------------------------------------------------------------------------
// Reporter format tests
// ---------------------------------------------------------------------------

fn make_ctx_no_date() -> LogContext {
    LogContext {
        options: Arc::new(ConsolaOptions {
            format_options: FormatOptions {
                date: false,
                colors: false,
                ..FormatOptions::default()
            },
            ..ConsolaOptions::default()
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
fn test_reporter_format() {
    let reporter = BasicReporter;
    let ctx = make_ctx_no_date();

    let obj = make_log_obj(LogType::Info, &["hello", "world"], "");
    let result = reporter.format(&obj, &ctx).unwrap();
    assert_eq!(result, "[info] hello world");

    // With tag
    let obj = make_log_obj(LogType::Info, &["hello"], "mytag");
    let result = reporter.format(&obj, &ctx).unwrap();
    assert_eq!(result, "[info] [mytag] hello");
}

#[test]
fn test_fancy_reporter_format() {
    set_color_enabled(false);
    let reporter = FancyReporter::new();
    let ctx = make_ctx_no_date();

    let obj = make_log_obj(LogType::Info, &["hello"], "");
    let result = reporter.format(&obj, &ctx).unwrap();
    // FancyReporter shows icon for Info type + message
    assert!(result.contains('ℹ') || result.contains('i'));
    assert!(result.contains("hello"));

    // With tag
    let obj = make_log_obj(LogType::Info, &["hello"], "mytag");
    let result = reporter.format(&obj, &ctx).unwrap();
    assert!(result.contains("mytag"));
}

// ---------------------------------------------------------------------------
// Prompt stub tests
// ---------------------------------------------------------------------------

#[cfg(not(any(
    feature = "prompt",
    feature = "prompt-inquire",
    feature = "prompt-dialoguer"
)))]
#[test]
fn test_k_cancel_exists() {
    // K_CANCEL is defined in the prompt module
    assert_eq!(::consola::prompt::K_CANCEL, "Symbol(cancel)");
}

#[cfg(not(any(
    feature = "prompt",
    feature = "prompt-inquire",
    feature = "prompt-dialoguer"
)))]
#[test]
fn test_stub_k_cancel_exists() {
    assert_eq!(::consola::prompt::K_CANCEL, "Symbol(cancel)");
}

#[cfg(not(any(
    feature = "prompt",
    feature = "prompt-inquire",
    feature = "prompt-dialoguer"
)))]
#[test]
fn test_stub_returns_feature_error() {
    let opts = ::consola::prompt::TextPromptOptions {
        common: ::consola::prompt::PromptCommonOptions { cancel: None },
        r#type: None,
        default: None,
        placeholder: None,
        initial: None,
    };
    let result = ::consola::prompt::text("test", &opts);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("prompt"));
    assert!(err.contains("feature"));
}

#[cfg(not(any(
    feature = "prompt",
    feature = "prompt-inquire",
    feature = "prompt-dialoguer"
)))]
#[test]
fn test_stub_confirm_returns_feature_error() {
    let opts = ::consola::prompt::ConfirmPromptOptions {
        common: ::consola::prompt::PromptCommonOptions { cancel: None },
        r#type: "confirm".into(),
        initial: None,
    };
    let result = ::consola::prompt::confirm("confirm?", &opts);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("prompt"));
    assert!(err.contains("feature"));
}

#[cfg(not(any(
    feature = "prompt",
    feature = "prompt-inquire",
    feature = "prompt-dialoguer"
)))]
#[test]
fn test_stub_select_returns_feature_error() {
    use ::consola::prompt::SelectOption;
    let opts = ::consola::prompt::SelectPromptOptions {
        common: ::consola::prompt::PromptCommonOptions { cancel: None },
        r#type: "select".into(),
        initial: None,
        options: vec![SelectOption {
            label: "A".into(),
            value: "a".into(),
            hint: None,
        }],
    };
    let result = ::consola::prompt::select("pick one", &opts);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("prompt"));
    assert!(err.contains("feature"));
}

#[cfg(not(any(
    feature = "prompt",
    feature = "prompt-inquire",
    feature = "prompt-dialoguer"
)))]
#[test]
fn test_stub_multiselect_returns_feature_error() {
    use ::consola::prompt::SelectOption;
    let opts = ::consola::prompt::MultiSelectOptions {
        common: ::consola::prompt::PromptCommonOptions { cancel: None },
        r#type: "multiselect".into(),
        initial: None,
        options: vec![SelectOption {
            label: "A".into(),
            value: "a".into(),
            hint: None,
        }],
        required: None,
    };
    let result = ::consola::prompt::multiselect("pick", &opts);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("prompt"));
    assert!(err.contains("feature"));
}
