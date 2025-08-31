use crate::levels::{LogLevel, level_for_type};
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    String(String),
    Number(f64),
    Bool(bool),
    Error(String),
    OtherDebug(String),
}

impl From<&str> for ArgValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for ArgValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<bool> for ArgValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}
impl From<f64> for ArgValue {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<i64> for ArgValue {
    fn from(n: i64) -> Self {
        Self::Number(n as f64)
    }
}

impl From<u64> for ArgValue {
    fn from(n: u64) -> Self {
        Self::Number(n as f64)
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
    // Additional structured fields (pending richer handling)
    pub additional: Option<Vec<ArgValue>>,
    pub meta: Option<Vec<(String, ArgValue)>>,
    pub stack: Option<Vec<String>>, // simple lines
    pub is_raw: bool,
    pub error_chain: Option<Vec<String>>, // collected error chain lines (unprefixed)
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
            additional: None,
            meta: None,
            stack: None,
            is_raw: false,
            error_chain: None,
        }
    }

    pub fn new_with_timestamp(
        type_name: &str,
        tag: Option<String>,
        args: Vec<ArgValue>,
        timestamp: Instant,
    ) -> Self {
        let level = level_for_type(type_name).unwrap_or(LogLevel::LOG);
        let message = build_message(&args);
        Self {
            timestamp,
            level,
            type_name: type_name.to_string(),
            tag,
            args,
            message,
            repetition_count: 0,
            additional: None,
            meta: None,
            stack: None,
            is_raw: false,
            error_chain: None,
        }
    }

    pub fn with_additional(mut self, additional: Vec<ArgValue>) -> Self {
        self.additional = Some(additional);
        self
    }
    pub fn with_meta(mut self, meta: Vec<(String, ArgValue)>) -> Self {
        self.meta = Some(meta);
        self
    }
    pub fn with_stack<S: Into<String>>(mut self, lines: Vec<S>) -> Self {
        self.stack = Some(lines.into_iter().map(Into::into).collect());
        self
    }

    pub fn raw(type_name: &str, tag: Option<String>, message: &str, timestamp: Instant) -> Self {
        let level = level_for_type(type_name).unwrap_or(LogLevel::LOG);
        Self {
            timestamp,
            level,
            type_name: type_name.to_string(),
            tag,
            args: Vec::new(),
            message: Some(message.to_string()),
            repetition_count: 0,
            additional: None,
            meta: None,
            stack: None,
            is_raw: true,
            error_chain: None,
        }
    }

    pub fn with_error_chain(mut self, chain: Vec<String>) -> Self {
        self.error_chain = Some(chain);
        self
    }

    pub fn attach_error<E: std::error::Error + 'static>(mut self, err: &E) -> Self {
        // push the top error string as an ArgValue::Error (for message concatenation) if message not already explicit
        self.args.push(ArgValue::Error(err.to_string()));
        if self.error_chain.is_none() {
            self.error_chain = Some(crate::error_chain::collect_chain(
                err as &dyn std::error::Error,
            ));
        }
        // Rebuild message to include this error
        self.message = build_message(&self.args);
        self
    }

    pub fn attach_dyn_error(mut self, err: &(dyn std::error::Error + 'static)) -> Self {
        self.args.push(ArgValue::Error(err.to_string()));
        if self.error_chain.is_none() {
            self.error_chain = Some(crate::error_chain::collect_chain(err));
        }
        self.message = build_message(&self.args);
        self
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
