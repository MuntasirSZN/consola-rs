//! ─── consola-rs: Elegant Console Logger ──────────────────────────────────────
//! Ported from consola-js v3.4.2
//!
//! Features:
//!   - `jiff` (default): timestamps via jiff
//!   - `chrono`: timestamps via chrono
//!   - `log`: emit via `log` crate
//!   - `tracing`: emit via `tracing` crate
//!   - `wasm`: WASM console via web-sys
#![deny(unsafe_code)]
#![warn(missing_docs)]

/// The core [`Consola`] struct and its methods for creating and managing log entries.
pub mod consola;
/// Log level and log type constants used throughout the library.
pub mod constants;
/// Built-in reporter implementations (`FancyReporter`, `BasicReporter`).
pub mod reporters;
/// Shared types and traits, including `Reporter`, `LogObject`, and option structs.
pub mod types;
/// Utility functions for formatting, colors, and string manipulation.
pub mod util;

use std::sync::LazyLock;

use reporters::{BasicReporter, FancyReporter};
use types::ConsolaOptions;

// ─── Factory functions ───────────────────────────────────────────────────────

/// Create a new Consola instance with the given reporters and options.
///
/// By default uses `FancyReporter`. Pass `Reporters::Basic` to use the basic reporter.
pub fn create_consola(
    level: Option<LogLevel>,
    reporters: Vec<Box<dyn types::Reporter>>,
) -> Consola {
    let level = level.unwrap_or(constants::log_levels::INFO);
    let reporters = if reporters.is_empty() {
        vec![Box::new(FancyReporter::new()) as Box<dyn types::Reporter>]
    } else {
        reporters
    };

    Consola::new(ConsolaOptions {
        level,
        reporters,
        ..ConsolaOptions::default()
    })
}

/// Create a Consola instance with only `BasicReporter`.
pub fn create_basic_consola(level: Option<LogLevel>) -> Consola {
    let level = level.unwrap_or(constants::log_levels::INFO);
    Consola::new(ConsolaOptions {
        level,
        reporters: vec![Box::new(BasicReporter) as Box<dyn types::Reporter>],
        ..ConsolaOptions::default()
    })
}

/// Create a Consola instance with only `FancyReporter`.
pub fn create_fancy_consola(level: Option<LogLevel>) -> Consola {
    let level = level.unwrap_or(constants::log_levels::INFO);
    Consola::new(ConsolaOptions {
        level,
        reporters: vec![Box::new(FancyReporter::new()) as Box<dyn types::Reporter>],
        ..ConsolaOptions::default()
    })
}

/// Create a minimal Consola instance (no reporters configured).
pub fn create_core_consola(
    level: Option<LogLevel>,
    reporters: Vec<Box<dyn types::Reporter>>,
) -> Consola {
    let level = level.unwrap_or(constants::log_levels::INFO);
    Consola::new(ConsolaOptions {
        level,
        reporters,
        ..ConsolaOptions::default()
    })
}

// ─── Default instance ─────────────────────────────────────────────────────────

/// A default, lazily-initialized [`Consola`] instance for convenience use.
pub static CONSOLA: LazyLock<Consola> = LazyLock::new(|| create_consola(None, vec![]));

// ─── Re-exports ───────────────────────────────────────────────────────────────

pub use consola::Consola;
pub use constants::{LogLevel, LogType, log_levels};
pub use types::{ConsolaOptions as ConsolaOpts, FormatOptions, LogObject, LogObjectInput};
pub use util::*;

// ─── Prompt module ────────────────────────────────────────────────────────

#[cfg(feature = "prompt")]
/// Interactive prompts for user input.
///
/// Provides [`text()`], [`confirm()`], [`select()`], and [`multiselect()`] functions,
/// plus the sentinel constant [`K_CANCEL`] returned when the user aborts.
pub mod prompt {
    /// Sentinel value returned when a user cancels a prompt.
    pub const K_CANCEL: &str = "Symbol(cancel)";

    pub use super::types::{
        ConfirmPromptOptions, MultiSelectOptions, PromptCommonOptions, PromptOptions, SelectOption,
        SelectPromptOptions, TextPromptOptions,
    };

    use demand::{Confirm, Input, MultiSelect, Select};

    fn map_io_error(e: std::io::Error) -> String {
        if e.kind() == std::io::ErrorKind::Interrupted {
            K_CANCEL.to_string()
        } else {
            e.to_string()
        }
    }

    /// Prompt the user for free-form text input.
    pub fn text(message: &str, opts: &TextPromptOptions) -> Result<String, String> {
        let mut input = Input::new(message);
        if let Some(ref placeholder) = opts.placeholder {
            input = input.placeholder(placeholder);
        }
        if let Some(ref default) = opts.default {
            input = input.default_value(default);
        }
        input.run().map_err(map_io_error)
    }

    /// Prompt the user for a yes/no confirmation.
    pub fn confirm(message: &str, opts: &ConfirmPromptOptions) -> Result<bool, String> {
        let mut dialog = Confirm::new(message);
        if let Some(initial) = opts.initial {
            dialog = dialog.selected(initial);
        }
        dialog.run().map_err(map_io_error)
    }

    /// Prompt the user to select a single option from a list.
    pub fn select(message: &str, opts: &SelectPromptOptions) -> Result<String, String> {
        let items: Vec<demand::DemandOption<String>> = opts
            .options
            .iter()
            .map(|opt| {
                let mut d = demand::DemandOption::with_label(&opt.label, opt.value.clone());
                if let Some(ref hint) = opt.hint {
                    d = d.description(hint);
                }
                d
            })
            .collect();
        Select::new(message)
            .options(items)
            .run()
            .map_err(map_io_error)
    }

    /// Prompt the user to select one or more options from a list.
    pub fn multiselect(message: &str, opts: &MultiSelectOptions) -> Result<Vec<String>, String> {
        let mut ms = MultiSelect::new(message);
        if opts.required.unwrap_or(false) {
            ms = ms.min(1);
        }
        let items: Vec<demand::DemandOption<String>> = opts
            .options
            .iter()
            .map(|opt| {
                let mut d = demand::DemandOption::with_label(&opt.label, opt.value.clone());
                if let Some(ref hint) = opt.hint {
                    d = d.description(hint);
                }
                d
            })
            .collect();
        ms.options(items).run().map_err(map_io_error)
    }
}

/// Interactive prompt support
#[cfg(not(feature = "prompt"))]
pub mod prompt {

    /// Sentinel value returned when a user cancels a prompt.
    pub const K_CANCEL: &str = "Symbol(cancel)";

    pub use super::types::{
        ConfirmPromptOptions, MultiSelectOptions, PromptCommonOptions, PromptOptions, SelectOption,
        SelectPromptOptions, TextPromptOptions,
    };

    /// Prompt the user for free-form text input (requires `prompt` feature).
    pub fn text(_message: &str, _opts: &TextPromptOptions) -> Result<String, String> {
        Err("interactive prompts require the `prompt` Cargo feature".into())
    }

    /// Prompt the user for a yes/no confirmation (requires `prompt` feature).
    pub fn confirm(_message: &str, _opts: &ConfirmPromptOptions) -> Result<bool, String> {
        Err("interactive prompts require the `prompt` Cargo feature".into())
    }

    /// Prompt the user to select a single option (requires `prompt` feature).
    pub fn select(_message: &str, _opts: &SelectPromptOptions) -> Result<String, String> {
        Err("interactive prompts require the `prompt` Cargo feature".into())
    }

    /// Prompt the user to select multiple options (requires `prompt` feature).
    pub fn multiselect(_message: &str, _opts: &MultiSelectOptions) -> Result<Vec<String>, String> {
        Err("interactive prompts require the `prompt` Cargo feature".into())
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporters::{BasicReporter, FancyReporter};
    use crate::types::{LogContext, LogObject, Reporter};

    #[test]
    fn test_can_set_level() {
        let c = create_consola(None, vec![]);
        assert_eq!(c.level(), log_levels::INFO);
        for i in 0..=5 {
            c.set_level(i);
            assert_eq!(c.level(), i);
        }
    }

    #[test]
    fn test_with_tag() {
        let c = create_consola(None, vec![]);
        let tagged = c.with_tag("api");
        assert!(tagged.info("test"));
    }

    #[test]
    fn test_pause_resume() {
        let c = create_consola(None, vec![]);
        c.pause_logs();
        c.info("queued");
        c.resume_logs();
    }

    #[test]
    fn test_log_methods_all_types() {
        let c = create_fancy_consola(Some(log_levels::VERBOSE));
        assert!(c.fatal("test"));
        assert!(c.error("test"));
        assert!(c.warn("test"));
        assert!(c.log("test"));
        assert!(c.info("test"));
        assert!(c.success("test"));
        assert!(c.fail("test"));
        assert!(c.ready("test"));
        assert!(c.start("test"));
        assert!(c.box_("test"));
        assert!(c.debug("test"));
        assert!(c.trace("test"));
        assert!(c.verbose("test"));
    }

    #[test]
    fn test_raw_methods() {
        let c = create_consola(None, vec![]);
        assert!(c.fatal_raw("test"));
        assert!(c.error_raw("test"));
        assert!(c.info_raw("test"));
        assert!(c.log_raw("test"));
    }

    #[test]
    fn test_log_obj() {
        let c = create_consola(None, vec![]);
        let input = LogObjectInput::new().message("hello").tag("test");
        assert!(c.log_obj(&input));
    }

    #[test]
    fn test_library_static_works() {
        let _ = &*CONSOLA;
    }

    #[test]
    fn test_reporter_format() {
        let r = BasicReporter;
        let log_obj = LogObject::new(LogType::Info);
        let ctx = LogContext {
            options: std::sync::Arc::new(ConsolaOptions::default()),
        };
        let result = r.format(&log_obj, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fancy_reporter_format() {
        let r = FancyReporter::new();
        let log_obj = LogObject::new(LogType::Info);
        let ctx = LogContext {
            options: std::sync::Arc::new(ConsolaOptions::default()),
        };
        let result = r.format(&log_obj, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_level_filter() {
        let c = create_consola(Some(log_levels::WARN), vec![]);
        assert!(c.error("should log")); // level 0
        assert!(c.warn("should log")); // level 1
        assert!(!c.info("should NOT log")); // level 3
    }

    #[test]
    fn test_throttle_basic() {
        let c = create_consola(Some(log_levels::INFO), vec![]);
        // Should not panic with repeated messages
        c.info("message");
        c.info("message");
    }
}
