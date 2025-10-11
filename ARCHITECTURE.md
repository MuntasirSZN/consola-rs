# Architecture

System design and implementation details for consola-rs

## Overview

consola-rs is structured as a modular logging library with several key components:

```
┌─────────────┐
│   Macros    │  (info!, warn!, error!, etc.)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Logger    │  (LoggerBuilder, filtering, pause/resume)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Throttling  │  (Deduplication, repetition counting)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Formatter  │  (Segment pipeline, styling)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Reporter   │  (Basic, Fancy, JSON)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Output    │  (stdout, stderr, custom sinks)
└─────────────┘
```

## Core Modules

### 1. Levels (`levels.rs`)

**Purpose**: Log level management and type registration

**Key Types**:

- `LogLevel(i16)`: Newtype wrapper for log levels
- `LogTypeSpec`: Specification for custom log types
- Type registry: Global `RwLock<HashMap<String, LogTypeSpec>>`

**Constants**:

```rust
pub const SILENT: LogLevel = LogLevel(-99);
pub const FATAL: LogLevel = LogLevel(0);
pub const ERROR: LogLevel = LogLevel(1);
pub const WARN: LogLevel = LogLevel(2);
pub const LOG: LogLevel = LogLevel(3);
pub const INFO: LogLevel = LogLevel(4);
pub const SUCCESS: LogLevel = LogLevel(5);
pub const DEBUG: LogLevel = LogLevel(6);
pub const TRACE: LogLevel = LogLevel(7);
pub const VERBOSE: LogLevel = LogLevel(99);
```

**Thread Safety**: Uses `parking_lot::RwLock` for lock-free reads in the common case.

### 2. Record (`record.rs`)

**Purpose**: Log record data structure and argument handling

**Key Types**:

```rust
pub struct LogRecord {
    pub timestamp: Instant,
    pub level: LogLevel,
    pub type_name: String,
    pub tag: Option<String>,
    pub args: Vec<ArgValue>,
    pub repetition_count: u32,
}

pub enum ArgValue {
    String(String),
    Number(f64),
    Bool(bool),
    Error(String),
    OtherDebug(String),
}
```

**Features**:

- Flexible argument handling (primitives, errors, debug values)
- JSON serialization support (feature: `json`)
- Efficient storage using `SmallVec` for common cases

### 3. Throttling (`throttling.rs`)

**Purpose**: Message deduplication and repetition counting

**Algorithm**:

1. Generate fingerprint: `blake3(type_name + args + tag + level)`
1. Check if fingerprint matches previous log
1. If within throttle window and count >= min_count: suppress and increment count
1. On window expiry or different fingerprint: flush with repetition count

**Configuration**:

- `throttle_window_ms`: Time window for deduplication (default: 500ms)
- `throttle_min_count`: Minimum occurrences before suppression (default: 2)

**Clock Abstraction**:

- `RealClock`: Uses system time
- `MockClock`: Deterministic time for testing

### 4. Formatter (`format.rs`)

**Purpose**: Transform LogRecord into styled segments

**Pipeline**:

```
LogRecord → FormatOptions → Segments → Styled Output
```

**Segments**:

- Time (optional)
- Type/Level indicator
- Tag (optional)
- Message
- Additional data (key-value pairs)
- Metadata
- Stack traces
- Repetition count

**FormatOptions**:

```rust
pub struct FormatOptions {
    pub date: bool,
    pub colors: bool,
    pub compact: bool,
    pub columns: Option<usize>,
    pub error_level: usize,
    pub unicode_mode: bool,
}
```

**Raw Path**: Bypass formatting for performance (`log_raw` methods)

### 5. Reporter (`reporter.rs`)

**Purpose**: Output formatted logs to sinks

**Implementations**:

#### BasicReporter

- Simple `[type] message` format
- stderr for levels < 2, stdout otherwise
- Error chain formatting with depth limiting

#### FancyReporter (feature: `fancy`)

- Icons for each log type (✔, ✖, ⚠, ℹ, etc.)
- ASCII fallback for non-unicode terminals
- Colored badges and type names
- Enhanced stack trace formatting

#### JsonReporter (feature: `json`)

- Single-line JSON per log
- Structured error chains
- Deterministic key ordering
- Schema version: `consola-rs/v1`

**Custom Reporters**: Implement the `Reporter` trait

### 6. Error Chain (`error_chain.rs`)

**Purpose**: Extract and format Rust error source chains

**Features**:

- Recursive source extraction via `Error::source()`
- Cycle detection (prevents infinite loops)
- Depth limiting (via `FormatOptions.error_level`)
- Multi-line message normalization

**Format**:

```
Error: Main error message
Caused by:
  - First cause
  - Second cause
  - Third cause
```

### 7. Utilities (`utils.rs`)

**Purpose**: Helper functions for formatting and output

**Components**:

- `strip_ansi`: Remove ANSI escape codes
- Box drawing: Unicode and ASCII box characters
- Tree formatting: Hierarchical output with proper indentation
- Alignment: Left/right/center text alignment
- Sinks: `StdoutSink`, `StderrSink`, `TestSink`

### 8. Clock (`clock.rs`)

**Purpose**: Time abstraction for testability

**Implementations**:

- `RealClock`: `Instant::now()` wrapper
- `MockClock`: Controllable time for deterministic tests

### 9. Prompt (`prompt.rs`)

**Purpose**: Interactive user input (feature: `prompt-demand`)

**Key Types**:

```rust
pub enum PromptCancelStrategy {
    Reject,     // Return error
    Default,    // Use default value
    Undefined,  // Return Undefined
    Null,       // Return NullValue
    Symbol,     // Return SymbolCancel
}

pub enum PromptOutcome<T> {
    Value(T),
    Undefined,
    NullValue,
    SymbolCancel,
    Cancelled,
}

pub trait PromptProvider {
    fn text(&self, prompt: &str, default: Option<&str>) -> Result<PromptOutcome<String>>;
    fn confirm(&self, prompt: &str, default: Option<bool>) -> Result<PromptOutcome<bool>>;
    fn select(&self, prompt: &str, options: &[&str]) -> Result<PromptOutcome<usize>>;
    fn multiselect(&self, prompt: &str, options: &[&str]) -> Result<PromptOutcome<Vec<usize>>>;
}
```

**WASM Handling**: `WasmPromptStub` returns `PromptError::NotSupported`

### 10. Macros (`macros.rs`)

**Purpose**: Ergonomic logging API

**Macro Types**:

- Standard: `info!`, `warn!`, `error!`, `success!`, `debug!`, `trace!`, `fatal!`, `ready!`, `start!`, `fail!`
- Custom: `log_type!(type_name, format, args)`
- Raw: `info_raw!`, `warn_raw!`, `error_raw!`, etc.

**Implementation**: Thin wrappers around helper functions

## Data Flow

### 1. Normal Logging Flow

```
User calls info!("message")
    ↓
Macro expands to log_message("info", "message")
    ↓
Create LogRecord with timestamp, level, type, args
    ↓
Level filtering (check if level >= configured level)
    ↓
Throttling check (fingerprint, window, count)
    ↓
Mock interception (if set)
    ↓
Pause check (buffer if paused)
    ↓
Format pipeline (LogRecord → Segments)
    ↓
Reporter (segments → styled output)
    ↓
Sink (stdout/stderr/custom)
```

### 2. Raw Logging Flow

```
User calls info_raw!("message")
    ↓
Macro expands to log_message_raw("info", "message")
    ↓
Create LogRecord (minimal)
    ↓
Level filtering
    ↓
Throttling check (same as normal)
    ↓
Direct output (bypass formatting)
    ↓
Sink
```

### 3. Throttled Flow

```
Same message repeated multiple times
    ↓
First: Normal flow
    ↓
Subsequent (within window): Suppress, increment count
    ↓
On flush/window expiry/different message:
    Emit "[type] message  (xN)"
```

### 4. Paused Flow

```
logger.pause() called
    ↓
New logs added to pause queue (VecDeque)
    ↓
logger.resume() called
    ↓
Flush suppressed throttled message (if any)
    ↓
Drain queue, process each log sequentially
```

## Thread Safety

### Global State

**Type Registry**:

```rust
static TYPE_REGISTRY: Lazy<RwLock<HashMap<String, LogTypeSpec>>> = ...;
```

- Thread-safe via `parking_lot::RwLock`
- Optimized for concurrent reads

**Logger Instances**: Not yet implemented (currently using global helpers)

### Concurrency Model

- **Read-heavy operations**: Type lookups use RwLock for minimal contention
- **Write operations**: Type registration locks for write
- **Independent logs**: No shared state between log calls (stateless formatters)

## Performance Considerations

### 1. Hot Path Optimizations

- **SmallVec**: Avoid heap allocations for typical argument counts
- **Raw logging**: Bypass entire formatting pipeline
- **Level guards**: Early return before expensive operations
- **Fingerprint caching**: Use fast blake3 hashing

### 2. Memory Management

- **String interning**: Considered for common log types
- **Arena allocation**: Potential for segment allocation
- **Zero-copy**: Where possible, use string references

### 3. Lock-Free Operations

- **RwLock**: parking_lot implementation for better performance
- **Atomic operations**: For simple counters and flags
- **Lock-free structures**: Considered for future async reporters

## Feature Gates

### Why Feature Gates?

- **Minimize dependencies**: Users only pay for what they use
- **Platform support**: Some features (like prompts) don't work in WASM
- **Build time**: Reduce compilation time for minimal builds

### Feature Matrix

| Feature | Dependencies | Adds |
|---------|-------------|------|
| `color` | anstream, anstyle | Color support |
| `fancy` | unicode-width | Fancy reporter, icons |
| `json` | serde, serde_json | JSON reporter |
| `prompt-demand` | demand | Interactive prompts |
| `wasm` | wasm-bindgen | WASM exports |
| `bridge-log` | log | log crate integration |
| `bridge-tracing` | tracing, tracing-subscriber | tracing integration |

### Default Features

```toml
default = ["color", "fancy"]
```

Provides colored output with fancy formatting out of the box.

## Testing Strategy

### Unit Tests

- Embedded in each module
- Test individual components in isolation
- Use `MockClock` for deterministic timing

### Integration Tests

- `tests/` directory
- Test component interactions
- Snapshot testing with `insta` crate

### Property Tests

- Use `proptest` for randomized testing
- Verify panic-free operation
- Test invariants (e.g., repetition counts)

### Benchmark Tests

- `benches/` directory (when created)
- Measure hot path performance
- Compare raw vs formatted overhead

## Future Architecture

### Planned Improvements

1. **Async Reporters**: Non-blocking output via async channels
1. **Plugin System**: Dynamic reporter and formatter plugins
1. **Middleware**: Pre/post processing hooks
1. **Multi-sink Routing**: Different outputs for different log levels
1. **Structured Logging**: First-class support for structured data
1. **Span Support**: Integration with tracing spans

### Potential Optimizations

1. **String Interning**: Reduce allocations for repeated strings
1. **Custom Allocators**: Arena allocation for log records
1. **SIMD**: Vectorized operations for ANSI stripping
1. **Lock-Free Queue**: For pause/resume buffering

______________________________________________________________________

For implementation details, see the source code in `src/`. For usage examples, see the [README](README.md) and other documentation files.
