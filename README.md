# consola-rs 🐨

Elegant Console Logger for Rust and Browser (WASM)

A Rust port of [unjs/consola](https://github.com/unjs/consola) providing beautiful, powerful logging for native Rust applications and WebAssembly.

[![CI](https://github.com/MuntasirSZN/consola-rs/workflows/CI/badge.svg)](https://github.com/MuntasirSZN/consola-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ Features

- 🎨 **Beautiful Output**: Colored, formatted console output with multiple reporters
- 📦 **Lightweight**: Minimal dependencies, feature-gated integrations
- 🦀 **Type-Safe**: Full Rust type safety with flexible logging API
- 🌐 **WASM Support**: Run in browsers via WebAssembly
- 🔄 **Throttling**: Built-in message deduplication and repetition counting
- ⏸️ **Pause/Resume**: Buffer logs and replay them later
- 🎯 **Multiple Reporters**: Basic, Fancy (with icons), JSON, and custom reporters
- 🔗 **Ecosystem Integration**: Optional `log` and `tracing` crate support
- 💬 **Interactive Prompts**: Optional interactive CLI prompts (native only)
- 📝 **Error Chains**: Automatic error source chain formatting

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
consola = "0.0.0-alpha.0"
```

### Feature Flags

```toml
[dependencies]
consola = { version = "0.0.0-alpha.0", features = ["color", "fancy", "json"] }
```

Available features:
- `color` (default) - ANSI color support via `anstream`
- `fancy` (default) - Fancy reporter with icons and enhanced formatting
- `json` - JSON reporter for structured logging
- `prompt-demand` - Interactive prompts using the `demand` crate (native only)
- `wasm` - WebAssembly support via `wasm-bindgen`
- `bridge-log` - Integration with the `log` crate
- `bridge-tracing` - Integration with the `tracing` crate

## 🚀 Quick Start

### Basic Usage

```rust
use consola::{info, warn, error, success};

fn main() {
    info!("Application started");
    success!("Database connected successfully");
    warn!("Cache miss for key: {}", "user:123");
    error!("Failed to process request");
}
```

### With Format Arguments

```rust
use consola::{info, debug};

let username = "alice";
let count = 42;

info!("User {} logged in", username);
debug!("Processing {} items", count);
```

### Raw Logging (No Formatting)

```rust
use consola::info_raw;

// Bypass formatting pipeline for maximum performance
info_raw!("This is a raw message");
```

### Custom Log Types

```rust
use consola::log_type;

// Register and use custom log types
log_type!("custom", "Custom message: {}", value);
```

## 📖 Documentation

- [Migration Guide](MIGRATION.md) - Migrating from JavaScript consola
- [Architecture](ARCHITECTURE.md) - System design and components
- [Custom Reporters](REPORTERS.md) - Creating custom reporters
- [Prompts](PROMPTS.md) - Using interactive prompts (native only)
- [Integrations](INTEGRATION.md) - `log` and `tracing` integration
- [Feature Flags](FEATURE-FLAGS.md) - Complete feature matrix
- [Benchmarks](BENCHMARKS.md) - Performance characteristics

## 🌐 WebAssembly Usage

Build for WASM:

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web target
wasm-pack build --target web --features wasm
```

Use in JavaScript:

```javascript
import init, { info, warn, error } from './pkg/consola.js';

await init();

info("Hello from WASM!");
warn("This is a warning");
error("This is an error");
```

**Note**: Interactive prompts are not available in WASM - calling prompt methods will return an error.

## 🎨 Reporters

### Basic Reporter (Default)

Simple, clean output:
```
[info] Application started
[warn] Low disk space
[error] Connection failed
```

### Fancy Reporter (with `fancy` feature)

Enhanced output with icons and colors:
```
ℹ info    Application started
⚠ warn    Low disk space  
✖ error   Connection failed
```

### JSON Reporter (with `json` feature)

Structured JSON output for log aggregation:
```json
{"time":"2024-01-01T00:00:00Z","level":4,"type":"info","message":"Application started"}
```

## ⚡ Advanced Features

### Throttling & Deduplication

Automatically deduplicate repeated messages:

```rust
use consola::info;

// These will be coalesced into a single log with repetition count
for _ in 0..100 {
    info!("Processing batch");
}
// Output: [info] Processing batch  (x100)
```

### Pause & Resume

Buffer logs and replay them:

```rust
// Pause logging
consola.pause();

info!("This will be buffered");
warn!("This too");

// Resume and flush all buffered logs
consola.resume();
```

### Error Chain Formatting

Automatically format error source chains:

```rust
use consola::error;
use std::error::Error;

fn handle_error(err: Box<dyn Error>) {
    error!("Operation failed: {}", err);
    // Automatically shows full error chain with "Caused by:" sections
}
```

## 🧪 Testing

```bash
# Run all tests
cargo test --all-features

# Run with nextest (recommended)
cargo nextest run --all-features

# Run doctests
cargo test --doc
```

## 🔧 Development

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Build
cargo build --all-features

# Run all checks
make check
```

## 📊 Performance

consola-rs is designed for high performance:

- **Zero-cost abstractions**: Minimal overhead when logging is disabled
- **Optimized hot paths**: Raw logging bypasses formatting
- **Smart throttling**: Efficient deduplication using blake3 hashing
- **Small allocations**: Uses `smallvec` for common cases

See [BENCHMARKS.md](BENCHMARKS.md) for detailed performance characteristics.

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [unjs/consola](https://github.com/unjs/consola)
- Built with Rust 2024 edition (MSRV: 1.85)

## 📚 Related Projects

- [unjs/consola](https://github.com/unjs/consola) - Original JavaScript implementation
- [env_logger](https://docs.rs/env_logger) - Simple Rust logger
- [tracing](https://docs.rs/tracing) - Application-level tracing

---

**Status**: Alpha - API may change. Not yet recommended for production use.

