use std::time::{Duration, Instant};

/// Clock abstraction to allow deterministic tests.
pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> Instant;
}

#[derive(Default)]
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// A manual advance mock clock for tests.
#[derive(Debug)]
pub struct MockClock {
    base: Instant,
    offset: Duration,
}
impl Default for MockClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MockClock {
    pub fn new() -> Self {
        Self {
            base: Instant::now(),
            offset: Duration::ZERO,
        }
    }
    pub fn advance(&mut self, dur: Duration) {
        self.offset += dur;
    }
}
impl Clock for MockClock {
    fn now(&self) -> Instant {
        self.base + self.offset
    }
}
