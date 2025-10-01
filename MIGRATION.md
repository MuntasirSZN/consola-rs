# Migration Guide

Migrating from JavaScript consola to Rust consola-rs

## Overview

consola-rs provides similar functionality to the JavaScript version but with Rust-specific adaptations and improvements. This guide covers the key differences and migration strategies.

## Key Differences

### 1. Log Levels

**JavaScript consola**: Dynamic infinite log levels
```javascript
consola.level = 5;
consola.log('message');
```

**Rust consola-rs**: Fixed numeric levels with type system
```rust
use consola::{LogLevel, register_type, LogTypeSpec};

register_type("custom", LogTypeSpec { level: LogLevel(42) });
```

Rust uses a strongly-typed approach with a fixed set of log levels:
- `SILENT` (-99): No output
- `FATAL` (0): Fatal errors
- `ERROR` (1): Errors
- `WARN` (2): Warnings
- `LOG` (3): General logs
- `INFO` (4): Informational
- `SUCCESS` (5): Success messages
- `DEBUG` (6): Debug info
- `TRACE` (7): Trace-level detail
- `VERBOSE` (99): Maximum verbosity

### 2. API Style

**JavaScript consola**: Object-oriented with dynamic methods
```javascript
consola.info('Hello');
consola.warn('Warning!');
consola.error('Error!');
```

**Rust consola-rs**: Macros for ergonomic usage
```rust
use consola::{info, warn, error};

info!("Hello");
warn!("Warning!");
error!("Error!");
```

### 3. Prompts

**JavaScript consola**: Uses `prompts` package
```javascript
const name = await consola.prompt('What is your name?');
const confirmed = await consola.prompt('Continue?', { type: 'confirm' });
```

**Rust consola-rs**: Uses `demand` crate (native only, feature-gated)
```rust
use consola::prompt::{DefaultDemandPrompt, PromptProvider, PromptCancelStrategy};

let prompt = DefaultDemandPrompt::new(PromptCancelStrategy::Default);
let name = prompt.text("What is your name?", None)?;
let confirmed = prompt.confirm("Continue?", Some(true))?;
```

**Important**: Prompts are NOT available in WASM builds. Calling prompt methods in WASM returns `PromptError::NotSupported`.

### 4. Reporters

**JavaScript consola**: 
```javascript
import { createConsola, BasicReporter, FancyReporter } from 'consola';

const logger = createConsola({
  reporters: [new FancyReporter()]
});
```

**Rust consola-rs**:
```rust
// Reporters are configured through features and builder pattern
// Basic reporter is default, Fancy requires "fancy" feature
// JSON reporter requires "json" feature
```

### 5. Tags

**JavaScript consola**:
```javascript
consola.withTag('api').info('Request received');
```

**Rust consola-rs**:
```rust
// Tags are part of the LogRecord structure
// Can be added via builder pattern (when LoggerBuilder is complete)
```

### 6. Error Handling

**JavaScript consola**: JavaScript Error objects
```javascript
try {
  throw new Error('Something went wrong');
} catch (error) {
  consola.error(error);
}
```

**Rust consola-rs**: Rust error types with source chain
```rust
use std::error::Error;
use consola::error;

fn handle_error(err: Box<dyn Error>) {
    error!("Operation failed: {}", err);
    // Automatically extracts and formats error source chain
}
```

### 7. Throttling

**JavaScript consola**: Not built-in
```javascript
// Must implement throttling manually or use external packages
```

**Rust consola-rs**: Built-in with configurable window
```rust
// Automatic deduplication and repetition counting
// Configurable via throttle_window_ms and throttle_min_count
for _ in 0..100 {
    info!("Same message");
}
// Output: [info] Same message  (x100)
```

### 8. Pause/Resume

**JavaScript consola**:
```javascript
consola.pause();
// ... buffered logs
consola.resume();
```

**Rust consola-rs**:
```rust
// Similar API through logger instance
logger.pause();
// ... buffered logs
logger.resume();
```

## Feature Comparison

| Feature | JavaScript consola | Rust consola-rs |
|---------|-------------------|-----------------|
| Basic logging | ✅ | ✅ |
| Fancy reporter | ✅ | ✅ (feature: `fancy`) |
| JSON reporter | ✅ | ✅ (feature: `json`) |
| Browser support | ✅ | ✅ (feature: `wasm`) |
| Interactive prompts | ✅ | ✅ (feature: `prompt-demand`, native only) |
| Throttling | ❌ | ✅ (built-in) |
| Pause/Resume | ✅ | ✅ |
| Tags | ✅ | ✅ |
| Mock/Test support | ✅ | ✅ |
| log crate bridge | N/A | ✅ (feature: `bridge-log`) |
| tracing bridge | N/A | ✅ (feature: `bridge-tracing`) |

## Migration Checklist

### 1. Update Dependencies

**Before** (JavaScript):
```json
{
  "dependencies": {
    "consola": "^3.0.0"
  }
}
```

**After** (Rust):
```toml
[dependencies]
consola = { version = "0.0.0-alpha.0", features = ["color", "fancy"] }
```

### 2. Replace Imports

**Before** (JavaScript):
```javascript
import consola from 'consola';
```

**After** (Rust):
```rust
use consola::{info, warn, error, success, debug, trace};
```

### 3. Update Log Calls

**Before** (JavaScript):
```javascript
consola.info('User logged in:', username);
consola.error('Failed:', error);
```

**After** (Rust):
```rust
info!("User logged in: {}", username);
error!("Failed: {}", error);
```

### 4. Update Prompt Usage (if applicable)

**Before** (JavaScript):
```javascript
const name = await consola.prompt('Name?');
```

**After** (Rust):
```rust
use consola::prompt::{DefaultDemandPrompt, PromptProvider};

let prompt = DefaultDemandPrompt::new_default();
let outcome = prompt.text("Name?", None)?;
if let PromptOutcome::Value(name) = outcome {
    info!("Hello, {}!", name);
}
```

### 5. Handle WASM Differences

**JavaScript** (works everywhere):
```javascript
const name = await consola.prompt('Name?');
```

**Rust WASM** (not supported):
```rust
// Prompts will return PromptError::NotSupported in WASM
// Use alternative input methods for browser environments
```

## Best Practices

### 1. Use Macros for Ergonomics

Prefer macros over direct function calls:

```rust
// Good
info!("User {} logged in", username);

// Less ergonomic (when full API is available)
logger.log_type("info", &format!("User {} logged in", username));
```

### 2. Feature Gates

Only enable features you need:

```toml
# Minimal (color output only)
consola = { version = "0.0.0-alpha.0", default-features = false, features = ["color"] }

# With fancy output and JSON
consola = { version = "0.0.0-alpha.0", features = ["fancy", "json"] }

# Everything except WASM
consola = { version = "0.0.0-alpha.0", features = ["color", "fancy", "json", "prompt-demand"] }
```

### 3. Error Handling

Leverage Rust's error handling:

```rust
use anyhow::Result;
use consola::error;

fn process() -> Result<()> {
    // ... operations that might fail
    Ok(())
}

if let Err(e) = process() {
    error!("Processing failed: {:?}", e);
}
```

### 4. Testing

Use mock support for testing:

```rust
#[cfg(test)]
mod tests {
    use consola::{info, warn};
    
    #[test]
    fn test_logging() {
        // Your test code that logs
        info!("Test started");
        // Assertions
    }
}
```

## Unsupported Features

The following JavaScript consola features are not currently supported:

1. **Dynamic method creation**: JavaScript's dynamic nature allowed arbitrary log type methods
2. **Middleware/hooks**: Not yet implemented
3. **Custom log objects**: Direct object logging requires serialization
4. **Stack trace customization**: Limited control over stack formatting
5. **Reporter plugins at runtime**: Reporters must be configured at build time via features

## Getting Help

- Check the [examples](./examples) directory
- Read the [API documentation](https://docs.rs/consola)
- See [ARCHITECTURE.md](ARCHITECTURE.md) for implementation details
- Open an issue on [GitHub](https://github.com/MuntasirSZN/consola-rs/issues)

## Future Compatibility

consola-rs is in alpha stage. The API may change before 1.0. We aim to:

- Maintain migration guide updates
- Provide deprecation warnings before breaking changes
- Follow semantic versioning after 1.0

---

For questions or issues during migration, please open a [GitHub issue](https://github.com/MuntasirSZN/consola-rs/issues).
