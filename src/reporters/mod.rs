//! Reporters that transform [`LogObject`]s into formatted output strings.
//!
//! Each module provides a reporter implementation with a different formatting style:
//! [`basic`] for plain text, [`browser`] for web console output, and [`fancy`] for
//! colored terminal output.

/// Plain-text reporter that formats log messages without colors or icons.
pub mod basic;
/// Browser console reporter with runtime browser detection.
pub mod browser;
/// Fancy reporter with colors, icons, and rich formatting for terminal output.
pub mod fancy;

pub use basic::BasicReporter;
pub use browser::BrowserReporter;
pub use fancy::FancyReporter;
