//! WASM bindings for consola-rs.
//!
//! This module provides JavaScript-friendly bindings for using consola in WebAssembly.
//!
//! # Example Usage (JavaScript)
//!
//! ```javascript
//! import init, { create_logger, log_info, log_error, set_level, free_logger } from './pkg/consola.js';
//!
//! await init();
//!
//! // Create a logger
//! const logger = create_logger();
//!
//! // Log messages
//! log_info(logger, "Hello from WASM!");
//! log_error(logger, "Something went wrong");
//!
//! // Set level
//! set_level(logger, 4); // INFO level
//!
//! // Clean up
//! free_logger(logger);
//! ```

use crate::{ArgValue, BasicLogger, LogLevel, LoggerBuilder};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

/// Opaque handle to a logger instance
#[wasm_bindgen]
pub struct WasmLogger {
    inner: Arc<Mutex<BasicLogger>>,
}

/// Create a new logger with default configuration
#[wasm_bindgen]
pub fn create_logger() -> WasmLogger {
    WasmLogger {
        inner: Arc::new(Mutex::new(BasicLogger::default())),
    }
}

/// Create a logger with a specific log level
#[wasm_bindgen]
pub fn create_logger_with_level(level: i16) -> WasmLogger {
    let logger = LoggerBuilder::new().with_level(LogLevel(level)).build();

    WasmLogger {
        inner: Arc::new(Mutex::new(logger)),
    }
}

/// Free a logger instance
#[wasm_bindgen]
pub fn free_logger(_logger: WasmLogger) {
    // Logger will be dropped automatically
}

/// Set the log level for a logger
#[wasm_bindgen]
pub fn set_level(logger: &WasmLogger, level: i16) {
    if let Ok(mut log) = logger.inner.lock() {
        log.set_level(LogLevel(level));
    }
}

/// Pause logging (messages will be buffered)
#[wasm_bindgen]
pub fn pause(logger: &WasmLogger) {
    if let Ok(mut log) = logger.inner.lock() {
        log.pause();
    }
}

/// Resume logging (flush buffered messages)
#[wasm_bindgen]
pub fn resume(logger: &WasmLogger) {
    if let Ok(mut log) = logger.inner.lock() {
        log.resume();
    }
}

/// Flush any buffered log messages
#[wasm_bindgen]
pub fn flush(logger: &WasmLogger) {
    if let Ok(mut log) = logger.inner.lock() {
        log.flush();
    }
}

/// Log a simple string message (fast path)
#[wasm_bindgen]
pub fn log_simple(logger: &WasmLogger, type_name: &str, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.log(type_name, None, [ArgValue::String(message.to_string())]);
    }
}

/// Log an info message
#[wasm_bindgen]
pub fn log_info(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.info(message);
    }
}

/// Log a warning message
#[wasm_bindgen]
pub fn log_warn(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.warn(message);
    }
}

/// Log an error message
#[wasm_bindgen]
pub fn log_error(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.error(message);
    }
}

/// Log a debug message
#[wasm_bindgen]
pub fn log_debug(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.debug(message);
    }
}

/// Log a trace message
#[wasm_bindgen]
pub fn log_trace(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.trace(message);
    }
}

/// Log a success message
#[wasm_bindgen]
pub fn log_success(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.log("success", None, [ArgValue::String(message.to_string())]);
    }
}

/// Log a fail message
#[wasm_bindgen]
pub fn log_fail(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.log("fail", None, [ArgValue::String(message.to_string())]);
    }
}

/// Log a fatal message
#[wasm_bindgen]
pub fn log_fatal(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.log("fatal", None, [ArgValue::String(message.to_string())]);
    }
}

/// Log a ready message
#[wasm_bindgen]
pub fn log_ready(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.log("ready", None, [ArgValue::String(message.to_string())]);
    }
}

/// Log a start message
#[wasm_bindgen]
pub fn log_start(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.log("start", None, [ArgValue::String(message.to_string())]);
    }
}

/// Log a box message (for multi-line content)
#[wasm_bindgen]
pub fn log_box(logger: &WasmLogger, message: &str) {
    if let Ok(mut log) = logger.inner.lock() {
        log.log("box", None, [ArgValue::String(message.to_string())]);
    }
}

/// Log a message with a JavaScript Error object
/// This extracts the error message and stack trace
#[wasm_bindgen]
pub fn log_error_with_js_error(logger: &WasmLogger, message: &str, error: &JsValue) {
    if let Ok(mut log) = logger.inner.lock() {
        let mut args = vec![ArgValue::String(message.to_string())];

        // Try to extract error message
        if let Some(err) = error.as_string() {
            args.push(ArgValue::Error(err));
        } else {
            // Fallback to debug representation
            args.push(ArgValue::OtherDebug(format!("{:?}", error)));
        }

        log.log("error", None, args);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_create_and_free_logger() {
        let logger = create_logger();
        free_logger(logger);
    }

    #[wasm_bindgen_test]
    fn test_log_simple() {
        let logger = create_logger();
        log_simple(&logger, "info", "test message");
        free_logger(logger);
    }

    #[wasm_bindgen_test]
    fn test_log_levels() {
        let logger = create_logger();
        log_info(&logger, "info message");
        log_warn(&logger, "warning message");
        log_error(&logger, "error message");
        log_debug(&logger, "debug message");
        log_trace(&logger, "trace message");
        free_logger(logger);
    }

    #[wasm_bindgen_test]
    fn test_set_level() {
        let logger = create_logger();
        set_level(&logger, 4); // INFO level
        log_info(&logger, "should appear");
        log_debug(&logger, "should not appear");
        free_logger(logger);
    }

    #[wasm_bindgen_test]
    fn test_pause_resume() {
        let logger = create_logger();
        pause(&logger);
        log_info(&logger, "buffered");
        resume(&logger);
        free_logger(logger);
    }
}
