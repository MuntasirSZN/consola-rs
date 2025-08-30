use once_cell::sync::Lazy;
use std::sync::RwLock;

/// Sentinel / numeric log levels.
/// Ordering: lower is more severe.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LogLevel(pub i16);

impl LogLevel {
    pub const SILENT: LogLevel = LogLevel(-99);
    pub const FATAL: LogLevel = LogLevel(0);
    pub const ERROR: LogLevel = LogLevel(1);
    pub const WARN: LogLevel = LogLevel(2);
    pub const LOG: LogLevel = LogLevel(3);
    pub const INFO: LogLevel = LogLevel(4);
    pub const SUCCESS: LogLevel = LogLevel(5);
    pub const DEBUG: LogLevel = LogLevel(6);
    pub const TRACE: LogLevel = LogLevel(7);
    pub const VERBOSE: LogLevel = LogLevel(99);
}

/// Specification for a log type.
#[derive(Debug, Clone)]
pub struct LogTypeSpec {
    pub level: LogLevel,
}

static TYPE_REGISTRY: Lazy<RwLock<Vec<(String, LogTypeSpec)>>> = Lazy::new(|| {
    let mut v = Vec::new();
    for (name, level) in [
        ("silent", LogLevel::SILENT),
        ("fatal", LogLevel::FATAL),
        ("error", LogLevel::ERROR),
        ("warn", LogLevel::WARN),
        ("log", LogLevel::LOG),
        ("info", LogLevel::INFO),
        ("success", LogLevel::SUCCESS),
        ("fail", LogLevel::SUCCESS),
        ("ready", LogLevel::INFO),
        ("start", LogLevel::LOG),
        ("box", LogLevel::LOG),
        ("debug", LogLevel::DEBUG),
        ("trace", LogLevel::TRACE),
        ("verbose", LogLevel::VERBOSE),
    ] {
        v.push((name.to_string(), LogTypeSpec { level }));
    }
    RwLock::new(v)
});

pub fn register_type(name: &str, spec: LogTypeSpec) {
    let mut guard = TYPE_REGISTRY.write().unwrap();
    if let Some(existing) = guard.iter_mut().find(|(n, _)| n == name) {
        *existing = (name.to_string(), spec);
    } else {
        guard.push((name.to_string(), spec));
    }
}

pub fn level_for_type(name: &str) -> Option<LogLevel> {
    TYPE_REGISTRY
        .read()
        .unwrap()
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, s)| s.level)
}

pub fn normalize_level(input: &str) -> Option<LogLevel> {
    if let Ok(num) = input.parse::<i16>() {
        return Some(LogLevel(num));
    }
    level_for_type(input)
}
