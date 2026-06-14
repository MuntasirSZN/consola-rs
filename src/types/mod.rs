//! Core types: log entries, the reporter trait, and consola options.

pub mod format;
pub mod prompt;

use std::sync::Arc;

use crate::constants::{LogLevel, LogType, log_levels};

pub use format::{ErrorInfo, FormatOptions};
pub use prompt::{
    ConfirmPromptOptions, MultiSelectOptions, PromptCommonOptions, PromptOptions, SelectOption,
    SelectPromptOptions, TextPromptOptions,
};

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

    /// Optional title displayed prominently by some reporters.
    pub title: Option<String>,
    /// Whether to render a badge indicator.
    pub badge: Option<bool>,
    /// Optional icon name or emoji string.
    pub icon: Option<String>,
    /// Optional CSS-like style string (reporter-specific).
    pub style: Option<String>,
    /// Optional error information for error-level logs.
    pub error: Option<ErrorInfo>,
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
    /// Optional title displayed prominently by some reporters.
    pub title: Option<String>,
    /// Whether to render a badge indicator.
    pub badge: bool,
    /// Optional icon name or emoji string.
    pub icon: Option<String>,
    /// Optional CSS-like style string (reporter-specific).
    pub style: Option<String>,
    /// Optional error information for error-level logs.
    pub error: Option<ErrorInfo>,
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
            error: None,
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

    /// Return the timestamp as a time::OffsetDateTime (feature = "time").
    /// Returns `None` if the timestamp is invalid.
    #[cfg(feature = "time")]
    pub fn timestamp_time(&self) -> Option<time::OffsetDateTime> {
        time::OffsetDateTime::from_unix_timestamp_nanos(self.timestamp_ms as i128 * 1_000_000).ok()
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

#[cfg(all(feature = "time", not(any(feature = "jiff", feature = "chrono"))))]
pub(crate) fn now_ms() -> i64 {
    let now = time::OffsetDateTime::now_utc();
    now.unix_timestamp() * 1000 + now.millisecond() as i64
}

#[cfg(not(any(feature = "jiff", feature = "chrono", feature = "time")))]
pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

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
    fn format(
        &self,
        log_obj: &LogObject,
        ctx: &LogContext,
    ) -> Result<String, crate::error::ConsolaError>;
    /// Clone the reporter into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Reporter>;
}

impl Clone for Box<dyn Reporter> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

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
