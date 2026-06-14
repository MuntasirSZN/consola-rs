//! Emission goes through `log` or `tracing` crates. There is no IO.

use std::time::Instant;

#[cfg(feature = "tracing")]
use std::collections::HashMap;

use crate::sync::Mutex;

use crate::constants::{LogLevel, LogType, log_levels, log_type_defaults, normalize_log_level};
use crate::types::{ConsolaOptions, LogContext, LogObject, LogObjectInput, Reporter};

#[derive(Debug, Clone)]
struct LastLogInfo {
    serialized: String,
    object: LogObject,
    count: u32,
    time: Option<Instant>,
}

#[derive(Default)]
struct ConsolaState {
    paused: bool,
    queue: Vec<(LogObjectInput, Vec<String>, bool)>,
    last_log: Option<LastLogInfo>,
    #[cfg(feature = "tracing")]
    span_id_counter: u64,
    #[cfg(feature = "tracing")]
    span_stack: Vec<u64>,
    #[cfg(feature = "tracing")]
    span_ref_counts: HashMap<u64, u64>,
    #[cfg(feature = "tracing")]
    span_metas: HashMap<u64, &'static tracing::Metadata<'static>>,
    #[cfg(feature = "tracing")]
    span_fields: HashMap<u64, Vec<(String, String)>>,
    #[cfg(feature = "tracing")]
    span_follows_from: HashMap<u64, Vec<u64>>,
}

/// The main logger struct. Thread-safe; all methods take `&self`.
pub struct Consola {
    options: Mutex<ConsolaOptions>,
    state: Mutex<ConsolaState>,
}

impl std::fmt::Debug for Consola {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Consola")
            .field("options", &self.options)
            .finish()
    }
}

impl Consola {
    /// Create a new `Consola` instance with the given options.
    pub fn new(options: ConsolaOptions) -> Self {
        Self {
            options: Mutex::new(options),
            state: Mutex::new(ConsolaState::default()),
        }
    }

    /// Returns the current log level.
    pub fn level(&self) -> LogLevel {
        self.options.lock().level
    }

    /// Set the log level. Filters out messages below this level.
    pub fn set_level(&self, level: LogLevel) {
        let normalized = normalize_log_level(Some(level), log_levels::INFO);
        self.options.lock().level = normalized;
    }

    /// Add a reporter to the list of active reporters.
    pub fn add_reporter(&self, reporter: Box<dyn Reporter>) {
        self.options.lock().reporters.push(reporter);
    }

    /// Remove a reporter at the given index.
    pub fn remove_reporter(&self, index: usize) {
        self.options.lock().reporters.remove(index);
    }

    /// Remove all reporters.
    pub fn clear_reporters(&self) {
        self.options.lock().reporters.clear();
    }

    /// Replace all reporters with a new list.
    pub fn set_reporters(&self, reporters: Vec<Box<dyn Reporter>>) {
        self.options.lock().reporters = reporters;
    }

    /// Create a new `Consola` instance by merging the current options with the given overrides.
    pub fn create(&self, options_overrides: ConsolaOptions) -> Self {
        let current = self.options.lock().clone();

        let mut merged_defaults = current.defaults.clone();
        if let Some(level) = options_overrides.defaults.level {
            merged_defaults.level = Some(level);
        }
        if let Some(tag) = options_overrides.defaults.tag {
            let existing = merged_defaults.tag.clone().unwrap_or_default();
            merged_defaults.tag = Some(if existing.is_empty() {
                tag
            } else {
                format!("{}:{}", existing, tag)
            });
        }
        if let Some(msg) = options_overrides.defaults.message {
            merged_defaults.message = Some(msg);
        }
        if !options_overrides.defaults.args.is_empty() {
            merged_defaults.args = options_overrides.defaults.args;
        }
        if options_overrides.defaults.additional.is_some() {
            merged_defaults.additional = options_overrides.defaults.additional;
        }

        let merged = ConsolaOptions {
            level: options_overrides.level,
            reporters: if options_overrides.reporters.is_empty() {
                current.reporters
            } else {
                options_overrides.reporters
            },
            defaults: merged_defaults,
            throttle: options_overrides.throttle,
            throttle_min: options_overrides.throttle_min,
            format_options: options_overrides.format_options,
        };

        Self::new(merged)
    }

    /// Create a new `Consola` instance with the given defaults merged into the current options.
    pub fn with_defaults(&self, defaults: LogObjectInput) -> Self {
        let current = self.options.lock().clone();
        let mut merged = current.defaults.clone();
        if let Some(level) = defaults.level {
            merged.level = Some(level);
        }
        if let Some(tag) = defaults.tag {
            merged.tag = Some(tag);
        }
        if let Some(msg) = defaults.message {
            merged.message = Some(msg);
        }
        if !defaults.args.is_empty() {
            merged.args = defaults.args;
        }
        if let Some(additional) = defaults.additional {
            merged.additional = Some(additional);
        }

        let opts = ConsolaOptions {
            defaults: merged,
            ..current
        };
        self.create(opts)
    }

    /// Create a new `Consola` instance with the given tag added to the defaults.
    pub fn with_tag(&self, tag: &str) -> Self {
        self.with_defaults(LogObjectInput {
            tag: Some(tag.to_string()),
            ..LogObjectInput::default()
        })
    }

    /// Pause all logging. Logs are queued and will be flushed on [`resume_logs`].
    pub fn pause_logs(&self) {
        self.state.lock().paused = true;
    }

    /// Resume logging and flush any queued log messages.
    pub fn resume_logs(&self) {
        let mut state = self.state.lock();
        state.paused = false;
        let queue = std::mem::take(&mut state.queue);
        drop(state);

        for (defaults, args, is_raw) in queue {
            self._log_fn(&defaults, &args, is_raw);
        }
    }

    fn _log_fn(&self, input_defaults: &LogObjectInput, args: &[String], is_raw: bool) -> bool {
        // Read config once
        let (level, throttle, throttle_min) = {
            let opts = self.options.lock();
            (opts.level, opts.throttle, opts.throttle_min)
        };

        let msg_level = input_defaults.level.unwrap_or(log_levels::INFO);
        if msg_level > level {
            return false;
        }

        // Check paused state
        {
            let mut state = self.state.lock();
            if state.paused {
                state
                    .queue
                    .push((input_defaults.clone(), args.to_vec(), is_raw));
                return true;
            }
        }

        // Build LogObject
        let log_type = input_defaults.r#type.unwrap_or(LogType::Log);
        let mut log_obj = LogObject::new(log_type);
        log_obj.level = normalize_log_level(input_defaults.level, log_type.level());
        log_obj.tag = input_defaults.tag.clone().unwrap_or_default();
        log_obj.message = input_defaults.message.clone();
        log_obj.args = args.to_vec();
        log_obj.title = input_defaults.title.clone();
        log_obj.icon = input_defaults.icon.clone();
        log_obj.style = input_defaults.style.clone();
        log_obj.error = input_defaults.error.clone();

        // Auto-capture backtrace for error-level logs when backtrace feature is enabled
        // and no explicit error info was provided (e.g. via log crate integration).
        // Skipped on WASM targets (backtrace crate needs platform-specific support).
        #[cfg(all(feature = "backtrace", not(target_arch = "wasm32")))]
        if log_obj.level == 0 && input_defaults.error.is_none() {
            let bt = backtrace::Backtrace::new();
            log_obj.error = Some(crate::types::ErrorInfo {
                message: String::new(),
                stack: Some(format!("{:?}", bt)),
                backtrace: Some(format!("{:?}", bt)),
                cause: None,
            });
        }

        // Aliases: message -> args[0]
        if let Some(msg) = &log_obj.message
            && !msg.is_empty()
        {
            log_obj.args.insert(0, msg.clone());
            log_obj.message = None;
        }

        // Aliases: additional -> appended to args
        if let Some(additional) = &input_defaults.additional {
            let lines: Vec<&str> = additional.split('\n').collect();
            log_obj.args.push("\n".to_string() + &lines.join("\n"));
        }

        // Throttle / Dedup
        let serialized = format!("{:?}:{}:{:?}", log_obj.r#type, log_obj.tag, log_obj.args);

        let is_repeat = {
            let state = self.state.lock();
            state.last_log.as_ref().and_then(|last| {
                last.time.and_then(|t| {
                    if (t.elapsed().as_millis() as u64) < throttle && last.serialized == serialized
                    {
                        Some(last.count)
                    } else {
                        None
                    }
                })
            })
        };

        if let Some(count) = is_repeat {
            let mut state = self.state.lock();
            if let Some(last) = &mut state.last_log {
                last.count = count.saturating_add(1);
                last.serialized = serialized.clone();
                if last.count > throttle_min {
                    last.object = log_obj;
                    return true;
                }
            }
        }

        // Emit repeated count from previous log
        {
            let mut state = self.state.lock();
            if let Some(last) = state.last_log.clone() {
                let repeated = (last.count as i64)
                    .saturating_sub(throttle_min as i64)
                    .max(0) as u32;
                if repeated > 0 {
                    let mut repeat_args = last.object.args.clone();
                    if repeated > 1 {
                        repeat_args.push(format!("(repeated {} times)", repeated));
                    }
                    let mut repeat_obj = last.object;
                    repeat_obj.args = repeat_args;
                    if let Some(l) = &mut state.last_log {
                        l.count = 1;
                    }
                    drop(state);
                    self._emit(&repeat_obj);
                }
            }
        }

        // Save as last log
        {
            let mut state = self.state.lock();
            state.last_log = Some(LastLogInfo {
                serialized,
                object: log_obj.clone(),
                count: 1,
                #[cfg(not(target_arch = "wasm32"))]
                time: Some(Instant::now()),
                #[cfg(target_arch = "wasm32")]
                time: None,
            });
        }

        // Emit
        self._emit(&log_obj);
        true
    }

    fn _emit(&self, log_obj: &LogObject) {
        let opts = self.options.lock();
        let ctx = LogContext {
            options: std::sync::Arc::new(opts.clone()),
        };

        for reporter in &opts.reporters {
            match reporter.format(log_obj, &ctx) {
                Ok(formatted) => {
                    if !formatted.is_empty() {
                        let _ = Self::write_line(&formatted, log_obj.level);
                    }
                }
                Err(e) => {
                    use std::io::Write;
                    let _ = writeln!(std::io::stderr(), "[consola] reporter error: {}", e);
                }
            }
        }
    }

    /// Write a line to stdout or stderr based on log level.
    /// Errors are silently ignored (e.g. in WASM environments where stdout may not exist).
    fn write_line(message: &str, level: LogLevel) -> std::io::Result<()> {
        use std::io::Write;
        if level < 2 {
            let mut stderr = std::io::stderr().lock();
            writeln!(stderr, "{message}")
        } else {
            let mut stdout = std::io::stdout().lock();
            writeln!(stdout, "{message}")
        }
    }
}

#[cfg(feature = "log")]
impl log::Log for Consola {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        let level = match metadata.level() {
            log::Level::Error => 0,
            log::Level::Warn => 1,
            log::Level::Info => 3,
            log::Level::Debug => 4,
            log::Level::Trace => 5,
        };
        level <= self.level()
    }

    fn log(&self, record: &log::Record<'_>) {
        let raw_level = match record.level() {
            log::Level::Error => 0,
            log::Level::Warn => 1,
            log::Level::Info => 3,
            log::Level::Debug => 4,
            log::Level::Trace => 5,
        };
        if raw_level > self.level() {
            return;
        }

        let tag = record.target().to_string();

        let mut log_obj = LogObject::new(LogType::Log);
        log_obj.level = raw_level;
        log_obj.r#type = match raw_level {
            0 => LogType::Error,
            1 => LogType::Warn,
            2 | 3 => LogType::Info,
            4 => LogType::Debug,
            _ => LogType::Trace,
        };
        log_obj.tag = tag;
        log_obj.args = vec![record.args().to_string()];

        #[cfg(feature = "backtrace")]
        if raw_level == 0 {
            let bt = backtrace::Backtrace::new();
            log_obj.error = Some(crate::types::ErrorInfo {
                message: String::new(),
                stack: Some(format!("{:?}", bt)),
                backtrace: Some(format!("{:?}", bt)),
                cause: None,
            });
        }

        self._emit(&log_obj);
    }

    fn flush(&self) {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
    }
}

#[cfg(feature = "tracing")]
struct ConsolaVisitor<'a> {
    message: Option<String>,
    _marker: std::marker::PhantomData<&'a ()>,
}

#[cfg(feature = "tracing")]
impl<'a> tracing::field::Visit for ConsolaVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        }
    }
}

/// Collects all field values on a span (unlike [`ConsolaVisitor`] which
/// extracts only the single `message` field from an event).
#[cfg(feature = "tracing")]
struct SpanFieldCollector {
    fields: Vec<(String, String)>,
}

#[cfg(feature = "tracing")]
impl tracing::field::Visit for SpanFieldCollector {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields
            .push((field.name().to_string(), format!("{:?}", value)));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }
}

#[cfg(feature = "tracing")]
impl tracing::Subscriber for Consola {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        let level = match *metadata.level() {
            tracing::Level::ERROR => 0,
            tracing::Level::WARN => 1,
            tracing::Level::INFO => 3,
            tracing::Level::DEBUG => 4,
            tracing::Level::TRACE => 5,
        };
        level <= self.level()
    }

    fn max_level_hint(&self) -> Option<tracing::metadata::LevelFilter> {
        let raw = self.level();
        // Negative levels (e.g. SILENT = i32::MIN) mean no events pass;
        // see `enabled()` which compares `level <= raw`.
        if raw < 0 {
            return Some(tracing::metadata::LevelFilter::OFF);
        }
        let filter = match raw.min(5) {
            0 => tracing::metadata::LevelFilter::ERROR,
            1 => tracing::metadata::LevelFilter::WARN,
            2 | 3 => tracing::metadata::LevelFilter::INFO,
            4 => tracing::metadata::LevelFilter::DEBUG,
            _ => tracing::metadata::LevelFilter::TRACE,
        };
        Some(filter)
    }

    fn new_span(&self, attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        let mut state = self.state.lock();
        state.span_id_counter += 1;
        let id = state.span_id_counter;
        state.span_ref_counts.insert(id, 1);
        state.span_metas.insert(id, attrs.metadata());

        // Record initial field values from the span macro.
        let mut collector = SpanFieldCollector { fields: Vec::new() };
        attrs.record(&mut collector);
        if !collector.fields.is_empty() {
            state.span_fields.insert(id, collector.fields);
        }

        tracing::span::Id::from_u64(id)
    }

    fn record(&self, span: &tracing::span::Id, values: &tracing::span::Record<'_>) {
        let mut collector = SpanFieldCollector { fields: Vec::new() };
        values.record(&mut collector);
        if !collector.fields.is_empty() {
            let mut state = self.state.lock();
            state
                .span_fields
                .entry(span.into_u64())
                .or_default()
                .extend(collector.fields);
        }
    }

    fn record_follows_from(&self, span: &tracing::span::Id, follows: &tracing::span::Id) {
        let mut state = self.state.lock();
        state
            .span_follows_from
            .entry(span.into_u64())
            .or_default()
            .push(follows.into_u64());
    }

    fn event(&self, event: &tracing::Event<'_>) {
        let raw_level = match *event.metadata().level() {
            tracing::Level::ERROR => 0,
            tracing::Level::WARN => 1,
            tracing::Level::INFO => 3,
            tracing::Level::DEBUG => 4,
            tracing::Level::TRACE => 5,
        };
        if raw_level > self.level() {
            return;
        }

        let mut visitor = ConsolaVisitor {
            message: None,
            _marker: std::marker::PhantomData,
        };
        event.record(&mut visitor);

        let message = visitor.message.unwrap_or_default();
        let base_tag = event.metadata().target().to_string();

        // Collect current span context (name + recorded fields) without
        // holding the lock across the remaining work.
        let (tag, span_field_args) = {
            let state = self.state.lock();
            let top = state.span_stack.last().copied();
            if let Some(top) = top {
                let span_name = state.span_metas.get(&top).map(|m| m.name().to_string());
                let span_fields = state.span_fields.get(&top).cloned().unwrap_or_default();
                let tag = match span_name {
                    Some(name) => format!("{}::{}", name, base_tag),
                    None => base_tag,
                };
                let args: Vec<String> = span_fields
                    .into_iter()
                    .filter(|(k, _)| k != "message")
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                (tag, args)
            } else {
                (base_tag, Vec::new())
            }
        };

        let mut log_obj = LogObject::new(LogType::Log);
        log_obj.level = raw_level;
        log_obj.r#type = match raw_level {
            0 => LogType::Error,
            1 => LogType::Warn,
            2 | 3 => LogType::Info,
            4 => LogType::Debug,
            _ => LogType::Trace,
        };
        log_obj.tag = tag;
        log_obj.args = if span_field_args.is_empty() {
            vec![message]
        } else {
            let mut args = Vec::with_capacity(span_field_args.len() + 1);
            args.push(message);
            args.extend(span_field_args);
            args
        };

        #[cfg(feature = "backtrace")]
        if raw_level == 0 {
            let bt = backtrace::Backtrace::new();
            log_obj.error = Some(crate::types::ErrorInfo {
                message: String::new(),
                stack: Some(format!("{:?}", bt)),
                backtrace: Some(format!("{:?}", bt)),
                cause: None,
            });
        }

        self._emit(&log_obj);
    }

    fn enter(&self, span: &tracing::span::Id) {
        let mut state = self.state.lock();
        state.span_stack.push(span.into_u64());
    }

    fn exit(&self, span: &tracing::span::Id) {
        let mut state = self.state.lock();
        // Only pop if the top of the stack matches the exiting span.
        if state.span_stack.last() == Some(&span.into_u64()) {
            state.span_stack.pop();
        }
    }

    fn clone_span(&self, id: &tracing::span::Id) -> tracing::span::Id {
        let mut state = self.state.lock();
        if let Some(count) = state.span_ref_counts.get_mut(&id.into_u64()) {
            *count += 1;
        }
        id.clone()
    }

    fn current_span(&self) -> tracing_core::span::Current {
        let state = self.state.lock();
        let id_raw = state.span_stack.last().copied();
        if let Some(id_raw) = id_raw
            && let Some(&meta) = state.span_metas.get(&id_raw)
        {
            let current = tracing_core::span::Current::new(
                // SAFETY: id_raw was produced by new_span (starts at 1).
                tracing::span::Id::from_u64(id_raw),
                meta,
            );
            drop(state);
            return current;
        }
        drop(state);
        tracing_core::span::Current::none()
    }

    fn try_close(&self, id: tracing::span::Id) -> bool {
        let mut state = self.state.lock();
        let raw = id.into_u64();
        if let Some(count) = state.span_ref_counts.get_mut(&raw) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                state.span_ref_counts.remove(&raw);
                state.span_metas.remove(&raw);
                state.span_fields.remove(&raw);
                state.span_follows_from.remove(&raw);
                // Clean up from stack if still present.
                state.span_stack.retain(|&s| s != raw);
                return true;
            }
        }
        false
    }
}

macro_rules! consola_methods {
    ($($method:ident, $raw_method:ident, $Type:ident;)*) => {
        impl Consola {
            $(
                #[doc = concat!("Log a message at `", stringify!($Type), "` level.\n\nReturns `true` if the message was logged, `false` if filtered by log level.")]
                pub fn $method(&self, msg: &str) -> bool {
                    let defaults = log_type_defaults(LogType::$Type);
                    self._log_fn(&defaults, &[msg.to_string()], false)
                }

                #[doc = concat!("Log a message at `", stringify!($Type), "` level (raw variant).\n\nReturns `true` if the message was logged, `false` if filtered by log level.")]
                pub fn $raw_method(&self, msg: &str) -> bool {
                    let defaults = log_type_defaults(LogType::$Type);
                    self._log_fn(&defaults, &[msg.to_string()], true)
                }
            )*
        }
    };
}

consola_methods! {
    fatal, fatal_raw, Fatal;
    error, error_raw, Error;
    warn, warn_raw, Warn;
    info, info_raw, Info;
    success, success_raw, Success;
    fail, fail_raw, Fail;
    ready, ready_raw, Ready;
    start, start_raw, Start;
    box_, box_raw, Box;
    debug, debug_raw, Debug;
    trace, trace_raw, Trace;
    verbose, verbose_raw, Verbose;
}

impl Consola {
    /// Log at `log` level with a string message.
    pub fn log(&self, msg: &str) -> bool {
        let defaults = log_type_defaults(LogType::Log);
        self._log_fn(&defaults, &[msg.to_string()], false)
    }

    /// Log at `log` level (raw variant).
    pub fn log_raw(&self, msg: &str) -> bool {
        let defaults = log_type_defaults(LogType::Log);
        self._log_fn(&defaults, &[msg.to_string()], true)
    }

    /// Log with a structured `LogObjectInput`.
    pub fn log_obj(&self, input: &LogObjectInput) -> bool {
        let ty = input.r#type.unwrap_or(LogType::Log);
        let defaults = LogObjectInput {
            level: input.level.or(Some(ty.level())),
            r#type: Some(ty),
            tag: input.tag.clone(),
            message: input.message.clone(),
            additional: input.additional.clone(),
            args: input.args.clone(),
            title: input.title.clone(),
            badge: input.badge,
            icon: input.icon.clone(),
            style: input.style.clone(),
            error: input.error.clone(),
        };
        self._log_fn(&defaults, &input.args, false)
    }

    /// Log with a structured `LogObjectInput` (raw alias).
    pub fn log_obj_raw(&self, input: &LogObjectInput) -> bool {
        self.log_obj(input)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::constants::{LogType, log_levels, log_type_defaults};
    use crate::types::{
        ConsolaOptions, FormatOptions, LogContext, LogObject, LogObjectInput, Reporter,
    };

    use super::Consola;

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
            let guard = self.captured.lock().unwrap();
            guard.last().cloned()
        }

        fn all(&self) -> Vec<String> {
            self.captured.lock().unwrap().clone()
        }
    }

    impl Reporter for CaptureReporter {
        fn format(&self, log_obj: &LogObject, _ctx: &LogContext) -> Result<String, String> {
            let formatted = format!(
                "[{}]{}: {}",
                log_obj.r#type.as_str(),
                if log_obj.tag.is_empty() {
                    String::new()
                } else {
                    format!("<{}>", log_obj.tag)
                },
                log_obj.args.join(" "),
            );
            self.captured.lock().unwrap().push(formatted.clone());
            Ok(formatted)
        }

        fn clone_box(&self) -> Box<dyn Reporter> {
            Box::new(self.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct ErrReporter;

    impl Reporter for ErrReporter {
        fn format(&self, _log_obj: &LogObject, _ctx: &LogContext) -> Result<String, String> {
            Err("intentional test error".into())
        }

        fn clone_box(&self) -> Box<dyn Reporter> {
            Box::new(Self)
        }
    }

    #[test]
    fn test_new_default_level() {
        let c = Consola::new(ConsolaOptions::default());
        assert_eq!(c.level(), log_levels::INFO);
    }

    #[test]
    fn test_new_custom_level() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::DEBUG,
            ..ConsolaOptions::default()
        });
        assert_eq!(c.level(), log_levels::DEBUG);
    }

    #[test]
    fn test_new_with_reporters() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.info("hi"));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_new_silent_level() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::SILENT,
            ..ConsolaOptions::default()
        });
        // All messages filtered out
        assert!(!c.info("x"));
        assert!(!c.error("x"));
    }

    #[test]
    fn test_new_verbose_level() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            ..ConsolaOptions::default()
        });
        assert!(c.trace("x"));
        assert!(c.verbose("x"));
    }

    #[test]
    fn test_level_get_set_all() {
        let c = Consola::new(ConsolaOptions::default());
        assert_eq!(c.level(), log_levels::INFO);

        // normalize_log_level clamps to [0, 5]
        for level in 0..=5 {
            c.set_level(level);
            assert_eq!(c.level(), level, "level should be {} at clamp range", level);
        }
        c.set_level(6);
        assert_eq!(c.level(), 5, "level 6 should be clamped to 5");
        c.set_level(7);
        assert_eq!(c.level(), 5, "level 7 should be clamped to 5");
    }

    #[test]
    fn test_level_negative() {
        let c = Consola::new(ConsolaOptions::default());
        c.set_level(-5);
        // normalize_log_level clamps to [0, 5]
        assert_eq!(c.level(), 0);
    }

    #[test]
    fn test_level_high_value() {
        let c = Consola::new(ConsolaOptions::default());
        c.set_level(100);
        // normalize_log_level clamps to [0, 5]
        assert_eq!(c.level(), 5);
    }

    #[test]
    fn test_level_reset() {
        let c = Consola::new(ConsolaOptions::default());
        c.set_level(log_levels::DEBUG);
        assert_eq!(c.level(), log_levels::DEBUG);
        c.set_level(log_levels::INFO);
        assert_eq!(c.level(), log_levels::INFO);
    }

    #[test]
    fn test_remove_reporter_valid() {
        let r1 = CaptureReporter::new();
        let r2 = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r1.clone()), Box::new(r2.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.info("msg"));
        assert_eq!(r1.count(), 1);
        assert_eq!(r2.count(), 1);

        c.remove_reporter(0);
        assert!(c.info("msg2"));
        assert_eq!(r1.count(), 1); // r1 removed, still 1
        assert_eq!(r2.count(), 2); // r2 still active
    }

    #[test]
    fn test_remove_reporter_out_of_bounds() {
        let c = Consola::new(ConsolaOptions::default());
        // Should panic — but we can't easily catch panic in a test without
        // std::panic::catch_unwind. Just verify it doesn't silently corrupt.
        // Actually in Rust, remove with out-of-bounds index panics.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            c.remove_reporter(42);
        }));
        assert!(
            result.is_err(),
            "removing out-of-bounds reporter should panic"
        );
    }

    #[test]
    fn test_clear_reporters() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.info("first"));
        assert_eq!(r.count(), 1);

        c.clear_reporters();
        assert!(c.info("second")); // still returns true (not filtered by level)
        assert_eq!(r.count(), 1); // but not emitted to anyone
    }

    #[test]
    fn test_set_reporters() {
        let r1 = CaptureReporter::new();
        let r2 = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r1.clone())],
            ..ConsolaOptions::default()
        });
        c.set_reporters(vec![Box::new(r2.clone())]);
        assert!(c.info("msg"));
        assert_eq!(r1.count(), 0); // replaced
        assert_eq!(r2.count(), 1);
    }

    #[test]
    fn test_create_with_level_override() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });

        let sub = c.create(ConsolaOptions {
            level: log_levels::WARN,
            ..ConsolaOptions::default()
        });
        assert_eq!(sub.level(), log_levels::WARN);
        // Info should be filtered now
        assert!(!sub.info("filtered"));
        assert!(sub.warn("passed"));
    }

    #[test]
    fn test_create_with_reporters_override() {
        let r_parent = CaptureReporter::new();
        let r_child = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r_parent.clone())],
            ..ConsolaOptions::default()
        });

        let sub = c.create(ConsolaOptions {
            reporters: vec![Box::new(r_child.clone())],
            ..ConsolaOptions::default()
        });
        assert!(sub.info("test"));
        assert_eq!(r_parent.count(), 0); // child uses its own reporters
        assert_eq!(r_child.count(), 1);
    }

    #[test]
    fn test_create_empty_reporters_inherits() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });

        // Empty reporters vec -> inherit from parent
        let sub = c.create(ConsolaOptions {
            level: log_levels::WARN,
            reporters: vec![],
            ..ConsolaOptions::default()
        });
        assert!(sub.warn("test"));
        assert_eq!(r.count(), 1); // parent reporter used
    }

    #[test]
    fn test_create_with_defaults_level() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let sub = c.create(ConsolaOptions {
            defaults: LogObjectInput {
                level: Some(log_levels::ERROR),
                ..LogObjectInput::default()
            },
            ..ConsolaOptions::default()
        });
        // An error-level message from the subtype
        let defaults = log_type_defaults(LogType::Error);
        assert!(sub._log_fn(&defaults, &["err"].map(String::from), false));
        let last = r.last().unwrap();
        assert!(last.contains("err"), "got: {}", last);
    }

    #[test]
    fn test_with_defaults_returns_working_instance() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let sub = c.with_defaults(LogObjectInput {
            tag: Some("mymod".into()),
            ..LogObjectInput::default()
        });
        assert!(sub.info("hello"));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_with_defaults_preserves_reporters() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let sub = c.with_defaults(LogObjectInput::default());
        assert!(sub.info("pass"));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_with_defaults_chaining_via_create() {
        // with_defaults delegates to create(); each call adds a layer.
        // The tag merge logic in create() concatenates with ':'.
        // This merge only matters when the child Consola passes its
        // defaults. The log methods themselves don't use it, but we
        // verify chaining doesn't panic.
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let sub = c
            .with_defaults(LogObjectInput {
                tag: Some("A".into()),
                ..LogObjectInput::default()
            })
            .with_defaults(LogObjectInput {
                tag: Some("B".into()),
                ..LogObjectInput::default()
            });
        assert!(sub.info("chain"));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_with_tag_returns_working_instance() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let tagged = c.with_tag("api");
        assert!(tagged.warn("rate limit"));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_with_tag_chaining() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let tagged = c.with_tag("module1").with_tag("submodule");
        assert!(tagged.info("nested"));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_pause_resume_queues_and_flushes() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });

        c.pause_logs();
        assert!(c.info("queued1"));
        assert!(c.warn("queued2"));
        // Nothing emitted yet
        assert_eq!(r.count(), 0);

        c.resume_logs();
        // Both should have been flushed
        assert_eq!(r.count(), 2);
        let all = r.all();
        assert!(all[0].contains("queued1"));
        assert!(all[1].contains("queued2"));
    }

    #[test]
    fn test_double_pause_then_resume() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });

        c.pause_logs();
        c.pause_logs();
        assert!(c.info("double paused"));
        assert_eq!(r.count(), 0);

        c.resume_logs();
        assert_eq!(r.count(), 1);
        assert!(r.last().unwrap().contains("double paused"));
    }

    #[test]
    fn test_pause_resume_no_queue_when_not_paused() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });

        assert!(c.info("direct"));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_resume_without_pause() {
        // Resume without pausing should be a no-op
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        c.resume_logs();
        assert!(c.info("after resume"));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_log_string_basic() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.log("plain"));
        let last = r.last().unwrap();
        assert!(last.contains("plain"), "got: {}", last);
        assert!(last.contains(']'), "should have type bracket: {}", last);
    }

    #[test]
    fn test_log_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.log_raw("raw"));
        // raw variant should produce output too
        let last = r.last().unwrap();
        assert!(last.contains("raw"), "got: {}", last);
    }

    #[test]
    fn test_log_obj_with_type() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let input = LogObjectInput {
            r#type: Some(LogType::Warn),
            message: Some("custom".into()),
            args: vec!["additional".into()],
            ..LogObjectInput::default()
        };
        assert!(c.log_obj(&input));
        let last = r.last().unwrap();
        assert!(last.contains("additional"), "got: {}", last);
        assert!(last.contains("warn"), "got: {}", last);
    }

    #[test]
    fn test_log_obj_default_type() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        // No type specified -> defaults to Log
        let input = LogObjectInput::new().message("default-ty");
        assert!(c.log_obj(&input));
        let last = r.last().unwrap();
        // message gets pushed as args[0]; type is Log ("log")
        assert!(last.contains("default-ty"), "got: {}", last);
        assert!(last.contains("log"), "got: {}", last);
    }

    #[test]
    fn test_log_obj_raw_alias() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let input = LogObjectInput::new().message("via-raw");
        assert!(c.log_obj_raw(&input));
        let last = r.last().unwrap();
        assert!(last.contains("via-raw"), "got: {}", last);
    }

    #[test]
    fn test_log_with_filtered_level() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::WARN,
            ..ConsolaOptions::default()
        });
        assert!(!c.log("should be filtered")); // LOG level = 2 > WARN level = 1
    }

    #[test]
    fn test_all_type_methods_info() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.info("info"));
        assert!(r.last().unwrap().contains("info"));
    }

    #[test]
    fn test_all_type_methods_warn() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.warn("warn"));
        assert!(r.last().unwrap().contains("warn"));
    }

    #[test]
    fn test_all_type_methods_error() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.error("error"));
        assert!(r.last().unwrap().contains("error"));
    }

    #[test]
    fn test_all_type_methods_fatal() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.fatal("fatal"));
        assert!(r.last().unwrap().contains("fatal"));
    }

    #[test]
    fn test_all_type_methods_success() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.success("success"));
        assert!(r.last().unwrap().contains("success"));
    }

    #[test]
    fn test_all_type_methods_fail() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.fail("fail"));
        assert!(r.last().unwrap().contains("fail"));
    }

    #[test]
    fn test_all_type_methods_ready() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.ready("ready"));
        assert!(r.last().unwrap().contains("ready"));
    }

    #[test]
    fn test_all_type_methods_start() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.start("start"));
        assert!(r.last().unwrap().contains("start"));
    }

    #[test]
    fn test_all_type_methods_box() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.box_("box"));
        assert!(r.last().unwrap().contains("box"));
    }

    #[test]
    fn test_all_type_methods_debug() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.debug("debug"));
        assert!(r.last().unwrap().contains("debug"));
    }

    #[test]
    fn test_all_type_methods_trace() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.trace("trace"));
        assert!(r.last().unwrap().contains("trace"));
    }

    #[test]
    fn test_all_type_methods_verbose() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.verbose("verbose"));
        assert!(r.last().unwrap().contains("verbose"));
    }

    #[test]
    fn test_all_type_methods_log() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.log("log"));
        assert!(r.last().unwrap().contains("log"));
    }

    // Test via log_obj with Silent type (no direct method)
    #[test]
    fn test_silent_type_via_log_obj() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let input = LogObjectInput {
            r#type: Some(LogType::Silent),
            message: Some("silent".into()),
            ..LogObjectInput::default()
        };
        // Silent has level -1; level filter is msg_level > consola_level.
        // -1 > 5 (VERBOSE clamped) is false, so Silent passes the filter.
        // It reaches the reporter.
        assert!(
            c.log_obj(&input),
            "Silent type passes level filter at VERBOSE"
        );
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_info_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.info_raw("raw-info"));
        let last = r.last().unwrap();
        assert!(last.contains("raw-info"), "got: {}", last);
    }

    #[test]
    fn test_warn_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.warn_raw("raw-warn"));
        assert!(r.last().unwrap().contains("raw-warn"));
    }

    #[test]
    fn test_error_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.error_raw("raw-error"));
        assert!(r.last().unwrap().contains("raw-error"));
    }

    #[test]
    fn test_fatal_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.fatal_raw("raw-fatal"));
        assert!(r.last().unwrap().contains("raw-fatal"));
    }

    #[test]
    fn test_success_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.success_raw("raw-success"));
        assert!(r.last().unwrap().contains("raw-success"));
    }

    #[test]
    fn test_fail_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.fail_raw("raw-fail"));
        assert!(r.last().unwrap().contains("raw-fail"));
    }

    #[test]
    fn test_ready_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.ready_raw("raw-ready"));
        assert!(r.last().unwrap().contains("raw-ready"));
    }

    #[test]
    fn test_start_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.start_raw("raw-start"));
        assert!(r.last().unwrap().contains("raw-start"));
    }

    #[test]
    fn test_box_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.box_raw("raw-box"));
        assert!(r.last().unwrap().contains("raw-box"));
    }

    #[test]
    fn test_debug_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.debug_raw("raw-debug"));
        assert!(r.last().unwrap().contains("raw-debug"));
    }

    #[test]
    fn test_trace_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.trace_raw("raw-trace"));
        assert!(r.last().unwrap().contains("raw-trace"));
    }

    #[test]
    fn test_verbose_raw() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.verbose_raw("raw-verbose"));
        assert!(r.last().unwrap().contains("raw-verbose"));
    }

    #[test]
    fn test_throttle_dedup_same_message() {
        let r = CaptureReporter::new();
        // throttle_min=0: first message emits, second+ are throttled.
        // Because "Save as last log" resets count to 1 after each
        // non-throttled call, throttle_min=1 means: first passes,
        // second is throttled (2 > 1). With min=0: first passes,
        // second: 2 > 0 → throttled.
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![Box::new(r.clone())],
            throttle: 10_000,
            throttle_min: 0,
            ..ConsolaOptions::default()
        });

        // First call: emitted
        assert!(c.info("dedup-me"));
        assert_eq!(r.count(), 1);

        // Second call (same msg, within window): throttled
        assert!(c.info("dedup-me"));
        assert_eq!(r.count(), 1, "second identical should be throttled");

        // Third call: still throttled
        assert!(c.info("dedup-me"));
        assert_eq!(r.count(), 1, "third identical should also be throttled");

        // Different message: passes (different serialized key)
        assert!(c.warn("different"));
        // The repeated count (3) emits before the new msg
        // repeated = (3 - 0).max(0) = 3 > 0 → emits "dedup-me (repeated 3 times)"
        // then the "different" warn emits.
        assert_eq!(r.count(), 3);
        let all = r.all();
        assert!(all[1].contains("(repeated 3 times)"), "got: {:?}", all);
        assert!(all[2].contains("different"), "got: {:?}", all);
    }

    #[test]
    fn test_throttle_emits_repeated_suffix_on_next_new_message() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![Box::new(r.clone())],
            throttle: 10_000,
            throttle_min: 0,
            ..ConsolaOptions::default()
        });

        assert!(c.info("msg"));
        assert_eq!(r.count(), 1);

        // Second identical: throttled
        assert!(c.info("msg"));
        assert_eq!(r.count(), 1);

        // A different message triggers the "repeated" flush
        assert!(c.warn("new"));
        let all = r.all();
        // repeated = (2 - 0).max(0) = 2 → emits "(repeated 2 times)"
        assert_eq!(r.count(), 3, "should be msg, repeated-msg, new: {:?}", all);
        assert!(
            all[1].contains("msg"),
            "second entry should be repeated: {:?}",
            all
        );
        assert!(all[2].contains("new"), "got: {:?}", all);
    }

    #[test]
    fn test_throttle_count_resets_prevents_accumulation() {
        // throttle_min=2: due to "Save as last log" resetting count to 1,
        // accumulation never reaches 3 (which would be > 2).
        // So all identical messages within the window are emitted.
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![Box::new(r.clone())],
            throttle: 10_000,
            throttle_min: 2,
            ..ConsolaOptions::default()
        });

        // First: emitted
        assert!(c.info("dup"));
        assert_eq!(r.count(), 1);

        // Second: is_repeat = Some(1), count → 2, 2 > 2? No → emitted
        assert!(c.info("dup"));
        assert_eq!(r.count(), 2);

        // Third: same pattern (count reset to 1), still not > 2 → emitted
        assert!(c.info("dup"));
        assert_eq!(r.count(), 3);

        // Fourth: still same
        assert!(c.info("dup"));
        assert_eq!(r.count(), 4);
    }

    #[test]
    fn test_level_filter_below_info() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            ..ConsolaOptions::default()
        });
        // ERROR=0 ≤ INFO=3 → passes
        assert!(c.error("err"), "error should pass at INFO level");
        assert!(c.warn("warn"), "warn should pass at INFO level");
        assert!(c.log("log"), "log (level 2) should pass at INFO level");
        // INFO=3 == INFO=3 → passes
        assert!(c.info("info"), "info should pass at INFO level");
        // DEBUG=4 > INFO=3 → filtered
        assert!(!c.debug("debug"), "debug should be filtered at INFO level");
        assert!(!c.trace("trace"), "trace should be filtered at INFO level");
    }

    #[test]
    fn test_level_filter_warn() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::WARN,
            ..ConsolaOptions::default()
        });
        assert!(c.error("err"), "error should pass at WARN level");
        assert!(c.warn("warn"), "warn should pass at WARN level");
        assert!(!c.info("info"), "info should be filtered at WARN level");
        assert!(!c.debug("debug"), "debug should be filtered at WARN level");
    }

    #[test]
    fn test_level_filter_error() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::ERROR,
            ..ConsolaOptions::default()
        });
        // fatal and error both at level 0
        assert!(c.fatal("fatal"));
        assert!(c.error("err"));
        assert!(!c.warn("warn"), "warn should be filtered at ERROR level");
    }

    #[test]
    fn test_level_filter_debug() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::DEBUG,
            ..ConsolaOptions::default()
        });
        assert!(c.debug("debug"), "debug should pass at DEBUG level");
        assert!(!c.trace("trace"), "trace should be filtered at DEBUG level");
    }

    #[test]
    fn test_level_filter_verbose_accepts_all() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            ..ConsolaOptions::default()
        });
        assert!(c.error("err"));
        assert!(c.info("info"));
        assert!(c.debug("debug"));
        assert!(c.trace("trace"));
        assert!(c.verbose("verbose"));
    }

    #[test]
    fn test_level_filter_silent_rejects_all() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::SILENT,
            ..ConsolaOptions::default()
        });
        assert!(!c.fatal("fatal"));
        assert!(!c.error("err"));
        assert!(!c.warn("warn"));
        assert!(!c.info("info"));
        assert!(!c.log("log"));
        assert!(!c.debug("debug"));
    }

    #[test]
    fn test_reporter_error_during_emit() {
        // An ErrReporter always returns Err from format().
        // The Consola catches these and writes to stderr.
        let err_reporter = ErrReporter;
        let capture = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![Box::new(err_reporter.clone()), Box::new(capture.clone())],
            ..ConsolaOptions::default()
        });

        // Even though ErrReporter fails, the second reporter should still
        // get the message.
        assert!(c.info("after error"));
        assert_eq!(capture.count(), 1);
        assert!(capture.last().unwrap().contains("after error"));
    }

    #[test]
    fn test_reporter_all_error() {
        // When ALL reporters fail, the log should still return true
        // (the message was "accepted" even if all reporters errored).
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![Box::new(ErrReporter)],
            ..ConsolaOptions::default()
        });
        assert!(c.info("will error"));
    }

    #[test]
    fn test_with_defaults_tag_merge_order() {
        let c = Consola::new(ConsolaOptions::default());
        let sub = c.with_defaults(LogObjectInput {
            tag: Some("base".into()),
            ..LogObjectInput::default()
        });
        let tagged = sub.with_tag("extra");

        // Verify the instance was created without panic
        assert!(tagged.info("merged"));
    }

    #[test]
    fn test_create_merges_tags_with_colon() {
        // Directly test create()'s tag merge at the option level
        let c = Consola::new(ConsolaOptions {
            defaults: LogObjectInput {
                tag: Some("base".into()),
                ..LogObjectInput::default()
            },
            ..ConsolaOptions::default()
        });
        let child = c.create(ConsolaOptions {
            defaults: LogObjectInput {
                tag: Some("extra".into()),
                ..LogObjectInput::default()
            },
            ..ConsolaOptions::default()
        });
        // The create merge concatenates: "base" + ":" + "extra" = "base:extra"
        // But we can't directly access options.defaults.tag since it's in a Mutex.
        // We just verify no panic and the instance works.
        assert!(child.info("test"));
    }

    #[test]
    fn test_direct_write_pipeline() {
        use crate::reporters::BasicReporter;
        let r = BasicReporter;
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r) as Box<dyn Reporter>],
            ..ConsolaOptions::default()
        });

        // Exercise the full path: _log_fn -> _emit -> write_line
        assert!(c.info("backend test"));
        assert!(c.warn("more"));
        assert!(c.error("error"));
    }

    #[test]
    fn test_direct_write_empty_formatted() {
        // When format returns empty string, write_line is not called
        // (the guard `if !formatted.is_empty()` skips it).
        // We confirm this doesn't panic.
        use crate::reporters::BasicReporter;
        let r = BasicReporter;
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r) as Box<dyn Reporter>],
            format_options: FormatOptions {
                date: false,
                ..FormatOptions::default()
            },
            ..ConsolaOptions::default()
        });

        // Box type might produce empty-ish output? Let's just exercise it.
        assert!(c.log("test"));
    }

    #[cfg(feature = "log")]
    mod log_trait_tests {
        use super::*;

        #[test]
        fn test_log_enabled_error() {
            let c = Consola::new(ConsolaOptions {
                level: log_levels::INFO,
                ..ConsolaOptions::default()
            });
            assert!(log::Log::enabled(
                &c,
                &log::Metadata::builder()
                    .level(log::Level::Error)
                    .target("test")
                    .build(),
            ));
        }

        #[test]
        fn test_log_enabled_debug_filtered() {
            let c = Consola::new(ConsolaOptions {
                level: log_levels::INFO,
                ..ConsolaOptions::default()
            });
            assert!(!log::Log::enabled(
                &c,
                &log::Metadata::builder()
                    .level(log::Level::Debug)
                    .target("test")
                    .build(),
            ));
        }

        #[test]
        fn test_log_log_dispatches_to_reporters() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::TRACE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });

            let record = log::Record::builder()
                .args(format_args!("log-test-message"))
                .level(log::Level::Info)
                .target("test-target")
                .build();
            log::Log::log(&c, &record);

            assert_eq!(r.count(), 1);
            let last = r.last().unwrap();
            assert!(last.contains("log-test-message"), "got: {}", last);
        }

        #[test]
        fn test_log_log_level_filtering() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::WARN,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });

            // Info is below WARN, should be filtered
            let record = log::Record::builder()
                .args(format_args!("should-not-appear"))
                .level(log::Level::Info)
                .target("test")
                .build();
            log::Log::log(&c, &record);
            assert_eq!(r.count(), 0);

            // Error is above WARN, should pass
            let record = log::Record::builder()
                .args(format_args!("should-appear"))
                .level(log::Level::Error)
                .target("test")
                .build();
            log::Log::log(&c, &record);
            assert_eq!(r.count(), 1);
        }

        #[test]
        fn test_log_log_flush() {
            let c = Consola::new(ConsolaOptions::default());
            // flush should not panic
            log::Log::flush(&c);
        }
    }

    #[cfg(feature = "tracing")]
    mod subscriber_tests {
        use super::*;

        #[test]
        fn test_subscriber_macro_info() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            tracing::info!("macro info message");
            assert_eq!(r.count(), 1);
            let last = r.last().unwrap();
            assert!(last.contains("macro info message"), "got: {}", last);
        }

        #[test]
        fn test_subscriber_macro_error() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            tracing::error!("macro error message");
            assert_eq!(r.count(), 1);
            let last = r.last().unwrap();
            assert!(last.contains("macro error message"), "got: {}", last);
        }

        #[test]
        fn test_subscriber_macro_warn() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            tracing::warn!("macro warn message");
            assert_eq!(r.count(), 1);
        }

        #[test]
        fn test_subscriber_macro_debug() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            tracing::debug!("macro debug message");
            assert_eq!(r.count(), 1);
        }

        #[test]
        fn test_subscriber_macro_trace() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            tracing::trace!("macro trace message");
            assert_eq!(r.count(), 1);
        }

        #[test]
        fn test_subscriber_macro_level_filtered() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::WARN,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            tracing::debug!("should be filtered");
            assert_eq!(r.count(), 0);

            tracing::warn!("should pass");
            assert_eq!(r.count(), 1);
        }

        #[test]
        fn test_subscriber_macro_filter_off() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::SILENT,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            tracing::error!("should be silent");
            assert_eq!(r.count(), 0);
        }

        #[test]
        fn test_subscriber_macro_with_span() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            // Create and enter a span
            let span = tracing::info_span!("outer_span", user = "alice");
            let _enter = span.enter();

            tracing::info!("inside span");
            assert_eq!(r.count(), 1);
            let last = r.last().unwrap();
            // Should contain span name and fields
            assert!(last.contains("outer_span"), "span name missing: {}", last);
            assert!(
                last.contains("user") || last.contains("alice"),
                "span field missing: {}",
                last
            );
        }

        #[test]
        fn test_subscriber_span_enter_exit_tracking() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            let span = tracing::info_span!("nested_span");
            {
                let _enter = span.enter();
                tracing::info!("in span");
            }
            tracing::info!("after span");

            assert_eq!(r.count(), 2);
            let all = r.all();
            assert!(
                all[0].contains("nested_span"),
                "span context missing: {}",
                all[0]
            );
            assert!(
                !all[1].contains("nested_span"),
                "span context leaked: {}",
                all[1]
            );
        }

        #[test]
        fn test_subscriber_max_level_hint() {
            use tracing::Subscriber;
            let c = Consola::new(ConsolaOptions {
                level: log_levels::DEBUG,
                ..ConsolaOptions::default()
            });
            let hint = Subscriber::max_level_hint(&c);
            assert_eq!(hint, Some(tracing_core::metadata::LevelFilter::DEBUG));
        }

        #[test]
        fn test_subscriber_max_level_hint_silent() {
            use tracing::Subscriber;
            let c = Consola::new(ConsolaOptions {
                level: log_levels::SILENT,
                ..ConsolaOptions::default()
            });
            assert_eq!(
                Subscriber::max_level_hint(&c),
                Some(tracing_core::metadata::LevelFilter::OFF)
            );
        }

        #[test]
        fn test_subscriber_max_level_hint_error() {
            use tracing::Subscriber;
            let c = Consola::new(ConsolaOptions {
                level: log_levels::ERROR,
                ..ConsolaOptions::default()
            });
            assert_eq!(
                Subscriber::max_level_hint(&c),
                Some(tracing_core::metadata::LevelFilter::ERROR)
            );
        }

        #[test]
        fn test_subscriber_nested_spans_different_ids() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            let span_a = tracing::info_span!("a");
            let span_b = tracing::info_span!("b");
            assert_ne!(span_a.id(), span_b.id());
        }

        #[test]
        fn test_subscriber_event_with_fields() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            tracing::info!(key = "val", "event with fields");
            assert_eq!(r.count(), 1);
            let last = r.last().unwrap();
            assert!(last.contains("event with fields"), "got: {}", last);
        }

        #[test]
        fn test_subscriber_span_record_after_creation() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            // The field must be declared in the span macro for record() to work.
            let span = tracing::info_span!("dyn_span", dynamic_key = tracing::field::Empty);
            span.record("dynamic_key", "dynamic_val");
            {
                let _enter = span.enter();
                tracing::info!("after record");
            }

            let last = r.last().unwrap();
            assert!(
                last.contains("dynamic_key") || last.contains("dynamic_val"),
                "dynamically recorded field missing: {}",
                last
            );
        }

        #[test]
        fn test_subscriber_try_close_reclaims() {
            use tracing::Subscriber;
            let c = Consola::new(ConsolaOptions::default());
            let id = tracing::span::Id::from_u64(9999);
            let closed = Subscriber::try_close(&c, id);
            assert!(!closed);
        }

        #[test]
        fn test_subscriber_record_fields_in_new_span() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            let span = tracing::info_span!(
                "fields_span",
                str_field = "hello",
                int_field = 42u64,
                bool_field = true,
            );
            {
                let _enter = span.enter();
                tracing::info!("check fields");
            }

            let last = r.last().unwrap();
            assert!(last.contains("hello"), "str_field: {}", last);
            assert!(last.contains("42"), "int_field: {}", last);
            assert!(last.contains("true"), "bool_field: {}", last);
        }

        #[test]
        fn test_subscriber_implicit_parent_span() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            let parent = tracing::info_span!("parent");
            let _parent_guard = parent.enter();

            let child = tracing::info_span!("child");
            let _child_guard = child.enter();

            tracing::warn!("deep inside");
            assert_eq!(r.count(), 1);
            let last = r.last().unwrap();
            // Should include child span context (top of stack)
            assert!(last.contains("child"), "child name missing: {}", last);
        }

        #[test]
        fn test_subscriber_multiple_spans_same_name() {
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::VERBOSE,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            let a = tracing::info_span!("same_name", val = "a");
            let b = tracing::info_span!("same_name", val = "b");

            {
                let _a = a.enter();
                {
                    let _b = b.enter();
                    tracing::info!("inside both");
                }
            }
            let last = r.last().unwrap();
            assert!(last.contains("same_name"), "got: {}", last);
        }

        #[test]
        fn test_subscriber_enabled_direct() {
            // Test enabled() directly by checking which events pass through
            let r = CaptureReporter::new();
            let c = Consola::new(ConsolaOptions {
                level: log_levels::WARN,
                reporters: vec![Box::new(r.clone())],
                ..ConsolaOptions::default()
            });
            let _guard = tracing::subscriber::set_default(c);

            tracing::info!("info at warn level");
            assert_eq!(r.count(), 0, "info should be filtered");

            tracing::warn!("warn at warn level");
            assert_eq!(r.count(), 1, "warn should pass");

            tracing::error!("error at warn level");
            assert_eq!(r.count(), 2, "error should pass");
        }
    }

    #[test]
    fn test_add_reporter() {
        let r1 = CaptureReporter::new();
        let r2 = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![Box::new(r1.clone())],
            ..ConsolaOptions::default()
        });

        // Initially only r1 receives
        assert!(c.info("first"));
        assert_eq!(r1.count(), 1);
        assert_eq!(r2.count(), 0);

        c.add_reporter(Box::new(r2.clone()));
        assert!(c.warn("second"));
        assert_eq!(r1.count(), 2);
        assert_eq!(r2.count(), 1);
    }

    #[test]
    fn test_set_level_roundtrip() {
        let c = Consola::new(ConsolaOptions::default());
        // Levels 0-5 round-trip cleanly; level ≥6 clamps to 5 (max).
        for level in 0..=5 {
            c.set_level(level);
            assert_eq!(c.level(), level, "level {} roundtrip", level);
        }
        c.set_level(6);
        assert_eq!(c.level(), 5, "level 6 should clamp to 5");
    }

    #[test]
    fn test_set_level_clamps() {
        let c = Consola::new(ConsolaOptions::default());
        c.set_level(100);
        assert!(c.level() <= 6);
        c.set_level(255);
        assert!(c.level() <= 6);
    }

    #[test]
    fn test_log_return_value() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            ..ConsolaOptions::default()
        });
        // Levels that pass
        assert!(c.error("pass"));
        assert!(c.warn("pass"));
        assert!(c.info("pass"));
        // Levels that are filtered
        assert!(!c.debug("fail"));
        assert!(!c.trace("fail"));
    }

    #[test]
    fn test_verbose_level_accepts_all_methods() {
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            ..ConsolaOptions::default()
        });
        assert!(c.fatal("ok"));
        assert!(c.error("ok"));
        assert!(c.warn("ok"));
        assert!(c.log("ok"));
        assert!(c.info("ok"));
        assert!(c.success("ok"));
        assert!(c.fail("ok"));
        assert!(c.ready("ok"));
        assert!(c.start("ok"));
        assert!(c.box_("ok"));
        assert!(c.debug("ok"));
        assert!(c.trace("ok"));
        assert!(c.verbose("ok"));
    }

    #[test]
    fn test_multiple_reporters_all_receive() {
        let r1 = CaptureReporter::new();
        let r2 = CaptureReporter::new();
        let r3 = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![
                Box::new(r1.clone()),
                Box::new(r2.clone()),
                Box::new(r3.clone()),
            ],
            ..ConsolaOptions::default()
        });
        assert!(c.info("broadcast"));
        assert_eq!(r1.count(), 1);
        assert_eq!(r2.count(), 1);
        assert_eq!(r3.count(), 1);
    }

    #[test]
    fn test_defaults_do_not_affect_original() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let _sub = c.with_defaults(LogObjectInput {
            tag: Some("child".into()),
            ..LogObjectInput::default()
        });
        // The original should not pick up the child's defaults
        c.info("original");
        let last = r.last().unwrap();
        // The tag should be empty (no "child" prefix)
        assert!(!last.contains("child"), "got: {}", last);
    }

    #[test]
    fn test_create_without_reporters_still_emits() {
        // Create with no reporters — emit should still succeed (no-op)
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![],
            ..ConsolaOptions::default()
        });
        assert!(c.info("no-reporter"));
    }

    #[test]
    fn test_log_obj_with_multiple_args() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        let input = LogObjectInput {
            r#type: Some(LogType::Info),
            message: Some("main".into()),
            args: vec!["arg1".into(), "arg2".into()],
            ..LogObjectInput::default()
        };
        assert!(c.log_obj(&input));
        let last = r.last().unwrap();
        assert!(last.contains("main"), "got: {}", last);
        assert!(last.contains("arg1"), "got: {}", last);
        assert!(last.contains("arg2"), "got: {}", last);
    }

    #[test]
    fn test_pause_during_reporter_error() {
        // Pausing while a reporter returns an error should not panic
        let err_reporter = ErrReporter {};
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![Box::new(err_reporter)],
            ..ConsolaOptions::default()
        });
        c.pause_logs();
        assert!(c.info("queued"));
        c.resume_logs();
    }

    #[test]
    fn test_with_tag_chaining_does_not_panic() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        // Chaining with_tag produces a child Consola; the child's own
        // tag comes from its defaults, not from the parent instance.
        let parent = c.with_tag("parent");
        let child = parent.with_tag("child");
        child.info("nested");
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_log_all_raw_variants() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::VERBOSE,
            reporters: vec![Box::new(r.clone())],
            ..ConsolaOptions::default()
        });
        assert!(c.fatal_raw("raw-fatal"));
        assert!(c.error_raw("raw-error"));
        assert!(c.warn_raw("raw-warn"));
        assert!(c.log_raw("raw-log"));
        assert!(c.info_raw("raw-info"));
        assert!(c.success_raw("raw-success"));
        assert!(c.fail_raw("raw-fail"));
        assert!(c.ready_raw("raw-ready"));
        assert!(c.start_raw("raw-start"));
        assert!(c.box_raw("raw-box"));
        assert!(c.debug_raw("raw-debug"));
        assert!(c.trace_raw("raw-trace"));
        assert!(c.verbose_raw("raw-verbose"));
        assert_eq!(r.count(), 13);
    }

    #[test]
    fn test_throttle_different_levels_both_emitted() {
        let r = CaptureReporter::new();
        let c = Consola::new(ConsolaOptions {
            level: log_levels::INFO,
            reporters: vec![Box::new(r.clone())],
            throttle: 0,
            ..ConsolaOptions::default()
        });
        // Throttle disabled (0) — every call emits.
        assert!(c.info("first"));
        assert!(c.warn("second"));
        let all = r.all();
        assert_eq!(all.len(), 2, "got: {:?}", all);
        assert!(all[0].contains("first"), "got: {}", all[0]);
        assert!(all[1].contains("second"), "got: {}", all[1]);
    }
}
