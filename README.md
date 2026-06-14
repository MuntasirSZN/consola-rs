# consola-rs

An elegant console logger for Rust, ported from [consola-js](https://github.com/unjs/consola) v3.4.2.

[![Crates.io](https://img.shields.io/crates/v/consola.svg)](https://crates.io/crates/consola)
[![CI](https://github.com/MuntasirSZN/consola-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/MuntasirSZN/consola-rs/actions/workflows/ci.yml)

---

## Quick Start

```toml
[dependencies]
consola = "0.1.0"
```

```rust
use consola::create_fancy_consola;

let consola = create_fancy_consola(None);
consola.info("Server starting...");
consola.start("Building project");
consola.success("Build complete");
consola.warn("Deprecated API detected");
consola.error("Connection refused");
```

## Log Types

14 log types with distinct styling:

| Level | Method     | Icon | Use                       |
|-------|------------|------|---------------------------|
| 0     | `.fatal()` | ✖    | Fatal error               |
| 0     | `.error()` | ✖    | Error                     |
| 1     | `.warn()`  | ⚠    | Warning                   |
| 2     | `.log()`   |      | General message           |
| 3     | `.info()`  | ℹ    | Informational             |
| 3     | `.success()`| ✔   | Success                   |
| 3     | `.fail()`  | ✖    | Failure                   |
| 3     | `.ready()` | ✔    | Ready state               |
| 3     | `.start()` | ◐    | Operation started         |
| 3     | `.box_()`  |      | Bordered box output       |
| 4     | `.debug()` | ⚙    | Debug message             |
| 5     | `.trace()` | →    | Trace message             |
| 5     | `.verbose()`|     | Verbose message           |

Each method has a `_raw` variant (e.g. `info_raw()`) that skips formatting.

```rust
consola.info_raw("raw json: {\"key\": \"value\"}");
```

### Setting log level

```rust
use consola::{create_fancy_consola, log_levels};

let consola = create_fancy_consola(Some(log_levels::VERBOSE));
consola.trace("only visible at VERBOSE or above");
```

Levels 0–5, where 0 = fatal only, 5 = verbose. `set_level()` at runtime:

```rust
consola.set_level(log_levels::DEBUG);
```

## Reporters

Two built-in reporters, both implementing the `Reporter` trait:

- **`FancyReporter`** — colored output with Unicode icons, backtick highlighting, underline, and badges (default)
- **`BasicReporter`** — plain text, suitable for CI or non-terminal output

```rust
use consola::{create_basic_consola, reporters::BasicReporter};

// Basic only
let consola = create_basic_consola(None);

// Add a reporter to an existing instance
consola.add_reporter(Box::new(BasicReporter));

// Replace all reporters
consola.set_reporters(vec![Box::new(FancyReporter::new())]);
```

### Custom reporters

```rust
use consola::{Reporter, LogObject, LogContext, ConsolaOpts};

#[derive(Debug, Clone)]
struct JsonReporter;
impl Reporter for JsonReporter {
    fn format(&self, log_obj: &LogObject, _ctx: &LogContext) -> Result<String, String> {
        // Return empty string to suppress output for certain entries
        Ok(serde_json::to_string(log_obj).unwrap_or_default())
    }
    fn clone_box(&self) -> Box<dyn Reporter> {
        Box::new(self.clone())
    }
}
```

## Tagged Logging

Scope logs with tags. Multiple tags are joined with `:`.

```rust
let http = consola.with_tag("http");
http.info("GET /api/users 200 OK");
http.warn("POST /api/login 429");

let svc = consola.with_tag("api").with_tag("v1");
svc.error("POST /api/v1/checkout 400");
```

## Structured Logs

Build a log entry from `LogObjectInput`:

```rust
use consola::{LogObjectInput, LogType};

consola.log_obj(
    &LogObjectInput::new()
        .type_(LogType::Info)
        .tag("db")
        .message("Query completed")
        .arg("duration_ms: 42"),
);
```

The builder provides: `type_()`, `tag()`, `message()`, `args()`, `arg()`, `additional()`, `title()`.

## Pause / Resume

Queues messages while paused, flushes on resume.

```rust
consola.pause_logs();
consola.info("queued");
consola.info("also queued");
consola.resume_logs(); // both flush
```

## Instance Derivation

Create a derived instance with overrides:

```rust
let child = consola.create(ConsolaOpts {
    level: log_levels::WARN,
    ..ConsolaOptions::default()
});

let verbose = consola.with_defaults(
    LogObjectInput::new().tag("background")
);
```

## Log & Tracing Integration

Enable with feature flags:

```rust
// install as log sink
log::set_boxed_logger(consola.clone())?;
log::set_max_level(log::LevelFilter::Trace);

// install as tracing subscriber
tracing::subscriber::set_global_default(consola.clone())?;
```

## Browser (WASM) Support

```rust
// wasm-pack test --headless --chrome --features browser
consola.info("works in the browser console");
```

## Interactive Prompts

```toml
[dependencies]
consola = { version = "0.1.0", features = ["prompt"] }
```

```rust
use consola::prompt::{self, TextPromptOptions, PromptCommonOptions};
let name = prompt::text("What is your name?", &TextPromptOptions {
    common: PromptCommonOptions { cancel: None },
    r#type: None,
    default: Some("world".into()),
    placeholder: Some("Enter name".into()),
    initial: None,
}).unwrap_or_else(|e| e); // K_CANCEL on Ctrl+C

let ok = prompt::confirm("Continue?", &ConfirmPromptOptions {
    common: PromptCommonOptions { cancel: None },
    r#type: "confirm".into(),
    initial: Some(true),
});

let choice = prompt::select("Pick one", &SelectPromptOptions {
    common: PromptCommonOptions { cancel: None },
    r#type: "select".into(),
    initial: None,
    options: vec![
        consola::SelectOption { label: "Alpha".into(), value: "a".into(), hint: None },
        consola::SelectOption { label: "Beta".into(), value: "b".into(), hint: None },
    ],
});
```

## Configuration

`ConsolaOptions` fields:

| Field           | Default                     | Description                              |
|-----------------|-----------------------------|------------------------------------------|
| `reporters`     | `vec![]`                    | Active reporters                         |
| `level`         | `log_levels::INFO` (3)      | Minimum log level                        |
| `defaults`      | `LogObjectInput::default()` | Defaults applied to every entry          |
| `throttle`      | `1000`                      | Min interval (ms) between duplicates     |
| `throttle_min`  | `5`                         | Min occurrences before throttling starts |
| `format_options`| `FormatOptions::default()`  | Formatting behavior                      |

`FormatOptions` fields:

| Field         | Default                    | Description                    |
|---------------|----------------------------|--------------------------------|
| `columns`     | terminal width or `None`   | Output column width            |
| `date`        | `true`                     | Include timestamp              |
| `colors`      | `false`                    | ANSI color codes               |
| `compact`     | `true`                     | Single-line format             |
| `error_level` | `0`                        | Max level for stack traces     |

## Feature Flags

| Feature    | Default  | Description                                    |
|------------|----------|------------------------------------------------|
| `jiff`     | yes      | Timestamps via `jiff`                          |
| `backtrace`| yes      | Error backtrace capture via `backtrace` crate  |
| `chrono`   | no       | Alternative timestamps via `chrono`            |
| `time`     | no       | Alternative timestamps via `time`              |
| `log`      | no       | Implement `log::Log` trait                     |
| `tracing`  | no       | Implement `tracing::Subscriber`                |
| `browser`  | no       | WASM browser console integration               |
| `parking_lot` | no    | Use `parking_lot::Mutex` instead of `std::sync::Mutex` |
| `prompt`   | no       | Interactive prompts (`text`, `confirm`, etc.) (demand backend)  |
| `prompt-inquire`   | no       | Interactive prompts (`text`, `confirm`, etc.) (inquire backend)  |
| `prompt-dialoguer`   | no       | Interactive prompts (`text`, `confirm`, etc.) (dialoguer backend)  |

## Thread Safety

`Consola` is `Send + Sync`. All methods take `&self`. Internal state is protected by `parking_lot::Mutex`.

## Safety

The crate uses `#![deny(unsafe_code)]` — no `unsafe` in the core library.

## Minimum Rust Version

Rust 1.96 (edition 2024).

## License

MIT
