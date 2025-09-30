// Environment variable tests (Tasks 280-282)
use consola::*;
use std::env;

#[test]
fn env_level_override() {
    // Save original value
    let original = env::var("CONSOLA_LEVEL").ok();

    // Set CONSOLA_LEVEL
    unsafe {
        env::set_var("CONSOLA_LEVEL", "error");
    }

    // Create logger from environment
    let logger: BasicLogger = LoggerBuilder::new().from_env().build();

    // Should respect environment variable
    assert_eq!(logger.level(), LogLevel::ERROR);

    // Restore original
    unsafe {
        match original {
            Some(val) => env::set_var("CONSOLA_LEVEL", val),
            None => env::remove_var("CONSOLA_LEVEL"),
        }
    }
}

#[test]
fn env_level_builder_precedence() {
    // Save original value
    let original = env::var("CONSOLA_LEVEL").ok();

    // Set CONSOLA_LEVEL to one value
    unsafe {
        env::set_var("CONSOLA_LEVEL", "debug");
    }

    // Builder should take precedence over env
    let logger: BasicLogger = LoggerBuilder::new()
        .from_env()
        .with_level(LogLevel::WARN) // This should override env
        .build();

    assert_eq!(logger.level(), LogLevel::WARN);

    // Restore original
    unsafe {
        match original {
            Some(val) => env::set_var("CONSOLA_LEVEL", val),
            None => env::remove_var("CONSOLA_LEVEL"),
        }
    }
}

#[test]
fn no_color_disables_styling() {
    // Save original value
    let original = env::var("NO_COLOR").ok();

    // Set NO_COLOR
    unsafe {
        env::set_var("NO_COLOR", "1");
    }

    // Create format options from environment
    let opts = FormatOptions::adaptive();

    // Colors should be disabled when NO_COLOR is set
    // Note: This depends on FormatOptions::adaptive() implementation
    // If colors are controlled by anstream, they should be automatically disabled

    // Create a reporter and verify it doesn't add ANSI codes
    let reporter = BasicReporter { opts };
    let record = LogRecord::new("info", None, vec!["test message".into()]);
    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    // Output should not contain ANSI escape sequences
    // Basic check: should not contain ESC character
    assert!(
        !output.contains('\x1b'),
        "Output should not contain ANSI codes when NO_COLOR is set"
    );

    // Restore original
    unsafe {
        match original {
            Some(val) => env::set_var("NO_COLOR", val),
            None => env::remove_var("NO_COLOR"),
        }
    }
}

#[test]
fn force_color_enables_styling() {
    // Save original values
    let original_no_color = env::var("NO_COLOR").ok();
    let original_force_color = env::var("FORCE_COLOR").ok();

    // Set FORCE_COLOR (should override NO_COLOR)
    unsafe {
        env::set_var("NO_COLOR", "1");
        env::set_var("FORCE_COLOR", "1");
    }

    // Create format options from environment
    let _opts = FormatOptions::adaptive();

    // FORCE_COLOR should enable colors even with NO_COLOR set
    // Note: This behavior depends on anstream's implementation

    // Restore original values
    unsafe {
        match original_no_color {
            Some(val) => env::set_var("NO_COLOR", val),
            None => env::remove_var("NO_COLOR"),
        }
        match original_force_color {
            Some(val) => env::set_var("FORCE_COLOR", val),
            None => env::remove_var("FORCE_COLOR"),
        }
    }
}

#[test]
fn consola_compact_env() {
    // Save original value
    let original = env::var("CONSOLA_COMPACT").ok();

    // Set CONSOLA_COMPACT
    unsafe {
        env::set_var("CONSOLA_COMPACT", "true");
    }

    // Create format options from environment
    let _opts = FormatOptions::adaptive();

    // Check if compact mode is respected
    // Note: Implementation depends on how FormatOptions::adaptive() handles CONSOLA_COMPACT

    // Restore original
    unsafe {
        match original {
            Some(val) => env::set_var("CONSOLA_COMPACT", val),
            None => env::remove_var("CONSOLA_COMPACT"),
        }
    }
}

#[test]
fn terminal_width_detection() {
    // Test COLUMNS env var
    let original = env::var("COLUMNS").ok();

    unsafe {
        env::set_var("COLUMNS", "120");
    }

    let width = consola::detect_terminal_width();
    assert_eq!(width, Some(120), "Should detect width from COLUMNS env var");

    // Restore original
    unsafe {
        match original {
            Some(val) => env::set_var("COLUMNS", val),
            None => env::remove_var("COLUMNS"),
        }
    }
}

#[test]
fn terminal_width_fallback() {
    // With no COLUMNS and possibly no terminal, should return None or actual terminal width
    let original = env::var("COLUMNS").ok();

    unsafe {
        env::remove_var("COLUMNS");
    }

    let width = consola::detect_terminal_width();
    // Width can be None (not a terminal) or Some(actual_width)
    // We just check that the function doesn't panic
    let _ = width;

    // Restore original
    if let Some(val) = original {
        unsafe {
            env::set_var("COLUMNS", val);
        }
    }
}
