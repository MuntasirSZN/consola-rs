//! Bridge implementation for the `tracing` crate.
//!
//! This module provides a `tracing_subscriber::Layer` implementation that routes
//! tracing events to a consola Logger instance.

use crate::{ArgValue, Logger, Reporter};
use std::cell::RefCell;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

thread_local! {
    /// Recursion guard to prevent infinite loops when consola logs to tracing
    static TRACING_RECURSION_GUARD: RefCell<bool> = const { RefCell::new(false) };
}

/// A field collector that gathers all fields from a tracing event.
struct FieldCollector {
    message: Option<String>,
    fields: Vec<(String, ArgValue)>,
}

impl FieldCollector {
    fn new() -> Self {
        Self {
            message: None,
            fields: Vec::new(),
        }
    }
}

impl Visit for FieldCollector {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let name = field.name();
        let value_str = format!("{:?}", value);

        if name == "message" {
            self.message = Some(value_str);
        } else {
            self.fields
                .push((name.to_string(), ArgValue::String(value_str)));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        let name = field.name();
        if name == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields
                .push((name.to_string(), ArgValue::String(value.to_string())));
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        let name = field.name();
        if name == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields
                .push((name.to_string(), ArgValue::Number(value as f64)));
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        let name = field.name();
        if name == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields
                .push((name.to_string(), ArgValue::Number(value as f64)));
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        let name = field.name();
        if name == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.push((name.to_string(), ArgValue::Bool(value)));
        }
    }
}

/// A `tracing_subscriber::Layer` implementation that bridges to a consola Logger.
pub struct ConsoLayer<R: Reporter + 'static> {
    logger: Logger<R>,
}

impl<R: Reporter + 'static> ConsoLayer<R> {
    /// Create a new ConsoLayer bridge with the given logger.
    pub fn new(logger: Logger<R>) -> Self {
        Self { logger }
    }

    /// Map tracing::Level to consola type name.
    fn level_to_type(level: &Level) -> &'static str {
        match *level {
            Level::ERROR => "error",
            Level::WARN => "warn",
            Level::INFO => "info",
            Level::DEBUG => "debug",
            Level::TRACE => "trace",
        }
    }
}

impl<S, R> Layer<S> for ConsoLayer<R>
where
    S: Subscriber,
    R: Reporter + 'static,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Recursion guard
        let guard = TRACING_RECURSION_GUARD.with(|g| {
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
                TRACING_RECURSION_GUARD.with(|g| *g.borrow_mut() = false);
            }
        }
        let _guard = Guard;

        let type_name = Self::level_to_type(event.metadata().level());

        // Check if this level would be filtered
        let level = crate::level_for_type(type_name);
        if let Some(level) = level {
            // Lower level numbers are more severe, so we check <=
            if level > self.logger.config().level {
                return;
            }
        } else {
            return;
        }

        // Collect all fields from the event
        let mut collector = FieldCollector::new();
        event.record(&mut collector);

        // Build args from the message
        let args = if let Some(message) = collector.message {
            vec![ArgValue::String(message)]
        } else {
            vec![ArgValue::String("(no message)".to_string())]
        };

        // Create log record with metadata
        let mut log_record = crate::LogRecord::new(type_name, None, args);

        // Add non-message fields to meta
        if !collector.fields.is_empty() {
            log_record.meta = Some(collector.fields);
        }

        // Emit the record
        self.logger.emit_record(log_record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BasicLogger, LoggerConfig, ThrottleConfig};
    use tracing::{info, warn};

    #[test]
    fn tracing_bridge_basic() {
        let logger = BasicLogger::default().with_config(LoggerConfig {
            level: crate::LogLevel::TRACE,
            throttle: ThrottleConfig::default(),
            queue_capacity: None,
            clock: None,
        });

        let layer = ConsoLayer::new(logger);

        // Set up tracing subscriber with our layer
        use tracing_subscriber::Registry;
        use tracing_subscriber::layer::SubscriberExt;
        let subscriber = Registry::default().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            info!("info message");
            warn!("warning message");
        });

        // Test passes if no panic occurs
    }

    #[test]
    fn tracing_bridge_field_capture() {
        let logger = BasicLogger::default().with_config(LoggerConfig {
            level: crate::LogLevel::TRACE,
            throttle: ThrottleConfig::default(),
            queue_capacity: None,
            clock: None,
        });

        let layer = ConsoLayer::new(logger);

        use tracing_subscriber::Registry;
        use tracing_subscriber::layer::SubscriberExt;
        let subscriber = Registry::default().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            info!(user_id = 42, status = "active", "User logged in");
        });

        // Test passes if no panic occurs
    }

    #[test]
    fn tracing_bridge_recursion_safety() {
        let logger = BasicLogger::default();
        let layer = ConsoLayer::new(logger);

        // Simulate recursion by setting the guard
        TRACING_RECURSION_GUARD.with(|g| {
            *g.borrow_mut() = true;
        });

        use tracing_subscriber::Registry;
        use tracing_subscriber::layer::SubscriberExt;
        let subscriber = Registry::default().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            info!("should not appear");
        });

        // Clear guard
        TRACING_RECURSION_GUARD.with(|g| {
            *g.borrow_mut() = false;
        });

        // Test passes if no panic occurs
    }
}
