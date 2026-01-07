use std::time::{Duration, Instant};

use crate::utils::Timer;

#[derive(Debug)]
pub struct BenchmarkState {
    pub print_benchmark_info: bool,
    pub update_profile_timer: Timer,
    pub print_info_timer: Timer,
    pub action_duration: Instant,
}

impl BenchmarkState {
    pub fn new() -> Self {
        Self {
            print_benchmark_info: false,
            update_profile_timer: Timer::new(Duration::from_millis(1000)),
            print_info_timer: Timer::new(Duration::from_millis(1000)),
            action_duration: Instant::now(),
        }
    }
}

impl Default for BenchmarkState {
    fn default() -> Self {
        Self::new()
    }
}
