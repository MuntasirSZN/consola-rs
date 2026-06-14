//! The [`Consola`] logger — core struct, log methods, and crate integrations.
//!
//! Emission goes through `log` or `tracing` crates. There is no IO.

use std::time::Instant;

#[cfg(feature = "tracing")]
use std::collections::HashMap;

use crate::sync::Mutex;

use crate::constants::{LogLevel, LogType, log_levels, log_type_defaults, normalize_log_level};
use crate::types::{ConsolaOptions, LogContext, LogObject, LogObjectInput, Reporter};

/// `log` crate integration.
#[cfg(feature = "log")]
pub mod log_impl;
/// `tracing` subscriber integration.
#[cfg(feature = "tracing")]
pub mod tracing_impl;

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
