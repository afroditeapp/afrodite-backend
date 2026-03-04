use std::{
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use error_stack::Report;
use test_mode_utils::client::{TestError, is_unauthorized_error};
use tracing::warn;

const MIN_RUNTIME_FOR_UNAUTHORIZED_RESTART: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotRunResult {
    Quit,
    Unauthorized,
}

pub fn run_result_from_error(error: &Report<TestError>) -> BotRunResult {
    if is_unauthorized_error(error) {
        BotRunResult::Unauthorized
    } else {
        BotRunResult::Quit
    }
}

pub fn should_restart_bot_after_run_result(
    run_result: BotRunResult,
    started_at: Instant,
    task_id: u32,
    bot_name: &str,
) -> bool {
    match run_result {
        BotRunResult::Unauthorized => {
            let runtime = started_at.elapsed();
            if runtime >= MIN_RUNTIME_FOR_UNAUTHORIZED_RESTART {
                // Most likely access token has been invalidated
                warn!(
                    "{} task {} received unauthorized after {:?}, restarting",
                    bot_name, task_id, runtime
                );
                true
            } else {
                warn!(
                    "{} task {} received unauthorized after {:?}, not restarting because minimum runtime is {:?}",
                    bot_name, task_id, runtime, MIN_RUNTIME_FOR_UNAUTHORIZED_RESTART
                );
                false
            }
        }
        BotRunResult::Quit => false,
    }
}

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
