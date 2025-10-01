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
    ($($arg:tt)*) => {
        $crate::log_message("info", &format!($($arg)*))
    };
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
    ($($arg:tt)*) => {
        $crate::log_message("debug", &format!($($arg)*))
    };
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
    ($($arg:tt)*) => {
        $crate::log_message("trace", &format!($($arg)*))
    };
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
}
