pub mod assert;
pub mod image;
pub mod location;

use std::{
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

#[derive(Default, Debug)]
pub struct Counters {
    get_profile: AtomicU64,
}

impl Counters {
    pub const fn new() -> Self {
        Self {
            get_profile: AtomicU64::new(0),
        }
    }

    pub fn inc_get_profile(&self) {
        self.get_profile.fetch_add(1, Ordering::Relaxed);
    }

    pub fn reset_get_profile(&self) -> u64 {
        self.get_profile.swap(0, Ordering::Relaxed)
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
