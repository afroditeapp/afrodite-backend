
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
    },
};

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

/// Create a new counter struct and statics related to it.
///
/// ```
/// use simple_backend::create_counters;
/// create_counters!(
///     AccountCounters,       // Struct name (private)
///     ACCOUNT,               // Static struct instance name (private)
///     ACCOUNT_COUNTERS_LIST, // Static counter list name (public)
///     check_access_token,    // Struct field name
///     get_account_state,     // Struct field name
///     // ...
/// );
/// ```
#[macro_export]
macro_rules! create_counters {
    (
        $counters_struct_type_name:ident,
        $counters_static_name:ident,
        $counters_list_name:ident,
        $( $name:ident , )*
    ) => {
        struct $counters_struct_type_name {
            $(
                pub $name: $crate::perf::counters::PerfCounter,
            )*
        }

        impl $counters_struct_type_name {
            const fn new() -> Self {
                Self {
                    $(
                        $name: $crate::perf::counters::PerfCounter::new(stringify!($name)),
                    )*
                }
            }
        }

        pub static $counters_list_name: &'static [&'static $crate::perf::counters::PerfCounter] = &[
            $(
                &$counters_static_name.$name,
            )*
        ];

        static $counters_static_name: $counters_struct_type_name =
            $counters_struct_type_name::new();
    };
}

/// Type for storing references to all counter categories.
///
/// ```
/// use simple_backend::create_counters;
/// create_counters!(
///     AccountCounters,       // Struct name (private)
///     ACCOUNT,               // Static struct instance name (private)
///     ACCOUNT_COUNTERS_LIST, // Static counter list name (public)
///     check_access_token,    // Struct field name
///     get_account_state,     // Struct field name
///     // ...
/// );
/// use simple_backend::perf::CounterCategory;
/// static ALL_COUNTERS: &'static [&'static CounterCategory] = &[
///     &CounterCategory::new("account", ACCOUNT_COUNTERS_LIST),
/// ];
/// ```
pub type AllCounters = &'static [&'static CounterCategory];

pub struct CounterCategory {
    name: &'static str,
    counter_list: &'static [&'static PerfCounter],
}

impl CounterCategory {
    pub const fn new(name: &'static str, counter_list: &'static [&'static PerfCounter]) -> Self {
        Self { name, counter_list }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn counter_list(&self) ->  &[&PerfCounter] {
        self.counter_list
    }
}
