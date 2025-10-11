use consola::*;
use strip_ansi_escapes::strip;

fn render_fancy_line(type_name: &str, msg: &str) -> (String, String) {
    let reporter = FancyReporter {
        opts: FormatOptions {
            date: false,
            ..FormatOptions::default()
        },
    };
    let record = LogRecord::new(type_name, None, vec![msg.into()]);
    // enable stack / error chain display explicitly for error types if any chain present
    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let colored = String::from_utf8(buf).unwrap();
    let plain_bytes = strip(colored.as_bytes());
    let plain = String::from_utf8(plain_bytes).unwrap();
    (colored, plain)
}

fn render_basic_line(type_name: &str, msg: &str) -> (String, String) {
    let reporter = BasicReporter {
        opts: FormatOptions {
            date: false,
            colors: true,
            ..FormatOptions::default()
        },
    };
    let record = LogRecord::new(type_name, None, vec![msg.into()]);
    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let colored = String::from_utf8(buf).unwrap();
    let plain_bytes = strip(colored.as_bytes());
    let plain = String::from_utf8(plain_bytes).unwrap();
    (colored, plain)
}

#[test]
fn snapshot_fancy_info_basic() {
    let (colored, plain) = render_fancy_line("info", "hello world");
    insta::assert_snapshot!("fancy_info_basic_colored", colored);
    insta::assert_snapshot!("fancy_info_basic_plain", plain);
}

#[test]
fn snapshot_fancy_error_chain_limited() {
    use anyhow::anyhow;
    let reporter = FancyReporter {
        opts: FormatOptions {
            date: false,
            show_stack: true,
            error_level: 2,
            ..FormatOptions::default()
        },
    };
    let err = anyhow!("root cause")
        .context("middle layer")
        .context("top context");
    let err_ref: &(dyn std::error::Error + 'static) = err.as_ref();
    let record =
        LogRecord::new("error", None, vec!["processing failed".into()]).attach_dyn_error(err_ref);
    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let colored = String::from_utf8(buf).unwrap();
    let plain_bytes = strip(colored.as_bytes());
    let plain = String::from_utf8(plain_bytes).unwrap();
    insta::assert_snapshot!("fancy_error_chain_colored", colored);
    insta::assert_snapshot!("fancy_error_chain_plain", plain);
}

// BasicReporter Tests (Task 159-161)
#[test]
fn snapshot_basic_single_line() {
    let (colored, plain) = render_basic_line("info", "Simple log message");
    insta::assert_snapshot!("basic_info_colored", colored);
    insta::assert_snapshot!("basic_info_plain", plain);
}

#[test]
fn snapshot_basic_with_box() {
    let reporter = BasicReporter {
        opts: FormatOptions {
            date: false,
            colors: true,
            ..FormatOptions::default()
        },
    };
    // Create a box record with multiple lines
    let box_builder = BoxBuilder::new(true).with_width(40);
    let content = vec![
        "Line 1".to_string(),
        "Line 2".to_string(),
        "Line 3".to_string(),
    ];
    let boxed_lines = box_builder.build("Box Title", &content);

    // Build a record with box content as additional args
    let mut record = LogRecord::new("box", None, vec!["Box content:".into()]);
    for line in boxed_lines {
        record.args.push(ArgValue::String(line));
    }

    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let colored = String::from_utf8(buf).unwrap();
    let plain_bytes = strip(colored.as_bytes());
    let plain = String::from_utf8(plain_bytes).unwrap();
    insta::assert_snapshot!("basic_box_colored", colored);
    insta::assert_snapshot!("basic_box_plain", plain);
}

#[test]
fn snapshot_basic_error_chain() {
    use anyhow::anyhow;
    let reporter = BasicReporter {
        opts: FormatOptions {
            date: false,
            colors: true,
            show_stack: true,
            error_level: 3,
            ..FormatOptions::default()
        },
    };
    let err = anyhow!("lowest level error")
        .context("middle error")
        .context("top error");
    let err_ref: &(dyn std::error::Error + 'static) = err.as_ref();
    let record =
        LogRecord::new("error", None, vec!["Error occurred".into()]).attach_dyn_error(err_ref);

    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let colored = String::from_utf8(buf).unwrap();
    let plain_bytes = strip(colored.as_bytes());
    let plain = String::from_utf8(plain_bytes).unwrap();
    insta::assert_snapshot!("basic_error_chain_colored", colored);
    insta::assert_snapshot!("basic_error_chain_plain", plain);
}

// FancyReporter Additional Tests (Tasks 176-178)
#[test]
fn snapshot_fancy_unicode_fallback() {
    let reporter = FancyReporter {
        opts: FormatOptions {
            date: false,
            unicode: false, // Force ASCII fallback
            ..FormatOptions::default()
        },
    };
    let record = LogRecord::new("info", None, vec!["ASCII fallback test".into()]);
    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let plain_bytes = strip(output.as_bytes());
    let plain = String::from_utf8(plain_bytes).unwrap();
    insta::assert_snapshot!("fancy_unicode_fallback", plain);
}

#[test]
fn snapshot_fancy_repetition_count() {
    let reporter = FancyReporter {
        opts: FormatOptions {
            date: false,
            ..FormatOptions::default()
        },
    };
    let mut record = LogRecord::new("warn", None, vec!["Repeated warning".into()]);
    record.repetition_count = 5; // Simulate throttling repetition

    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let colored = String::from_utf8(buf).unwrap();
    let plain_bytes = strip(colored.as_bytes());
    let plain = String::from_utf8(plain_bytes).unwrap();
    insta::assert_snapshot!("fancy_repetition_colored", colored);
    insta::assert_snapshot!("fancy_repetition_plain", plain);
}

// Test BasicReporter with date enabled (Task 48)
#[test]
fn snapshot_basic_with_date() {
    let reporter = BasicReporter {
        opts: FormatOptions {
            date: true, // Enable date
            colors: true,
            ..FormatOptions::default()
        },
    };
    let record = LogRecord::new("info", None, vec!["Message with timestamp".into()]);
    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    // Verify date is present (timestamp format includes 'T' between date and time)
    assert!(
        output.contains('T'),
        "Output should contain timestamp with ISO8601 format"
    );
    assert!(output.contains("[info]"), "Output should contain log type");
    assert!(
        output.contains("Message with timestamp"),
        "Output should contain message"
    );
}

// Test FancyReporter with Box (Task 53)
#[test]
fn snapshot_fancy_with_box() {
    let reporter = FancyReporter {
        opts: FormatOptions {
            date: false,
            colors: true,
            ..FormatOptions::default()
        },
    };

    // Create a box record
    let mut record = LogRecord::new("box", None, vec!["Box Title".into()]);
    record.args.push("Line 1 content".into());
    record.args.push("Line 2 content".into());
    record.args.push("Line 3 content".into());

    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let colored = String::from_utf8(buf).unwrap();
    let plain_bytes = strip(colored.as_bytes());
    let plain = String::from_utf8(plain_bytes).unwrap();

    // Verify box structure is present
    assert!(plain.contains("Box Title"), "Should contain box title");
    assert!(plain.contains("Line 1"), "Should contain first line");
    assert!(plain.contains("Line 2"), "Should contain second line");

    insta::assert_snapshot!("fancy_box_colored", colored);
    insta::assert_snapshot!("fancy_box_plain", plain);
}

// Test BasicReporter with raw logging (Task 111)
#[test]
fn snapshot_basic_raw() {
    let reporter = BasicReporter {
        opts: FormatOptions {
            date: false,
            colors: false,
            ..FormatOptions::default()
        },
    };
    let mut record = LogRecord::new("info", None, vec!["Raw log message".into()]);
    record.is_raw = true;

    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("basic_raw", output);
}

// Test FancyReporter with raw logging (Task 111)
#[test]
fn snapshot_fancy_raw() {
    let reporter = FancyReporter {
        opts: FormatOptions {
            date: false,
            colors: false,
            ..FormatOptions::default()
        },
    };
    let mut record = LogRecord::new("info", None, vec!["Raw log message".into()]);
    record.is_raw = true;

    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    insta::assert_snapshot!("fancy_raw", output);
}

// Test various log types with BasicReporter (Task 111)
#[test]
fn snapshot_basic_log_types() {
    let reporter = BasicReporter {
        opts: FormatOptions {
            date: false,
            colors: false,
            ..FormatOptions::default()
        },
    };

    let types = vec!["success", "warn", "error", "debug", "trace"];
    let mut outputs = Vec::new();

    for type_name in types {
        let record = LogRecord::new(type_name, None, vec![format!("{} message", type_name).into()]);
        let mut buf: Vec<u8> = Vec::new();
        reporter.emit(&record, &mut buf).unwrap();
        outputs.push(String::from_utf8(buf).unwrap());
    }

    let combined = outputs.join("");
    insta::assert_snapshot!("basic_log_types", combined);
}

// Test various log types with FancyReporter (Task 111)
#[test]
fn snapshot_fancy_log_types() {
    let reporter = FancyReporter {
        opts: FormatOptions {
            date: false,
            colors: false,
            ..FormatOptions::default()
        },
    };

    let types = vec!["success", "warn", "error", "debug", "trace"];
    let mut outputs = Vec::new();

    for type_name in types {
        let record = LogRecord::new(type_name, None, vec![format!("{} message", type_name).into()]);
        let mut buf: Vec<u8> = Vec::new();
        reporter.emit(&record, &mut buf).unwrap();
        outputs.push(String::from_utf8(buf).unwrap());
    }

    let combined = outputs.join("");
    insta::assert_snapshot!("fancy_log_types", combined);
}
