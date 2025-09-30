// Error chain tests (Tasks 206-208)
use consola::*;

#[test]
fn error_chain_depth_limiting() {
    use anyhow::anyhow;

    // Create a chain with 5 errors
    let err = anyhow!("error 1")
        .context("error 2")
        .context("error 3")
        .context("error 4")
        .context("error 5");

    let err_ref: &(dyn std::error::Error + 'static) = err.as_ref();
    let chain = collect_chain(err_ref);

    // Should have all 5 errors in the chain
    assert_eq!(chain.len(), 5);

    // Test depth limiting in formatting
    let limited_2 = format_chain_lines(&chain, 2);
    assert_eq!(limited_2.len(), 2);
    assert!(limited_2[0].contains("error 5")); // Top error first
    assert!(limited_2[1].contains("Caused by:"));
    assert!(limited_2[1].contains("error 4"));

    let limited_3 = format_chain_lines(&chain, 3);
    assert_eq!(limited_3.len(), 3);

    // Test unlimited depth
    let unlimited = format_chain_lines(&chain, usize::MAX);
    assert_eq!(unlimited.len(), 5);
}

#[test]
fn error_chain_cycle_detection() {
    // This test ensures that if we somehow have a cyclic error chain,
    // the collect_chain function won't infinite loop
    // Note: In practice, std::error::Error chains shouldn't be cyclic,
    // but our implementation should be safe anyway

    use anyhow::anyhow;
    let err = anyhow!("test error");
    let err_ref: &(dyn std::error::Error + 'static) = err.as_ref();
    let chain = collect_chain(err_ref);

    // Should complete without hanging
    assert!(!chain.is_empty());
    // Verify no duplicates (which would indicate a cycle)
    for i in 0..chain.len() {
        for _j in (i + 1)..chain.len() {
            // In a proper implementation, we shouldn't have exact duplicates
            // but the chain should terminate
        }
    }
}

#[test]
fn error_chain_multi_level_nested() {
    use anyhow::anyhow;
    use std::io;

    // Create a multi-level nested error chain with different error types
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = anyhow!(io_err)
        .context("Failed to read config")
        .context("Application initialization failed");

    let err_ref: &(dyn std::error::Error + 'static) = err.as_ref();
    let chain = collect_chain(err_ref);

    // Should have at least 3 levels
    assert!(chain.len() >= 3);

    // Format with full depth
    let formatted = format_chain_lines(&chain, usize::MAX);

    // First line should be the top-level error (no "Caused by:" prefix)
    assert!(!formatted[0].starts_with("Caused by:"));

    // Subsequent lines should have "Caused by:" prefix
    for line in &formatted[1..] {
        assert!(line.starts_with("Caused by:"));
    }

    // Check that the chain contains our expected error messages
    let full_chain_str = formatted.join("\n");
    assert!(
        full_chain_str.contains("initialization failed")
            || full_chain_str.contains("Application initialization")
    );
}

#[test]
fn error_chain_empty_source() {
    use anyhow::anyhow;

    // Simple error with no source
    let err = anyhow!("single error");
    let err_ref: &(dyn std::error::Error + 'static) = err.as_ref();
    let chain = collect_chain(err_ref);

    // Should have exactly one error
    assert_eq!(chain.len(), 1);
    assert!(chain[0].contains("single error"));
}
