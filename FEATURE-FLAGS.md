# Feature Flags

Complete feature matrix for consola-rs

## ⚠️ Unstable Features (Task 277)

The following features are **experimental or planned** and their API may change:

- **`async-reporters`** (🚧 Planned): Non-blocking async reporters - API not yet stable
- **`wasm`** (⚠️ Experimental): WebAssembly support - limited functionality, API may change
- **`bridge-log`** (🚧 Planned): Integration with log crate - not yet implemented
- **`bridge-tracing`** (🚧 Planned): Integration with tracing crate - not yet implemented

**Recommendation**: Do not use unstable features in production without understanding the risks. APIs may change between versions without notice.

## Available Features

### Default Features

```toml
default = ["color", "fancy"]
```

These features are enabled by default when you add consola without specifying features:

```toml
[dependencies]
consola = "0.0.0-alpha.0"
```

### All Features

| Feature | Status | Dependencies | Description |
|---------|--------|--------------|-------------|
| `color` | ✅ Stable | anstream, anstyle | ANSI color support, NO_COLOR/FORCE_COLOR handling |
| `fancy` | ✅ Stable | unicode-width | Fancy reporter with icons, badges, enhanced formatting |
| `json` | ✅ Stable | serde, serde_json | JSON reporter for structured logging |
| `prompt-demand` | ✅ Stable | demand | Interactive CLI prompts (text, confirm, select) |
| `wasm` | ⚠️ Experimental | wasm-bindgen | WebAssembly support |
| `bridge-log` | 🚧 Planned | log | Integration with log crate |
| `bridge-tracing` | 🚧 Planned | tracing, tracing-subscriber | Integration with tracing crate |
| `async-reporters` | 🚧 Planned | - | Non-blocking async reporters |

## Feature Combinations

### Minimal Build

No colors, no fancy output, smallest binary:

```toml
[dependencies]
consola = { version = "0.0.0-alpha.0", default-features = false }
```

**Size**: ~200KB
**Output**: Plain text only

### Standard Build (Default)

Colors and fancy formatting:

```toml
[dependencies]
consola = "0.0.0-alpha.0"
# Equivalent to:
# consola = { version = "0.0.0-alpha.0", features = ["color", "fancy"] }
```

**Size**: ~250KB
**Output**: Colored text with icons

### Full-Featured Build

Everything except WASM:

```toml
[dependencies]
consola = { version = "0.0.0-alpha.0", features = [
    "color",
    "fancy",
    "json",
    "prompt-demand",
] }
```

**Size**: ~400KB
**Output**: All reporters available, prompts enabled

### WASM Build

For browser/WASM targets:

```toml
[dependencies]
consola = { version = "0.0.0-alpha.0", features = ["wasm", "color", "fancy"] }
```

**Note**: Do NOT enable `prompt-demand` with `wasm` - prompts are not supported in browsers.

### Server/CLI Build

For backend applications:

```toml
[dependencies]
consola = { version = "0.0.0-alpha.0", features = [
    "color",
    "fancy",
    "json",
    "bridge-log",
    "bridge-tracing",
] }
```

**Size**: ~450KB
**Output**: Full logging with ecosystem integration

## Feature Details

### `color`

**Status**: ✅ Stable

**Enables**:

- ANSI color codes via `anstyle`
- Cross-platform color support via `anstream`
- `NO_COLOR` environment variable support
- `FORCE_COLOR` environment variable support

**Dependencies**:

```toml
anstream = "0.6"
anstyle = "1.0"
```

**Example**:

```rust
use consola::info;

// Will output with colors on supported terminals
info!("Colored output!");
```

**Environment Variables**:

- `NO_COLOR=1`: Disable all colors
- `FORCE_COLOR=1`: Force colors even if not a TTY

**Binary Impact**: +50KB

### `fancy`

**Status**: ✅ Stable

**Enables**:

- `FancyReporter` with icons
- Unicode box characters
- Enhanced formatting
- Badge-style log types

**Dependencies**:

```toml
unicode-width = "0.2"
```

**Icons**:

- ℹ info
- ✔ success
- ⚠ warn
- ✖ error / fatal
- 🐛 debug
- 📝 trace
- 🚀 ready / start
- ✖ fail

**ASCII Fallback**: Automatically uses ASCII alternatives on non-unicode terminals

**Binary Impact**: +30KB

### `json`

**Status**: ✅ Stable

**Enables**:

- `JsonReporter` for structured logging
- JSON serialization of LogRecord
- Structured error chains
- Machine-readable output

**Dependencies**:

```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Schema**:

```json
{
  "time": "2024-01-01T00:00:00Z",
  "level": 4,
  "level_name": "info",
  "type": "info",
  "message": "Log message",
  "args": [...],
  "additional": {...},
  "repeat": 1,
  "schema": "consola-rs/v1"
}
```

**Use Cases**:

- Log aggregation (ELK, Splunk, etc.)
- Structured log parsing
- API logging
- Monitoring and alerting

**Binary Impact**: +100KB

### `prompt-demand`

**Status**: ✅ Stable

**Enables**:

- Interactive CLI prompts
- Text input
- Yes/No confirmation
- Single selection
- Multiple selection
- Cancellation handling

**Dependencies**:

```toml
demand = "1.7"
```

**Example**:

```rust
use consola::prompt::{DefaultDemandPrompt, PromptProvider, PromptCancelStrategy};

let prompt = DefaultDemandPrompt::new(PromptCancelStrategy::Default);

// Text input
let name = prompt.text("What is your name?", None)?;

// Confirmation
let confirmed = prompt.confirm("Continue?", Some(true))?;

// Selection
let choice = prompt.select("Choose option:", &["Option 1", "Option 2"])?;
```

**Platform Support**:

- ✅ Linux
- ✅ macOS
- ✅ Windows
- ❌ WASM (returns error)

**Binary Impact**: +150KB

### `wasm`

**Status**: ⚠️ Experimental

**Enables**:

- WebAssembly compilation
- Browser console output
- WASM-bindgen exports

**Dependencies**:

```toml
wasm-bindgen = "0.2"
```

**Limitations**:

- No interactive prompts
- Limited color support (browser-dependent)
- No file I/O
- Reduced functionality compared to native

**Build Command**:

```bash
wasm-pack build --target web --features wasm
```

**JavaScript Usage**:

```javascript
import init, { info, warn, error } from './pkg/consola.js';

await init();

info("Hello from WASM!");
```

**Binary Impact**: +100KB (WASM bundle)

### `bridge-log` (Planned)

**Status**: 🚧 Planned

**Enables**:

- Integration with `log` crate
- Route log crate messages through consola
- Level mapping
- Metadata preservation

**Dependencies**:

```toml
log = "0.4"
```

**Example** (planned):

```rust
use log::info;
use consola::bridge::init_log_bridge;

init_log_bridge();

// This will go through consola
info!("Message from log crate");
```

**Binary Impact**: +20KB

### `bridge-tracing` (Planned)

**Status**: 🚧 Planned

**Enables**:

- Integration with `tracing` crate
- Event and span support
- Field capture
- Context preservation

**Dependencies**:

```toml
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Example** (planned):

```rust
use tracing::info;
use consola::bridge::ConsolaLayer;

tracing_subscriber::registry()
    .with(ConsolaLayer::new())
    .init();

// This will go through consola
info!("Message from tracing");
```

**Binary Impact**: +80KB

### `async-reporters` (Planned)

**Status**: 🚧 Planned

**Enables**:

- Non-blocking log output
- Async channel-based reporters
- Background worker thread
- Buffered output

**Use Cases**:

- High-throughput logging
- Network-based reporters
- Database logging
- Minimal blocking in hot paths

**Binary Impact**: +50KB

## Compatibility Matrix

### Platform Support

| Feature | Linux | macOS | Windows | WASM |
|---------|-------|-------|---------|------|
| `color` | ✅ | ✅ | ✅ | ⚠️ |
| `fancy` | ✅ | ✅ | ✅ | ⚠️ |
| `json` | ✅ | ✅ | ✅ | ✅ |
| `prompt-demand` | ✅ | ✅ | ✅ | ❌ |
| `wasm` | N/A | N/A | N/A | ✅ |
| `bridge-log` | ✅ | ✅ | ✅ | ✅ |
| `bridge-tracing` | ✅ | ✅ | ✅ | ✅ |

⚠️ = Limited support or browser-dependent
❌ = Not supported
🚧 = Not yet implemented

### Rust Version Support

- **MSRV**: 1.85 (Rust 2024 edition)
- All features support the same MSRV

### Feature Dependencies

```
color
  ├── anstream
  └── anstyle

fancy
  ├── unicode-width
  └── (implicitly requires color for styling)

json
  ├── serde
  └── serde_json

prompt-demand
  └── demand

wasm
  └── wasm-bindgen

bridge-log
  └── log

bridge-tracing
  ├── tracing
  └── tracing-subscriber

async-reporters
  └── (tokio or async-std, TBD)
```

## Recommended Configurations

### CLI Application

```toml
consola = { version = "0.0.0-alpha.0", features = ["color", "fancy", "prompt-demand"] }
```

### Web Service

```toml
consola = { version = "0.0.0-alpha.0", features = ["color", "json", "bridge-tracing"] }
```

### Library

```toml
# Don't enable features in library crates - let consumers decide
consola = { version = "0.0.0-alpha.0", default-features = false }
```

### WASM Application

```toml
consola = { version = "0.0.0-alpha.0", features = ["wasm", "color", "fancy"] }
```

### Embedded/Resource-Constrained

```toml
consola = { version = "0.0.0-alpha.0", default-features = false }
```

## Binary Size Impact

Approximate size impact of each feature (Release build with LTO):

| Configuration | Size (approx) |
|--------------|---------------|
| Minimal (no features) | ~200 KB |
| Default (color + fancy) | ~250 KB |
| + json | ~350 KB |
| + prompt-demand | ~500 KB |
| All features | ~550 KB |

**Note**: Actual sizes depend on your code and other dependencies. Use `cargo bloat` to analyze your specific build.

## FAQ

### Q: Which features should I enable?

**A**: Start with defaults. Add `json` if you need structured logging, `prompt-demand` for CLI interactivity.

### Q: Can I use prompts in WASM?

**A**: No. Prompts require terminal interaction not available in browsers. Enable `wasm` instead of `prompt-demand`.

### Q: Does `fancy` require `color`?

**A**: While not a hard dependency, fancy output looks best with colors. Both are enabled by default.

### Q: How do I minimize binary size?

**A**: Use `default-features = false` and only enable what you need.

### Q: Are features mutually exclusive?

**A**: No, you can enable any combination except `wasm` + `prompt-demand`.

### Q: What's the performance impact?

**A**: Features mainly affect binary size. Runtime overhead is minimal, especially with level filtering.

______________________________________________________________________

For more information, see the [README](README.md) and [ARCHITECTURE](ARCHITECTURE.md).
