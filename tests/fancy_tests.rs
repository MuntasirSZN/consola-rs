use consola::*;

#[allow(unused)]
fn render_basic<F: FnOnce(&mut FancyLogger)>(f: F) -> String {
    let mut logger = FancyLogger::new(FancyReporter {
        opts: FormatOptions {
            unicode: true,
            ..FormatOptions::default()
        },
    });
    logger.opts_mut().date = false; // deterministic
    f(&mut logger);
    // No direct capture; for test we will create a MemoryReporter in future. For now we just ensure no panic.
    String::new()
}

#[test]
fn fancy_icon_ascii_fallback() {
    let reporter = FancyReporter {
        opts: FormatOptions {
            unicode: false,
            date: false,
            ..FormatOptions::default()
        },
    };
    let mut logger = FancyLogger::new(reporter);
    logger.log("info", None, ["hello"]);
    logger.flush();
}

#[test]
fn fancy_error_chain_depth_limit() {
    use anyhow::anyhow;
    let err = anyhow!("root cause").context("intermediate").context("top");
    let mut record = LogRecord::new("error", None, vec!["processing failed".into()]);
    let err_ref: &(dyn std::error::Error + 'static) = err.as_ref();
    // Use concrete trait object via helper method that takes dyn reference
    record = record.attach_dyn_error(err_ref);
    // ensure chain stored
    assert!(record.error_chain.as_ref().unwrap().len() >= 3);
}
