//! Bridge implementation for the `log` crate.
//!
//! This module provides a `log::Log` implementation that routes log records
//! to a consola Logger instance.

use crate::{ArgValue, Logger, Reporter};
use log::{Level, Log, Metadata, Record};
use std::cell::RefCell;

thread_local! {
    /// Recursion guard to prevent infinite loops when consola logs to the log crate
    static RECURSION_GUARD: RefCell<bool> = const { RefCell::new(false) };
}

/// A `log::Log` implementation that bridges to a consola Logger.
pub struct ConsoLog<R: Reporter + 'static> {
    logger: Logger<R>,
}

impl<R: Reporter + 'static> ConsoLog<R> {
    /// Create a new ConsoLog bridge with the given logger.
    pub fn new(logger: Logger<R>) -> Self {
        Self { logger }
    }

    /// Map log::Level to consola type name.
    fn level_to_type(level: Level) -> &'static str {
        match level {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Info => "info",
            Level::Debug => "debug",
            Level::Trace => "trace",
        }
    }
}

impl<R: Reporter + 'static> Log for ConsoLog<R> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Check if the level would be filtered by consola
        let type_name = Self::level_to_type(metadata.level());
        let level = crate::level_for_type(type_name);
        if let Some(level) = level {
            // Lower level numbers are more severe, so we check <=
            level <= self.logger.config().level
        } else {
            false
        }
    }

    fn log(&self, record: &Record) {
        // Recursion guard
        let guard = RECURSION_GUARD.with(|g| {
            if *g.borrow() {
                return false;
            }
            *g.borrow_mut() = true;
            true
        });

        if !guard {
            return;
        }

        // Ensure we clear the guard even if something panics
        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) {
                RECURSION_GUARD.with(|g| *g.borrow_mut() = false);
            }
        }
        let _guard = Guard;

        if !self.enabled(record.metadata()) {
            return;
        }

        let type_name = Self::level_to_type(record.level());

        // Build args from the message
        let args = vec![ArgValue::String(record.args().to_string())];

        // Build meta from module_path, file, and line
        let mut meta = Vec::new();
        if let Some(module) = record.module_path() {
            meta.push(("module".to_string(), ArgValue::String(module.to_string())));
        }
        if let Some(file) = record.file() {
            meta.push(("file".to_string(), ArgValue::String(file.to_string())));
        }
        if let Some(line) = record.line() {
            meta.push(("line".to_string(), ArgValue::Number(line as f64)));
        }

        // Create log record
        let mut log_record = crate::LogRecord::new(type_name, None, args);
        if !meta.is_empty() {
            log_record.meta = Some(meta);
        }

        // Emit the record
        self.logger.emit_record(log_record);
    }

    fn flush(&self) {
        // Note: Logger::flush requires &mut self, but Log::flush only provides &self
        // Users should call flush directly on the Logger if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BasicLogger, LoggerConfig, ThrottleConfig};

    #[test]
    fn log_bridge_basic() {
        let logger = BasicLogger::default().with_config(LoggerConfig {
            level: crate::LogLevel::TRACE,
            throttle: ThrottleConfig::default(),
            queue_capacity: None,
            clock: None,
        });

        let bridge = ConsoLog::new(logger);

        // Just verify these don't panic
        assert!(
            bridge.enabled(
                &log::Metadata::builder()
                    .level(log::Level::Info)
                    .target("test")
                    .build()
            )
        );

        bridge.flush();
    }

    #[test]
    fn log_bridge_level_filtering() {
        let logger = BasicLogger::default().with_config(LoggerConfig {
            level: crate::LogLevel::INFO,
            throttle: ThrottleConfig::default(),
            queue_capacity: None,
            clock: None,
        });

        let bridge = ConsoLog::new(logger);

        // Debug should not be enabled
        assert!(
            !bridge.enabled(
                &log::Metadata::builder()
                    .level(log::Level::Debug)
                    .target("test")
                    .build()
            )
        );

        // Info should be enabled
        assert!(
            bridge.enabled(
                &log::Metadata::builder()
                    .level(log::Level::Info)
                    .target("test")
                    .build()
            )
        );
    }

    #[test]
    fn log_bridge_recursion_safety() {
        let logger = BasicLogger::default();
        let bridge = ConsoLog::new(logger);

        // Simulate recursion by setting the guard
        RECURSION_GUARD.with(|g| {
            *g.borrow_mut() = true;
        });

        // This should not panic or recurse - the log call will be ignored
        let record = log::Record::builder()
            .level(log::Level::Info)
            .target("test")
            .args(format_args!("should not appear"))
            .build();
        bridge.log(&record);

        // Clear guard
        RECURSION_GUARD.with(|g| {
            *g.borrow_mut() = false;
        });
    }
}
