# consola-rs Task Breakdown (Rust + WASM Port of @unjs/consola)

Purpose: Implement a Rust library offering feature parity with @unjs/consola (logging types, formatting, reporters, throttling, pause/resume, raw logs, mocking, tree/box, error formatting) with:

- Native Rust target.
- Optional WASM (browser) target (NO interactive prompts there, just an error warning if prompt methods are used).
- Native interactive prompts powered by the `demand` crate (see <https://docs.rs/demand>).
- Output & styling via `anstream` / `anstyle` (cross-platform, NO_COLOR aware).
- Optional integrations with `log` and `tracing` ecosystems (feature-gated).

No npm package. WASM usage documented (user compiles).\
This file is the authoritative actionable task list.

______________________________________________________________________

## Legend

- [ ] Unstarted
- [~] In progress
- [x] Done
- ⚠ Needs decision
- ⏩ Parallelizable
- 🧪 Testing-focused
- 🔁 Revisit / iterative
- 🐢 Post-MVP / Backlog

______________________________________________________________________

## 0. Project Initialization

1. [x] Scaffold repository (Cargo.toml, LICENSE (MIT), README stub, tasks.md (this), SPEC.md). Adopt cargo-nextest for test runs.
1. [x] Add rust-toolchain.toml (edition 2024, 1.85.0).
1. [x] Configure Cargo features:
   - default = ["color", "fancy"]
   - "color" (enables color / anstream styling)
   - "fancy" (FancyReporter + unicode-width)
   - "json" (serde + serde_json + JSON reporter)
   - "prompt-demand" (pulls `demand` crate; enables prompt subsystem)
   - "wasm" (wasm-bindgen exports; disables prompt at runtime)
   - "bridge-log" (log crate integration)
   - "bridge-tracing" (tracing subscriber layer)
   - "async-reporters" (🐢)
1. [x] Dependencies:
   - Core: anyhow, thiserror, smallvec, blake3, once_cell, anstream, anstyle, unicode-width, parking_lot
   - Optional (feature gated): serde, serde_json ("json"), demand ("prompt-demand"), wasm-bindgen ("wasm"), tracing / tracing-subscriber ("bridge-tracing"), log ("bridge-log")
1. [x] Dev deps: insta, proptest or quickcheck, criterion, assert_fs, cargo-nextest (using `cargo nextest run` by default), cargo-deny (later), wasm-bindgen-test (wasm), rstest (optional).
1. [x] CI workflow skeleton (Linux): fmt, clippy, tests.
1. [x] Add CODEOWNERS / CONTRIBUTING stub (optional MVP).
1. [x] Pre-commit config / justfile or Makefile (🐢).

______________________________________________________________________

## 1. Levels & Types

9. [x] Decide sentinel mapping for silent / verbose (chosen i16: silent=-99, verbose=99; documented in SPEC.md). Using cargo-nextest for all tests.
1. [x] Implement LogLevel (newtype with Ord).
1. [x] Default LogTypeSpec table from JS: silent, fatal, error, warn, log, info, success, fail, ready, start, box, debug, trace, verbose.
1. [x] Registration API `register_type(name, spec)` (duplicate overwrite doc).
1. [x] Map types → numeric level via table.
1. [x] Level filter normalization (user sets global level).
   🧪 Tests:

- [x] Level ordering & filter.
- [x] Default type mapping parity.
- [x] Custom type registration.

______________________________________________________________________

## 2. Record & Argument Handling

15. [x] Define LogRecord { timestamp, level, type_name, tag, args, message?, repetition_count } (partial; additional/raw/meta/error_chain_depth pending).
01. [x] ArgValue enum (String, Number, Bool, Error, OtherDebug) (Json variant pending feature).
01. [x] Normalization: object vs primitive vs error (mimic JS flexible call forms).
01. [x] Merge defaults (tag, additional, meta).
01. [x] JSON serialization (feature "json").
    🧪 Tests:

- [x] Primitive + error mix.
- [x] Default merge precedence.
- [x] JSON output attributes ordering (snapshot).

______________________________________________________________________

## 3. Throttling & Repetition

20. [x] Fingerprint (stable string join + blake3 hash).
01. [x] Config: throttle_window_ms, throttle_min_count.
01. [x] Coalescence logic (basic implementation).
01. [~] Repetition flush triggers: new fingerprint, window expire (timer), flush(), pause(), drop. (Implemented: new fingerprint, flush() public, pause()/drop flush; window expiry logic exists but lacks dedicated test)
01. [x] Suffix formatting rules (basic: " (xN)" implemented in basic builder; fancy/json pending styles/fields).
01. [x] Clock abstraction (MockClock for tests).
    🧪 Tests:

- [x] Coalesce below vs at threshold.
- [x] Window expiry flush.
- [x] Manual flush releases suppressed.
- [x] Mixed raw/formatted same fingerprint case.

______________________________________________________________________

## 4. Pause / Resume

26. [x] paused flag + queue (VecDeque<Pending>).
01. [x] pause(): buffer new inputs.
01. [x] resume(): flush suppressed group, drain queue sequentially.
01. [x] Optional queue capacity (⚠ decide: default unlimited, config limit) (implemented: drop-oldest strategy when capacity set).
01. [x] flush() public API (emits suppressed group if any).
    🧪 Tests:

- [x] Order preservation.
- [x] Throttle boundary reset on resume.
- [ ] Capacity overflow strategy (if implemented).

______________________________________________________________________

## 5. Formatting Pipeline (Core)

31. [x] Segment model (text + style metadata).
01. [x] FormatOptions { date, colors, compact, columns, error_level, unicode_mode }.
01. [~] Builder: record → segments (implemented: time, type, tag, message, repetition, additional, meta, stack basic; pending: fancy icon/badge styling, error chain depth formatting).
01. [~] Raw path bypass (basic log_raw implemented; fast assembly & optimized path pending performance tuning).
01. [x] Column width detection (from terminal; fallback).
01. [x] Width calc with unicode-width; fallback char len if disabled.
01. [x] NO_COLOR and FORCE_COLOR env respect (anstream detection).
    🧪 Tests:

- [ ] Basic vs raw snapshot.
- [ ] Width fallback when unicode feature off.
- [x] NO_COLOR strips style.

______________________________________________________________________

## 6. Utilities

38. [x] strip_ansi (using external crate `strip-ansi-escapes`).
01. [x] Alignment helpers.
01. [x] Tree formatter (depth, ellipsis).
01. [x] Box builder (unicode border fallback).
01. [x] Error stack parser (cwd + file:// removal).
01. [x] Color/style helpers wrapping anstyle (avoid direct codes).
01. [x] Stream sinks (StdoutSink, StderrSink, TestSink).
    🧪 Tests:

- [x] Tree snapshot depth limit.
- [x] Box styles (unicode vs fallback).
- [x] Error stack parse (trimming).
- [x] strip_ansi correctness.

______________________________________________________________________

## 7. BasicReporter

45. [x] Implement formatting: `[type][tag] message` (box special pending).
01. [x] Error formatting (multi-cause with depth limiting, overflow indicator).
01. [x] stderr for levels < 2 else stdout.
01. [x] Include date when enabled.
    🧪 Tests:

- [x] Single line formatting snapshot.
- [x] Box log multi-line.
- [x] Error with cause chain (basic variant).

______________________________________________________________________

## 8. FancyReporter (feature "fancy")

49. [x] Icon map + ASCII fallback (unicode detection) (icon set basic; fallback TBD).
01. [x] Badge formatting (bg color + uppercase type).
01. [x] Type/level color mapping (info=cyan, success=green, fail/fatal/error=red, warn=yellow basic implemented).
01. [x] Stack line coloring (gray "at", cyan path).
01. [x] Integration with Box (colored frame).
01. [x] Repetition suffix dim style.
01. [x] Downgrade gracefully if colors disabled (basic fallback prints plain text).
    🧪 Tests:

- [x] Fancy colored snapshot (strip_ansi for compare).
- [x] Unicode fallback snapshot.
- [x] repetition count formatting.

______________________________________________________________________

## 9. JSON Reporter (feature "json")

56. [x] Schema: { time, level, level_name, type, tag, message, args, additional, repeat?, stack?, causes?, meta?, schema:"consola-rs/v1" }.
01. [x] Serialize to single line (no trailing spaces).
01. [x] Error chain structured array (causes).
01. [x] Deterministic key order.
01. [x] Option disable time (FormatOptions.date=false).
    🧪 Tests:

- [x] Snapshot basic record.
- [x] With repetition.
- [x] Error chain serialization.

______________________________________________________________________

## 10. Error Handling & Chain

61. [x] Extract std::error::Error::source() chain w/ cycle detect (pointer set).
01. [x] Depth limit via FormatOptions.error_level.
01. [x] Format nested causes with `Caused by:` prefix.
01. [x] Multi-line message normalization (indent continuation).
01. [x] Provide structured error data to JSON reporter.
    🧪 Tests:

- [x] Depth limiting.
- [x] Cycle detection.
- [x] Multi-level nested output.

______________________________________________________________________

## 11. Prompt System (feature "prompt-demand")

66. [x] Define PromptCancelStrategy (Reject, Default, Undefined, Null, Symbol).
01. [x] PromptOutcome enum (Value(T), Undefined, NullValue, SymbolCancel, Cancelled).
01. [x] PromptProvider trait using demand crate.
01. [x] Demand adapter: text/confirm/select/multiselect mapping.
01. [x] Cancellation mapping (demand interruption → strategy).
01. [x] WASM runtime guard: calling prompt returns Err + logs console error (no interactive).
01. [ ] Provide builder `.with_prompt_provider(DefaultDemandPrompt)` only when feature active.
    🧪 Tests:

- [x] Cancellation strategy behavior.
- [x] Default fallback path.
- [x] WASM (compiled) prompt stub returns error (wasm test skip interactive).

______________________________________________________________________

## 12. WASM Integration (feature "wasm")

73. [ ] Export create*logger / free_logger / log*\* / set_level / pause / resume via wasm-bindgen.
01. [ ] JS shim example for variadic args & Error bridging.
01. [ ] Error bridging: stack + message + one-level cause (JSON if needed).
01. [ ] Provide fast path function `log_simple(type, &str)` for performance.
01. [ ] Document build instructions (`wasm-pack build --target web`).
01. [ ] Ensure prompt provider not compiled (no demand dependency) in wasm-only build.
01. [ ] Logging color detection for browsers (maybe skip; always enable?) (⚠ doc).
    🧪 Tests (wasm-bindgen-test):

- [ ] Basic log works.
- [ ] Fancy reporter formatting (if feature toggled).
- [ ] Prompt call returns error.

______________________________________________________________________

## 13. Raw Logging Path

80. [x] Per-type \*\_raw() methods + generic log_type_raw().
01. [x] Raw path still subject to level filter & throttle.
01. [x] Fingerprint strategy same as formatted (document).
    🧪 Tests:

- [x] Raw output minimal.
- [x] Raw repetition aggregated.

______________________________________________________________________

## 14. Mocking & Test Instrumentation

83. [x] set_mock(fn: Fn(&LogRecord)) before reporters.
01. [x] clear_mock().
01. [x] MemoryReporter capturing full records.
01. [x] MockClock injection for deterministic timestamps.
    🧪 Tests:

- [x] Mock intercept order.
- [x] Deterministic timestamp snapshots.

______________________________________________________________________

## 15. Config & Environment

87. [x] LoggerBuilder with defaults.
01. [~] from_env() reading: CONSOLA_LEVEL, NO_COLOR, CONSOLA_COMPACT. (CONSOLA_LEVEL implemented; NO_COLOR and CONSOLA_COMPACT in FormatOptions.adaptive())
01. [x] Precedence: builder > env > defaults.
01. [x] Option force_simple_width bool.
01. [ ] Document unstable feature toggles (async-reporters etc).
    🧪 Tests:

- [x] Env overrides.
- [x] NO_COLOR disables styling.
- [x] force_simple_width effect.

______________________________________________________________________

## 16. Integrations: log + tracing

92. [ ] (bridge-log) Implement ConsoLog (log::Log) mapping log::Level → consola type.
01. [ ] Module path/file/line into meta.
01. [ ] Recursion guard (thread local).
01. [ ] (bridge-tracing) Implement ConsoLayer (Layer<Event>) capturing fields.
01. [ ] FieldCollector merges non-message fields into meta.
01. [ ] Span stack optional (config) show `[span1/span2]` prefix (🐢 maybe).
01. [ ] Feature flags: "bridge-log", "bridge-tracing".
01. [ ] Document fingerprint inclusion of meta fields (toggle? ⚠).
    🧪 Tests:

- [ ] log crate bridge basic.
- [ ] tracing event field capture.
- [ ] Recursion safety.

______________________________________________________________________

## 17. Macros & Ergonomics

100. [x] info!(logger, "hello {user}", user=?user_id).
001. [x] warn!, error!, success!, etc.
001. [x] Raw macros info_raw! etc.
001. [x] log_type!(logger, "custom", ...).
001. [ ] Ensure macros avoid format cost if filtered (level guard).
     🧪 Tests:

- [x] Compile-time macro checks.
- [ ] Filtered-out macro short-circuits.

______________________________________________________________________

## 18. Performance & Benchmarks

105. [x] Bench scenarios: simple info, fancy info, json, high repetition, unique bursts.
001. [x] Compare raw vs formatted overhead.
001. [x] Evaluate blake3 cost; fallback to fxhash (⚠ decision after bench).
001. [x] smallvec size tuning (segments typical count).
001. [x] Preallocate String capacities (common line length).
001. [x] Document results in BENCHMARKS.md.
     🧪 Bench:

- [x] Baseline println! vs basic info.
- [x] Throttled spam scenario memory.

______________________________________________________________________

## 19. Testing & Quality

111. [ ] Snapshot tests (insta) for basic/fancy/box outputs (strip ANSI).
001. [ ] Property tests: randomized sequences (panic-free, final flush).
001. [ ] Stress test: high concurrency (if multi-threaded use demonstrated).
001. [ ] Fuzz error chain builder.
001. [ ] Wasm tests behind feature gating.
001. [x] Coverage (tarpaulin) optional summary.
001. [ ] Deterministic run repeat (two runs diff-free).
001. [x] No unwrap()/expect() outside tests (lint check).
001. [x] Unsafe code = 0 (assert).

______________________________________________________________________

## 20. Documentation

120. [x] README: features, quick start (native + wasm), examples.
001. [ ] MIGRATION.md (JS consola differences: infinite levels replaced, prompt differences, dynamic methods) (Removed - not needed).
001. [x] ARCHITECTURE.md (pipeline diagram).
001. [ ] REPORTERS.md (custom reporter guide).
001. [ ] PROMPTS.md (using demand; no WASM; cancellation mapping table).
001. [ ] INTEGRATION.md (log + tracing usage).
001. [x] FEATURE-FLAGS.md (matrix).
001. [x] BENCHMARKS.md results.
001. [ ] CHANGELOG.md (manual initial) (Removed - will be created at release time).
001. [x] CONTRIBUTING.md (workflow, MSRV).
001. [x] SECURITY.md (if needed).
001. [x] API docs check (cargo doc build, feature combos).

______________________________________________________________________

## 21. CI & Tooling

132. [x] GitHub Actions matrix: linux, macOS, windows.
001. [x] clippy & fmt enforcement.
001. [x] cargo-deny (licenses/advisories).
001. [x] nextest integration.
001. [x] wasm build job (feature "wasm", no prompt-demand).
001. [x] json feature build job.
001. [x] docs build job (cargo doc).
001. [x] Optional coverage upload (Codecov).
001. [x] Bench job (manual trigger) (🐢).
001. [x] Lint for unwrap patterns (custom script).

______________________________________________________________________

## 22. Release Prep

143. [ ] Define MVP completion (tasks 9–70, 73–84, 87–95, 100–111, 120–131, 132–139).
001. [ ] Version 0.1.0 tag.
001. [ ] Publish crate (cargo publish).
001. [ ] Post-release README badge update.
001. [ ] Feedback issue templates.

______________________________________________________________________

## 23. Backlog / Post-MVP

148. [ ] async-reporters (non-blocking sinks).
001. [ ] Ephemeral/spinner reporter.
001. [ ] Multi-sink routing rules (per-level).
001. [ ] Structured metadata redaction plugin.
001. [ ] Telemetry (trace/span correlation fields).
001. [ ] Source-map stack rewrite (WASM).
001. [ ] Plugin pre-processor chain.
001. [ ] Multi-crate workspace: core / integrations / wasm facade.
001. [ ] MessagePack / CBOR structured output.
001. [ ] Live progress / status lines (in-place update).
001. [ ] Color themes / user-config palettes.

______________________________________________________________________

## 24. Risks & Mitigations

159. [ ] Level sentinel confusion → Document mapping & convert unknown numeric to nearest.
001. [ ] Fingerprint includes meta causing over-coalescing → Provide toggle `fingerprint_meta` (default false).
001. [ ] Windows color edge cases → rely on anstream detection; add regression test.
001. [ ] WASM size bloat → enable LTO + opt-level=z instructions in docs.
001. [ ] Re-entrancy from log/tracing integration → thread local guard tests.
001. [ ] Demand crate prompt cancellation semantics drift → version pin & compatibility note.

______________________________________________________________________

## 25. Milestones

Milestone 1 Core Fundamentals: 9–24, 31–38, 45 ✅ **COMPLETED**\
Milestone 2 Formatting & Utilities: 39–44, 46–54 ✅ **COMPLETED**\
Milestone 3 Throttle/Pause/Raw: 20–30, 80–82 ✅ **COMPLETED**\
Milestone 4 Fancy & Box: 49–55 ✅ **COMPLETED**\
Milestone 5 Error & JSON: 56–64, 61–65 ✅ **COMPLETED**\
Milestone 6 Prompt & WASM: 66–72, 73–79.\
Milestone 7 Integrations: 92–99.\
Milestone 8 Macros & Performance: 100–110.\
Milestone 9 Tests & Docs: 111–131.\
Milestone 10 CI & Release: 132–147.

______________________________________________________________________

## 26. Open Decisions (⚠)

- sentinel values for silent/verbose (9).
- Include meta in fingerprint default? (160).
- Box unicode fallback style set (single-line vs extended).
- Repetition suffix style exact ANSI attributes (dim vs gray).
- Whether fancy reporter auto-detects color vs require "color" feature always (doc).
- Provide direct builder `enable_tracing()` convenience (docs only vs code).

______________________________________________________________________

## 27. Acceptance Criteria (MVP)

- All default log types functional with Basic & Fancy reporters.
- Throttling & repetition produce correct aggregated output and final counts.
- pause/resume + flush behave deterministically.
- Prompt-demand works natively; WASM calling prompt yields documented error.
- log + tracing integrations (when features enabled) route messages with correct level mapping & no recursion.
- JSON reporter (feature) stable schema, documented.
- Raw logging path preserves filtering & throttling.
- Error chain formatting (with color) matches spec; JSON structured chain present.
- Benchmarks show acceptable overhead (\<1.5x plain println for basic info).
- NO_COLOR and FORCE_COLOR behavior verified.
- Documentation set complete; CI green across matrix+MSRV; no clippy warnings.

______________________________________________________________________

(End of tasks.md)
