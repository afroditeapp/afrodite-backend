pub mod assert;
pub mod image;
pub mod location;
pub mod encrypt;

use std::{
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

/// Benchmark counters
#[derive(Default, Debug)]
pub struct Counters {
    /// Counter for one benchmark iteration
    main: AtomicU64,
    /// Counter for details about one benchmark iteration
    sub: AtomicU64,
}

impl Counters {
    pub const fn new() -> Self {
        Self {
            main: AtomicU64::new(0),
            sub: AtomicU64::new(0),
        }
    }

    pub fn inc_main(&self) {
        self.main.fetch_add(1, Ordering::Relaxed);
    }

    pub fn reset_main(&self) -> u64 {
        self.main.swap(0, Ordering::Relaxed)
    }

    pub fn inc_sub(&self) {
        self.sub.fetch_add(1, Ordering::Relaxed);
    }

    pub fn reset_sub(&self) -> u64 {
        self.sub.swap(0, Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct Timer {
    previous: Instant,
    time: Duration,
}

impl Timer {
    pub fn new(time: Duration) -> Self {
        Self {
            previous: Instant::now(),
            time,
        }
    }

    pub fn passed(&mut self) -> bool {
        if self.previous.elapsed() >= self.time {
            self.previous = Instant::now();
            true
        } else {
            false
        }
    }
}
