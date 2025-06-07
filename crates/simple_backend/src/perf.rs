//! Server performance info
//!
//!

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use counters::AllCounters;
use simple_backend_model::{
    MetricKey, PerfMetricQueryResult, PerfMetricValueArea, PerfMetricValues, TimeGranularity, UnixTime
};
use sysinfo::MemoryRefreshKind;
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{error, warn};

use crate::ServerQuitWatcher;

pub mod websocket;
pub mod counters;

struct SystemInfo {
    cpu_usage: u32,
    ram_usage_mib: u32,
}

impl SystemInfo {
    fn new(mut system: Box<sysinfo::System>) -> (Box<sysinfo::System>, SystemInfo) {
        system.refresh_cpu_usage();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();
        system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let info = SystemInfo {
            cpu_usage: system.global_cpu_usage() as u32,
            ram_usage_mib: (system.used_memory() / 1024 / 1024) as u32,
        };
        (system, info)
    }
}

/// History has performance metric values every minute 24 hours
pub struct PerformanceMetricsHistory {
    first_item_time: Option<UnixTime>,
    data: VecDeque<HashMap<MetricKey, u32>>,
    counters: AllCounters,
    system: Option<Box<sysinfo::System>>,
}

impl PerformanceMetricsHistory {
    const MINUTES_PER_DAY: usize = 24 * 60;

    fn new(counters: AllCounters) -> Self {
        let mut data = VecDeque::new();
        for _ in 0..Self::MINUTES_PER_DAY {
            data.push_front(HashMap::new());
        }

        Self {
            data,
            first_item_time: None,
            counters,
            system: Some(Box::new(sysinfo::System::new())),
        }
    }

    async fn append_and_reset_counters(&mut self) {
        self.first_item_time = Some(UnixTime::current_time());
        let mut first_item = self.data.pop_back().expect("Buffer is empty");

        for category in self.counters {
            for counter in category.counter_list() {
                let key = MetricKey::new(
                    category.name(),
                    counter.name(),
                );
                first_item.insert(key, counter.load_and_reset());
            }
        }

        let system = self.system.take();
        let result = tokio::task::spawn_blocking(|| {
            SystemInfo::new(system.unwrap())
        }).await;
        match result {
            Ok((system, info)) => {
                self.system = Some(system);
                first_item.insert(MetricKey::SYSTEM_CPU_USAGE, info.cpu_usage);
                first_item.insert(MetricKey::SYSTEM_RAM_USAGE_MIB, info.ram_usage_mib);
            }
            Err(e) => {
                error!("Getting system info failed: {e}");
            }
        }

        first_item.insert(
            MetricKey::CONNECTIONS,
            websocket::Connections::connection_count(),
        );
        first_item.insert(
            MetricKey::CONNECTIONS_MEN,
            websocket::ConnectionsMen::connection_count(),
        );
        first_item.insert(
            MetricKey::CONNECTIONS_WOMEN,
            websocket::ConnectionsWomen::connection_count(),
        );
        first_item.insert(
            MetricKey::CONNECTIONS_NONBINARIES,
            websocket::ConnectionsNonbinaries::connection_count(),
        );

        first_item.insert(
            MetricKey::BOT_CONNECTIONS,
            websocket::BotConnections::connection_count(),
        );
        first_item.insert(
            MetricKey::BOT_CONNECTIONS_MEN,
            websocket::BotConnectionsMen::connection_count(),
        );
        first_item.insert(
            MetricKey::BOT_CONNECTIONS_WOMEN,
            websocket::BotConnectionsWomen::connection_count(),
        );
        first_item.insert(
            MetricKey::BOT_CONNECTIONS_NONBINARIES,
            websocket::BotConnectionsNonbinaries::connection_count(),
        );

        self.data.push_front(first_item);
    }

    fn get_history(&self, only_latest_hour: bool) -> HashMap<MetricKey, PerfMetricValueArea> {
        let mut counter_data = HashMap::new();

        self.copy_current_data_to(&mut counter_data, only_latest_hour);

        counter_data
    }

    fn copy_current_data_to(
        &self,
        counter_data: &mut HashMap<MetricKey, PerfMetricValueArea>,
        only_latest_hour: bool,
    ) {
        let Some(first_time_value) = self.first_item_time else {
            return;
        };
        let max_count = if only_latest_hour {
            60
        } else {
            Self::MINUTES_PER_DAY
        };
        for counter_values in self.data.iter().take(max_count) {
            for (k, &v) in counter_values.iter() {
                if let Some(area) = counter_data.get_mut(k) {
                    area.values.push(v);
                } else {
                    let area = PerfMetricValueArea {
                        first_time_value,
                        time_granularity: TimeGranularity::Minutes,
                        values: vec![v],
                    };
                    counter_data.insert(*k, area);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct PerfMetricsManagerQuitHandle {
    task: JoinHandle<()>,
}

impl PerfMetricsManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("PefCounterManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct PerfMetricsManagerData {
    history: RwLock<PerformanceMetricsHistory>,
}

impl PerfMetricsManagerData {
    pub fn new(counters: AllCounters) -> Self {
        Self {
            history: RwLock::new(PerformanceMetricsHistory::new(counters)),
        }
    }

    pub async fn get_history(&self, only_latest_hour: bool) -> PerfMetricQueryResult {
        let counter_data = self.history.read().await.get_history(only_latest_hour);
        let mut counters = vec![];
        for (counter_name, values) in counter_data {
            let value = PerfMetricValues {
                name: counter_name.to_name(),
                values: vec![values],
            };
            counters.push(value);
        }

        PerfMetricQueryResult { metrics: counters }
    }

    pub async fn get_history_raw(&self, only_latest_hour: bool) -> HashMap<MetricKey, PerfMetricValueArea> {
        self.history.read().await.get_history(only_latest_hour)
    }
}

pub struct PerfMetricsManager {
    data: Arc<PerfMetricsManagerData>,
}

impl PerfMetricsManager {
    pub fn new_manager(
        data: Arc<PerfMetricsManagerData>,
        quit_notification: ServerQuitWatcher,
    ) -> PerfMetricsManagerQuitHandle {
        let manager = Self { data };

        let task = tokio::spawn(manager.run(quit_notification));

        PerfMetricsManagerQuitHandle { task }
    }

    pub async fn run(self, mut quit_notification: ServerQuitWatcher) {
        let mut timer = tokio::time::interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                // It is assumed that missed ticks can not happen as interval time
                // is a minute. The default Burst recovery strategy will only result
                // as wrong information in data and original tick timing will recover
                // eventually.
                _ = timer.tick() => {
                    self.data.history.write().await.append_and_reset_counters().await;
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }
}
