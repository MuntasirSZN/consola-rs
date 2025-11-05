//! Ergonomic macros for logging
//!
//! Provides convenient macros like `info!`, `warn!`, `error!`, etc.
//! that work similar to `println!` but with the consola logging system.

/// Log an info message
///
/// # Examples
///
/// ```
/// use consola::info;
///
/// info!("Hello, world!");
/// info!("User {} logged in", "alice");
/// ```
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        if $crate::is_log_type_enabled("info") {
            $crate::log_message("info", &format!($($arg)*))
        }
    }};
}

/// Log a warning message
///
/// # Examples
///
/// ```
/// use consola::warn;
///
/// warn!("Low disk space: {} MB remaining", 100);
/// ```
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log_message("warn", &format!($($arg)*))
    };
}

/// Log an error message
///
/// # Examples
///
/// ```
/// use consola::error;
///
/// error!("Failed to connect to database");
/// ```
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log_message("error", &format!($($arg)*))
    };
}

/// Log a success message
///
/// # Examples
///
/// ```
/// use consola::success;
///
/// success!("Build completed successfully!");
/// ```
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        $crate::log_message("success", &format!($($arg)*))
    };
}

/// Log a debug message
///
/// # Examples
///
/// ```
/// use consola::debug;
///
/// let some_value = 42;
/// debug!("Variable value: {:?}", some_value);
/// ```
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        if $crate::is_log_type_enabled("debug") {
            $crate::log_message("debug", &format!($($arg)*))
        }
    }};
}

/// Log a trace message
///
/// # Examples
///
/// ```
/// use consola::trace;
///
/// trace!("Entering function foo()");
/// ```
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        if $crate::is_log_type_enabled("trace") {
            $crate::log_message("trace", &format!($($arg)*))
        }
    }};
}

/// Log a fatal message
///
/// # Examples
///
/// ```
/// use consola::fatal;
///
/// fatal!("Critical system error!");
/// ```
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {
        $crate::log_message("fatal", &format!($($arg)*))
    };
}

/// Log a ready message
///
/// # Examples
///
/// ```
/// use consola::ready;
///
/// ready!("Server listening on port {}", 8080);
/// ```
#[macro_export]
macro_rules! ready {
    ($($arg:tt)*) => {
        $crate::log_message("ready", &format!($($arg)*))
    };
}

/// Log a start message
///
/// # Examples
///
/// ```
/// use consola::start;
///
/// start!("Starting application...");
/// ```
#[macro_export]
macro_rules! start {
    ($($arg:tt)*) => {
        $crate::log_message("start", &format!($($arg)*))
    };
}

/// Log a fail message
///
/// # Examples
///
/// ```
/// use consola::fail;
///
/// let expected = 10;
/// let actual = 20;
/// fail!("Test failed: expected {}, got {}", expected, actual);
/// ```
#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {
        $crate::log_message("fail", &format!($($arg)*))
    };
}

/// Log with a custom type
///
/// # Examples
///
/// ```
/// use consola::log_type;
///
/// let value = 42;
/// log_type!("custom", "Custom message: {}", value);
/// ```
#[macro_export]
macro_rules! log_type {
    ($type_name:expr, $($arg:tt)*) => {
        $crate::log_message($type_name, &format!($($arg)*))
    };
}

// Raw logging macros (bypass formatting pipeline)

/// Log an info message (raw, no formatting)
#[macro_export]
macro_rules! info_raw {
    ($($arg:tt)*) => {
        $crate::log_message_raw("info", &format!($($arg)*))
    };
}

/// Log a warning message (raw, no formatting)
#[macro_export]
macro_rules! warn_raw {
    ($($arg:tt)*) => {
        $crate::log_message_raw("warn", &format!($($arg)*))
    };
}

/// Log an error message (raw, no formatting)
#[macro_export]
macro_rules! error_raw {
    ($($arg:tt)*) => {
        $crate::log_message_raw("error", &format!($($arg)*))
    };
}

/// Log a success message (raw, no formatting)
#[macro_export]
macro_rules! success_raw {
    ($($arg:tt)*) => {
        $crate::log_message_raw("success", &format!($($arg)*))
    };
}

/// Log a debug message (raw, no formatting)
#[macro_export]
macro_rules! debug_raw {
    ($($arg:tt)*) => {
        $crate::log_message_raw("debug", &format!($($arg)*))
    };
}

/// Log a trace message (raw, no formatting)
#[macro_export]
macro_rules! trace_raw {
    ($($arg:tt)*) => {
        $crate::log_message_raw("trace", &format!($($arg)*))
    };
}

/// Check if a log type is enabled (used by macros for level guard optimization)
/// Checks against CONSOLA_LEVEL environment variable if set, otherwise always enabled
pub fn is_log_type_enabled(type_name: &str) -> bool {
    use crate::{LogLevel, level_for_type};
    use std::env;

    // Get the type's level
    let type_level = match level_for_type(type_name) {
        Some(level) => level,
        None => return true, // Unknown types are always enabled
    };

    // Check if we have a configured level from environment
    if let Ok(level_str) = env::var("CONSOLA_LEVEL") {
        // Try to parse as numeric level
        if let Ok(level_num) = level_str.parse::<i16>() {
            let configured_level = LogLevel(level_num);
            return type_level <= configured_level;
        }

        // Try to parse as type name
        if let Some(configured_level) = level_for_type(&level_str) {
            return type_level <= configured_level;
        }
    }

    // Default: all types enabled
    true
}

/// Helper function to log a message (used by macros)
pub fn log_message(type_name: &str, message: &str) {
    // This is a placeholder - in a real implementation, this would use
    // the logger instance or a global logger
    println!("[{}] {}", type_name, message);
}

/// Helper function to log a raw message (used by macros)
pub fn log_message_raw(_type_name: &str, message: &str) {
    // This is a placeholder - in a real implementation, this would use
    // the logger's raw logging path
    println!("{}", message);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_info_macro() {
        info!("Test info message");
        info!("Test with arg: {}", 42);
    }

    #[test]
    fn test_warn_macro() {
        warn!("Test warning");
    }

    #[test]
    fn test_error_macro() {
        error!("Test error");
    }

    #[test]
    fn test_success_macro() {
        success!("Test success");
    }

    #[test]
    fn test_debug_macro() {
        debug!("Test debug");
    }

    #[test]
    fn test_trace_macro() {
        trace!("Test trace");
    }

    #[test]
    fn test_fatal_macro() {
        fatal!("Test fatal");
    }

    #[test]
    fn test_ready_macro() {
        ready!("Test ready");
    }

    #[test]
    fn test_start_macro() {
        start!("Test start");
    }

    #[test]
    fn test_fail_macro() {
        fail!("Test fail");
    }

    #[test]
    fn test_log_type_macro() {
        log_type!("custom", "Test custom type");
    }

    #[test]
    fn test_raw_macros() {
        info_raw!("Raw info");
        warn_raw!("Raw warning");
        error_raw!("Raw error");
        success_raw!("Raw success");
        debug_raw!("Raw debug");
        trace_raw!("Raw trace");
    }

    // Test filtered-out macro short-circuits
    #[test]
    fn test_filtered_macro_short_circuit() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        // This test demonstrates that when is_log_type_enabled returns false,
        // the format! macro should not be evaluated

        // Create a flag to track if expensive operation runs
        let expensive_called = Arc::new(AtomicBool::new(false));
        let expensive_called_clone = expensive_called.clone();

        let expensive_operation = || {
            expensive_called_clone.store(true, Ordering::SeqCst);
            "expensive result"
        };

        // Test that macros compile and run without panic
        info!("Test with expensive: {}", expensive_operation());

        // By default (no CONSOLA_LEVEL set), all types are enabled,
        // so expensive_operation will be called
        assert!(expensive_called.load(Ordering::SeqCst));

        // When CONSOLA_LEVEL is set to a higher level (e.g., warn or error),
        // info messages are filtered and expensive_operation won't be called.
        expensive_called.store(false, Ordering::SeqCst);
        let previous_level = std::env::var("CONSOLA_LEVEL").ok();

        unsafe {
            std::env::set_var("CONSOLA_LEVEL", "warn");
        }

        info!("Test with expensive: {}", expensive_operation());
        assert!(!expensive_called.load(Ordering::SeqCst));

        if let Some(prev) = previous_level {
            unsafe {
                std::env::set_var("CONSOLA_LEVEL", prev);
            }
        } else {
            unsafe {
                std::env::remove_var("CONSOLA_LEVEL");
            }
        }
    }
}
