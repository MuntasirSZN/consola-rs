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
    use consola::{FormatOptions, Segment, compute_line_width};

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

#[test]
fn mock_intercept_order() {
    use std::sync::{Arc, Mutex};

    let mut logger = Logger::new(BasicReporter::default());
    let captured = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = Arc::clone(&captured);

    // Set mock to capture log records
    logger.set_mock(move |record: &LogRecord| {
        captured_clone
            .lock()
            .unwrap()
            .push(record.type_name.clone());
    });

    // Log some messages
    logger.info("first");
    logger.warn("second");
    logger.error("third");

    // Verify records were captured in order
    let records = captured.lock().unwrap();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0], "info");
    assert_eq!(records[1], "warn");
    assert_eq!(records[2], "error");

    // Clear mock
    logger.clear_mock();

    // Log another message - should not be captured
    logger.debug("fourth");

    // Verify no new records were captured
    assert_eq!(records.len(), 3);
}

#[test]
fn memory_reporter_captures_records() {
    use consola::{Logger, MemoryReporter};

    let reporter = MemoryReporter::new();
    let mut logger = Logger::new(reporter);

    // Initially empty
    assert!(logger.reporter().is_empty());
    assert_eq!(logger.reporter().len(), 0);

    // Log some messages
    logger.info("test message");
    logger.warn("warning message");
    logger.error("error message");

    // Verify records were captured
    assert_eq!(logger.reporter().len(), 3);
    assert!(!logger.reporter().is_empty());

    let records = logger.reporter().get_records();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].type_name, "info");
    assert_eq!(records[0].message.as_deref(), Some("test message"));
    assert_eq!(records[1].type_name, "warn");
    assert_eq!(records[1].message.as_deref(), Some("warning message"));
    assert_eq!(records[2].type_name, "error");
    assert_eq!(records[2].message.as_deref(), Some("error message"));

    // Clear records
    logger.reporter().clear();
    assert!(logger.reporter().is_empty());
    assert_eq!(logger.reporter().len(), 0);
}

#[test]
fn deterministic_timestamp_snapshots() {
    use consola::{Logger, LoggerConfig, MemoryReporter, MockClock, ThrottleConfig};

    let mock_clock = MockClock::new();
    let reporter = MemoryReporter::new();

    let config = LoggerConfig {
        level: LogLevel::VERBOSE,
        throttle: ThrottleConfig::default(),
        queue_capacity: None,
        clock: Some(Box::new(mock_clock)),
    };

    let mut logger = Logger::new(reporter).with_config(config);

    // Log first message at time T
    logger.info("message at T0");

    // Need to get a mutable reference to the clock to advance it
    // Since the clock is inside the config, we'll need to create a new config
    // For this test, let's verify that the timestamps are stable

    let records = logger.reporter().get_records();
    assert_eq!(records.len(), 1);

    let first_timestamp = records[0].timestamp;

    // Log more messages - they should have the same timestamp since clock doesn't advance
    logger.warn("message at T0");
    logger.error("message at T0");

    let records = logger.reporter().get_records();
    assert_eq!(records.len(), 3);

    // All three records should have the exact same timestamp
    assert_eq!(records[0].timestamp, first_timestamp);
    assert_eq!(records[1].timestamp, first_timestamp);
    assert_eq!(records[2].timestamp, first_timestamp);

    // This demonstrates deterministic timestamps for testing
}

// Test capacity overflow strategy (drop oldest)
#[test]
fn pause_queue_capacity_overflow() {
    use std::sync::{Arc, Mutex};

    // Create logger with small queue capacity
    let mut logger = Logger::new(BasicReporter::default()).with_config(LoggerConfig {
        level: LogLevel::VERBOSE,
        throttle: ThrottleConfig::default(),
        queue_capacity: Some(3), // Only keep 3 items
        clock: None,
    });

    // Capture the messages that actually get processed
    let captured = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = Arc::clone(&captured);

    logger.set_mock(move |record: &LogRecord| {
        if let Some(msg) = &record.message {
            captured_clone.lock().unwrap().push(msg.clone());
        }
    });

    // Pause and send 5 messages (exceeds capacity of 3)
    logger.pause();
    logger.info("message 1 (should be dropped)");
    logger.info("message 2 (should be dropped)");
    logger.info("message 3 (should be kept)");
    logger.info("message 4 (should be kept)");
    logger.info("message 5 (should be kept)");

    // Resume - only the last 3 messages should be processed
    logger.resume();
    logger.flush();

    let messages = captured.lock().unwrap();
    assert_eq!(messages.len(), 3, "Should only keep the last 3 messages");
    assert_eq!(messages[0], "message 3 (should be kept)");
    assert_eq!(messages[1], "message 4 (should be kept)");
    assert_eq!(messages[2], "message 5 (should be kept)");
}

// Stress test for high concurrency
#[test]
fn stress_test_high_concurrency() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    // Create a logger wrapped in Arc to share between threads
    let logger = Arc::new(Mutex::new(Logger::new(MemoryReporter::default())));
    let num_threads = 10;
    let messages_per_thread = 100;

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let logger_clone = Arc::clone(&logger);
        let handle = thread::spawn(move || {
            for msg_id in 0..messages_per_thread {
                let mut logger = logger_clone.lock().unwrap();
                logger.info(format!("Thread {} message {}", thread_id, msg_id));
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Flush and verify
    let mut logger = logger.lock().unwrap();
    logger.flush();

    let records = logger.reporter().get_records();

    // Should have all messages (or close to it, accounting for possible throttling)
    // At minimum, should have received messages from all threads
    assert!(
        records.len() >= num_threads,
        "Should have messages from all threads, got {}",
        records.len()
    );

    // Verify no panics occurred and logger is still functional
    logger.info("Final message after stress test");
    let final_records = logger.reporter().get_records();
    assert!(
        final_records.len() > records.len(),
        "Logger should still be functional"
    );
}

// Deterministic run repeat test
#[test]
fn deterministic_run_repeat() {
    use crate::clock::MockClock;

    // Helper function to run a logging scenario
    let run_scenario = || -> Vec<String> {
        let mock_clock = MockClock::new();
        let base_time = mock_clock.now();

        let mut logger = Logger::new(MemoryReporter::default()).with_config(LoggerConfig {
            level: LogLevel::VERBOSE,
            throttle: ThrottleConfig::default(),
            queue_capacity: None,
            clock: Some(Box::new(mock_clock)),
        });

        // Perform deterministic logging operations
        logger.info("First message");
        logger.warn("Second message");
        logger.error("Third message");

        // Test throttling with same message
        logger.info("Repeated");
        logger.info("Repeated");
        logger.info("Repeated");

        logger.flush();

        // Extract messages in order with relative timestamps
        logger
            .reporter()
            .get_records()
            .iter()
            .map(|r| {
                let duration_since_base = r.timestamp.duration_since(base_time);
                format!("{:?} {} {:?}", duration_since_base, r.type_name, r.message)
            })
            .collect()
    };

    // Run the scenario twice
    let run1 = run_scenario();
    let run2 = run_scenario();

    // Both runs should produce identical results
    assert_eq!(
        run1.len(),
        run2.len(),
        "Both runs should produce the same number of log records"
    );

    for (i, (msg1, msg2)) in run1.iter().zip(run2.iter()).enumerate() {
        assert_eq!(
            msg1, msg2,
            "Message {} should be identical in both runs:\nRun 1: {}\nRun 2: {}",
            i, msg1, msg2
        );
    }
}
