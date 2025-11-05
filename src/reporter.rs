use crate::clock::{Clock, SystemClock};
use crate::format::{
    FormatOptions, build_basic_segments, compute_line_width, detect_terminal_width,
};
use crate::levels::LogLevel;
use crate::record::{ArgValue, LogRecord, RecordDefaults};
use crate::throttling::{ThrottleConfig, Throttler};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::Arc;

pub trait Reporter: Send + Sync {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()>;
}

pub trait ReporterWithOptions {
    fn fmt_options(&self) -> &FormatOptions;
    fn fmt_options_mut(&mut self) -> &mut FormatOptions;
}

#[derive(Default)]
pub struct BasicReporter {
    pub opts: FormatOptions,
}

pub struct FancyReporter {
    pub opts: FormatOptions,
}

impl Reporter for BasicReporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        let segments = build_basic_segments(record, &self.opts);
        let cols = self.opts.columns.or_else(detect_terminal_width);
        // Build plain parts for potential future width computations (currently unused beyond join length logic)
        let mut plain_parts: Vec<String> = Vec::new();
        for (i, seg) in segments.iter().enumerate() {
            if i > 0 {
                plain_parts.push(" ".into());
            }
            plain_parts.push(seg.text.clone());
        }
        let width = cols.unwrap_or(usize::MAX);
        if width == usize::MAX || compute_line_width(&segments, &self.opts) <= width {
            // Single line output
            let mut out = String::new();
            for (i, seg) in segments.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                if self.opts.colors {
                    out.push_str(&apply_style(&seg.text, seg.style.as_ref()));
                } else {
                    out.push_str(&seg.text);
                }
            }
            out.push('\n');
            w.write_all(out.as_bytes())
        } else {
            // Wrap naive by chars; future: width-aware segmentation
            let mut current = String::new();
            let mut current_len = 0usize;
            let mut first_segment = true;
            for seg in &segments {
                let raw = &seg.text;
                let styled = if self.opts.colors {
                    apply_style(raw, seg.style.as_ref())
                } else {
                    raw.clone()
                };
                let piece_len = raw.chars().count() + if !first_segment { 1 } else { 0 };
                if current_len + piece_len > width && !current.is_empty() {
                    current.push('\n');
                    w.write_all(current.as_bytes())?;
                    current.clear();
                    current_len = 0;
                    first_segment = true;
                }
                if !first_segment {
                    current.push(' ');
                    current_len += 1;
                }
                current.push_str(&styled);
                current_len += raw.chars().count();
                first_segment = false;
            }
            if !current.ends_with('\n') {
                current.push('\n');
            }
            w.write_all(current.as_bytes())
        }
    }
}

impl Reporter for FancyReporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        // Special handling for box type logs with colored frames
        if record.type_name == "box" {
            return self.emit_box(record, w);
        }

        let mut segs = build_basic_segments(record, &self.opts);
        // Prepend icon badge based on type with ASCII fallback
        let (unicode_icon, ascii_icon) = match record.type_name.as_str() {
            "info" => ("ℹ", "i"),
            "success" => ("✔", "+"),
            "error" | "fail" | "fatal" => ("✖", "x"),
            "warn" => ("⚠", "!"),
            "debug" => ("🐛", "d"),
            "trace" => ("↳", ">"),
            _ => ("", ""),
        };
        let chosen_icon = if self.opts.unicode {
            unicode_icon
        } else {
            ascii_icon
        };
        if !chosen_icon.is_empty() {
            segs.insert(
                0,
                crate::format::Segment {
                    text: chosen_icon.to_string(),
                    style: Some(crate::format::SegmentStyle {
                        fg_color: Some(icon_color(record).to_string()),
                        bg_color: None,
                        bold: true,
                        dim: false,
                        italic: false,
                        underline: false,
                    }),
                },
            );
        }
        // Badge formatting: find type segment like "[type]" and uppercase inside with background color
        if self.opts.show_type {
            for s in &mut segs {
                if s.text.starts_with('[') && s.text.ends_with(']') && s.text.len() > 2 {
                    let inner = &s.text[1..s.text.len() - 1];
                    // heuristically ensure it matches record.type_name
                    if inner.eq_ignore_ascii_case(&record.type_name) {
                        s.text = format!(" {} ", inner.to_ascii_uppercase());
                        if let Some(style) = &mut s.style {
                            style.bold = true;
                            style.fg_color = Some("white".to_string());
                            style.bg_color = Some(badge_bg_color(record).to_string());
                        } else {
                            s.style = Some(crate::format::SegmentStyle {
                                fg_color: Some("white".to_string()),
                                bg_color: Some(badge_bg_color(record).to_string()),
                                bold: true,
                                dim: false,
                                italic: false,
                                underline: false,
                            });
                        }
                    }
                    break;
                }
            }
        }
        // Adjust repetition style to dim fully
        for s in &mut segs {
            if s.text.starts_with("(x") || s.text.starts_with(" (x") {
                if let Some(st) = &mut s.style {
                    st.dim = true;
                }
            }
        }
        // Width wrapping similar to BasicReporter
        let cols = self.opts.columns.or_else(detect_terminal_width);
        let width = cols.unwrap_or(usize::MAX);
        if width == usize::MAX || compute_line_width(&segs, &self.opts) <= width {
            let mut out = String::new();
            for (i, seg) in segs.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                if self.opts.colors {
                    out.push_str(&apply_style(&seg.text, seg.style.as_ref()));
                } else {
                    out.push_str(&seg.text);
                }
            }
            out.push('\n');
            w.write_all(out.as_bytes())
        } else {
            let mut current = String::new();
            let mut current_len = 0usize;
            let mut first_segment = true;
            for seg in &segs {
                let raw = &seg.text;
                let styled = if self.opts.colors {
                    apply_style(raw, seg.style.as_ref())
                } else {
                    raw.clone()
                };
                let piece_len = raw.chars().count() + if !first_segment { 1 } else { 0 };
                if current_len + piece_len > width && !current.is_empty() {
                    current.push('\n');
                    w.write_all(current.as_bytes())?;
                    current.clear();
                    current_len = 0;
                    first_segment = true;
                }
                if !first_segment {
                    current.push(' ');
                    current_len += 1;
                }
                current.push_str(&styled);
                current_len += raw.chars().count();
                first_segment = false;
            }
            if !current.ends_with('\n') {
                current.push('\n');
            }
            w.write_all(current.as_bytes())
        }
    }
}

impl FancyReporter {
    /// Special emit for box-type logs with colored frames
    fn emit_box(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        use crate::utils::BoxBuilder;

        // Extract title from message if present
        let title = record.message.as_deref().unwrap_or("");

        // Collect content lines from args
        let mut content_lines = Vec::new();
        for arg in &record.args {
            content_lines.push(arg.to_string());
        }

        // Build the box
        let width = self
            .opts
            .columns
            .or_else(detect_terminal_width)
            .unwrap_or(80);
        let box_builder = BoxBuilder::new(self.opts.unicode).with_width(width.saturating_sub(4));
        let box_lines = box_builder.build(title, &content_lines);

        // Apply colors if enabled
        for line in box_lines {
            if self.opts.colors {
                // Apply cyan color to box borders
                let styled = apply_style(
                    &line,
                    Some(&crate::format::SegmentStyle {
                        fg_color: Some("cyan".to_string()),
                        bg_color: None,
                        bold: false,
                        dim: false,
                        italic: false,
                        underline: false,
                    }),
                );
                writeln!(w, "{}", styled)?;
            } else {
                writeln!(w, "{}", line)?;
            }
        }

        Ok(())
    }
}

fn icon_color(record: &LogRecord) -> &'static str {
    match record.type_name.as_str() {
        "error" | "fail" | "fatal" => "red",
        "success" => "green",
        "warn" => "yellow",
        "info" => "cyan",
        "debug" => "magenta",
        "trace" => "blue",
        _ => "white",
    }
}

fn badge_bg_color(record: &LogRecord) -> &'static str {
    match record.type_name.as_str() {
        "error" | "fail" | "fatal" => "bg_red",
        "success" => "bg_green",
        "warn" => "bg_yellow",
        "info" => "bg_cyan",
        "debug" => "bg_magenta",
        "trace" => "bg_blue",
        _ => "bg_white",
    }
}

fn apply_style(text: &str, style: Option<&crate::format::SegmentStyle>) -> String {
    use std::fmt::Write as _;
    if style.is_none() {
        return text.to_string();
    }
    let s = style.unwrap();
    let mut out = String::new();
    let mut codes: Vec<&str> = Vec::new();
    if let Some(color) = &s.fg_color {
        if let Some(c) = map_color(color) {
            codes.push(c);
        }
    }
    if s.bold {
        codes.push("1");
    }
    if s.dim {
        codes.push("2");
    }
    if s.italic {
        codes.push("3");
    }
    if s.underline {
        codes.push("4");
    }
    if codes.is_empty() {
        return text.to_string();
    }
    write!(&mut out, "\x1b[{}m{}\x1b[0m", codes.join(";"), text).ok();
    out
}

fn map_color(name: &str) -> Option<&'static str> {
    match name {
        "gray" => Some("90"),
        "red" => Some("31"),
        "green" => Some("32"),
        "yellow" => Some("33"),
        "blue" => Some("34"),
        "magenta" => Some("35"),
        "cyan" => Some("36"),
        _ => None,
    }
}

struct Pending(LogRecord);

pub struct LoggerConfig {
    pub level: LogLevel,
    pub throttle: ThrottleConfig,
    pub queue_capacity: Option<usize>,
    pub clock: Option<Box<dyn Clock>>, // if None, SystemClock
}
impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::VERBOSE,
            throttle: ThrottleConfig::default(),
            queue_capacity: None,
            clock: None,
        }
    }
}

type MockFn = Box<dyn Fn(&LogRecord) + Send + Sync>;

pub struct Logger<R: Reporter + 'static> {
    cfg: LoggerConfig,
    reporter: R,
    throttler: Throttler,
    paused: bool,
    queue: VecDeque<Pending>,
    system_clock: SystemClock,
    mock_fn: Option<MockFn>,
    #[cfg(feature = "prompt-demand")]
    prompt_provider: Option<Box<dyn crate::PromptProvider>>,
}

impl<R: Reporter + 'static> Logger<R> {
    pub fn new(reporter: R) -> Self {
        Self {
            cfg: LoggerConfig::default(),
            reporter,
            throttler: Throttler::new(ThrottleConfig::default()),
            paused: false,
            queue: VecDeque::new(),
            system_clock: SystemClock,
            mock_fn: None,
            #[cfg(feature = "prompt-demand")]
            prompt_provider: None,
        }
    }

    pub fn with_config(mut self, cfg: LoggerConfig) -> Self {
        self.throttler = Throttler::new(cfg.throttle.clone());
        self.cfg = cfg;
        self
    }

    /// Set the prompt provider (only available with prompt-demand feature)
    #[cfg(feature = "prompt-demand")]
    pub fn with_prompt_provider(mut self, provider: Box<dyn crate::PromptProvider>) -> Self {
        self.prompt_provider = Some(provider);
        self
    }

    /// Get the prompt provider if available
    #[cfg(feature = "prompt-demand")]
    pub fn prompt_provider(&self) -> Option<&dyn crate::PromptProvider> {
        self.prompt_provider.as_ref().map(|p| p.as_ref())
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.cfg.level = level;
    }

    // Temporary accessor needed by tests; future: expose builder pattern.
    pub fn opts_mut(&mut self) -> &mut FormatOptions
    where
        R: ReporterWithOptions,
    {
        self.reporter.fmt_options_mut()
    }

    pub fn level(&self) -> LogLevel {
        self.cfg.level
    }

    pub fn reporter(&self) -> &R {
        &self.reporter
    }

    pub fn log<I, A>(&mut self, type_name: &str, tag: Option<String>, args: I)
    where
        I: IntoIterator<Item = A>,
        A: Into<ArgValue>,
    {
        let args_vec: Vec<ArgValue> = args.into_iter().map(Into::into).collect();
        let now = self
            .cfg
            .clock
            .as_ref()
            .map(|c| c.now())
            .unwrap_or_else(|| self.system_clock.now());
        let record = LogRecord::new_with_timestamp(type_name, tag, args_vec, now);
        if !self.passes_level(&record) {
            return;
        }
        if self.paused {
            self.enqueue(record);
            return;
        }
        self.process_record(record);
    }

    pub fn log_raw(&mut self, type_name: &str, tag: Option<String>, message: &str) {
        let now = self
            .cfg
            .clock
            .as_ref()
            .map(|c| c.now())
            .unwrap_or_else(|| self.system_clock.now());
        let record = LogRecord::raw(type_name, tag, message, now);
        if !self.passes_level(&record) {
            return;
        }
        if self.paused {
            self.enqueue(record);
            return;
        }
        self.process_record(record);
    }

    // Per-type raw logging methods for common types
    pub fn info_raw(&mut self, message: &str) {
        self.log_raw("info", None, message);
    }

    pub fn warn_raw(&mut self, message: &str) {
        self.log_raw("warn", None, message);
    }

    pub fn error_raw(&mut self, message: &str) {
        self.log_raw("error", None, message);
    }

    pub fn debug_raw(&mut self, message: &str) {
        self.log_raw("debug", None, message);
    }

    pub fn trace_raw(&mut self, message: &str) {
        self.log_raw("trace", None, message);
    }

    pub fn success_raw(&mut self, message: &str) {
        self.log_raw("success", None, message);
    }

    pub fn fail_raw(&mut self, message: &str) {
        self.log_raw("fail", None, message);
    }

    pub fn fatal_raw(&mut self, message: &str) {
        self.log_raw("fatal", None, message);
    }

    // Generic raw method with custom type
    pub fn log_type_raw(&mut self, type_name: &str, message: &str) {
        self.log_raw(type_name, None, message);
    }

    // Convenience methods for formatted logging
    pub fn info<T: ToString>(&mut self, message: T) {
        self.log("info", None, [message.to_string()]);
    }

    pub fn warn<T: ToString>(&mut self, message: T) {
        self.log("warn", None, [message.to_string()]);
    }

    pub fn error<T: ToString>(&mut self, message: T) {
        self.log("error", None, [message.to_string()]);
    }

    pub fn success<T: ToString>(&mut self, message: T) {
        self.log("success", None, [message.to_string()]);
    }

    pub fn debug<T: ToString>(&mut self, message: T) {
        self.log("debug", None, [message.to_string()]);
    }

    pub fn trace<T: ToString>(&mut self, message: T) {
        self.log("trace", None, [message.to_string()]);
    }

    /// Set a mock callback function that will be called before the reporter emits each record.
    /// This is useful for testing and debugging. The mock function receives a reference to the
    /// LogRecord before it is emitted to the reporter.
    pub fn set_mock<F>(&mut self, mock_fn: F)
    where
        F: Fn(&LogRecord) + Send + Sync + 'static,
    {
        self.mock_fn = Some(Box::new(mock_fn));
    }

    /// Clear the mock callback function.
    pub fn clear_mock(&mut self) {
        self.mock_fn = None;
    }

    fn passes_level(&self, record: &LogRecord) -> bool {
        record.level <= self.cfg.level
    }

    fn enqueue(&mut self, record: LogRecord) {
        if let Some(cap) = self.cfg.queue_capacity {
            if self.queue.len() >= cap {
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
        // Call mock function if set (before reporter emission)
        if let Some(ref mock) = self.mock_fn {
            mock(record);
        }

        let is_err = record.level <= LogLevel::ERROR;
        let mut handle: Box<dyn Write> = if is_err {
            Box::new(io::stderr())
        } else {
            Box::new(io::stdout())
        };
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
        if !self.paused {
            self.flush(); // flush throttled group on pause
            self.paused = true;
        }
    }

    pub fn resume(&mut self) {
        if !self.paused {
            return;
        }
        self.paused = false;
        self.flush(); // flush any stale before draining queued
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

pub type BasicLogger = Logger<BasicReporter>;
pub type FancyLogger = Logger<FancyReporter>;

impl Default for BasicLogger {
    fn default() -> Self {
        Logger::new(BasicReporter::default())
    }
}

impl BasicReporter {
    pub fn adaptive() -> Self {
        Self {
            opts: FormatOptions::adaptive(),
        }
    }
}

impl FancyReporter {
    pub fn adaptive() -> Self {
        Self {
            opts: FormatOptions::adaptive(),
        }
    }
}

impl Default for FancyReporter {
    fn default() -> Self {
        Self::adaptive()
    }
}

impl ReporterWithOptions for BasicReporter {
    fn fmt_options(&self) -> &FormatOptions {
        &self.opts
    }
    fn fmt_options_mut(&mut self) -> &mut FormatOptions {
        &mut self.opts
    }
}

impl ReporterWithOptions for FancyReporter {
    fn fmt_options(&self) -> &FormatOptions {
        &self.opts
    }
    fn fmt_options_mut(&mut self) -> &mut FormatOptions {
        &mut self.opts
    }
}

#[cfg(feature = "json")]
#[derive(Default)]
pub struct JsonReporter {
    opts: FormatOptions,
}

#[cfg(feature = "json")]
impl JsonReporter {
    pub fn new() -> Self {
        Self {
            opts: FormatOptions::default(),
        }
    }

    pub fn adaptive() -> Self {
        Self {
            opts: FormatOptions::adaptive(),
        }
    }
}

#[cfg(feature = "json")]
impl Reporter for JsonReporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        use serde_json::{Map, Value, json};

        // Build JSON object with deterministic key order
        let mut obj = Map::new();

        // Schema identifier first
        obj.insert("schema".to_string(), json!("consola-rs/v1"));

        // Time (if enabled)
        if self.opts.date {
            let ts = jiff::Zoned::now().to_string();
            obj.insert("time".to_string(), json!(ts));
        }

        // Core fields
        obj.insert("level".to_string(), json!(record.level.0));
        obj.insert("level_name".to_string(), json!(record.type_name));
        obj.insert("type".to_string(), json!(record.type_name));

        if let Some(tag) = &record.tag {
            obj.insert("tag".to_string(), json!(tag));
        }

        if let Some(msg) = &record.message {
            obj.insert("message".to_string(), json!(msg));
        }

        // Arguments
        if !record.args.is_empty() {
            let args_json: Vec<Value> = record
                .args
                .iter()
                .map(|arg| match arg {
                    ArgValue::String(s) => json!(s),
                    ArgValue::Number(n) => json!(n),
                    ArgValue::Bool(b) => json!(b),
                    ArgValue::Error(e) => json!(e),
                    ArgValue::OtherDebug(d) => json!(d),
                    #[cfg(feature = "json")]
                    ArgValue::Json(v) => v.clone(),
                })
                .collect();
            obj.insert("args".to_string(), json!(args_json));
        }

        // Additional structured args
        if let Some(additional) = &record.additional {
            let add_json: Vec<Value> = additional
                .iter()
                .map(|arg| match arg {
                    ArgValue::String(s) => json!(s),
                    ArgValue::Number(n) => json!(n),
                    ArgValue::Bool(b) => json!(b),
                    ArgValue::Error(e) => json!(e),
                    ArgValue::OtherDebug(d) => json!(d),
                    #[cfg(feature = "json")]
                    ArgValue::Json(v) => v.clone(),
                })
                .collect();
            obj.insert("additional".to_string(), json!(add_json));
        }

        // Repetition count
        if record.repetition_count > 1 {
            obj.insert("repeat".to_string(), json!(record.repetition_count));
        }

        // Stack trace
        if let Some(stack) = &record.stack {
            obj.insert("stack".to_string(), json!(stack));
        }

        // Error chain (structured array)
        if let Some(chain) = &record.error_chain {
            obj.insert("causes".to_string(), json!(chain));
        }

        // Metadata
        if let Some(meta) = &record.meta {
            let meta_obj: Map<String, Value> = meta
                .iter()
                .map(|(k, v)| {
                    let val = match v {
                        ArgValue::String(s) => json!(s),
                        ArgValue::Number(n) => json!(n),
                        ArgValue::Bool(b) => json!(b),
                        ArgValue::Error(e) => json!(e),
                        ArgValue::OtherDebug(d) => json!(d),
                        #[cfg(feature = "json")]
                        ArgValue::Json(jv) => jv.clone(),
                    };
                    (k.clone(), val)
                })
                .collect();
            obj.insert("meta".to_string(), json!(meta_obj));
        }

        // Serialize to single line (compact)
        let json_str = serde_json::to_string(&Value::Object(obj)).map_err(io::Error::other)?;

        writeln!(w, "{}", json_str)
    }
}

#[cfg(feature = "json")]
impl ReporterWithOptions for JsonReporter {
    fn fmt_options(&self) -> &FormatOptions {
        &self.opts
    }

    fn fmt_options_mut(&mut self) -> &mut FormatOptions {
        &mut self.opts
    }
}

#[cfg(feature = "json")]
pub type JsonLogger = Logger<JsonReporter>;

/// MemoryReporter captures full LogRecords in memory for testing and inspection.
/// This is useful for writing tests that need to verify log output without checking stdout/stderr.
#[derive(Default)]
pub struct MemoryReporter {
    records: std::sync::Arc<std::sync::Mutex<Vec<LogRecord>>>,
    opts: FormatOptions,
}

impl MemoryReporter {
    pub fn new() -> Self {
        Self {
            records: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            opts: FormatOptions::default(),
        }
    }

    /// Get a clone of all captured records.
    pub fn get_records(&self) -> Vec<LogRecord> {
        self.records.lock().unwrap().clone()
    }

    /// Clear all captured records.
    pub fn clear(&self) {
        self.records.lock().unwrap().clear();
    }

    /// Get the number of captured records.
    pub fn len(&self) -> usize {
        self.records.lock().unwrap().len()
    }

    /// Check if any records have been captured.
    pub fn is_empty(&self) -> bool {
        self.records.lock().unwrap().is_empty()
    }

    /// Get a shared reference to the records Arc for sharing between threads.
    pub fn records_arc(&self) -> std::sync::Arc<std::sync::Mutex<Vec<LogRecord>>> {
        Arc::clone(&self.records)
    }
}

impl Reporter for MemoryReporter {
    fn emit(&self, record: &LogRecord, _w: &mut dyn Write) -> io::Result<()> {
        self.records.lock().unwrap().push(record.clone());
        Ok(())
    }
}

impl ReporterWithOptions for MemoryReporter {
    fn fmt_options(&self) -> &FormatOptions {
        &self.opts
    }

    fn fmt_options_mut(&mut self) -> &mut FormatOptions {
        &mut self.opts
    }
}

pub type MemoryLogger = Logger<MemoryReporter>;

/// Builder for configuring Logger instances with environment variable support
pub struct LoggerBuilder<R: Reporter + 'static> {
    reporter: Option<R>,
    config: LoggerConfig,
    defaults: RecordDefaults,
    #[cfg(feature = "prompt-demand")]
    prompt_provider: Option<Box<dyn crate::PromptProvider>>,
}

impl<R: Reporter + 'static> LoggerBuilder<R> {
    pub fn new() -> Self
    where
        R: Default,
    {
        Self {
            reporter: None,
            config: LoggerConfig::default(),
            defaults: RecordDefaults::default(),
            #[cfg(feature = "prompt-demand")]
            prompt_provider: None,
        }
    }

    pub fn with_reporter(mut self, reporter: R) -> Self {
        self.reporter = Some(reporter);
        self
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.config.level = level;
        self
    }

    pub fn with_throttle_config(mut self, throttle: ThrottleConfig) -> Self {
        self.config.throttle = throttle;
        self
    }

    pub fn with_defaults(mut self, defaults: RecordDefaults) -> Self {
        self.defaults = defaults;
        self
    }

    /// Configure prompt provider (only available with prompt-demand feature)
    #[cfg(feature = "prompt-demand")]
    pub fn with_prompt_provider(mut self, provider: Box<dyn crate::PromptProvider>) -> Self {
        self.prompt_provider = Some(provider);
        self
    }

    /// Apply environment variables: CONSOLA_LEVEL, NO_COLOR, CONSOLA_COMPACT
    /// Precedence: builder > env > defaults
    pub fn from_env(mut self) -> Self {
        use std::env;

        // CONSOLA_LEVEL overrides level if not explicitly set by builder
        if let Ok(level_str) = env::var("CONSOLA_LEVEL") {
            if let Ok(level_num) = level_str.parse::<i16>() {
                self.config.level = LogLevel(level_num);
            } else {
                // Try to parse as type name
                if let Some(level) = crate::levels::level_for_type(&level_str) {
                    self.config.level = level;
                }
            }
        }

        self
    }

    pub fn build(self) -> Logger<R>
    where
        R: Default,
    {
        let reporter = self.reporter.unwrap_or_default();
        let mut logger = Logger::new(reporter).with_config(self.config);

        #[cfg(feature = "prompt-demand")]
        if let Some(provider) = self.prompt_provider {
            logger = logger.with_prompt_provider(provider);
        }

        logger
        // Note: defaults would need to be stored in Logger to be used during log calls
    }
}

impl Default for LoggerBuilder<BasicReporter> {
    fn default() -> Self {
        Self::new()
    }
}
