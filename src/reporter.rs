use crate::clock::{Clock, SystemClock};
use crate::format::{
    FormatOptions, build_basic_segments, compute_line_width, detect_terminal_width,
};
use crate::levels::LogLevel;
use crate::record::{ArgValue, LogRecord};
use crate::throttling::{ThrottleConfig, Throttler};
use std::collections::VecDeque;
use std::io::{self, Write};

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
        if width == usize::MAX || compute_line_width(&segments) <= width {
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
        // Badge formatting: find type segment like "[type]" and uppercase inside
        if self.opts.show_type {
            for s in &mut segs {
                if s.text.starts_with('[') && s.text.ends_with(']') && s.text.len() > 2 {
                    let inner = &s.text[1..s.text.len() - 1];
                    // heuristically ensure it matches record.type_name
                    if inner.eq_ignore_ascii_case(&record.type_name) {
                        s.text = format!("[{}]", inner.to_ascii_uppercase());
                        if let Some(style) = &mut s.style {
                            style.bold = true;
                            style.fg_color = Some(icon_color(record).to_string());
                        } else {
                            s.style = Some(crate::format::SegmentStyle {
                                fg_color: Some(icon_color(record).to_string()),
                                bg_color: None,
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
        if width == usize::MAX || compute_line_width(&segs) <= width {
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

fn icon_color(record: &LogRecord) -> &'static str {
    if record.level <= LogLevel::ERROR {
        "red"
    } else if record.type_name == "success" {
        "green"
    } else if record.type_name == "warn" {
        "yellow"
    } else if record.type_name == "info" {
        "cyan"
    } else {
        "magenta"
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

pub struct Logger<R: Reporter + 'static> {
    cfg: LoggerConfig,
    reporter: R,
    throttler: Throttler,
    paused: bool,
    queue: VecDeque<Pending>,
    system_clock: SystemClock,
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
