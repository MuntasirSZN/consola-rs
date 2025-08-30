//! consola-rs core library (stages 0-5 groundwork)
//!
//! Modules currently implemented:
//! - levels/types registry
//! - record + arg handling (basic)
//! - throttling (basic coalescence)
//! - logger core with pause/resume & level filtering
//! - basic reporter (minimal formatting)
//! - utils (strip ansi placeholder)
//! - formatting stubs (to be expanded)
//! - error chain extraction stub
//!
//! This is incremental per `tasks.md`.

use blake3::Hasher;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::fmt;
use std::io::{self, Write};
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub mod utils {
    /// Strip ANSI escape sequences using the `strip-ansi-escapes` crate (robust implementation).
    pub fn strip_ansi(input: &str) -> String {
        let bytes = strip_ansi_escapes::strip(input);
        String::from_utf8(bytes).unwrap_or_else(|_| input.to_string())
    }
}

pub mod error_chain {
    use std::collections::HashSet;
    use std::error::Error;

    pub fn collect_chain(err: &(dyn Error + 'static)) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen: HashSet<usize> = HashSet::new();
        let mut cur: Option<&(dyn Error + 'static)> = Some(err);
        while let Some(e) = cur {
            // Obtain a stable address for cycle detection (cast via *const () first)
            let ptr = (e as *const dyn std::error::Error) as *const () as usize;
            if !seen.insert(ptr) {
                break;
            }
            out.push(e.to_string());
            cur = e.source();
        }
        out
    }
}

pub mod format {
    use super::LogRecord;
    #[derive(Debug, Clone)]
    pub struct FormatOptions {
        pub date: bool,
        pub colors: bool,
        pub compact: bool,
        pub columns: Option<usize>,
        pub error_level: usize,
        pub unicode: bool,
        pub show_tag: bool,
        pub show_type: bool,
        pub show_repetition: bool,
        pub show_stack: bool,
    }
    impl Default for FormatOptions {
        fn default() -> Self {
            Self {
                date: true,
                colors: true,
                compact: false,
                columns: None,
                error_level: 16,
                unicode: true,
                show_tag: true,
                show_type: true,
                show_repetition: true,
                show_stack: false,
            }
        }
    }
    #[derive(Debug, Clone)]
    pub struct Segment {
        pub text: String,
        pub style: Option<SegmentStyle>,
    }

    #[derive(Debug, Clone)]
    pub struct SegmentStyle {
        pub fg_color: Option<String>,
        pub bg_color: Option<String>,
        pub bold: bool,
        pub dim: bool,
        pub italic: bool,
        pub underline: bool,
    }
    pub fn build_basic_segments(record: &LogRecord, opts: &FormatOptions) -> Vec<Segment> {
        let mut v = Vec::new();
        if opts.date {
            let ts = {
                #[allow(unused)]
                {
                    // Use jiff crate for local timestamp; fall back to debug if API changes.
                    let z = jiff::Zoned::now();
                    z.to_string()
                }
            };
            v.push(Segment {
                text: ts,
                style: Some(SegmentStyle {
                    fg_color: Some("gray".into()),
                    bg_color: None,
                    bold: false,
                    dim: true,
                    italic: false,
                    underline: false,
                }),
            });
        }
        if opts.show_type {
            v.push(Segment {
                text: format!("[{}]", record.type_name),
                style: Some(SegmentStyle {
                    fg_color: Some("cyan".into()),
                    bg_color: None,
                    bold: true,
                    dim: false,
                    italic: false,
                    underline: false,
                }),
            });
        }
        if opts.show_tag {
            if let Some(tag) = &record.tag {
                v.push(Segment {
                    text: format!("[{tag}]"),
                    style: Some(SegmentStyle {
                        fg_color: Some("magenta".into()),
                        bg_color: None,
                        bold: false,
                        dim: false,
                        italic: true,
                        underline: false,
                    }),
                });
            }
        }
        if let Some(msg) = &record.message {
            v.push(Segment {
                text: msg.clone(),
                style: None,
            });
        }
        if opts.show_repetition && record.repetition_count > 1 {
            v.push(Segment {
                text: format!(" (x{})", record.repetition_count),
                style: Some(SegmentStyle {
                    fg_color: Some("gray".into()),
                    bg_color: None,
                    bold: false,
                    dim: true,
                    italic: false,
                    underline: false,
                }),
            });
        }
        v
    }
}

// ---------------- Levels & Types (Stage 1) ----------------

/// Sentinel / numeric log levels.
/// Ordering: lower is more severe.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LogLevel(pub i16);

impl LogLevel {
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
}

/// Specification for a log type.
#[derive(Debug, Clone)]
pub struct LogTypeSpec {
    pub level: LogLevel,
}

static TYPE_REGISTRY: Lazy<RwLock<Vec<(String, LogTypeSpec)>>> = Lazy::new(|| {
    let mut v = Vec::new();
    // default mapping parity attempt
    for (name, level) in [
        ("silent", LogLevel::SILENT),
        ("fatal", LogLevel::FATAL),
        ("error", LogLevel::ERROR),
        ("warn", LogLevel::WARN),
        ("log", LogLevel::LOG),
        ("info", LogLevel::INFO),
        ("success", LogLevel::SUCCESS),
        ("fail", LogLevel::SUCCESS), // alias to success level for now
        ("ready", LogLevel::INFO),
        ("start", LogLevel::LOG),
        ("box", LogLevel::LOG),
        ("debug", LogLevel::DEBUG),
        ("trace", LogLevel::TRACE),
        ("verbose", LogLevel::VERBOSE),
    ] {
        v.push((name.to_string(), LogTypeSpec { level }));
    }
    RwLock::new(v)
});

/// Register or overwrite a log type.
pub fn register_type(name: &str, spec: LogTypeSpec) {
    let mut guard = TYPE_REGISTRY.write().unwrap();
    if let Some(existing) = guard.iter_mut().find(|(n, _)| n == name) {
        *existing = (name.to_string(), spec);
    } else {
        guard.push((name.to_string(), spec));
    }
}

pub fn level_for_type(name: &str) -> Option<LogLevel> {
    let guard = TYPE_REGISTRY.read().unwrap();
    guard.iter().find(|(n, _)| n == name).map(|(_, s)| s.level)
}

// --------------- Level Filter Normalization ---------------

/// Normalize a user provided level filter input (string or numeric string) into a LogLevel.
/// Accepts known type names ("info", etc.) or raw integer values.
pub fn normalize_level(input: &str) -> Option<LogLevel> {
    if let Ok(num) = input.parse::<i16>() {
        return Some(LogLevel(num));
    }
    level_for_type(input)
}

// --------------- Record & Argument Handling (Stage 2) ---------------

#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    String(String),
    Number(f64),
    Bool(bool),
    Error(String), // placeholder; later structured chain
    OtherDebug(String),
}

impl From<&str> for ArgValue {
    fn from(s: &str) -> Self {
        ArgValue::String(s.to_string())
    }
}
impl From<String> for ArgValue {
    fn from(s: String) -> Self {
        ArgValue::String(s)
    }
}
impl From<bool> for ArgValue {
    fn from(b: bool) -> Self {
        ArgValue::Bool(b)
    }
}
impl From<f64> for ArgValue {
    fn from(n: f64) -> Self {
        ArgValue::Number(n)
    }
}
impl From<i64> for ArgValue {
    fn from(n: i64) -> Self {
        ArgValue::Number(n as f64)
    }
}
impl From<u64> for ArgValue {
    fn from(n: u64) -> Self {
        ArgValue::Number(n as f64)
    }
}

impl fmt::Display for ArgValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgValue::String(s) => write!(f, "{s}"),
            ArgValue::Number(n) => write!(f, "{n}"),
            ArgValue::Bool(b) => write!(f, "{b}"),
            ArgValue::Error(e) => write!(f, "{e}"),
            ArgValue::OtherDebug(d) => write!(f, "{d}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogRecord {
    pub timestamp: Instant,
    pub level: LogLevel,
    pub type_name: String,
    pub tag: Option<String>,
    pub args: Vec<ArgValue>,
    pub message: Option<String>,
    pub repetition_count: u32,
}

impl LogRecord {
    pub fn new(type_name: &str, tag: Option<String>, args: Vec<ArgValue>) -> Self {
        let level = level_for_type(type_name).unwrap_or(LogLevel::LOG);
        let message = build_message(&args);
        Self {
            timestamp: Instant::now(),
            level,
            type_name: type_name.to_string(),
            tag,
            args,
            message,
            repetition_count: 0,
        }
    }
}

fn build_message(args: &[ArgValue]) -> Option<String> {
    if args.is_empty() {
        return None;
    }
    let mut out = String::new();
    for (i, a) in args.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&a.to_string());
    }
    Some(out)
}

// --------------- Throttling Skeleton (Stage 3) ---------------

#[derive(Debug, Clone)]
pub struct ThrottleConfig {
    pub window: Duration,
    pub min_count: u32,
}
impl Default for ThrottleConfig {
    fn default() -> Self {
        Self {
            window: Duration::from_millis(500),
            min_count: 2,
        }
    }
}

#[derive(Debug)]
struct ThrottleState {
    current_fp: Option<[u8; 32]>,
    first_time: Option<Instant>,
    count: u32,
    stored: Option<LogRecord>,
    emitted: bool,
}

impl ThrottleState {
    fn new() -> Self {
        Self {
            current_fp: None,
            first_time: None,
            count: 0,
            stored: None,
            emitted: false,
        }
    }
    fn reset(&mut self) {
        self.current_fp = None;
        self.first_time = None;
        self.count = 0;
        self.stored = None;
        self.emitted = false;
    }
}

pub struct Throttler {
    cfg: ThrottleConfig,
    state: ThrottleState,
}

impl Throttler {
    pub fn new(cfg: ThrottleConfig) -> Self {
        Self {
            cfg,
            state: ThrottleState::new(),
        }
    }

    pub fn fingerprint(record: &LogRecord) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(record.type_name.as_bytes());
        if let Some(tag) = &record.tag {
            hasher.update(tag.as_bytes());
        }
        hasher.update(&record.level.0.to_le_bytes());
        if let Some(msg) = &record.message {
            hasher.update(msg.as_bytes());
        }
        for a in &record.args {
            hasher.update(format!("{a:?}").as_bytes());
        }
        *hasher.finalize().as_bytes()
    }

    pub fn on_record<F>(&mut self, mut record: LogRecord, mut emit: F)
    where
        F: FnMut(&LogRecord),
    {
        let fp = Self::fingerprint(&record);
        let now = record.timestamp;
        if let (Some(_), Some(first)) = (self.state.current_fp, self.state.first_time) {
            if now.duration_since(first) > self.cfg.window && self.state.count > 0 {
                self.flush_inner(true, &mut emit);
            }
        }
        match self.state.current_fp {
            Some(current) if current == fp => {
                self.state.count += 1;
                if let Some(stored) = &mut self.state.stored {
                    stored.repetition_count = self.state.count;
                }
                if self.state.count == self.cfg.min_count {
                    if let Some(stored) = &self.state.stored {
                        emit(stored);
                    }
                    self.state.emitted = true;
                }
                return;
            }
            Some(_) => {
                self.flush_inner(true, &mut emit);
            }
            None => {}
        }
        self.state.current_fp = Some(fp);
        self.state.first_time = Some(now);
        self.state.count = 1;
        record.repetition_count = 1;
        self.state.stored = Some(record);
        if let Some(stored) = &self.state.stored {
            emit(stored);
            self.state.emitted = true;
        }
    }

    fn flush_inner<F>(&mut self, forced: bool, emit: &mut F)
    where
        F: FnMut(&LogRecord),
    {
        if let Some(stored) = &self.state.stored {
            if forced && (self.state.count > 1 || !self.state.emitted) {
                emit(stored);
            }
        }
        self.state.reset();
    }

    pub fn flush<F>(&mut self, mut emit: F)
    where
        F: FnMut(&LogRecord),
    {
        self.flush_inner(true, &mut emit);
    }
}

// --------------- Pause / Resume & Logger Core (Stage 4 + integration) ---------------

/// Reporter trait (minimal for now). Future reporters can implement advanced formatting.
pub trait Reporter: Send + Sync {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()>;
}

/// BasicReporter: `[type][tag] message` with repetition suffix.
pub struct BasicReporter {
    pub opts: crate::format::FormatOptions,
}
impl Default for BasicReporter { fn default() -> Self { Self { opts: crate::format::FormatOptions::default() } } }
impl Reporter for BasicReporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        let segments = crate::format::build_basic_segments(record, &self.opts);
        let mut line = String::new();
        for (i, seg) in segments.iter().enumerate() {
            if i > 0 { line.push(' '); }
            line.push_str(&seg.text);
        }
        line.push('\n');
        w.write_all(line.as_bytes())
    }
}

/// Pending item stored during pause.
struct Pending(LogRecord);

pub struct LoggerConfig {
    pub level: LogLevel,
    pub throttle: ThrottleConfig,
    pub queue_capacity: Option<usize>,
}
impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::VERBOSE, // show everything by default
            throttle: ThrottleConfig::default(),
            queue_capacity: None,
        }
    }
}

pub struct Logger<R: Reporter + 'static> {
    cfg: LoggerConfig,
    reporter: R,
    throttler: Throttler,
    paused: bool,
    queue: VecDeque<Pending>,
}

impl<R: Reporter + 'static> Logger<R> {
    pub fn new(reporter: R) -> Self {
        Self {
            cfg: LoggerConfig::default(),
            reporter,
            throttler: Throttler::new(ThrottleConfig::default()),
            paused: false,
            queue: VecDeque::new(),
        }
    }

    pub fn with_config(mut self, cfg: LoggerConfig) -> Self {
        self.throttler = Throttler::new(cfg.throttle.clone());
        self.cfg = cfg;
        self
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.cfg.level = level;
    }
    pub fn level(&self) -> LogLevel {
        self.cfg.level
    }

    /// Public logging entrypoint.
    pub fn log<I, A>(&mut self, type_name: &str, tag: Option<String>, args: I)
    where
        I: IntoIterator<Item = A>,
        A: Into<ArgValue>,
    {
        let args_vec: Vec<ArgValue> = args.into_iter().map(Into::into).collect();
        let record = LogRecord::new(type_name, tag, args_vec);
        if !self.passes_level(&record) {
            return;
        }
        if self.paused {
            self.enqueue(record);
            return;
        }
        self.process_record(record);
    }

    fn passes_level(&self, record: &LogRecord) -> bool {
        record.level <= self.cfg.level
    }

    fn enqueue(&mut self, record: LogRecord) {
        if let Some(cap) = self.cfg.queue_capacity {
            if self.queue.len() >= cap {
                // simple overflow strategy: drop oldest
                self.queue.pop_front();
            }
        }
        self.queue.push_back(Pending(record));
    }

    fn process_record(&mut self, record: LogRecord) {
        let mut to_emit: Vec<LogRecord> = Vec::new();
        self.throttler
            .on_record(record, |r| to_emit.push(r.clone()));
        for rec in to_emit {
            self.emit(&rec);
        }
    }

    fn emit(&self, record: &LogRecord) {
        // choose stderr for level < WARN (i.e., fatal/error)
        let is_err = record.level <= LogLevel::ERROR;
        let mut handle: Box<dyn Write> = if is_err {
            Box::new(io::stderr())
        } else {
            Box::new(io::stdout())
        };
        // ignore errors silently for now (no panic). TODO: surface?
        let _ = self.reporter.emit(record, &mut *handle);
    }

    pub fn flush(&mut self) {
        let mut to_emit: Vec<LogRecord> = Vec::new();
        self.throttler.flush(|r| to_emit.push(r.clone()));
        for rec in to_emit {
            self.emit(&rec);
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }
    pub fn resume(&mut self) {
        if !self.paused {
            return;
        }
        self.paused = false;
        while let Some(Pending(rec)) = self.queue.pop_front() {
            if !self.passes_level(&rec) {
                continue;
            }
            self.process_record(rec);
        }
    }
}

impl<R: Reporter + 'static> Drop for Logger<R> {
    fn drop(&mut self) {
        self.flush();
    }
}

// Convenience type alias with BasicReporter
pub type BasicLogger = Logger<BasicReporter>;

impl Default for BasicLogger {
    fn default() -> Self {
    Logger::new(BasicReporter::default())
    }
}

// --------------- Tests (basic) ---------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_ordering() {
        assert!(LogLevel::FATAL < LogLevel::ERROR);
        assert!(LogLevel::TRACE < LogLevel::VERBOSE);
        assert!(LogLevel::SILENT < LogLevel::FATAL);
    }

    #[test]
    fn default_type_mapping() {
        assert_eq!(level_for_type("info"), Some(LogLevel::INFO));
        assert_eq!(level_for_type("trace"), Some(LogLevel::TRACE));
        assert_eq!(level_for_type("verbose"), Some(LogLevel::VERBOSE));
    }

    #[test]
    fn custom_type_registration() {
        register_type(
            "custom",
            LogTypeSpec {
                level: LogLevel(42),
            },
        );
        assert_eq!(level_for_type("custom"), Some(LogLevel(42)));
    }

    #[test]
    fn record_message_building() {
        let r = LogRecord::new("info", None, vec!["hello".into(), 5i64.into(), true.into()]);
        assert_eq!(r.message.as_deref(), Some("hello 5 true"));
    }

    #[test]
    fn throttle_basic() {
        let mut throttler = Throttler::new(ThrottleConfig {
            window: Duration::from_millis(200),
            min_count: 3,
        });
        let mut emitted: Vec<LogRecord> = Vec::new();
        throttler.on_record(LogRecord::new("info", None, vec!["x".into()]), |r| {
            emitted.push(r.clone())
        });
        assert_eq!(emitted.len(), 1); // first
        throttler.on_record(LogRecord::new("info", None, vec!["x".into()]), |r| {
            emitted.push(r.clone())
        });
        assert_eq!(emitted.len(), 1); // suppressed
        throttler.on_record(LogRecord::new("info", None, vec!["x".into()]), |r| {
            emitted.push(r.clone())
        });
        assert_eq!(emitted.len(), 2); // aggregated
        assert_eq!(emitted[1].repetition_count, 3);
    }

    #[test]
    fn level_filtering() {
        let mut logger = BasicLogger::default().with_config(super::LoggerConfig {
            level: LogLevel::INFO,
            throttle: ThrottleConfig::default(),
            queue_capacity: None,
        });
        // debug should not pass (6 > 4)
        logger.log("debug", None, ["hidden"]);
        // info should pass
        logger.log("info", None, ["shown"]);
    }

    #[test]
    fn pause_resume_order() {
        let mut logger = BasicLogger::default();
        logger.pause();
        logger.log("info", None, ["a"]);
        logger.log("info", None, ["b"]);
        logger.resume();
        // Order should be preserved (manual inspection; advanced test would capture output with custom reporter)
    }

    #[test]
    fn strip_ansi_basic() {
        let colored = "\u{1b}[31mred\u{1b}[0m text";
        let stripped = crate::utils::strip_ansi(colored);
        assert_eq!(stripped, "red text");
    }
}
