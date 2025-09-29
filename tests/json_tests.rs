#[cfg(feature = "json")]
use consola::*;

#[cfg(feature = "json")]
#[test]
fn json_basic_record() {
    let reporter = JsonReporter::new();
    let mut logger = JsonLogger::new(reporter);
    logger.opts_mut().date = false; // deterministic

    // Test basic log
    logger.log("info", None, ["hello world"]);
    logger.flush();

    // Test with tag
    logger.log("error", Some("test".to_string()), ["failed"]);
    logger.flush();
}

#[cfg(feature = "json")]
#[test]
fn json_error_chain_serialization() {
    use anyhow::anyhow;

    let reporter = JsonReporter::new();
    let mut logger = JsonLogger::new(reporter);
    logger.opts_mut().date = false;

    let err = anyhow!("root cause").context("intermediate").context("top");
    let mut record = LogRecord::new("error", None, vec!["processing failed".into()]);
    let err_ref: &(dyn std::error::Error + 'static) = err.as_ref();
    record = record.attach_dyn_error(err_ref);

    // Verify error chain is present
    assert!(record.error_chain.is_some());
    assert!(!record.error_chain.as_ref().unwrap().is_empty());
}

#[cfg(feature = "json")]
#[test]
fn json_repetition_handling() {
    let reporter = JsonReporter::new();
    let mut logger = JsonLogger::new(reporter);
    logger.opts_mut().date = false;

    // Create repeated records
    for _ in 0..3 {
        logger.log("info", None, ["repeated message"]);
    }
    logger.flush();
}

#[cfg(feature = "json")]
#[test]
fn metadata_merge_precedence() {
    // Test default merging behavior
    let defaults = RecordDefaults {
        tag: Some("default-tag".to_string()),
        additional: Some(vec!["default-arg".into()]),
        meta: Some(vec![
            ("default-key".to_string(), "default-value".into()),
            ("shared-key".to_string(), "default-shared".into()),
        ]),
    };

    let record = LogRecord::new("info", None, vec!["test message".into()])
        .with_additional(vec!["record-arg".into()])
        .with_meta(vec![
            ("record-key".to_string(), "record-value".into()),
            ("shared-key".to_string(), "record-shared".into()), // This should win
        ])
        .merge_defaults(&defaults);

    // Should have tag from defaults
    assert_eq!(record.tag, Some("default-tag".to_string()));

    // Should have merged additional args (defaults first)
    assert!(record.additional.is_some());
    let additional = record.additional.unwrap();
    assert_eq!(additional.len(), 2);
    assert_eq!(additional[0], "default-arg".into());

    // Should have merged meta with record values taking precedence
    assert!(record.meta.is_some());
    let meta = record.meta.unwrap();
    let meta_map: std::collections::HashMap<String, ArgValue> = meta.into_iter().collect();
    assert_eq!(meta_map.get("shared-key"), Some(&"record-shared".into()));
    assert_eq!(meta_map.get("default-key"), Some(&"default-value".into()));
    assert_eq!(meta_map.get("record-key"), Some(&"record-value".into()));
}
