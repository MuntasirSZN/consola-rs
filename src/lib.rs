//! `consola-rs` Elegant Console Logger
//!
//! Features:
//!   - `jiff` (default): timestamps via jiff
//!   - `backtrace` (default): error backtrace capture via `backtrace` crate
//!   - `chrono`: timestamps via chrono
//!   - `time`: timestamps via time
//!   - `log`: implement `log::Log` trait (receive from `log` crate)
//!   - `tracing`: implement `tracing::Subscriber` (receive from `tracing` crate)
//!   - `browser`: browser console styling via `web-sys` (runtime detection)
//!   - `parking_lot`: use `parking_lot::Mutex` (default: std::sync::Mutex)
//!   - `prompt`: interactive prompts via demand
//!   - `prompt-inquire`: interactive prompts via inquire
//!   - `prompt-dialoguer`: interactive prompts via dialoguer
#![deny(unsafe_code)]
#![warn(missing_docs)]

/// Typed error definitions.
pub mod error;

/// The core [`Consola`] struct and its methods for creating and managing log entries.
pub mod consola;
/// Log level and log type constants used throughout the library.
pub mod constants;
/// Built-in reporter implementations (`FancyReporter`, `BasicReporter`).
pub mod reporters;
/// Internal synchronization primitives (parking_lot or std).
pub(crate) mod sync;
/// Shared types and traits, including `Reporter`, `LogObject`, and option structs.
pub mod types;
/// Utility functions for formatting, colors, and string manipulation.
pub mod util;

/// Interactive prompts for user input.
///
/// Provides [`text()`], [`confirm()`], [`select()`], and [`multiselect()`] functions,
/// plus the sentinel constant [`K_CANCEL`] returned when the user aborts.
///
/// Backend selection (priority):
/// - `prompt` → demand (default)
/// - `prompt-inquire` → inquire
/// - `prompt-dialoguer` → dialoguer
pub mod prompt;

use std::sync::LazyLock;

use reporters::{BasicReporter, FancyReporter};
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

/// A default, lazily-initialized [`Consola`] instance for convenience use.
pub static CONSOLA: LazyLock<Consola> = LazyLock::new(|| create_consola(None, vec![]));

pub use consola::Consola;
pub use constants::{LogLevel, LogType, log_levels};
pub use types::{ConsolaOptions as ConsolaOpts, FormatOptions, LogObject, LogObjectInput};
pub use types::{ConsolaOptions, LogContext, Reporter};
pub use util::*;
