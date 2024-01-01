//! Server performance info
//!
//!

use simple_backend::{
    create_counters,
    perf::{CounterCategory, PerfCounter},
};

use crate::api::{media::{MEDIA_CONTENT_COUNTERS_LIST, MEDIA_PROFILE_CONTENT_COUNTERS_LIST, MEDIA_SECURITY_IMAGE_COUNTERS_LIST, MEDIA_MODERATION_REQUEST_COUNTERS_LIST, MEDIA_TILE_MAP_COUNTERS_LIST}, media_admin::MEDIA_ADMIN_MODERATION_COUNTERS_LIST, chat::CHAT_COUNTERS_LIST, profile::PROFILE_COUNTERS_LIST, profile_internal::PROFILE_INTERNAL_COUNTERS_LIST, account_internal::ACCOUNT_INTERNAL_COUNTERS_LIST, account::ACCOUNT_COUNTERS_LIST, media_internal::MEDIA_INTERNAL_COUNTERS_LIST, common::COMMON_COUNTERS_LIST, common_admin::COMMON_ADMIN_COUNTERS_LIST};

pub static ALL_COUNTERS: &'static [&'static CounterCategory] = &[
    &CounterCategory::new("common", COMMON_COUNTERS_LIST),
    &CounterCategory::new("common_admin", COMMON_ADMIN_COUNTERS_LIST),
    &CounterCategory::new("account_internal", ACCOUNT_INTERNAL_COUNTERS_LIST),
    &CounterCategory::new("account", ACCOUNT_COUNTERS_LIST),

    // Media
    &CounterCategory::new("media_content", MEDIA_CONTENT_COUNTERS_LIST),
    &CounterCategory::new("media_profile_content", MEDIA_PROFILE_CONTENT_COUNTERS_LIST),
    &CounterCategory::new("media_security_image", MEDIA_SECURITY_IMAGE_COUNTERS_LIST),
    &CounterCategory::new("media_moderation_request", MEDIA_MODERATION_REQUEST_COUNTERS_LIST),
    &CounterCategory::new("media_tile_map", MEDIA_TILE_MAP_COUNTERS_LIST),

    // Media admin
    &CounterCategory::new("media_admin_moderation", MEDIA_ADMIN_MODERATION_COUNTERS_LIST),

    // Media internal
    &CounterCategory::new("media_internal", MEDIA_INTERNAL_COUNTERS_LIST),

    &CounterCategory::new("profile", PROFILE_COUNTERS_LIST),
    &CounterCategory::new("profile_internal", PROFILE_INTERNAL_COUNTERS_LIST),
    &CounterCategory::new("chat", CHAT_COUNTERS_LIST),
];
