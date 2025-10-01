# consola-rs Task Breakdown (Rust + WASM Port of @unjs/consola)

Purpose: Implement a Rust library offering feature parity with @unjs/consola (logging types, formatting, reporters, throttling, pause/resume, raw logs, mocking, tree/box, error formatting) with:

- Native Rust target.
- Optional WASM (browser) target (NO interactive prompts there, just an error warning if prompt methods are used).
- Native interactive prompts powered by the `demand` crate (see <https://docs.rs/demand>).
- Output & styling via `anstream` / `anstyle` (cross-platform, NO_COLOR aware).
- Optional integrations with `log` and `tracing` ecosystems (feature-gated).

No npm package. WASM usage documented (user compiles).  
This file is the authoritative actionable task list.

---

## Legend

- [ ] Unstarted
- [~] In progress
- [x] Done
- ⚠ Needs decision
- ⏩ Parallelizable
- 🧪 Testing-focused
- 🔁 Revisit / iterative
- 🐢 Post-MVP / Backlog

---

## 0. Project Initialization

1. [x] Scaffold repository (Cargo.toml, LICENSE (MIT), README stub, tasks.md (this), SPEC.md). Adopt cargo-nextest for test runs.
2. [x] Add rust-toolchain.toml (edition 2024, 1.85.0).
3. [x] Configure Cargo features:
   - default = ["color", "fancy"]
   - "color" (enables color / anstream styling)
   - "fancy" (FancyReporter + unicode-width)
   - "json" (serde + serde_json + JSON reporter)
   - "prompt-demand" (pulls `demand` crate; enables prompt subsystem)
   - "wasm" (wasm-bindgen exports; disables prompt at runtime)
   - "bridge-log" (log crate integration)
   - "bridge-tracing" (tracing subscriber layer)
   - "async-reporters" (🐢)
4. [x] Dependencies:
   - Core: anyhow, thiserror, smallvec, blake3, once_cell, anstream, anstyle, unicode-width, parking_lot
   - Optional (feature gated): serde, serde_json ("json"), demand ("prompt-demand"), wasm-bindgen ("wasm"), tracing / tracing-subscriber ("bridge-tracing"), log ("bridge-log")
5. [x] Dev deps: insta, proptest or quickcheck, criterion, assert_fs, cargo-nextest (using `cargo nextest run` by default), cargo-deny (later), wasm-bindgen-test (wasm), rstest (optional).
6. [x] CI workflow skeleton (Linux): fmt, clippy, tests.
7. [x] Add CODEOWNERS / CONTRIBUTING stub (optional MVP).
8. [ ] Pre-commit config / justfile or Makefile (🐢).

---

## 1. Levels & Types

9. [x] Decide sentinel mapping for silent / verbose (chosen i16: silent=-99, verbose=99; documented in SPEC.md). Using cargo-nextest for all tests.
10. [x] Implement LogLevel (newtype with Ord).
11. [x] Default LogTypeSpec table from JS: silent, fatal, error, warn, log, info, success, fail, ready, start, box, debug, trace, verbose.
12. [x] Registration API `register_type(name, spec)` (duplicate overwrite doc).
13. [x] Map types → numeric level via table.
14. [x] Level filter normalization (user sets global level).
        🧪 Tests:

- [x] Level ordering & filter.
- [x] Default type mapping parity.
- [x] Custom type registration.

---

## 2. Record & Argument Handling

15. [x] Define LogRecord { timestamp, level, type_name, tag, args, message?, repetition_count } (partial; additional/raw/meta/error_chain_depth pending).
16. [x] ArgValue enum (String, Number, Bool, Error, OtherDebug) (Json variant pending feature).
17. [x] Normalization: object vs primitive vs error (mimic JS flexible call forms).
18. [x] Merge defaults (tag, additional, meta).
19. [x] JSON serialization (feature "json").
        🧪 Tests:

- [x] Primitive + error mix.
- [x] Default merge precedence.
- [x] JSON output attributes ordering (snapshot).

---

## 3. Throttling & Repetition

20. [x] Fingerprint (stable string join + blake3 hash).
21. [x] Config: throttle_window_ms, throttle_min_count.
22. [x] Coalescence logic (basic implementation).
23. [~] Repetition flush triggers: new fingerprint, window expire (timer), flush(), pause(), drop. (Implemented: new fingerprint, flush() public, pause()/drop flush; window expiry logic exists but lacks dedicated test)
24. [x] Suffix formatting rules (basic: " (xN)" implemented in basic builder; fancy/json pending styles/fields).
25. [x] Clock abstraction (MockClock for tests).
        🧪 Tests:

- [x] Coalesce below vs at threshold.
- [x] Window expiry flush.
- [x] Manual flush releases suppressed.
- [x] Mixed raw/formatted same fingerprint case.

---

## 4. Pause / Resume

26. [x] paused flag + queue (VecDeque<Pending>).
27. [x] pause(): buffer new inputs.
28. [x] resume(): flush suppressed group, drain queue sequentially.
29. [x] Optional queue capacity (⚠ decide: default unlimited, config limit) (implemented: drop-oldest strategy when capacity set).
30. [x] flush() public API (emits suppressed group if any).
        🧪 Tests:

- [x] Order preservation.
- [x] Throttle boundary reset on resume.
- [ ] Capacity overflow strategy (if implemented).

---

## 5. Formatting Pipeline (Core)

31. [x] Segment model (text + style metadata).
32. [x] FormatOptions { date, colors, compact, columns, error_level, unicode_mode }.
33. [~] Builder: record → segments (implemented: time, type, tag, message, repetition, additional, meta, stack basic; pending: fancy icon/badge styling, error chain depth formatting).
34. [~] Raw path bypass (basic log_raw implemented; fast assembly & optimized path pending performance tuning).
35. [x] Column width detection (from terminal; fallback).
36. [x] Width calc with unicode-width; fallback char len if disabled.
37. [x] NO_COLOR and FORCE_COLOR env respect (anstream detection).
        🧪 Tests:

- [ ] Basic vs raw snapshot.
- [ ] Width fallback when unicode feature off.
- [x] NO_COLOR strips style.

---

## 6. Utilities

38. [x] strip_ansi (using external crate `strip-ansi-escapes`).
39. [x] Alignment helpers.
40. [x] Tree formatter (depth, ellipsis).
41. [x] Box builder (unicode border fallback).
42. [x] Error stack parser (cwd + file:// removal).
43. [x] Color/style helpers wrapping anstyle (avoid direct codes).
44. [x] Stream sinks (StdoutSink, StderrSink, TestSink).
        🧪 Tests:

- [x] Tree snapshot depth limit.
- [x] Box styles (unicode vs fallback).
- [x] Error stack parse (trimming).
- [x] strip_ansi correctness.

---

## 7. BasicReporter

45. [x] Implement formatting: `[type][tag] message` (box special pending).
46. [x] Error formatting (multi-cause with depth limiting, overflow indicator).
47. [x] stderr for levels < 2 else stdout.
48. [x] Include date when enabled.
        🧪 Tests:

- [x] Single line formatting snapshot.
- [x] Box log multi-line.
- [x] Error with cause chain (basic variant).

---

## 8. FancyReporter (feature "fancy")

49. [x] Icon map + ASCII fallback (unicode detection) (icon set basic; fallback TBD).
50. [x] Badge formatting (bg color + uppercase type).
51. [x] Type/level color mapping (info=cyan, success=green, fail/fatal/error=red, warn=yellow basic implemented).
52. [x] Stack line coloring (gray "at", cyan path).
53. [x] Integration with Box (colored frame).
54. [x] Repetition suffix dim style.
55. [x] Downgrade gracefully if colors disabled (basic fallback prints plain text).
        🧪 Tests:

- [x] Fancy colored snapshot (strip_ansi for compare).
- [x] Unicode fallback snapshot.
- [x] repetition count formatting.

---

## 9. JSON Reporter (feature "json")

56. [x] Schema: { time, level, level_name, type, tag, message, args, additional, repeat?, stack?, causes?, meta?, schema:"consola-rs/v1" }.
57. [x] Serialize to single line (no trailing spaces).
58. [x] Error chain structured array (causes).
59. [x] Deterministic key order.
60. [x] Option disable time (FormatOptions.date=false).
        🧪 Tests:

- [x] Snapshot basic record.
- [x] With repetition.
- [x] Error chain serialization.

---

## 10. Error Handling & Chain

61. [x] Extract std::error::Error::source() chain w/ cycle detect (pointer set).
62. [x] Depth limit via FormatOptions.error_level.
63. [x] Format nested causes with `Caused by:` prefix.
64. [x] Multi-line message normalization (indent continuation).
65. [x] Provide structured error data to JSON reporter.
        🧪 Tests:

- [x] Depth limiting.
- [x] Cycle detection.
- [x] Multi-level nested output.

---

## 11. Prompt System (feature "prompt-demand")

66. [x] Define PromptCancelStrategy (Reject, Default, Undefined, Null, Symbol).
67. [x] PromptOutcome enum (Value(T), Undefined, NullValue, SymbolCancel, Cancelled).
68. [x] PromptProvider trait using demand crate.
69. [x] Demand adapter: text/confirm/select/multiselect mapping.
70. [x] Cancellation mapping (demand interruption → strategy).
71. [x] WASM runtime guard: calling prompt returns Err + logs console error (no interactive).
72. [ ] Provide builder `.with_prompt_provider(DefaultDemandPrompt)` only when feature active.
        🧪 Tests:

- [x] Cancellation strategy behavior.
- [x] Default fallback path.
- [x] WASM (compiled) prompt stub returns error (wasm test skip interactive).

---

## 12. WASM Integration (feature "wasm")

73. [ ] Export create*logger / free_logger / log*\* / set_level / pause / resume via wasm-bindgen.
74. [ ] JS shim example for variadic args & Error bridging.
75. [ ] Error bridging: stack + message + one-level cause (JSON if needed).
76. [ ] Provide fast path function `log_simple(type, &str)` for performance.
77. [ ] Document build instructions (`wasm-pack build --target web`).
78. [ ] Ensure prompt provider not compiled (no demand dependency) in wasm-only build.
79. [ ] Logging color detection for browsers (maybe skip; always enable?) (⚠ doc).
        🧪 Tests (wasm-bindgen-test):

- [ ] Basic log works.
- [ ] Fancy reporter formatting (if feature toggled).
- [ ] Prompt call returns error.

---

## 13. Raw Logging Path

80. [x] Per-type \*\_raw() methods + generic log_type_raw().
81. [x] Raw path still subject to level filter & throttle.
82. [x] Fingerprint strategy same as formatted (document).
        🧪 Tests:

- [x] Raw output minimal.
- [x] Raw repetition aggregated.

---

## 14. Mocking & Test Instrumentation

83. [x] set_mock(fn: Fn(&LogRecord)) before reporters.
84. [x] clear_mock().
85. [x] MemoryReporter capturing full records.
86. [x] MockClock injection for deterministic timestamps.
        🧪 Tests:

- [x] Mock intercept order.
- [x] Deterministic timestamp snapshots.

---

## 15. Config & Environment

87. [x] LoggerBuilder with defaults.
88. [~] from_env() reading: CONSOLA_LEVEL, NO_COLOR, CONSOLA_COMPACT. (CONSOLA_LEVEL implemented; NO_COLOR and CONSOLA_COMPACT in FormatOptions.adaptive())
89. [x] Precedence: builder > env > defaults.
90. [x] Option force_simple_width bool.
91. [ ] Document unstable feature toggles (async-reporters etc).
        🧪 Tests:

- [x] Env overrides.
- [x] NO_COLOR disables styling.
- [x] force_simple_width effect.

---

## 16. Integrations: log + tracing

92. [ ] (bridge-log) Implement ConsoLog (log::Log) mapping log::Level → consola type.
93. [ ] Module path/file/line into meta.
94. [ ] Recursion guard (thread local).
95. [ ] (bridge-tracing) Implement ConsoLayer (Layer<Event>) capturing fields.
96. [ ] FieldCollector merges non-message fields into meta.
97. [ ] Span stack optional (config) show `[span1/span2]` prefix (🐢 maybe).
98. [ ] Feature flags: "bridge-log", "bridge-tracing".
99. [ ] Document fingerprint inclusion of meta fields (toggle? ⚠).
        🧪 Tests:

- [ ] log crate bridge basic.
- [ ] tracing event field capture.
- [ ] Recursion safety.

---

## 17. Macros & Ergonomics

100. [x] info!(logger, "hello {user}", user=?user_id).
101. [x] warn!, error!, success!, etc.
102. [x] Raw macros info_raw! etc.
103. [x] log_type!(logger, "custom", ...).
104. [ ] Ensure macros avoid format cost if filtered (level guard).
         🧪 Tests:

- [x] Compile-time macro checks.
- [ ] Filtered-out macro short-circuits.

---

## 18. Performance & Benchmarks

105. [x] Bench scenarios: simple info, fancy info, json, high repetition, unique bursts.
106. [x] Compare raw vs formatted overhead.
107. [x] Evaluate blake3 cost; fallback to fxhash (⚠ decision after bench).
108. [x] smallvec size tuning (segments typical count).
109. [x] Preallocate String capacities (common line length).
110. [x] Document results in BENCHMARKS.md.
         🧪 Bench:

- [x] Baseline println! vs basic info.
- [x] Throttled spam scenario memory.

---

## 19. Testing & Quality

111. [ ] Snapshot tests (insta) for basic/fancy/box outputs (strip ANSI).
112. [ ] Property tests: randomized sequences (panic-free, final flush).
113. [ ] Stress test: high concurrency (if multi-threaded use demonstrated).
114. [ ] Fuzz error chain builder.
115. [ ] Wasm tests behind feature gating.
116. [ ] Coverage (tarpaulin) optional summary.
117. [ ] Deterministic run repeat (two runs diff-free).
118. [ ] No unwrap()/expect() outside tests (lint check).
119. [ ] Unsafe code = 0 (assert).

---

## 20. Documentation

120. [x] README: features, quick start (native + wasm), examples.
121. [x] MIGRATION.md (JS consola differences: infinite levels replaced, prompt differences, dynamic methods).
122. [x] ARCHITECTURE.md (pipeline diagram).
123. [ ] REPORTERS.md (custom reporter guide).
124. [ ] PROMPTS.md (using demand; no WASM; cancellation mapping table).
125. [ ] INTEGRATION.md (log + tracing usage).
126. [x] FEATURE-FLAGS.md (matrix).
127. [ ] BENCHMARKS.md results.
128. [x] CHANGELOG.md (manual initial).
129. [ ] CONTRIBUTING.md (workflow, MSRV).
130. [x] SECURITY.md (if needed).
131. [ ] API docs check (cargo doc build, feature combos).

---

## 21. CI & Tooling

132. [x] GitHub Actions matrix: linux, macOS, windows.
133. [x] MSRV job (deny warnings).
134. [x] clippy & fmt enforcement.
135. [x] cargo-deny (licenses/advisories).
136. [x] nextest integration.
137. [x] wasm build job (feature "wasm", no prompt-demand).
138. [x] json feature build job.
139. [x] docs build job (cargo doc).
140. [x] Optional coverage upload (Codecov).
141. [x] Bench job (manual trigger) (🐢).
142. [x] Lint for unwrap patterns (custom script).

---

## 22. Release Prep

143. [ ] Define MVP completion (tasks 9–70, 73–84, 87–95, 100–111, 120–131, 132–139).
144. [ ] Version 0.1.0 tag.
145. [ ] Publish crate (cargo publish) (if public).
146. [ ] Post-release README badge update.
147. [ ] Feedback issue templates.

---

## 23. Backlog / Post-MVP

148. [ ] async-reporters (non-blocking sinks).
149. [ ] Ephemeral/spinner reporter.
150. [ ] Multi-sink routing rules (per-level).
151. [ ] Structured metadata redaction plugin.
152. [ ] Telemetry (trace/span correlation fields).
153. [ ] Source-map stack rewrite (WASM).
154. [ ] Plugin pre-processor chain.
155. [ ] Multi-crate workspace: core / integrations / wasm facade.
156. [ ] MessagePack / CBOR structured output.
157. [ ] Live progress / status lines (in-place update).
158. [ ] Color themes / user-config palettes.

---

## 24. Risks & Mitigations

159. [ ] Level sentinel confusion → Document mapping & convert unknown numeric to nearest.
160. [ ] Fingerprint includes meta causing over-coalescing → Provide toggle `fingerprint_meta` (default false).
161. [ ] Windows color edge cases → rely on anstream detection; add regression test.
162. [ ] WASM size bloat → enable LTO + opt-level=z instructions in docs.
163. [ ] Re-entrancy from log/tracing integration → thread local guard tests.
164. [ ] Demand crate prompt cancellation semantics drift → version pin & compatibility note.
165. [ ] Performance regression → baseline lock & compare before release.

---

## 25. Milestones

Milestone 1 Core Fundamentals: 9–24, 31–38, 45 ✅ **COMPLETED**  
Milestone 2 Formatting & Utilities: 39–44, 46–54 ✅ **COMPLETED**  
Milestone 3 Throttle/Pause/Raw: 20–30, 80–82 ✅ **COMPLETED**  
Milestone 4 Fancy & Box: 49–55 ✅ **COMPLETED**  
Milestone 5 Error & JSON: 56–64, 61–65 ✅ **COMPLETED**  
Milestone 6 Prompt & WASM: 66–72, 73–79.  
Milestone 7 Integrations: 92–99.  
Milestone 8 Macros & Performance: 100–110.  
Milestone 9 Tests & Docs: 111–131.  
Milestone 10 CI & Release: 132–147.

---

## 26. Open Decisions (⚠)

- sentinel values for silent/verbose (9).
- Include meta in fingerprint default? (160).
- Box unicode fallback style set (single-line vs extended).
- Repetition suffix style exact ANSI attributes (dim vs gray).
- Whether fancy reporter auto-detects color vs require "color" feature always (doc).
- Provide direct builder `enable_tracing()` convenience (docs only vs code).

---

## 27. Acceptance Criteria (MVP)

- All default log types functional with Basic & Fancy reporters.
- Throttling & repetition produce correct aggregated output and final counts.
- pause/resume + flush behave deterministically.
- Prompt-demand works natively; WASM calling prompt yields documented error.
- log + tracing integrations (when features enabled) route messages with correct level mapping & no recursion.
- JSON reporter (feature) stable schema, documented.
- Raw logging path preserves filtering & throttling.
- Error chain formatting (with color) matches spec; JSON structured chain present.
- Benchmarks show acceptable overhead (<1.5x plain println for basic info).
- NO_COLOR and FORCE_COLOR behavior verified.
- Documentation set complete; CI green across matrix+MSRV; no clippy warnings.

---

(End of tasks.md)
