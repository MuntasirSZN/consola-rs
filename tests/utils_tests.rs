use consola::*;

#[test]
fn utils_tree_formatter() {
    let formatter = TreeFormatter::new(3, true);
    let lines = vec!["item1".to_string(), "item2".to_string(), "item3".to_string()];
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
    let lines = vec!["item1".to_string(), "item2".to_string(), "item3".to_string()];
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
    let logger: Logger<BasicReporter> = LoggerBuilder::new()
        .with_level(LogLevel::WARN)
        .build();
    
    assert_eq!(logger.level(), LogLevel::WARN);
}