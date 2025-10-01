use consola::*;

#[test]
fn utils_tree_formatter() {
    let formatter = TreeFormatter::new(3, true);
    let lines = vec![
        "item1".to_string(),
        "item2".to_string(),
        "item3".to_string(),
    ];
    let formatted = formatter.format_lines(&lines, 0);

    assert_eq!(formatted.len(), 3);
    assert!(formatted[0].contains("├─"));
    assert!(formatted[2].contains("└─"));
}

#[test]
fn utils_tree_formatter_ascii() {
    let formatter = TreeFormatter::new(3, false);
    let lines = vec!["item1".to_string(), "item2".to_string()];
    let formatted = formatter.format_lines(&lines, 0);

    assert_eq!(formatted.len(), 2);
    assert!(formatted[0].contains("|-"));
    assert!(formatted[1].contains("`-"));
}

#[test]
fn utils_tree_formatter_depth_limit() {
    let formatter = TreeFormatter::new(1, true);
    let lines = vec![
        "item1".to_string(),
        "item2".to_string(),
        "item3".to_string(),
    ];
    let formatted = formatter.format_lines(&lines, 1);

    assert_eq!(formatted.len(), 2); // Should truncate with ellipsis
    assert!(formatted[1].contains("..."));
}

#[test]
fn utils_box_builder_unicode() {
    let builder = BoxBuilder::new(true).with_width(20);
    let content = vec!["Hello".to_string(), "World".to_string()];
    let boxed = builder.build("Title", &content);

    assert!(!boxed.is_empty());
    assert!(boxed[0].contains("┌"));
    assert!(boxed[0].contains("┐"));
    assert!(boxed.last().unwrap().contains("└"));
    assert!(boxed.last().unwrap().contains("┘"));
}

#[test]
fn utils_box_builder_ascii() {
    let builder = BoxBuilder::new(false).with_width(15);
    let content = vec!["Test".to_string()];
    let boxed = builder.build("", &content);

    assert!(!boxed.is_empty());
    assert!(boxed[0].contains("+"));
    assert!(boxed[0].contains("-"));
}

#[test]
fn utils_align_text() {
    assert_eq!(align_text("hello", 10, Alignment::Left), "hello     ");
    assert_eq!(align_text("hello", 10, Alignment::Right), "     hello");
    assert_eq!(align_text("hello", 10, Alignment::Center), "  hello   ");
}

#[test]
fn raw_logging_methods() {
    let mut logger = Logger::new(BasicReporter::default());

    // Test per-type raw methods
    logger.info_raw("info message");
    logger.warn_raw("warning message");
    logger.error_raw("error message");
    logger.debug_raw("debug message");
    logger.trace_raw("trace message");
    logger.success_raw("success message");
    logger.fail_raw("fail message");
    logger.fatal_raw("fatal message");

    // Test generic raw method
    logger.log_type_raw("custom", "custom message");

    logger.flush();
}

#[test]
fn logger_builder_basic() {
    let logger: Logger<BasicReporter> = LoggerBuilder::new().with_level(LogLevel::WARN).build();

    assert_eq!(logger.level(), LogLevel::WARN);
}

#[test]
fn error_stack_parser_basic() {
    let stack = "    at main (file:///home/user/project/src/main.rs:10:5)\n    at init (/home/user/project/src/lib.rs:20:10)";
    let parsed = consola::parse_error_stack(stack);

    assert_eq!(parsed.len(), 2);
    assert!(parsed[0].contains("main.rs"));
    assert!(parsed[1].contains("lib.rs"));
    // The parser removes file:// prefix from paths
    // Check that at least the main function processing works
}

#[test]
fn test_sink_basic() {
    let sink = consola::TestSink::new();
    sink.write_all(b"hello").unwrap();
    sink.write_all(b" world").unwrap();

    assert_eq!(sink.contents(), "hello world");

    sink.clear();
    assert_eq!(sink.contents(), "");
}

#[cfg(feature = "color")]
#[test]
fn style_helpers() {
    let colored = consola::style::colored("test", consola::style::info_color());
    assert!(colored.contains("test"));
    assert!(colored.len() > 4); // Should have ANSI codes

    let dim = consola::style::dim("dim text");
    assert!(dim.contains("dim text"));
}

#[cfg(not(feature = "color"))]
#[test]
fn style_helpers_no_color() {
    let colored = consola::style::colored("test", ());
    assert_eq!(colored, "test");

    let dim = consola::style::dim("dim text");
    assert_eq!(dim, "dim text");
}

#[test]
fn fancy_reporter_badge_formatting() {
    let mut logger = Logger::new(FancyReporter::default());
    // This test mainly ensures badge formatting doesn't panic
    logger.info("test badge formatting");
    logger.error("test error badge");
    logger.success("test success badge");
    logger.flush();
}

#[test]
fn fancy_reporter_enhanced_colors() {
    let fancy = FancyReporter::default();
    // Test that different log types get different icon colors
    let info_record = LogRecord::new("info", None, vec!["info test".into()]);
    let error_record = LogRecord::new("error", None, vec!["error test".into()]);

    // Just verify the records can be processed without panic
    let mut buffer = Vec::new();
    fancy.emit(&info_record, &mut buffer).unwrap();
    fancy.emit(&error_record, &mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("info test"));
    assert!(output.contains("error test"));
}

#[test]
fn pause_resume_order_preservation() {
    let mut logger = Logger::new(BasicReporter::default());

    logger.pause();
    logger.info("first message");
    logger.warn("second message");
    logger.error("third message");

    logger.resume();
    logger.flush();

    // Messages should be in correct order (though we can't easily capture them with the current setup)
    // This test mainly ensures no panic occurs during pause/resume cycle
}

#[test]
fn throttle_boundary_reset_on_resume() {
    let mut logger = Logger::new(BasicReporter::default());

    // Send same message twice
    logger.info("duplicate message");
    logger.info("duplicate message");

    logger.pause();
    logger.info("duplicate message"); // This should be queued

    logger.resume(); // Should reset throttle boundaries and process queued message
    logger.flush();

    // Test completes if no panic occurs
}

#[test]
fn force_simple_width_effect() {
    use consola::{compute_line_width, FormatOptions, Segment};

    // Test with unicode characters that have different display width
    let segments = vec![Segment {
        text: "你好世界".to_string(), // Chinese characters
        style: None,
    }];

    // With force_simple_width = false (default), uses unicode width
    let opts = FormatOptions {
        force_simple_width: false,
        ..Default::default()
    };
    let unicode_width = compute_line_width(&segments, &opts);

    // With force_simple_width = true, uses character count
    let opts = FormatOptions {
        force_simple_width: true,
        ..Default::default()
    };
    let simple_width = compute_line_width(&segments, &opts);

    // Character count (4) should be less than unicode display width (8)
    // But if fancy feature is disabled, both will be the same
    #[cfg(feature = "fancy")]
    {
        assert_eq!(simple_width, 4); // char count
        assert_eq!(unicode_width, 8); // unicode display width
    }
    #[cfg(not(feature = "fancy"))]
    {
        assert_eq!(simple_width, 4);
        assert_eq!(unicode_width, 4);
    }
}
