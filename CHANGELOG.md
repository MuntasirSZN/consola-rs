# Changelog

All notable changes to consola-rs will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Prompt system with demand integration (feature: `prompt-demand`)
  - `PromptCancelStrategy` enum with 5 strategies
  - `PromptOutcome` enum for result handling
  - `PromptProvider` trait for custom prompt implementations
  - `DefaultDemandPrompt` with text, confirm, select, multiselect support
  - `WasmPromptStub` for WASM targets (returns NotSupported error)
- Logging macros for ergonomic API
  - Standard macros: `info!`, `warn!`, `error!`, `success!`, `debug!`, `trace!`, `fatal!`, `ready!`, `start!`, `fail!`
  - Custom type macro: `log_type!`
  - Raw logging macros: `info_raw!`, `warn_raw!`, etc.
- Comprehensive documentation
  - Updated README with features, examples, and quick start
  - MIGRATION.md guide for JavaScript consola users
  - ARCHITECTURE.md with system design details
  - FEATURE-FLAGS.md with complete feature matrix
  - CHANGELOG.md (this file)

### Changed
- Updated project status from WIP to Alpha

### Fixed
- Clippy warnings in prompt module (unnecessary unwrap)
- Doc test failures in macro examples

## [0.0.0-alpha.0] - 2024-10-01

### Added
- Initial alpha release
- Core logging infrastructure
  - Log level system with sentinel values (SILENT=-99, VERBOSE=99)
  - Type registration with global registry
  - LogRecord structure with argument handling
  - ArgValue enum for flexible log arguments
- Throttling and deduplication
  - Message fingerprinting with blake3
  - Configurable throttle window and minimum count
  - Repetition counting and suffix formatting
  - Clock abstraction (RealClock, MockClock)
- Pause/Resume functionality
  - Log buffering via VecDeque
  - Sequential replay on resume
  - Flush API for manual control
- Formatting pipeline
  - Segment-based formatting
  - FormatOptions for customization
  - Raw logging path for performance
  - Unicode width calculation
  - NO_COLOR and FORCE_COLOR support
- Reporters
  - BasicReporter with simple formatting
  - FancyReporter with icons and colors (feature: `fancy`)
  - JsonReporter for structured output (feature: `json`)
  - Custom reporter trait
- Error handling
  - Error source chain extraction
  - Cycle detection
  - Depth limiting
  - Multi-line message normalization
- Utilities
  - Box drawing (unicode and ASCII)
  - Tree formatting with depth control
  - Text alignment helpers
  - ANSI stripping
  - Stream sinks (Stdout, Stderr, Test)
- Testing infrastructure
  - MockClock for deterministic tests
  - TestSink for capturing output
  - Snapshot tests with insta
  - Property tests with proptest
- Feature gates
  - `color`: ANSI color support (default)
  - `fancy`: Fancy reporter with icons (default)
  - `json`: JSON reporter
  - `prompt-demand`: Interactive prompts (planned)
  - `wasm`: WebAssembly support (planned)
  - `bridge-log`: log crate integration (planned)
  - `bridge-tracing`: tracing crate integration (planned)
- CI/CD
  - GitHub Actions workflow
  - Format checking
  - Clippy linting
  - Test execution
  - Cargo deny for security
- Documentation
  - README with project overview
  - SPEC.md with technical specification
  - CONTRIBUTING.md with development workflow
  - AGENTS.md for AI coding assistants
  - Comprehensive tasks.md with roadmap

### Known Issues
- LoggerBuilder not yet implemented (task 72, 104 deferred)
- Level guard optimization for macros pending (task 104)
- Prompt integration with logger builder pending (task 72)
- WASM exports not yet implemented (tasks 73-79)
- log/tracing bridges not yet implemented (tasks 92-99)
- Benchmark suite not yet created (tasks 105-110)
- Some CI jobs not yet configured (tasks 137-142)

## Version History

- `0.0.0-alpha.0` - Initial alpha release with core functionality
- More versions coming as features are completed

## Upgrade Guide

### From Pre-Alpha to Alpha

This is the first alpha release. No upgrade path needed.

## Breaking Changes Policy

During alpha (0.0.0-alpha.x):
- Breaking changes may occur without major version bump
- Changes will be documented in this changelog
- Deprecation warnings provided where possible

After 1.0:
- Breaking changes only in major versions
- Deprecation period of at least one minor version
- Clear migration guides provided

## Versioning Strategy

- **0.0.0-alpha.x**: Alpha stage, API unstable
- **0.1.0**: Beta release, API mostly stable
- **1.0.0**: First stable release, semantic versioning starts

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to contribute to this changelog.

## Links

- [Repository](https://github.com/MuntasirSZN/consola-rs)
- [Issues](https://github.com/MuntasirSZN/consola-rs/issues)
- [Documentation](https://docs.rs/consola)

[Unreleased]: https://github.com/MuntasirSZN/consola-rs/compare/v0.0.0-alpha.0...HEAD
[0.0.0-alpha.0]: https://github.com/MuntasirSZN/consola-rs/releases/tag/v0.0.0-alpha.0
