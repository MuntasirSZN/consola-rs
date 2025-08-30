use consola::*;
use std::time::{Duration, Instant};

// Helper to build a record with custom timestamp
fn rec_at(ts: Instant, msg: &str) -> LogRecord {
    LogRecord::new_with_timestamp("info", None, vec![msg.into()], ts)
}

#[test]
fn coalesce_below_vs_at_threshold() {
    let mut t = Throttler::new(ThrottleConfig {
        window: Duration::from_millis(200),
        min_count: 3,
    });
    let base = Instant::now();
    let mut emitted = Vec::new();
    t.on_record(rec_at(base, "a"), |r| emitted.push(r.clone())); // first emit
    t.on_record(rec_at(base + Duration::from_millis(10), "a"), |r| {
        emitted.push(r.clone())
    }); // suppressed
    assert_eq!(emitted.len(), 1);
    t.on_record(rec_at(base + Duration::from_millis(20), "a"), |r| {
        emitted.push(r.clone())
    }); // aggregated
    assert_eq!(emitted.len(), 2);
    assert_eq!(emitted[0].repetition_count, 1);
    assert_eq!(emitted[1].repetition_count, 3); // aggregated count
}

#[test]
fn manual_flush_releases_suppressed() {
    let mut t = Throttler::new(ThrottleConfig {
        window: Duration::from_millis(500),
        min_count: 5,
    });
    let base = Instant::now();
    let mut emitted = Vec::new();
    t.on_record(rec_at(base, "x"), |r| emitted.push(r.clone())); // first
    t.on_record(rec_at(base + Duration::from_millis(50), "x"), |r| {
        emitted.push(r.clone())
    }); // suppressed (count=2)
    assert_eq!(emitted.len(), 1);
    t.flush(|r| emitted.push(r.clone())); // should emit aggregated with count=2
    assert_eq!(emitted.len(), 2);
    assert_eq!(emitted[1].repetition_count, 2);
}

#[test]
fn window_expiry_flushes_group() {
    let mut t = Throttler::new(ThrottleConfig {
        window: Duration::from_millis(100),
        min_count: 5,
    });
    let base = Instant::now();
    let mut emitted = Vec::new();
    t.on_record(rec_at(base, "y"), |r| emitted.push(r.clone())); // first (count=1)
    t.on_record(rec_at(base + Duration::from_millis(10), "y"), |r| {
        emitted.push(r.clone())
    }); // suppressed (count=2)
    assert_eq!(emitted.len(), 1);
    // Advance beyond window; next record with same fingerprint should first flush previous aggregated (count=2) then emit new first
    t.on_record(rec_at(base + Duration::from_millis(150), "y"), |r| {
        emitted.push(r.clone())
    });
    // After third call we expect two more emissions: aggregated old group (count=2) and new group first (count=1)
    assert_eq!(emitted.len(), 3);
    assert_eq!(emitted[1].repetition_count, 2);
    assert_eq!(emitted[2].repetition_count, 1);
}

#[test]
fn raw_logging_path_basic() {
    // Ensure raw records can be produced and throttled through logger interface
    let mut logger = BasicLogger::default();
    logger.log_raw("info", None, "raw message one");
    logger.log_raw("info", None, "raw message one"); // may aggregate later depending on min_count=2 default
    logger.flush();
}

#[test]
fn no_duplicate_on_final_flush_after_aggregate() {
    let mut t = Throttler::new(ThrottleConfig {
        window: Duration::from_millis(200),
        min_count: 2,
    });
    let base = Instant::now();
    let mut emitted = Vec::new();
    t.on_record(rec_at(base, "z"), |r| emitted.push(r.clone())); // first
    t.on_record(rec_at(base + Duration::from_millis(10), "z"), |r| {
        emitted.push(r.clone())
    }); // aggregated (count=2 triggers)
    assert_eq!(emitted.len(), 2);
    assert_eq!(emitted[1].repetition_count, 2);
    t.flush(|r| emitted.push(r.clone())); // should NOT duplicate aggregated emission
    assert_eq!(emitted.len(), 2);
}
