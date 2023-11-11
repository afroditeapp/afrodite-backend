//! Server performance info
//!
//!

use std::{sync::atomic::{AtomicU32, Ordering}, collections::HashMap};

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

pub struct PerformanceCounterManager {


}

const MINUTES_PER_DAY: usize = 24 * 60;

/// History has counter values every minute 24 hours
pub struct PerformanceCounterHistory {
    pub data: Vec<HashMap<&'static str, u32>>,
}

impl PerformanceCounterHistory {
    pub fn new() -> Self {
        let mut data = vec![];
        for _ in 0..MINUTES_PER_DAY {
            data.push(HashMap::new());
        }

        Self {
            data,
        }
    }
}