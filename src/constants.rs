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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn log_level_constant_values() {
        assert_eq!(log_levels::SILENT, LogLevel::MIN);
        assert_eq!(log_levels::FATAL, 0);
        assert_eq!(log_levels::ERROR, 0);
        assert_eq!(log_levels::WARN, 1);
        assert_eq!(log_levels::LOG, 2);
        assert_eq!(log_levels::INFO, 3);
        assert_eq!(log_levels::SUCCESS, 3);
        assert_eq!(log_levels::FAIL, 3);
        assert_eq!(log_levels::READY, 3);
        assert_eq!(log_levels::START, 3);
        assert_eq!(log_levels::BOX, 3);
        assert_eq!(log_levels::DEBUG, 4);
        assert_eq!(log_levels::TRACE, 5);
        assert_eq!(log_levels::VERBOSE, LogLevel::MAX);
    }

    #[test]
    fn log_type_as_str() {
        assert_eq!(LogType::Silent.as_str(), "silent");
        assert_eq!(LogType::Fatal.as_str(), "fatal");
        assert_eq!(LogType::Error.as_str(), "error");
        assert_eq!(LogType::Warn.as_str(), "warn");
        assert_eq!(LogType::Log.as_str(), "log");
        assert_eq!(LogType::Info.as_str(), "info");
        assert_eq!(LogType::Success.as_str(), "success");
        assert_eq!(LogType::Fail.as_str(), "fail");
        assert_eq!(LogType::Ready.as_str(), "ready");
        assert_eq!(LogType::Start.as_str(), "start");
        assert_eq!(LogType::Box.as_str(), "box");
        assert_eq!(LogType::Debug.as_str(), "debug");
        assert_eq!(LogType::Trace.as_str(), "trace");
        assert_eq!(LogType::Verbose.as_str(), "verbose");
    }

    #[test]
    fn log_type_level_method() {
        assert_eq!(LogType::Silent.level(), log_type_level(LogType::Silent));
        assert_eq!(LogType::Fatal.level(), log_type_level(LogType::Fatal));
        assert_eq!(LogType::Error.level(), log_type_level(LogType::Error));
        assert_eq!(LogType::Warn.level(), log_type_level(LogType::Warn));
        assert_eq!(LogType::Log.level(), log_type_level(LogType::Log));
        assert_eq!(LogType::Info.level(), log_type_level(LogType::Info));
        assert_eq!(LogType::Success.level(), log_type_level(LogType::Success));
        assert_eq!(LogType::Fail.level(), log_type_level(LogType::Fail));
        assert_eq!(LogType::Ready.level(), log_type_level(LogType::Ready));
        assert_eq!(LogType::Start.level(), log_type_level(LogType::Start));
        assert_eq!(LogType::Box.level(), log_type_level(LogType::Box));
        assert_eq!(LogType::Debug.level(), log_type_level(LogType::Debug));
        assert_eq!(LogType::Trace.level(), log_type_level(LogType::Trace));
        assert_eq!(LogType::Verbose.level(), log_type_level(LogType::Verbose));
    }

    #[test]
    fn log_type_from_str_ok() {
        assert_eq!(LogType::from_str("silent"), Ok(LogType::Silent));
        assert_eq!(LogType::from_str("fatal"), Ok(LogType::Fatal));
        assert_eq!(LogType::from_str("error"), Ok(LogType::Error));
        assert_eq!(LogType::from_str("warn"), Ok(LogType::Warn));
        assert_eq!(LogType::from_str("log"), Ok(LogType::Log));
        assert_eq!(LogType::from_str("info"), Ok(LogType::Info));
        assert_eq!(LogType::from_str("success"), Ok(LogType::Success));
        assert_eq!(LogType::from_str("fail"), Ok(LogType::Fail));
        assert_eq!(LogType::from_str("ready"), Ok(LogType::Ready));
        assert_eq!(LogType::from_str("start"), Ok(LogType::Start));
        assert_eq!(LogType::from_str("box"), Ok(LogType::Box));
        assert_eq!(LogType::from_str("debug"), Ok(LogType::Debug));
        assert_eq!(LogType::from_str("trace"), Ok(LogType::Trace));
        assert_eq!(LogType::from_str("verbose"), Ok(LogType::Verbose));
    }

    #[test]
    fn log_type_from_str_invalid() {
        assert_eq!(LogType::from_str("unknown"), Err(()));
        assert_eq!(LogType::from_str(""), Err(()));
        assert_eq!(LogType::from_str("INFO"), Err(()));
    }

    #[test]
    fn log_type_parse() {
        assert_eq!("info".parse::<LogType>(), Ok(LogType::Info));
        assert_eq!("error".parse::<LogType>(), Ok(LogType::Error));
        assert_eq!("nope".parse::<LogType>(), Err(()));
    }

    #[test]
    fn log_type_level_fn() {
        assert_eq!(log_type_level(LogType::Silent), -1);
        assert_eq!(log_type_level(LogType::Fatal), 0);
        assert_eq!(log_type_level(LogType::Error), 0);
        assert_eq!(log_type_level(LogType::Warn), 1);
        assert_eq!(log_type_level(LogType::Log), 2);
        assert_eq!(log_type_level(LogType::Info), 3);
        assert_eq!(log_type_level(LogType::Success), 3);
        assert_eq!(log_type_level(LogType::Fail), 3);
        assert_eq!(log_type_level(LogType::Ready), 3);
        assert_eq!(log_type_level(LogType::Start), 3);
        assert_eq!(log_type_level(LogType::Box), 3);
        assert_eq!(log_type_level(LogType::Debug), 4);
        assert_eq!(log_type_level(LogType::Trace), 5);
        assert_eq!(log_type_level(LogType::Verbose), log_levels::VERBOSE);
    }

    #[test]
    fn log_types_slice() {
        assert_eq!(LOG_TYPES.len(), 14);
        let expected = [
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
        assert_eq!(LOG_TYPES, &expected[..]);
    }

    #[test]
    fn log_type_defaults_fn() {
        for ty in LOG_TYPES {
            let input = log_type_defaults(*ty);
            assert_eq!(input.r#type, Some(*ty), "type mismatch for {:?}", ty);
            assert_eq!(
                input.level,
                Some(log_type_level(*ty)),
                "level mismatch for {:?}",
                ty
            );
            // Remaining fields should be default (None / empty)
            assert!(input.tag.is_none(), "tag should be None for {:?}", ty);
            assert!(
                input.message.is_none(),
                "message should be None for {:?}",
                ty
            );
            assert!(
                input.additional.is_none(),
                "additional should be None for {:?}",
                ty
            );
            assert!(input.title.is_none(), "title should be None for {:?}", ty);
            assert!(input.badge.is_none(), "badge should be None for {:?}", ty);
            assert!(input.icon.is_none(), "icon should be None for {:?}", ty);
            assert!(input.style.is_none(), "style should be None for {:?}", ty);
            assert!(input.args.is_empty(), "args should be empty for {:?}", ty);
        }
    }

    #[test]
    fn normalize_log_level_values() {
        // None defaults to info level (3)
        assert_eq!(normalize_log_level(None, log_levels::INFO), 3);
        // None with custom default
        assert_eq!(normalize_log_level(None, log_levels::WARN), 1);
        // Values 0-5 pass through unchanged
        for level in 0..=5 {
            assert_eq!(
                normalize_log_level(Some(level), log_levels::INFO),
                level,
                "level {} should pass through",
                level
            );
        }
        // Under 0 maps to 0
        assert_eq!(normalize_log_level(Some(-1), log_levels::INFO), 0);
        assert_eq!(normalize_log_level(Some(-100), log_levels::INFO), 0);
        assert_eq!(normalize_log_level(Some(i32::MIN), log_levels::INFO), 0);
        // Over 5 maps to 5
        assert_eq!(normalize_log_level(Some(6), log_levels::INFO), 5);
        assert_eq!(normalize_log_level(Some(7), log_levels::INFO), 5);
        assert_eq!(normalize_log_level(Some(100), log_levels::INFO), 5);
        assert_eq!(normalize_log_level(Some(i32::MAX), log_levels::INFO), 5);
    }
}
