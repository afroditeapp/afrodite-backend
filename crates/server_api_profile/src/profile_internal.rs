//! Handlers for internal from Server to Server state transfers and messages

use simple_backend::create_counters;

create_counters!(
    ProfileInternalCounters,
    PROFILE_INTERNAL,
    PROFILE_INTERNAL_COUNTERS_LIST,
);
