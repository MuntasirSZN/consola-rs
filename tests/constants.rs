use consola::constants::{LOG_TYPES, log_type_defaults, log_type_level, normalize_log_level};
use consola::{LogLevel, LogType, log_levels};
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
    assert_eq!(LogType::Silent.level(), -1);
    assert_eq!(LogType::Fatal.level(), 0);
    assert_eq!(LogType::Error.level(), 0);
    assert_eq!(LogType::Warn.level(), 1);
    assert_eq!(LogType::Log.level(), 2);
    assert_eq!(LogType::Info.level(), 3);
    assert_eq!(LogType::Success.level(), 3);
    assert_eq!(LogType::Fail.level(), 3);
    assert_eq!(LogType::Ready.level(), 3);
    assert_eq!(LogType::Start.level(), 3);
    assert_eq!(LogType::Box.level(), 3);
    assert_eq!(LogType::Debug.level(), 4);
    assert_eq!(LogType::Trace.level(), 5);
    assert_eq!(LogType::Verbose.level(), LogLevel::MAX);
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
    assert!(LogType::from_str("").is_err());
    assert!(LogType::from_str("unknown").is_err());
    assert!(LogType::from_str("INFO").is_err());
    assert!(LogType::from_str("Error").is_err());
    assert!(LogType::from_str("log_type").is_err());
    assert!(LogType::from_str(" ").is_err());
}

#[test]
fn log_type_parse() {
    assert_eq!("info".parse::<LogType>(), Ok(LogType::Info));
    assert_eq!("error".parse::<LogType>(), Ok(LogType::Error));
    assert_eq!("debug".parse::<LogType>(), Ok(LogType::Debug));
    assert_eq!("trace".parse::<LogType>(), Ok(LogType::Trace));
    assert_eq!("warn".parse::<LogType>(), Ok(LogType::Warn));
    assert!("bogus".parse::<LogType>().is_err());
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
    assert_eq!(log_type_level(LogType::Verbose), LogLevel::MAX);
}

#[test]
fn log_types_slice() {
    assert_eq!(LOG_TYPES.len(), 14);
    assert_eq!(LOG_TYPES[0], LogType::Silent);
    assert_eq!(LOG_TYPES[1], LogType::Fatal);
    assert_eq!(LOG_TYPES[2], LogType::Error);
    assert_eq!(LOG_TYPES[3], LogType::Warn);
    assert_eq!(LOG_TYPES[4], LogType::Log);
    assert_eq!(LOG_TYPES[5], LogType::Info);
    assert_eq!(LOG_TYPES[6], LogType::Success);
    assert_eq!(LOG_TYPES[7], LogType::Fail);
    assert_eq!(LOG_TYPES[8], LogType::Ready);
    assert_eq!(LOG_TYPES[9], LogType::Start);
    assert_eq!(LOG_TYPES[10], LogType::Box);
    assert_eq!(LOG_TYPES[11], LogType::Debug);
    assert_eq!(LOG_TYPES[12], LogType::Trace);
    assert_eq!(LOG_TYPES[13], LogType::Verbose);
}

#[test]
fn log_type_defaults_fn() {
    let silent = log_type_defaults(LogType::Silent);
    assert_eq!(silent.level, Some(-1));
    assert_eq!(silent.r#type, Some(LogType::Silent));

    let info = log_type_defaults(LogType::Info);
    assert_eq!(info.level, Some(3));
    assert_eq!(info.r#type, Some(LogType::Info));
    // Tag, message, additional should be None (default)
    assert!(info.tag.is_none());
    assert!(info.message.is_none());
    assert!(info.additional.is_none());
    assert!(info.args.is_empty());

    let debug = log_type_defaults(LogType::Debug);
    assert_eq!(debug.level, Some(4));
    assert_eq!(debug.r#type, Some(LogType::Debug));
}

#[test]
fn normalize_log_level_values() {
    // None + default => default clamped to [0, 5]
    assert_eq!(normalize_log_level(None, 0), 0);
    assert_eq!(normalize_log_level(None, 3), 3);
    assert_eq!(normalize_log_level(None, 5), 5);
    assert_eq!(normalize_log_level(None, 10), 5);
    assert_eq!(normalize_log_level(None, -1), 0);

    // Some value returns as-is (clamped)
    assert_eq!(normalize_log_level(Some(0), 3), 0);
    assert_eq!(normalize_log_level(Some(2), 3), 2);
    assert_eq!(normalize_log_level(Some(3), 2), 3);
    assert_eq!(normalize_log_level(Some(5), 2), 5);

    // Clamping to [0, 5]
    assert_eq!(normalize_log_level(Some(-1), 3), 0);
    assert_eq!(normalize_log_level(Some(10), 3), 5);
    assert_eq!(normalize_log_level(Some(i32::MIN), 0), 0);
    assert_eq!(normalize_log_level(Some(i32::MAX), 0), 5);
}
