# Benchmarks

Performance characteristics and optimization notes for consola-rs

## Overview

consola-rs is designed for high performance while maintaining rich functionality. This document describes the benchmark suite and performance characteristics.

## Running Benchmarks

### Prerequisites

Benchmarks use the `criterion` crate, which is included in dev-dependencies.

### Commands

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench --bench logging_benchmark

# Compare against a baseline
cargo bench --bench logging_benchmark -- --save-baseline my-baseline

# Compare current performance to baseline
cargo bench --bench logging_benchmark -- --baseline my-baseline

# Generate detailed reports
cargo bench --bench logging_benchmark -- --verbose
```

### Results Location

Benchmark results are saved to:
- `target/criterion/` - Detailed HTML reports
- Console output - Summary statistics

## Benchmark Suites

### 1. Basic Logging

Tests fundamental logging operations:

- `info_macro`: Simple info! macro call
- `info_with_args`: Info with format arguments
- `warn_macro`: Warning messages
- `error_macro`: Error messages
- `debug_macro`: Debug messages

**Expected Performance**: 
- Simple logging: ~100-500 ns/op
- With arguments: ~200-800 ns/op

**Optimization Focus**:
- Minimize allocation overhead
- Efficient format string processing
- Fast type lookups

### 2. Raw vs Formatted

Compares raw logging (bypass formatting) vs normal formatted logging:

- `formatted_info`: Standard formatted output
- `raw_info`: Raw output (bypass pipeline)
- `formatted_with_args`: Formatted with arguments
- `raw_with_args`: Raw with arguments

**Expected Performance**:
- Raw logging should be 30-50% faster than formatted
- Overhead of formatting: ~100-300 ns

**Key Insight**: Use raw logging for hot paths where formatting isn't needed.

### 3. Repeated Messages

Tests throttling and deduplication with varying repetition counts:

- `same_message/10`: 10 identical messages
- `same_message/100`: 100 identical messages
- `same_message/1000`: 1000 identical messages
- `unique_messages/*`: Unique messages (no throttling)

**Expected Performance**:
- First message: Normal cost
- Subsequent throttled: ~50-100 ns/op (just fingerprint check)
- Unique messages: Full cost each time

**Throughput**:
- Throttled: 10-20M ops/sec
- Unique: 1-5M ops/sec

### 4. Format Complexity

Tests performance impact of different argument counts:

- `no_args`: Plain string
- `one_arg`: Single format argument
- `three_args`: Three format arguments
- `five_args`: Five format arguments

**Expected Performance**:
- No args: ~150 ns/op
- One arg: ~200 ns/op
- Three args: ~250 ns/op
- Five args: ~300 ns/op

**Linear Growth**: Each additional argument adds ~50 ns overhead.

### 5. String Sizes

Tests performance with different message lengths:

- `short_message`: ~5 characters
- `medium_message`: ~50 characters
- `long_message`: ~200+ characters

**Expected Performance**:
- Short: ~150 ns/op
- Medium: ~200 ns/op
- Long: ~300 ns/op

**Memory Impact**: Larger strings require more allocation but shouldn't significantly impact throughput.

### 6. Baseline Comparison

Compares consola-rs against raw `println!`:

- `println`: Standard library println macro
- `consola_info`: consola info! macro
- `println_with_args`: println with format args
- `consola_info_with_args`: info! with format args

**Expected Performance**:
- Target: consola overhead ≤ 2x println
- Acceptable: 1.5-2x println overhead
- Excellent: < 1.5x println overhead

**Goal**: Minimize overhead while providing rich features.

## Performance Targets

### Latency Targets (per operation)

| Operation | Target | Acceptable | Current* |
|-----------|--------|-----------|----------|
| Simple log | < 200 ns | < 500 ns | TBD |
| With args | < 300 ns | < 800 ns | TBD |
| Raw log | < 100 ns | < 300 ns | TBD |
| Throttled (cached) | < 100 ns | < 200 ns | TBD |

*Current performance to be measured and documented

### Throughput Targets

| Scenario | Target | Acceptable | Current* |
|----------|--------|-----------|----------|
| Simple info | > 5M ops/sec | > 2M ops/sec | TBD |
| Throttled | > 10M ops/sec | > 5M ops/sec | TBD |
| Unique messages | > 1M ops/sec | > 500K ops/sec | TBD |

*Current performance to be measured and documented

### Memory Targets

| Metric | Target | Acceptable |
|--------|--------|-----------|
| Per log overhead | < 100 bytes | < 200 bytes |
| Throttle state | < 1 KB | < 5 KB |
| Type registry | < 10 KB | < 50 KB |

## Optimization Strategies

### Completed Optimizations

1. **SmallVec for Arguments**
   - Use `SmallVec` to avoid heap allocation for common cases
   - Typical log has 0-5 arguments, inline 4-8
   - Impact: Reduces allocations by ~80%

2. **Blake3 for Fingerprinting**
   - Fast cryptographic hash for deduplication
   - Consistent cross-platform behavior
   - Impact: ~2-5 µs per fingerprint

3. **RwLock for Type Registry**
   - `parking_lot::RwLock` for better read performance
   - Most operations are reads (type lookups)
   - Impact: Minimal contention in multi-threaded scenarios

4. **Raw Logging Path**
   - Bypass formatting pipeline entirely
   - Direct output for maximum performance
   - Impact: 30-50% faster than formatted

### Planned Optimizations

1. **String Interning** (Task 108)
   - Cache common log type names
   - Reduce allocations for repeated strings
   - Estimated impact: 10-20% faster for high-volume logging

2. **Preallocated Buffers** (Task 109)
   - Pre-allocate string buffers with typical sizes
   - Avoid reallocations during formatting
   - Estimated impact: 5-15% faster

3. **Segment Arena Allocation** (Future)
   - Pool allocator for format segments
   - Reduce allocator pressure
   - Estimated impact: 10-20% faster

4. **SIMD for ANSI Stripping** (Future)
   - Vectorized ANSI escape code detection
   - Faster snapshot testing
   - Estimated impact: 2-5x faster ANSI stripping

## Profiling

### CPU Profiling

Use `perf` on Linux:

```bash
# Record profile data
cargo build --release --benches
perf record -F 99 --call-graph dwarf target/release/deps/logging_benchmark-*

# View flamegraph
perf report
```

Or use `cargo-flamegraph`:

```bash
cargo install flamegraph
cargo flamegraph --bench logging_benchmark
```

### Memory Profiling

Use `heaptrack` or `valgrind`:

```bash
# Heaptrack
heaptrack target/release/deps/logging_benchmark-*

# Valgrind
valgrind --tool=massif target/release/deps/logging_benchmark-*
```

### Allocation Tracking

Use `dhat` for allocation profiling:

```bash
# Add dhat to dev-dependencies, instrument code
cargo bench --features dhat-heap
```

## Comparison with Other Loggers

### Baseline (println!)

Pure println! overhead: ~50-100 ns/op

consola-rs target: ≤ 2x (100-200 ns/op)

### env_logger

Typical performance: ~200-500 ns/op

consola-rs target: Comparable or faster

### tracing

Typical performance: ~300-800 ns/op (with subscriber)

consola-rs target: Faster for simple logging, comparable with features

### log crate

Typical performance: ~50-150 ns/op (just log! macro, excluding backend)

consola-rs target: Competitive with backend included

## Performance Tips for Users

### 1. Use Raw Logging for Hot Paths

```rust
// Hot path
for _ in 0..1_000_000 {
    info_raw!("Fast message");
}
```

### 2. Leverage Throttling

```rust
// Automatically deduplicated
for _ in 0..1000 {
    info!("Repeated message");
}
// Output: [info] Repeated message  (x1000)
```

### 3. Minimize Format Complexity

```rust
// Faster
info!("Value: {}", value);

// Slower
info!("Complex: {:?}, {:?}, {:?}", a, b, c);
```

### 4. Disable Features You Don't Need

```toml
# Minimal build
consola = { version = "*", default-features = false }
```

### 5. Use Level Filtering

```rust
// Won't format if level is filtered out (when implemented)
debug!("Expensive: {}", expensive_operation());
```

## CI Integration

Benchmarks run in CI via manual workflow trigger:

```bash
# Trigger via GitHub UI or CLI
gh workflow run benchmarks.yml
```

Results are uploaded as artifacts for historical tracking.

## Regression Testing

To detect performance regressions:

1. Establish baseline:
   ```bash
   cargo bench -- --save-baseline main
   ```

2. Make changes

3. Compare:
   ```bash
   cargo bench -- --baseline main
   ```

4. Criterion will report performance changes

## Hardware Notes

Benchmark results depend heavily on hardware:

- **CPU**: Modern 3+ GHz recommended
- **Memory**: At least 8GB RAM
- **Disk**: SSD for faster compilation
- **OS**: Linux preferred for consistent results

Reference system (CI):
- CPU: GitHub Actions Ubuntu runner (2 cores)
- Memory: 7GB
- OS: Ubuntu Latest

## Contributing

When optimizing:

1. Run benchmarks before changes
2. Save baseline: `cargo bench -- --save-baseline before`
3. Make optimization
4. Compare: `cargo bench -- --baseline before`
5. Document improvements in PR
6. Update this file with new strategies

## Questions?

For performance-related questions or optimization ideas, open an issue on GitHub.

---

**Last Updated**: 2024-10-01  
**Benchmark Version**: 0.0.0-alpha.0  
**Criterion Version**: 0.7.0
