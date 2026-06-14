use crate::types::LogObjectInput;

/// Numeric log level. Higher values mean more verbosity.
pub type LogLevel = i32;

/// Predefined log level constants.
pub mod log_levels {
    use super::LogLevel;

    /// Silent log level; suppresses all output.
    pub const SILENT: LogLevel = LogLevel::MIN;
    /// Fatal log level.
    pub const FATAL: LogLevel = 0;
    /// Error log level.
    pub const ERROR: LogLevel = 0;
    /// Warning log level.
    pub const WARN: LogLevel = 1;
    /// General-purpose log level.
    pub const LOG: LogLevel = 2;
    /// Informational log level.
    pub const INFO: LogLevel = 3;
    /// Success log level.
    pub const SUCCESS: LogLevel = 3;
    /// Failure log level.
    pub const FAIL: LogLevel = 3;
    /// Ready state log level.
    pub const READY: LogLevel = 3;
    /// Start of an operation log level.
    pub const START: LogLevel = 3;
    /// Boxed output log level.
    pub const BOX: LogLevel = 3;
    /// Debug log level.
    pub const DEBUG: LogLevel = 4;
    /// Trace log level.
    pub const TRACE: LogLevel = 5;
    /// Verbose log level; maximum verbosity.
    pub const VERBOSE: LogLevel = LogLevel::MAX;
}

/// Category of a log message, determining its label and default log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogType {
    /// No output.
    Silent,
    /// Fatal error.
    Fatal,
    /// Error.
    Error,
    /// Warning.
    Warn,
    /// General log entry.
    Log,
    /// Informational message.
    Info,
    /// Success outcome.
    Success,
    /// Failure outcome.
    Fail,
    /// Ready state notification.
    Ready,
    /// Start of an operation.
    Start,
    /// Boxed/bordered output.
    Box,
    /// Debug message.
    Debug,
    /// Trace message.
    Trace,
    /// Verbose message.
    Verbose,
}

impl LogType {
    /// Returns the string label for this log type (e.g. `"info"`, `"error"`).
    #[inline]
    pub fn as_str(self) -> &'static str {
        match self {
            LogType::Silent => "silent",
            LogType::Fatal => "fatal",
            LogType::Error => "error",
            LogType::Warn => "warn",
            LogType::Log => "log",
            LogType::Info => "info",
            LogType::Success => "success",
            LogType::Fail => "fail",
            LogType::Ready => "ready",
            LogType::Start => "start",
            LogType::Box => "box",
            LogType::Debug => "debug",
            LogType::Trace => "trace",
            LogType::Verbose => "verbose",
        }
    }

    /// Returns the default numeric log level for this log type.
    #[inline]
    pub fn level(self) -> LogLevel {
        log_type_level(self)
    }
}

impl std::str::FromStr for LogType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "silent" => LogType::Silent,
            "fatal" => LogType::Fatal,
            "error" => LogType::Error,
            "warn" => LogType::Warn,
            "log" => LogType::Log,
            "info" => LogType::Info,
            "success" => LogType::Success,
            "fail" => LogType::Fail,
            "ready" => LogType::Ready,
            "start" => LogType::Start,
            "box" => LogType::Box,
            "debug" => LogType::Debug,
            "trace" => LogType::Trace,
            "verbose" => LogType::Verbose,
            _ => return Err(()),
        })
    }
}

/// All log type variants, in the order they appear in the JS source.
pub const LOG_TYPES: &[LogType] = &[
    LogType::Silent,
    LogType::Fatal,
    LogType::Error,
    LogType::Warn,
    LogType::Log,
    LogType::Info,
    LogType::Success,
    LogType::Fail,
    LogType::Ready,
    LogType::Start,
    LogType::Box,
    LogType::Debug,
    LogType::Trace,
    LogType::Verbose,
];

/// Returns the default numeric log level for a given [`LogType`].
#[inline]
pub fn log_type_level(ty: LogType) -> LogLevel {
    match ty {
        LogType::Silent => -1,
        LogType::Fatal => log_levels::FATAL,
        LogType::Error => log_levels::ERROR,
        LogType::Warn => log_levels::WARN,
        LogType::Log => log_levels::LOG,
        LogType::Info => log_levels::INFO,
        LogType::Success => log_levels::SUCCESS,
        LogType::Fail => log_levels::FAIL,
        LogType::Ready => log_levels::INFO,
        LogType::Start => log_levels::INFO,
        LogType::Box => log_levels::INFO,
        LogType::Debug => log_levels::DEBUG,
        LogType::Trace => log_levels::TRACE,
        LogType::Verbose => log_levels::VERBOSE,
    }
}

/// The per-type default partial input (as in JS `LogTypes`).
#[inline]
pub fn log_type_defaults(ty: LogType) -> LogObjectInput {
    let level = log_type_level(ty);
    LogObjectInput {
        level: Some(level),
        r#type: Some(ty),
        ..LogObjectInput::default()
    }
}

/// Normalize an optional level / type to a concrete numeric level.
pub fn normalize_log_level(input: Option<LogLevel>, default_level: LogLevel) -> LogLevel {
    let level = input.unwrap_or(default_level);
    level.clamp(0, 5)
}
