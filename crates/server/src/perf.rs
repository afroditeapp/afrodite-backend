//! Server performance info
//!
//!

use std::{sync::{atomic::{AtomicU32, Ordering}, Arc}, collections::HashMap, time::Duration};

use config::Config;
use model::{UnixTime, PerfHistoryQueryResult, PerfValueArea, PerfHistoryValue};
use tokio::{task::JoinHandle, sync::RwLock};
use tracing::{error, warn};

use crate::app::connection::ServerQuitWatcher;

pub struct PerfCounter {
    name: &'static str,
    value: AtomicU32,
}

impl PerfCounter {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            value: AtomicU32::new(0),
        }
    }

    fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }

    /// Increment counter
    pub fn incr(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn value(&self) -> u32 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn load_and_reset(&self) -> u32 {
        self.value.swap(0, Ordering::Relaxed)
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

macro_rules! create_counters {
    (
        $counters_struct_type_name:ident,
        $counters_static_name:ident,
        $counters_list_name:ident,
        $( $name:ident , )*
    ) => {
        pub struct $counters_struct_type_name {
            $(
                pub $name: PerfCounter,
            )*
        }

        impl $counters_struct_type_name {
            const fn new() -> Self {
                Self {
                    $(
                        $name: PerfCounter::new(stringify!($name)),
                    )*
                }
            }
        }

        static $counters_list_name: &'static [&'static PerfCounter] = &[
            $(
                &$counters_static_name.$name,
            )*
        ];

        pub static $counters_static_name: $counters_struct_type_name =
            $counters_struct_type_name::new();
    };
}

create_counters!(
    AccountInternalCounters,
    ACCOUNT_INTERNAL,
    ACCOUNT_INTERNAL_COUNTERS_LIST,
    check_access_token,
    internal_get_account_state,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_COUNTERS_LIST,
    post_register,
    post_login,
    post_sign_in_with_login,
    get_account_state,
    get_account_setup,
    post_account_setup,
    get_account_data,
    post_account_data,
    post_complete_setup,
    put_setting_profile_visiblity,
    post_delete,
    get_deletion_status,
    delete_cancel_deletion,
);

// TODO: Add missing counters here

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_COUNTERS_LIST,
    get_image,
    get_primary_image_info,
    get_all_normal_images,
    put_primary_image,
    get_moderation_request,
    put_moderation_request,
    put_image_to_moderation_slot,
    get_map_tile,
);

static ALL_COUNTERS: &'static [&'static CounterCategory] = &[
    &CounterCategory::new("account_internal", ACCOUNT_INTERNAL_COUNTERS_LIST),
    &CounterCategory::new("account", ACCOUNT_COUNTERS_LIST),
    &CounterCategory::new("media", MEDIA_COUNTERS_LIST),
];

struct CounterCategory {
    name: &'static str,
    counter_list: &'static [&'static PerfCounter],
}

impl CounterCategory {
    const fn new(name: &'static str, counter_list: &'static [&'static PerfCounter]) -> Self {
        Self {
            name, counter_list
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CounterKey {
    category: &'static str,
    counter: &'static str,
}

const MINUTES_PER_DAY: usize = 24 * 60;


/// History has counter values every minute 24 hours
pub struct PerformanceCounterHistory {
    pub previous_start_time: Option<UnixTime>,
    pub start_time: Option<UnixTime>,
    pub next_index: usize,
    pub data: Vec<HashMap<CounterKey, u32>>,
}

impl PerformanceCounterHistory {
    pub fn new() -> Self {
        let mut data = vec![];
        for _ in 0..MINUTES_PER_DAY {
            data.push(HashMap::new());
        }

        Self {
            data,
            start_time: None,
            previous_start_time: None,
            next_index: 0,
        }
    }

    pub fn append_and_reset_counters(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(UnixTime::current_time())
        }

        for category in ALL_COUNTERS {
            for counter in category.counter_list {
                let key = CounterKey {
                    category: category.name,
                    counter: counter.name,
                };
                if let Some(map) = self.data.get_mut(self.next_index) {
                    map.insert(key, counter.load_and_reset());
                } else {
                    error!("Index {} not available", self.next_index);
                }
            }
        }

        if self.next_index >= self.data.len() {
            self.next_index = 0;
            self.previous_start_time = self.start_time;
        } else {
            self.next_index += 0;
        }
    }

    pub fn get_history(&self) -> PerfHistoryQueryResult {
        let mut counter_data: HashMap<String, Vec<PerfValueArea>> = HashMap::new();

        self.handle_area(&mut counter_data, self.previous_area());
        self.handle_area(&mut counter_data, self.current_area());

        let mut counters = vec![];
        for (counter_name, values) in counter_data {
            let value = PerfHistoryValue {
                counter_name,
                values
            };
            counters.push(value);
        }

        PerfHistoryQueryResult { counters }
    }

    pub fn handle_area(
        &self,
        counter_data: &mut HashMap<String, Vec<PerfValueArea>>,
        area: Option<(UnixTime, &[HashMap<CounterKey, u32>])>,
    ) {
        if let Some((start_time, data)) = area {
            for counter_values in data.iter() {
                for (k, &v) in counter_values.iter() {
                    let key = format!("{}_{}", k.category, k.counter);
                    if let Some(area) = counter_data.get_mut(&key) {
                        area[0].values.push(v);
                    } else {
                        let area = PerfValueArea {
                            start_time,
                            time_granularity: model::TimeGranularity::Minutes,
                            values: vec![v],
                        };
                        counter_data.insert(key, vec![area]);
                    }
                }
            }
        }
    }

    pub fn current_area(&self) -> Option<(UnixTime, &[HashMap<CounterKey, u32>])> {
        if let Some(start_time) = self.start_time {
            if self.next_index == 0 {
                None
            } else {
                let seconds = 60 * ((self.next_index as i64) - 1);
                let area_start_time = UnixTime { unix_time: start_time.unix_time + seconds };
                let data = &self.data[..self.next_index];
                Some((area_start_time, data))
            }
        } else {
            None
        }
    }

    /// Start time for previous area. Also the data.
    pub fn previous_area(&self) -> Option<(UnixTime, &[HashMap<CounterKey, u32>])> {
        if let Some(previous_start_time) = self.previous_start_time {
            if self.next_index == 0 {
                Some((previous_start_time, &self.data))
            } else {
                let seconds = 60 * self.next_index as i64;
                let area_start_time = UnixTime { unix_time: previous_start_time.unix_time + seconds };
                let data = &self.data[self.next_index..];
                Some((area_start_time, data))
            }
        } else {
            None
        }
    }
}


#[derive(Debug)]
pub struct PerfCounterManagerQuitHandle {
    task: JoinHandle<()>,
}

impl PerfCounterManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("PefCounterManager quit failed. Error: {:?}", e);
            }
        }
    }
}


pub struct PerfCounterManagerData {
    history: RwLock<PerformanceCounterHistory>,
}

impl PerfCounterManagerData {
    pub fn new() -> Self {
        Self {
            history: RwLock::new(PerformanceCounterHistory::new()),
        }
    }

    pub async fn get_history(&self) -> PerfHistoryQueryResult {
        self.history.read().await.get_history()
    }
}

// TODO: Database for perf counters (hourly granularity)

pub struct PerfCounterManager {
    data: Arc<PerfCounterManagerData>,
    config: Arc<Config>,
}

impl PerfCounterManager {
    pub fn new(
        data: Arc<PerfCounterManagerData>,
        config: Arc<Config>,
        quit_notification: ServerQuitWatcher,
    ) -> PerfCounterManagerQuitHandle {
        let manager = Self {
            data,
            config,
        };

        let task = tokio::spawn(manager.run(quit_notification));

        let quit_handle = PerfCounterManagerQuitHandle {
            task,
        };

        quit_handle
    }

    pub async fn run(mut self, mut quit_notification: ServerQuitWatcher) {
        let mut timer = tokio::time::interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                // It is assumed that missed ticks can not happen as interval time
                // is a minute. The default Burst recovery strategy will only result
                // as wrong information in data and original tick timing will recover
                // eventually.
                _ = timer.tick() => {
                    self.data.history.write().await.append_and_reset_counters();
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }
}
