# Risk Mitigations & Known Issues

This document outlines potential risks in the consola-rs implementation and the mitigations that have been put in place.

## 1. Level Sentinel Confusion

**Risk**: The numeric level system uses sentinel values (SILENT=-99, VERBOSE=99) which could be confusing when users provide arbitrary numeric levels.

**Mitigation**:
- Levels are ordered where lower values are more severe (FATAL=0, ERROR=1, etc.)
- Unknown numeric levels are handled by the comparison operators naturally
- The `normalize_level()` function accepts both numeric and named levels
- When an unknown numeric level is encountered, the comparison logic still works correctly (e.g., a custom level of 10 would be more verbose than DEBUG=6)

**Documentation**: See `src/levels.rs` for the level constants and ordering rules.

## 2. Fingerprint Includes Meta Causing Over-Coalescing

**Risk**: If metadata fields are included in the fingerprint calculation, messages with identical content but different metadata might be incorrectly coalesced.

**Mitigation**:
- Currently, fingerprints are calculated based on `type_name`, `tag`, and `args` only
- Metadata fields are NOT included in the fingerprint by default
- This ensures that only truly identical log messages are throttled together
- Future enhancement: Add a configurable `fingerprint_meta: bool` toggle if users want metadata-aware throttling

**Current Implementation**: See `src/throttling.rs` - the `compute_fingerprint()` function explicitly excludes metadata.

## 3. Windows Color Edge Cases

**Risk**: Windows terminal color support varies across different Windows versions and terminal emulators.

**Mitigation**:
- Rely on `anstream` crate's automatic detection of color support
- `anstream` handles Windows-specific quirks including:
  - Enabling ANSI escape code support on Windows 10+
  - Detecting Windows Terminal vs legacy console
  - Respecting NO_COLOR and FORCE_COLOR environment variables
- All color output goes through `anstream` writers which handle platform differences

**Testing**: The CI pipeline runs on Windows to catch platform-specific issues. Environment variable handling (NO_COLOR, FORCE_COLOR) is tested across all platforms.

## 4. WASM Size Bloat

**Risk**: WebAssembly builds might be larger than necessary due to included dependencies and debug information.

**Mitigation**:
- Use feature flags to exclude unnecessary dependencies in WASM builds:
  - `prompt-demand` feature should NOT be enabled for WASM (prompts don't work in browsers)
  - `wasm` feature adds minimal dependencies (wasm-bindgen, js-sys, web-sys)
- Recommended build configuration for minimal WASM size:

```bash
# Build with wasm-pack
wasm-pack build --target web --release

# Or with cargo directly
cargo build --target wasm32-unknown-unknown --release \
  --features wasm \
  --no-default-features
```

- Recommended Cargo.toml profile for production WASM:

```toml
[profile.release]
lto = true              # Enable link-time optimization
codegen-units = 1       # Single codegen unit for better optimization
opt-level = "z"         # Optimize for size
strip = true            # Strip debug symbols
```

**Size Expectations**:
- With default features (color + fancy): ~200-300 KB (gzipped)
- With minimal features (no color, no fancy): ~100-150 KB (gzipped)
- JSON reporter adds ~50 KB due to serde_json

## 5. Re-entrancy from log/tracing Integration

**Risk**: If consola uses the `log` or `tracing` crates internally for its own logging, and a bridge is active, infinite recursion could occur.

**Mitigation**:
- **Thread-local recursion guards**: Both `bridge_log` and `bridge_tracing` modules use thread-local guards to detect and prevent recursion
- If a bridge detects that it's being called recursively (guard is already set), it immediately returns without logging
- The guards are automatically cleared when the scope exits, even if a panic occurs
- Tests verify that recursion is properly prevented

**Implementation Details**:
```rust
// In bridge_log.rs and bridge_tracing.rs
thread_local! {
    static RECURSION_GUARD: RefCell<bool> = const { RefCell::new(false) };
}

// In log/emit methods:
let guard = RECURSION_GUARD.with(|g| {
    if *g.borrow() {
        return false;  // Already in a log call, skip
    }
    *g.borrow_mut() = true;
    true
});
```

**Testing**: The test suite includes `recursion_safety` tests that verify the guards work correctly.

## 6. Demand Crate Prompt Cancellation Semantics

**Risk**: The `demand` crate's cancellation behavior might change in future versions, breaking the consola prompt abstraction.

**Mitigation**:
- Version pinning: Cargo.toml specifies `demand = "1.7.2"` (exact minor version)
- Cancellation strategy abstraction: consola defines its own `PromptCancelStrategy` enum that maps to demand's behavior
- If demand's API changes, only the adapter code in `src/prompt.rs` needs updating
- WASM builds explicitly error if prompt methods are called, preventing runtime surprises

**Version Compatibility Note**: 
- Tested with demand 1.7.2
- Future versions should be tested against the prompt test suite before upgrading
- See `src/prompt.rs` tests for cancellation behavior validation

## Additional Considerations

### Thread Safety
All global state (type registry) uses `RwLock` from `parking_lot` for thread-safe access. Multiple loggers can be used concurrently without issues.

### Memory Usage
- Throttling uses Blake3 for fingerprinting, which is fast but does allocate for the hash
- Paused loggers buffer records in a `VecDeque`, which can grow unbounded by default
- Use `LoggerConfig::queue_capacity` to limit buffering if memory is a concern

### Performance
- See BENCHMARKS.md for detailed performance characteristics
- Logging overhead is typically <100ns per message for basic output
- Throttling adds minimal overhead (~10-20ns for fingerprint check)

## Reporting Issues

If you encounter any issues related to these risks or other unexpected behavior, please file an issue on GitHub with:
1. Platform and Rust version
2. Feature flags enabled
3. Minimal reproduction case
4. Expected vs actual behavior
