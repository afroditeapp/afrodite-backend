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
use system::SystemInfoManager;
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::warn;

use crate::ServerQuitWatcher;

pub mod websocket;
pub mod counters;
pub mod system;

pub struct MetricValues {
    metrics: HashMap<MetricKey, u32>,
}

impl MetricValues {
    fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    async fn save_metrics(&mut self, counters: AllCounters, system: &mut SystemInfoManager) {
         for category in counters {
            for counter in category.counter_list() {
                let key = MetricKey::new(
                    category.name(),
                    counter.name(),
                );
                self.metrics.insert(key, counter.load_and_reset());
            }
        }

        if let Some(info) = system.get_system_info().await {
            self.metrics.insert(MetricKey::SYSTEM_CPU_USAGE, info.cpu_usage());
            self.metrics.insert(MetricKey::SYSTEM_RAM_USAGE_MIB, info.ram_usage_mib());
        }

        self.metrics.insert(
            MetricKey::CONNECTIONS,
            websocket::Connections::connection_count(),
        );
        self.metrics.insert(
            MetricKey::CONNECTIONS_MEN,
            websocket::ConnectionsMen::connection_count(),
        );
        self.metrics.insert(
            MetricKey::CONNECTIONS_WOMEN,
            websocket::ConnectionsWomen::connection_count(),
        );
        self.metrics.insert(
            MetricKey::CONNECTIONS_NONBINARIES,
            websocket::ConnectionsNonbinaries::connection_count(),
        );

        self.metrics.insert(
            MetricKey::BOT_CONNECTIONS,
            websocket::BotConnections::connection_count(),
        );
        self.metrics.insert(
            MetricKey::BOT_CONNECTIONS_MEN,
            websocket::BotConnectionsMen::connection_count(),
        );
        self.metrics.insert(
            MetricKey::BOT_CONNECTIONS_WOMEN,
            websocket::BotConnectionsWomen::connection_count(),
        );
        self.metrics.insert(
            MetricKey::BOT_CONNECTIONS_NONBINARIES,
            websocket::BotConnectionsNonbinaries::connection_count(),
        );
    }
}

/// History has performance metric values every minute 24 hours
pub struct PerformanceMetricsHistory {
    latest_metrics_save_time: Option<UnixTime>,
    data: VecDeque<MetricValues>,
    counters: AllCounters,
    system: SystemInfoManager,
}

impl PerformanceMetricsHistory {
    const MINUTES_PER_DAY: usize = 24 * 60;

    fn new(counters: AllCounters) -> Self {
        let mut data = VecDeque::new();
        for _ in 0..Self::MINUTES_PER_DAY {
            data.push_front(MetricValues::new());
        }

        Self {
            data,
            latest_metrics_save_time: None,
            counters,
            system: SystemInfoManager::new(),
        }
    }

    async fn save_and_reset_counters(&mut self) {
        self.latest_metrics_save_time = Some(UnixTime::current_time());
        let mut last_item = self.data.pop_back().expect("Buffer is empty");
        last_item.save_metrics(self.counters, &mut self.system).await;
        self.data.push_front(last_item);
    }

    fn get_history(&self, only_latest_hour: bool) -> HashMap<MetricKey, PerfMetricValueArea> {
        let mut counter_data = HashMap::new();

        let Some(latest_metrics_save_time) = self.latest_metrics_save_time else {
            return counter_data;
        };

        let max_count = if only_latest_hour {
            60
        } else {
            Self::MINUTES_PER_DAY
        };

        let mut measurements = 0;
        for counter_values in self.data.iter().take(max_count) {
            if counter_values.metrics.contains_key(&MetricKey::CONNECTIONS) {
                measurements += 1;
            }
        }

        let first_time_value = UnixTime::new(latest_metrics_save_time.ut - (measurements * 60));

        for counter_values in self.data.iter().take(max_count).rev() {
            for (k, &v) in counter_values.metrics.iter() {
                let area = counter_data
                    .entry(*k)
                    .or_insert_with(|| PerfMetricValueArea {
                        first_time_value,
                        time_granularity: TimeGranularity::Minutes,
                        values: vec![],
                    });
                area.values.push(v);
            }
        }

        counter_data
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
        timer.tick().await; // Prevent saving metrics when backend starts

        loop {
            tokio::select! {
                // It is assumed that missed ticks can not happen as interval time
                // is a minute. The default Burst recovery strategy will only result
                // as wrong information in data and original tick timing will recover
                // eventually.
                _ = timer.tick() => {
                    self.data.history.write().await.save_and_reset_counters().await;
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }
}
