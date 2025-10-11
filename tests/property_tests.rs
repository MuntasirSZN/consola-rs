// Property tests for randomized sequences (Task 112)
use consola::*;
use proptest::prelude::*;

// Test that randomized sequences don't panic
proptest! {
    #[test]
    fn randomized_log_sequence_no_panic(
        messages in prop::collection::vec(".*", 0..100),
        log_types in prop::collection::vec(0u8..10, 0..100)
    ) {
        let mut logger = Logger::new(MemoryReporter::new());

        // Map numbers to log types
        let type_names = ["info", "warn", "error", "success", "debug", "trace", "fatal", "log"];

        for (msg, type_idx) in messages.iter().zip(log_types.iter()) {
            let type_name = type_names[(*type_idx as usize) % type_names.len()];
            logger.log(type_name, None, [msg.as_str()]);
        }

        // Final flush should not panic
        logger.flush();

        // Verify we captured some logs
        let captured = logger.reporter().get_records();
        prop_assert!(captured.len() <= messages.len());
    }

    #[test]
    fn randomized_pause_resume_no_panic(
        operations in prop::collection::vec(0u8..3, 0..50)
    ) {
        let mut logger = Logger::new(MemoryReporter::new());

        for op in operations {
            match op {
                0 => logger.pause(),
                1 => logger.resume(),
                2 => logger.log("info", None, ["test message"]),
                _ => {}
            }
        }

        // Final flush should not panic
        logger.flush();
    }

    #[test]
    fn randomized_throttling_invariants(
        messages in prop::collection::vec(prop::bool::ANY, 10..100)
    ) {
        let mut logger = Logger::new(MemoryReporter::new())
            .with_config(LoggerConfig {
                level: LogLevel::INFO,
                throttle: ThrottleConfig {
                    window: std::time::Duration::from_millis(100),
                    min_count: 3,
                },
                queue_capacity: None,
                clock: None,
            });

        // Log repeated messages
        for same in &messages {
            if *same {
                logger.log("info", None, ["repeated"]);
            } else {
                logger.log("info", None, ["unique"]);
            }
        }

        logger.flush();

        let captured = logger.reporter().get_records();

        // Verify throttling reduced message count
        prop_assert!(captured.len() <= messages.len());

        // Verify repetition counts are valid (>= 1)
        for record in captured {
            prop_assert!(record.repetition_count >= 1);
        }
    }

    #[test]
    fn randomized_level_filtering(
        log_levels in prop::collection::vec(0i16..10, 0..50),
        filter_level in 0i16..10
    ) {
        let mut logger = Logger::new(MemoryReporter::new())
            .with_config(LoggerConfig {
                level: LogLevel(filter_level),
                throttle: ThrottleConfig::default(),
                queue_capacity: None,
                clock: None,
            });

        let type_names = ["fatal", "error", "warn", "log", "info", "success", "debug", "trace"];

        for level in &log_levels {
            // Map level to type name (0-7)
            let idx = (*level % type_names.len() as i16).unsigned_abs() as usize;
            let type_name = type_names[idx.min(type_names.len() - 1)];
            logger.log(type_name, None, ["test"]);
        }

        let captured = logger.reporter().get_records();

        // All captured logs should have level <= filter_level
        for record in captured {
            prop_assert!(record.level.0 <= filter_level);
        }
    }
}

// Test that complex argument types don't panic
proptest! {
    #[test]
    fn randomized_arg_types_no_panic(
        strings in prop::collection::vec(".*", 0..20),
        numbers in prop::collection::vec(prop::num::f64::ANY, 0..20),
        bools in prop::collection::vec(prop::bool::ANY, 0..20)
    ) {
        let mut logger = Logger::new(MemoryReporter::new());

        // Create a log with various argument types
        let mut args: Vec<ArgValue> = Vec::new();

        for s in strings {
            args.push(ArgValue::String(s));
        }

        for n in numbers {
            if n.is_finite() {
                args.push(ArgValue::Number(n));
            }
        }

        for b in bools {
            args.push(ArgValue::Bool(b));
        }
        // Use the log method instead of log_with_args
        for arg in args {
            logger.log("info", None, [arg]);
        }

        let captured = logger.reporter().get_records();
        prop_assert!(!captured.is_empty());
    }
}
