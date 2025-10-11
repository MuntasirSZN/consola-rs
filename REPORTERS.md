# Custom Reporters Guide

This guide explains how to create custom reporters for consola-rs to control how log messages are formatted and output.

## Overview

Reporters in consola-rs are responsible for formatting and emitting log records. The library provides three built-in reporters:

- **BasicReporter**: Simple text-based output with optional colors
- **FancyReporter**: Enhanced output with icons, badges, and rich formatting
- **JsonReporter**: Structured JSON output for machine parsing
- **MemoryReporter**: In-memory capture for testing

## The Reporter Trait

To create a custom reporter, implement the `Reporter` trait:

```rust
use consola::{Reporter, LogRecord};
use std::io::{self, Write};

pub trait Reporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()>;
}
```

The `emit` method is called for each log record that passes filtering and throttling. You write the formatted output to the provided `Write` instance.

## Basic Custom Reporter Example

Here's a minimal custom reporter that outputs logs in a custom format:

```rust
use consola::*;
use std::io::{self, Write};

pub struct CustomReporter;

impl Reporter for CustomReporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        // Simple format: [TYPE] message
        write!(w, "[{}] ", record.type_name.to_uppercase())?;
        
        // Write message if available
        if let Some(msg) = &record.message {
            write!(w, "{}", msg)?;
        }
        
        // Write args
        for arg in &record.args {
            write!(w, " {}", format_arg(arg))?;
        }
        
        writeln!(w)?;
        Ok(())
    }
}

fn format_arg(arg: &ArgValue) -> String {
    match arg {
        ArgValue::String(s) => s.clone(),
        ArgValue::Number(n) => n.to_string(),
        ArgValue::Bool(b) => b.to_string(),
        ArgValue::Error(e) => format!("Error: {}", e),
        ArgValue::OtherDebug(d) => d.clone(),
        #[cfg(feature = "json")]
        ArgValue::Json(j) => j.to_string(),
    }
}

// Usage
fn main() {
    let mut logger = Logger::new(CustomReporter);
    logger.log("info", None, ["Hello, world!"]);
}
```

## Advanced: Reporter with Options

For more control, implement the `ReporterWithOptions` trait to allow configuration:

```rust
use consola::*;
use std::io::{self, Write};

pub struct ConfigurableReporter {
    opts: FormatOptions,
    prefix: String,
}

impl ConfigurableReporter {
    pub fn new(prefix: String) -> Self {
        Self {
            opts: FormatOptions::default(),
            prefix,
        }
    }
}

impl Reporter for ConfigurableReporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        // Add timestamp if enabled
        if self.opts.date {
            write!(w, "[{}] ", record.timestamp.elapsed().as_secs())?;
        }
        
        // Add custom prefix
        write!(w, "{} ", self.prefix)?;
        
        // Add type
        if self.opts.show_type {
            write!(w, "[{}] ", record.type_name)?;
        }
        
        // Add message
        if let Some(msg) = &record.message {
            write!(w, "{}", msg)?;
        }
        
        writeln!(w)?;
        Ok(())
    }
}

impl ReporterWithOptions for ConfigurableReporter {
    fn fmt_options(&self) -> &FormatOptions {
        &self.opts
    }
    
    fn fmt_options_mut(&mut self) -> &mut FormatOptions {
        &mut self.opts
    }
}
```

## Using FormatOptions

The `FormatOptions` struct provides common configuration options:

```rust
pub struct FormatOptions {
    pub date: bool,              // Include timestamps
    pub colors: bool,            // Enable color output
    pub compact: bool,           // Compact formatting
    pub columns: Option<usize>,  // Terminal width
    pub error_level: usize,      // Error chain depth limit
    pub unicode: bool,           // Use unicode characters
    pub show_tag: bool,          // Show tags
    pub show_type: bool,         // Show log type
    pub show_repetition: bool,   // Show repetition counts
    pub show_stack: bool,        // Show stack traces
    pub show_additional: bool,   // Show additional fields
    pub show_meta: bool,         // Show metadata
    pub force_simple_width: bool,// Force simple width calculation
}
```

## Handling LogRecord Fields

The `LogRecord` struct contains all information about a log entry:

```rust
pub struct LogRecord {
    pub timestamp: Instant,       // When the log was created
    pub level: LogLevel,          // Numeric log level
    pub type_name: String,        // Log type (info, warn, error, etc.)
    pub tag: Option<String>,      // Optional tag
    pub args: Vec<ArgValue>,      // Log arguments
    pub message: Option<String>,  // Pre-formatted message
    pub repetition_count: u32,    // Number of repeated occurrences
    pub is_raw: bool,            // Raw logging mode
    pub error_chain: Option<Vec<String>>, // Error cause chain
    pub additional: Option<HashMap<String, ArgValue>>, // Additional fields
    pub meta: Option<HashMap<String, ArgValue>>,       // Metadata
}
```

## Color and Styling

Use the `anstyle` crate for terminal colors:

```rust
use anstyle::{Color, Style, AnsiColor};

impl Reporter for ColoredReporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        let style = match record.type_name.as_str() {
            "error" => Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))),
            "warn" => Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
            "success" => Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            "info" => Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
            _ => Style::new(),
        };
        
        write!(w, "{}", style.render())?;
        write!(w, "[{}]", record.type_name)?;
        write!(w, "{}", style.render_reset())?;
        
        // ... rest of formatting
        Ok(())
    }
}
```

## Error Handling

Your reporter should handle errors gracefully:

```rust
impl Reporter for RobustReporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        // Handle potential errors in formatting
        let formatted = match self.format_message(record) {
            Ok(msg) => msg,
            Err(e) => {
                // Fallback formatting
                format!("[{}] <formatting error: {}>", record.type_name, e)
            }
        };
        
        writeln!(w, "{}", formatted)
    }
}
```

## Destination Control

Reporters receive a `Write` trait object. The logger determines the destination:

- By default, logs with level < 2 (fatal, error) go to stderr
- Other logs go to stdout
- You can override this by checking `record.level` and returning custom behavior

## Testing Custom Reporters

Use `MemoryReporter` or a custom buffer for testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_custom_reporter_format() {
        let reporter = CustomReporter::new();
        let record = LogRecord::new("info", None, vec!["test message".into()]);
        
        let mut buf = Vec::new();
        reporter.emit(&record, &mut buf).unwrap();
        
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("INFO"));
        assert!(output.contains("test message"));
    }
}
```

## Performance Considerations

1. **Avoid allocations**: Use write! macros directly instead of building strings
1. **Check options early**: Skip expensive formatting if options disable it
1. **Cache styles**: Don't recreate style objects for every log
1. **Lazy evaluation**: Only format error chains if show_stack is true

## Complete Example: CSV Reporter

```rust
use consola::*;
use std::io::{self, Write};

pub struct CsvReporter {
    opts: FormatOptions,
}

impl CsvReporter {
    pub fn new() -> Self {
        Self {
            opts: FormatOptions::default(),
        }
    }
}

impl Reporter for CsvReporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        // CSV format: timestamp,level,type,tag,message,repetitions
        
        // Timestamp
        if self.opts.date {
            write!(w, "{},", record.timestamp.elapsed().as_secs())?;
        } else {
            write!(w, ",")?;
        }
        
        // Level
        write!(w, "{},", record.level.0)?;
        
        // Type
        write!(w, "{},", escape_csv(&record.type_name))?;
        
        // Tag
        if let Some(tag) = &record.tag {
            write!(w, "{},", escape_csv(tag))?;
        } else {
            write!(w, ",")?;
        }
        
        // Message
        if let Some(msg) = &record.message {
            write!(w, "{},", escape_csv(msg))?;
        } else {
            write!(w, ",")?;
        }
        
        // Repetitions
        if self.opts.show_repetition && record.repetition_count > 1 {
            write!(w, "{}", record.repetition_count)?;
        }
        
        writeln!(w)?;
        Ok(())
    }
}

impl ReporterWithOptions for CsvReporter {
    fn fmt_options(&self) -> &FormatOptions {
        &self.opts
    }
    
    fn fmt_options_mut(&mut self) -> &mut FormatOptions {
        &mut self.opts
    }
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
```

## See Also

- [PROMPTS.md](PROMPTS.md) - Interactive prompts
- [INTEGRATION.md](INTEGRATION.md) - Integrating with log/tracing crates
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [API Documentation](https://docs.rs/consola) - Full API reference
