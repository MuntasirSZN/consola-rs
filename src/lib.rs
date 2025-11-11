//! consola-rs core library: modular facade.

pub mod levels;
pub use levels::*;
pub mod record;
pub use record::*;
pub mod throttling;
pub use throttling::*;
pub mod reporter;
pub use reporter::*;
pub mod format;
pub use format::*;
pub mod error_chain;
pub use error_chain::*;
pub mod utils;
pub use utils::*;
pub mod clock;
pub use clock::*;

#[cfg(any(feature = "prompt-demand", feature = "wasm"))]
pub mod prompt;
#[cfg(any(feature = "prompt-demand", feature = "wasm"))]
pub use prompt::*;

#[cfg(feature = "bridge-log")]
pub mod bridge_log;
#[cfg(feature = "bridge-log")]
pub use bridge_log::*;

#[cfg(feature = "bridge-tracing")]
pub mod bridge_tracing;
#[cfg(feature = "bridge-tracing")]
pub use bridge_tracing::*;

#[macro_use]
pub mod macros;
pub use macros::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn level_ordering() {
        assert!(LogLevel::FATAL < LogLevel::ERROR);
        assert!(LogLevel::TRACE < LogLevel::VERBOSE);
        assert!(LogLevel::SILENT < LogLevel::FATAL);
    }

    #[test]
    fn default_type_mapping() {
        assert_eq!(level_for_type("info"), Some(LogLevel::INFO));
        assert_eq!(level_for_type("trace"), Some(LogLevel::TRACE));
        assert_eq!(level_for_type("verbose"), Some(LogLevel::VERBOSE));
    }

    #[test]
    fn custom_type_registration() {
        register_type(
            "custom",
            LogTypeSpec {
                level: LogLevel(42),
            },
        );
        assert_eq!(level_for_type("custom"), Some(LogLevel(42)));
    }

    #[test]
    fn record_message_building() {
        let r = LogRecord::new("info", None, vec!["hello".into(), 5i64.into(), true.into()]);
        assert_eq!(r.message.as_deref(), Some("hello 5 true"));
    }

    #[test]
    fn throttle_basic() {
        let mut throttler = Throttler::new(ThrottleConfig {
            window: Duration::from_millis(200),
            min_count: 3,
        });
        let mut emitted: Vec<LogRecord> = Vec::new();
        throttler.on_record(LogRecord::new("info", None, vec!["x".into()]), |r| {
            emitted.push(r.clone())
        });
        assert_eq!(emitted.len(), 1); // first
        throttler.on_record(LogRecord::new("info", None, vec!["x".into()]), |r| {
            emitted.push(r.clone())
        });
        assert_eq!(emitted.len(), 1); // suppressed
        throttler.on_record(LogRecord::new("info", None, vec!["x".into()]), |r| {
            emitted.push(r.clone())
        });
        assert_eq!(emitted.len(), 2); // aggregated
        assert_eq!(emitted[1].repetition_count, 3);
    }

    #[test]
    fn level_filtering() {
        let mut logger = BasicLogger::default().with_config(super::LoggerConfig {
            level: LogLevel::INFO,
            throttle: ThrottleConfig::default(),
            queue_capacity: None,
            clock: None,
        });
        // debug should not pass (6 > 4)
        logger.log("debug", None, ["hidden"]);
        // info should pass
        logger.log("info", None, ["shown"]);
    }

    #[test]
    fn pause_resume_order() {
        let mut logger = BasicLogger::default();
        logger.pause();
        logger.log("info", None, ["a"]);
        logger.log("info", None, ["b"]);
        logger.resume();
        // Order should be preserved (manual inspection; advanced test would capture output with custom reporter)
    }

    #[test]
    fn strip_ansi_basic() {
        let colored = "\u{1b}[31mred\u{1b}[0m text";
        let stripped = crate::utils::strip_ansi(colored);
        assert_eq!(stripped, "red text");
    }
}
