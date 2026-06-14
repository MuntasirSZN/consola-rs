//! Formatting options, terminal width detection, and error info.

/// Controls formatting behavior of log output.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Override the terminal column width for output wrapping.
    pub columns: Option<u16>,
    /// Whether to include a timestamp prefix in log output.
    pub date: bool,
    /// Whether to use ANSI color codes in formatted output.
    pub colors: bool,
    /// Whether to use compact formatting (single-line output).
    pub compact: bool,
    /// Maximum error level to display in stack traces.
    pub error_level: u32,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            columns: terminal_width(),
            date: true,
            colors: false,
            compact: true,
            error_level: 0,
        }
    }
}

/// Attempt to detect terminal width at runtime.
/// Returns `None` when not connected to a terminal.
pub fn terminal_width() -> Option<u16> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::IsTerminal;
        if std::io::stdout().is_terminal() {
            terminal_size::terminal_size().map(|(width, _)| width.0)
        } else {
            None
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

/// Information about an error for rich error-chain formatting.
#[derive(Debug, Clone, Default)]
pub struct ErrorInfo {
    /// The error message.
    pub message: String,
    /// Optional stack trace as a string.
    pub stack: Option<String>,
    /// Optional captured backtrace (populated when `backtrace` feature is enabled).
    pub backtrace: Option<String>,
    /// The cause of this error (next in the chain).
    pub cause: Option<Box<ErrorInfo>>,
}
