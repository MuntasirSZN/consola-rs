use crate::record::LogRecord;
use blake3::Hasher;
use std::time::{Duration, Instant};
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
        let mut h = Hasher::new();
        h.update(record.type_name.as_bytes());
        if let Some(tag) = &record.tag {
            h.update(tag.as_bytes());
        }
        h.update(&record.level.0.to_le_bytes());
        if let Some(msg) = &record.message {
            h.update(msg.as_bytes());
        }
        for a in &record.args {
            h.update(format!("{a:?}").as_bytes());
        }
        *h.finalize().as_bytes()
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
