# consola-rs Specification (Initial Skeleton)

This document will evolve. Currently capturing early design notes for stages 1-3.

## Log Levels
Sentinel values:
- silent = -99
- verbose = 99

Intermediate numeric levels (tentative mapping):
- fatal=0, error=1, warn=2, log=3, info=4, success=5, fail=5 (alias?), ready=4, start=3, box=3, debug=6, trace=7

## Type Registration
A global (process-wide) registry mapping type name -> TypeSpec { level: i16 } with overwrite allowed (last write wins). Thread-safe via RwLock.

## Log Record Model (simplified v0)
LogRecord { timestamp: Instant, level: i16, type_name: String, tag: Option<String>, args: Vec<ArgValue>, repetition: u32 }
ArgValue draft variants: String, Number(f64), Bool(bool), Error(String), OtherDebug(String)

## Throttling (Stage 3 Skeleton)
Config: throttle_window_ms (default 500), throttle_min_count (default 2).
Fingerprint: join of type_name + normalized args debug + tag + level -> blake3 hash hex (first 16 chars maybe).
State holds last fingerprint, count, first_instant.

Further details pending.
