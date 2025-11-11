# WebAssembly Usage Guide

This document explains how to use consola-rs in WebAssembly/browser environments.

## Building for WASM

### Prerequisites

Install `wasm-pack`:

```bash
cargo install wasm-pack
```

### Build Command

Build the library for web targets:

```bash
wasm-pack build --target web --release
```

For smaller bundle sizes in production, use these optimizations:

```bash
# Build with size optimizations
wasm-pack build --target web --release -- --no-default-features --features wasm
```

The built package will be in the `pkg/` directory and can be imported directly in JavaScript modules.

### Other Target Types

wasm-pack supports multiple target types:

```bash
# For Node.js
wasm-pack build --target nodejs

# For bundlers (webpack, rollup, etc.)
wasm-pack build --target bundler

# For no modules (creates global variables)
wasm-pack build --target no-modules
```

## JavaScript Usage

### Basic Example

```javascript
import init, { 
  create_logger, 
  log_info, 
  log_error, 
  log_warn,
  free_logger 
} from './pkg/consola.js';

// Initialize the WASM module
await init();

// Create a logger
const logger = create_logger();

// Log messages
log_info(logger, "Application started");
log_warn(logger, "This is a warning");
log_error(logger, "Something went wrong!");

// Clean up when done
free_logger(logger);
```

### With Log Levels

```javascript
import init, { 
  create_logger_with_level,
  set_level, 
  log_info,
  log_debug,
  free_logger 
} from './pkg/consola.js';

await init();

// Create logger with INFO level (4)
const logger = create_logger_with_level(4);

log_info(logger, "This will appear");
log_debug(logger, "This will NOT appear (debug=6 > info=4)");

// Change level at runtime
set_level(logger, 6); // DEBUG level
log_debug(logger, "Now debug messages appear");

free_logger(logger);
```

### Available Log Levels

```javascript
const LEVELS = {
  SILENT: -99,
  FATAL: 0,
  ERROR: 1,
  WARN: 2,
  LOG: 3,
  INFO: 4,
  SUCCESS: 5,
  DEBUG: 6,
  TRACE: 7,
  VERBOSE: 99
};
```

Lower values are more severe. A logger with level INFO (4) will show FATAL, ERROR, WARN, LOG, and INFO messages, but not DEBUG or TRACE.

### Pause and Resume

```javascript
import init, { 
  create_logger, 
  pause,
  resume,
  log_info,
  free_logger 
} from './pkg/consola.js';

await init();
const logger = create_logger();

// Pause logging (messages are buffered)
pause(logger);

log_info(logger, "Message 1 - buffered");
log_info(logger, "Message 2 - buffered");

// Resume (flushes buffered messages)
resume(logger);

log_info(logger, "Message 3 - appears immediately");

free_logger(logger);
```

### Using Different Log Types

```javascript
import init, { 
  create_logger,
  log_info,
  log_warn,
  log_error,
  log_success,
  log_fail,
  log_debug,
  log_trace,
  log_fatal,
  log_ready,
  log_start,
  log_box,
  free_logger
} from './pkg/consola.js';

await init();
const logger = create_logger();

// Standard levels
log_fatal(logger, "Fatal error!");
log_error(logger, "Error message");
log_warn(logger, "Warning message");
log_info(logger, "Info message");
log_debug(logger, "Debug message");
log_trace(logger, "Trace message");

// Special types
log_success(logger, "✓ Operation completed");
log_fail(logger, "✗ Operation failed");
log_ready(logger, "Ready to accept connections");
log_start(logger, "Starting server...");
log_box(logger, "Multi-line\ncontent\nin a box");

free_logger(logger);
```

### Error Logging with Stack Traces

```javascript
import init, { 
  create_logger,
  log_error_with_js_error,
  free_logger
} from './pkg/consola.js';

await init();
const logger = create_logger();

try {
  throw new Error("Something went wrong!");
} catch (error) {
  // Log with JavaScript Error object (includes stack trace)
  log_error_with_js_error(logger, "Caught an error:", error);
}

free_logger(logger);
```

## JavaScript Wrapper (Recommended)

For better ergonomics, wrap the WASM functions in a JavaScript class:

```javascript
import init, * as consola_wasm from './pkg/consola.js';

class ConsolaLogger {
  constructor(level = 4) {
    this.logger = null;
    this.level = level;
  }

  async init() {
    await init();
    this.logger = consola_wasm.create_logger_with_level(this.level);
    return this;
  }

  info(message) {
    if (this.logger) consola_wasm.log_info(this.logger, message);
  }

  warn(message) {
    if (this.logger) consola_wasm.log_warn(this.logger, message);
  }

  error(message, error = null) {
    if (!this.logger) return;
    
    if (error) {
      consola_wasm.log_error_with_js_error(this.logger, message, error);
    } else {
      consola_wasm.log_error(this.logger, message);
    }
  }

  debug(message) {
    if (this.logger) consola_wasm.log_debug(this.logger, message);
  }

  success(message) {
    if (this.logger) consola_wasm.log_success(this.logger, message);
  }

  fail(message) {
    if (this.logger) consola_wasm.log_fail(this.logger, message);
  }

  setLevel(level) {
    if (this.logger) {
      this.level = level;
      consola_wasm.set_level(this.logger, level);
    }
  }

  pause() {
    if (this.logger) consola_wasm.pause(this.logger);
  }

  resume() {
    if (this.logger) consola_wasm.resume(this.logger);
  }

  destroy() {
    if (this.logger) {
      consola_wasm.free_logger(this.logger);
      this.logger = null;
    }
  }
}

// Usage
const logger = await new ConsolaLogger().init();
logger.info("Hello from wrapped logger!");
logger.destroy();
```

## Limitations in WASM

### Interactive Prompts Not Supported

The `prompt-demand` feature (interactive prompts) is **not supported** in WebAssembly builds. Attempting to call prompt methods will result in an error.

```javascript
// ❌ This will NOT work in browser/WASM:
// - text_prompt()
// - confirm_prompt()
// - select_prompt()
// - multiselect_prompt()
```

If you need user input in a browser, use standard HTML form elements or browser-native prompts instead.

### Output Goes to Browser Console

All log output in WASM is directed to the browser's console (stdout → console.log, stderr → console.error). You cannot redirect output to files or custom streams in WASM.

### Color Support

Colors are always enabled in WASM builds by default, as most browser consoles support ANSI color codes. The NO_COLOR environment variable is not checked in WASM.

## Bundle Size Optimization

For production builds, follow these recommendations:

### 1. Minimal Features

Only enable the features you need:

```toml
# In Cargo.toml
[features]
default = []  # No default features
wasm-minimal = ["wasm"]  # WASM only, no color/fancy
wasm-color = ["wasm", "color"]  # WASM with basic colors
wasm-full = ["wasm", "color", "fancy"]  # All WASM features
```

Build with minimal features:

```bash
wasm-pack build --target web --release -- \
  --no-default-features --features wasm-minimal
```

### 2. Cargo Profile Optimization

Add this to your `Cargo.toml`:

```toml
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization, slower compile
opt-level = "z"         # Optimize for size
strip = true            # Strip debug symbols
panic = "abort"         # Smaller panic handler
```

### 3. wasm-opt Post-Processing

Install `wasm-opt` (part of Binaryen):

```bash
# Ubuntu/Debian
sudo apt install binaryen

# macOS
brew install binaryen

# Or download from https://github.com/WebAssembly/binaryen/releases
```

Optimize the built WASM:

```bash
# After wasm-pack build
wasm-opt pkg/consola_bg.wasm -O4 -o pkg/consola_bg.wasm
```

### Expected Sizes

With aggressive optimization:
- Minimal (no features): ~100-150 KB
- With color: ~200-250 KB  
- With color + fancy: ~250-300 KB
- With JSON reporter: Add ~50 KB

Sizes are uncompressed. With gzip/brotli, expect 30-40% smaller.

## TypeScript Definitions

wasm-pack automatically generates TypeScript definitions in `pkg/consola.d.ts`. Import them for type safety:

```typescript
import init, { 
  WasmLogger,
  create_logger, 
  log_info,
  free_logger 
} from './pkg/consola.js';

await init();

const logger: WasmLogger = create_logger();
log_info(logger, "Type-safe logging!");
free_logger(logger);
```

## Troubleshooting

### "Module not found" or Import Errors

Make sure you've run `wasm-pack build` and the `pkg/` directory exists. Also ensure your bundler or HTML is configured to load ES modules.

### "RuntimeError: memory access out of bounds"

This usually indicates a bug. Please file an issue with reproduction steps.

### Colors Not Appearing

Most modern browser consoles support ANSI escape codes. If colors don't appear:
1. Check your browser console settings
2. Try a different browser (Chrome, Firefox, Safari, Edge all support colors)
3. Some browser extensions may strip colors

### Large Bundle Size

See the "Bundle Size Optimization" section above for tips on reducing size.

## Examples

See the `examples/wasm/` directory for complete working examples:
- `examples/wasm/basic.html` - Simple HTML page with inline script
- `examples/wasm/app.js` - ES module example
- `examples/wasm/wrapper.js` - JavaScript wrapper class

## Further Reading

- [wasm-pack documentation](https://rustwasm.github.io/wasm-pack/)
- [WebAssembly.org](https://webassembly.org/)
- [MDN WebAssembly Guide](https://developer.mozilla.org/en-US/docs/WebAssembly)
