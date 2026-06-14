//! `log` crate integration for `Consola`.

// This module is conditionally compiled when feature = "log".
// See `super::log_impl` declaration in mod.rs.

use crate::constants::LogType;
#[cfg(feature = "backtrace")]
use crate::types::ErrorInfo;
use crate::types::LogObject;

use super::Consola;

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
            log_obj.error = Some(ErrorInfo {
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
