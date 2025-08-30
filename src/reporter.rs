use crate::format::{FormatOptions, build_basic_segments};
use crate::levels::LogLevel;
use crate::record::{ArgValue, LogRecord};
use crate::throttling::{ThrottleConfig, Throttler};
use std::collections::VecDeque;
use std::io::{self, Write};
pub trait Reporter: Send + Sync {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()>;
}
#[derive(Default)]
pub struct BasicReporter {
    pub opts: FormatOptions,
}
impl Reporter for BasicReporter {
    fn emit(&self, record: &LogRecord, w: &mut dyn Write) -> io::Result<()> {
        let segments = build_basic_segments(record, &self.opts);
        let mut line = String::new();
        for (i, seg) in segments.iter().enumerate() {
            if i > 0 {
                line.push(' ');
            }
            line.push_str(&seg.text);
        }
        line.push('\n');
        w.write_all(line.as_bytes())
    }
}
struct Pending(LogRecord);
pub struct LoggerConfig {
    pub level: LogLevel,
    pub throttle: ThrottleConfig,
    pub queue_capacity: Option<usize>,
}
impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::VERBOSE,
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
pub type BasicLogger = Logger<BasicReporter>;
impl Default for BasicLogger {
    fn default() -> Self {
        Logger::new(BasicReporter::default())
    }
}
