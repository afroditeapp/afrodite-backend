pub mod name;
pub mod assert;
pub mod image;

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant}, fmt::Debug,
};

use api_client::{
    apis::{
        account_api::{post_login, post_register},
        profile_api::{get_profile, post_profile},
    },
    models::{AccountIdLight, Profile},
};

use async_trait::async_trait;
use tokio::{
    select,
    sync::{mpsc, watch},
    time::sleep,
};

use error_stack::{Result, ResultExt};

use tracing::{error, log::warn};

use super::{benchmark::Benchmark, actions::BotAction};

use super::super::client::{ApiClient, TestError};

use crate::{
    config::args::{Test, TestMode},
    utils::IntoReportExt,
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

pub struct AvgTime {
    previous: Instant,
    total: u64,
    counter: u64,
    calculate_avg_when_couter: u64,
    current_avg: Duration,
}

impl AvgTime {
    pub fn new(calculate_avg_when_couter: u64) -> Self {
        Self {
            previous: Instant::now(),
            total: 0,
            counter: 0,
            calculate_avg_when_couter,
            current_avg: Duration::from_micros(0),
        }
    }

    pub fn track(&mut self) {
        self.previous = Instant::now();
    }

    pub fn complete(&mut self) {
        let time = self.previous.elapsed();
        self.total += time.as_micros() as u64;
        self.counter += 1;

        if self.counter >= self.calculate_avg_when_couter {
            self.current_avg = Duration::from_micros(self.total / self.counter);

            self.counter = 0;
            self.total = 0;
        }
    }

    pub fn current_avg(&self) -> Duration {
        self.current_avg
    }
}
