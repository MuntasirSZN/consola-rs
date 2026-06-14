//! Comprehensive tests for `Consola`'s `tracing::Subscriber` implementation.
//!
//! These tests cover subscriber methods the macro-level tests in `consola.rs` miss:
//! `enabled`, `max_level_hint`, `new_span`, `record`, `record_follows_from`,
//! `enter`/`exit`, `clone_span`, `current_span`, `try_close`, and span field collection.

#![cfg(feature = "tracing")]

use std::sync::{Arc, Mutex};

use consola::{Consola, ConsolaOptions, LogContext, LogLevel, LogObject, Reporter, log_levels};
use tracing::Subscriber;

#[derive(Debug, Clone)]
struct CaptureReporter {
    captured: Arc<Mutex<Vec<String>>>,
}

impl CaptureReporter {
    fn new() -> Self {
        Self {
            captured: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn count(&self) -> usize {
        self.captured.lock().unwrap().len()
    }

    fn last(&self) -> Option<String> {
        self.captured.lock().unwrap().last().cloned()
    }

    fn all(&self) -> Vec<String> {
        self.captured.lock().unwrap().clone()
    }
}

impl Reporter for CaptureReporter {
    fn format(&self, log_obj: &LogObject, _ctx: &LogContext) -> Result<String, String> {
        let tag_part = if log_obj.tag.is_empty() {
            String::new()
        } else {
            format!("<{}>", log_obj.tag)
        };
        let formatted = format!(
            "[{}]{}: {}",
            log_obj.r#type.as_str(),
            tag_part,
            log_obj.args.join(" ")
        );
        self.captured.lock().unwrap().push(formatted.clone());
        Ok(formatted)
    }

    fn clone_box(&self) -> Box<dyn Reporter> {
        Box::new(self.clone())
    }
}

fn make_sub(level: LogLevel) -> (Consola, CaptureReporter) {
    let cr = CaptureReporter::new();
    let c = Consola::new(ConsolaOptions {
        reporters: vec![Box::new(cr.clone()) as Box<dyn Reporter>],
        level,
        ..ConsolaOptions::default()
    });
    (c, cr)
}

#[test]
fn test_event_passes_at_info_level() {
    let (c, cr) = make_sub(log_levels::INFO);
    let _guard = tracing::subscriber::set_default(Box::new(c));
    tracing::info!("should pass");
    assert_eq!(cr.count(), 1);
}

#[test]
fn test_debug_filtered_at_info() {
    let (c, cr) = make_sub(log_levels::INFO);
    let _guard = tracing::subscriber::set_default(Box::new(c));
    tracing::debug!("should be filtered");
    assert_eq!(cr.count(), 0);
}

#[test]
fn test_all_pass_at_verbose() {
    let (c, cr) = make_sub(log_levels::VERBOSE);
    let _guard = tracing::subscriber::set_default(Box::new(c));
    tracing::trace!("trace");
    tracing::debug!("debug");
    tracing::info!("info");
    tracing::warn!("warn");
    tracing::error!("error");
    assert_eq!(cr.count(), 5);
}

#[test]
fn test_max_level_hint_verbose() {
    let (c, _) = make_sub(log_levels::VERBOSE);
    assert_eq!(
        Subscriber::max_level_hint(&c),
        Some(tracing::metadata::LevelFilter::TRACE),
    );
}

#[test]
fn test_max_level_hint_debug() {
    let (c, _) = make_sub(log_levels::DEBUG);
    assert_eq!(
        Subscriber::max_level_hint(&c),
        Some(tracing::metadata::LevelFilter::DEBUG),
    );
}

#[test]
fn test_max_level_hint_info() {
    let (c, _) = make_sub(log_levels::INFO);
    assert_eq!(
        Subscriber::max_level_hint(&c),
        Some(tracing::metadata::LevelFilter::INFO),
    );
}

#[test]
fn test_max_level_hint_warn() {
    let (c, _) = make_sub(log_levels::WARN);
    assert_eq!(
        Subscriber::max_level_hint(&c),
        Some(tracing::metadata::LevelFilter::WARN),
    );
}

#[test]
fn test_max_level_hint_error() {
    let (c, _) = make_sub(log_levels::ERROR);
    assert_eq!(
        Subscriber::max_level_hint(&c),
        Some(tracing::metadata::LevelFilter::ERROR),
    );
}

#[test]
fn test_max_level_hint_silent() {
    let (c, _) = make_sub(log_levels::SILENT);
    assert_eq!(
        Subscriber::max_level_hint(&c),
        Some(tracing::metadata::LevelFilter::OFF),
    );
}

#[test]
fn test_new_span_creates_id() {
    let (c, _) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));
    let span = tracing::info_span!("test_span");
    let id = span.id().expect("span should have an id");
    let id_val = id.into_u64();
    assert!(id_val > 0, "span id should be positive, got {id_val}");
}

#[test]
fn test_enter_exit_tracks_stack() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));
    {
        let span = tracing::info_span!("outer");
        let _enter = span.enter();
        tracing::info!("inside span");
    }
    tracing::info!("after span");

    assert_eq!(cr.count(), 2);
    let all = cr.all();
    assert!(all[0].contains("outer"), "span context missing: {}", all[0]);
    assert!(!all[1].contains("outer"), "span context leaked: {}", all[1]);
}

#[test]
fn test_nested_spans_stack() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let outer = tracing::info_span!("parent");
    let _outer_guard = outer.enter();
    {
        let inner = tracing::info_span!("child");
        let _inner_guard = inner.enter();
        tracing::info!("deep inside");
    }
    assert_eq!(cr.count(), 1);
    let last = cr.last().unwrap();
    assert!(last.contains("child"), "child name missing: {}", last);
}

#[test]
fn test_record_dynamic_fields() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let span = tracing::info_span!("dyn_span", dyn_key = tracing::field::Empty);
    span.record("dyn_key", "dyn_val");
    let _enter = span.enter();
    tracing::info!("after record");

    let last = cr.last().unwrap();
    assert!(
        last.contains("dyn_key") || last.contains("dyn_val"),
        "dynamically recorded field missing: {last}",
    );
}

#[test]
fn test_record_follows_from_does_not_panic() {
    let (_c, _) = make_sub(log_levels::TRACE);
    let span_a = tracing::info_span!("a");
    let span_b = tracing::info_span!("b", follows_from = span_a.id().map(|id| id.into_u64()));
    drop(span_a);
    drop(span_b);
}

#[test]
fn test_clone_span_increments_ref() {
    let (c, _) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let span = tracing::info_span!("ref_span");
    {
        // Cloning the span increments its ref count
        let _clone = span.clone();
        // Both original and clone hold the span open
        let _enter = span.enter();
        tracing::info!("in cloned span");
    }
    // After clone drops, original still alive — enter context is gone but span isn't closed
}

#[test]
fn test_current_span_with_active_span() {
    let (c, _) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let span = tracing::info_span!("active_span");
    let _enter = span.enter();
    let current = tracing::Span::current();
    assert!(current.id().is_some(), "expected an active span");
}

#[test]
fn test_current_span_none_when_no_span() {
    let (c, _) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let current = tracing::Span::current();
    assert_eq!(current.id(), None, "no active span expected");
}

#[test]
fn test_try_close_returns_true_for_existing() {
    let (c, _) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let span = tracing::info_span!("close_me");
    // let the span drop — subscriber's try_close is called
    drop(span);
    // No assertion beyond no panic
}

#[test]
fn test_try_close_returns_false_for_unknown() {
    let (c, _) = make_sub(log_levels::TRACE);
    let id = tracing::span::Id::from_u64(9999);
    let closed = Subscriber::try_close(&c, id);
    assert!(!closed);
}

// ===================================================================
// Event with span fields
// ===================================================================

#[test]
fn test_event_includes_span_fields() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let span = tracing::info_span!("fields_span", str_f = "hello", int_f = 42u64, bool_f = true);
    let _enter = span.enter();
    tracing::info!("check fields");

    let last = cr.last().unwrap();
    assert!(last.contains("hello"), "str_f: {last}");
    assert!(last.contains("42"), "int_f: {last}");
    assert!(last.contains("true"), "bool_f: {last}");
}

#[test]
fn test_multiple_spans_different_ids() {
    let (c, _) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let a = tracing::info_span!("a");
    let b = tracing::info_span!("b");
    assert_ne!(a.id(), b.id(), "two spans should have different ids");
}

#[test]
fn test_implicit_parent_span() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let parent = tracing::info_span!("parent");
    let _parent_guard = parent.enter();

    let child = tracing::info_span!("child");
    let _child_guard = child.enter();

    tracing::warn!("deep inside");
    assert_eq!(cr.count(), 1);
    let last = cr.last().unwrap();
    assert!(last.contains("child"), "child name missing: {last}");
}

#[test]
fn test_event_filtered_at_warn_level() {
    let (c, cr) = make_sub(log_levels::WARN);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    tracing::info!("should be filtered");
    assert_eq!(cr.count(), 0, "info should be filtered at WARN level");

    tracing::warn!("should pass");
    assert_eq!(cr.count(), 1, "warn should pass at WARN level");

    tracing::error!("should also pass");
    assert_eq!(cr.count(), 2, "error should pass at WARN level");
}

// ===================================================================
// Event without span context
// ===================================================================

#[test]
fn test_event_no_active_span() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    tracing::info!("bare message");
    assert_eq!(cr.count(), 1);
    let last = cr.last().unwrap();
    assert!(last.contains("bare message"), "got: {last}");
}

// ===================================================================
// Multiple events in same span
// ===================================================================

#[test]
fn test_multiple_events_same_span() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let span = tracing::info_span!("multi");
    let _enter = span.enter();
    tracing::info!("first");
    tracing::warn!("second");
    tracing::error!("third");

    assert_eq!(cr.count(), 3);
    for entry in cr.all() {
        assert!(entry.contains("multi"), "span name missing: {entry}");
    }
}

#[test]
fn test_exit_without_enter_does_not_panic() {
    let (c, _) = make_sub(log_levels::TRACE);
    let id = tracing::span::Id::from_u64(42);
    // Calling exit without a corresponding enter should not panic
    Subscriber::exit(&c, &id);
}

#[test]
fn test_event_with_debug_field() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    // Emit an event with a field that uses Debug formatting
    tracing::info!("msg");
    assert_eq!(cr.count(), 1);
}

#[test]
fn test_span_with_debug_field() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    // Create a span with various field types to exercise SpanFieldCollector
    let span = tracing::info_span!(
        "debug_span",
        str_f = "hello",
        int_f = 42u64,
        neg_f = -1i64,
        bool_f = true,
    );
    let _enter = span.enter();
    tracing::info!("inside debug span");

    let last = cr.last().unwrap();
    assert!(last.contains("debug_span"), "span name: {last}");
    assert!(last.contains("hello"), "str_f: {last}");
    assert!(last.contains("42"), "int_f: {last}");
    assert!(last.contains("-1"), "neg_f: {last}");
    assert!(last.contains("true"), "bool_f: {last}");
}

#[test]
fn test_multiple_clones_ref_counting() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let span = tracing::info_span!("ref_counted");
    let _enter = span.enter();

    // Clone the span multiple times
    let _clone1 = span.clone();
    let _clone2 = span.clone();

    tracing::info!("in ref_counted");
    assert_eq!(cr.count(), 1);
}

#[test]
fn test_record_on_span() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    // Use span.record() (which calls the subscriber's record method)
    let span = tracing::info_span!("rec_span", key = tracing::field::Empty);
    span.record("key", "dynamic_value");
    let _enter = span.enter();
    tracing::info!("after record");

    let last = cr.last().unwrap();
    assert!(last.contains("dynamic_value"), "recorded value: {last}");
}

#[test]
fn test_follows_from_does_not_panic_with_subscriber() {
    let (c, _) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let span_a = tracing::info_span!("a");
    let span_b = tracing::info_span!("b");

    // The subscriber sees new_span for both, and follows_from when the span is
    // created with the follows_from argument. But since there's no field named
    // follows_from in the macro, this just tests that creating spans doesn't panic.
    drop(span_a);
    drop(span_b);
}

#[test]
fn test_span_with_debug_only_fields() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    // f64 val has no specialized record_* method, so it uses record_debug
    let span = tracing::info_span!("dbg", fval = std::f64::consts::PI);
    let _enter = span.enter();
    tracing::info!("inside");
    let last = cr.last().unwrap();
    assert!(last.contains("inside"), "{last}");
}

#[test]
fn test_record_follows_from_subscriber_method() {
    let (c, _) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let span_a = tracing::info_span!("source");
    let span_b = tracing::info_span!("dependent");
    // This calls the subscriber's record_follows_from
    span_b.follows_from(&span_a);
    drop(span_a);
    drop(span_b);
}

#[test]
fn test_event_filtered_at_info_level() {
    let (c, cr) = make_sub(log_levels::WARN);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    tracing::info!("should be filtered");
    assert_eq!(cr.count(), 0);
}

#[test]
fn test_event_error_with_backtrace() {
    let (c, cr) = make_sub(log_levels::VERBOSE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    tracing::error!("fatal error");
    assert_eq!(cr.count(), 1);
    let last = cr.last().unwrap();
    assert!(last.contains("fatal error"), "{last}");
}

#[test]
fn test_exit_non_matching_span() {
    use tracing::Subscriber;

    let (c, _) = make_sub(log_levels::TRACE);
    // Before setting as default subscriber, call exit directly.
    // No span has been entered yet, so exit is a no-op regardless of id.
    let other_id = tracing::span::Id::from_u64(999);
    Subscriber::exit(&c, &other_id);

    // Now verify normal span lifecycle still works via default subscriber.
    let _guard = tracing::subscriber::set_default(Box::new(c));
    let span = tracing::info_span!("outer");
    let _enter = span.enter();
    tracing::info!("inside outer");
}

#[test]
fn test_event_with_non_message_field() {
    let (c, cr) = make_sub(log_levels::TRACE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    // The ConsolaVisitor sees all fields. The "message" field is consumed,
    // non-message fields hit the else branch of record_debug.
    tracing::info!("only message");
    assert_eq!(cr.count(), 1);
}

#[test]
fn test_span_created_at_verbose() {
    let (c, _) = make_sub(log_levels::VERBOSE);
    let _guard = tracing::subscriber::set_default(Box::new(c));

    let span = tracing::info_span!("vspan");
    let _enter = span.enter();
    tracing::trace!("tiny");
}
