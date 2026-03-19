use std::{
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
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
