//! Server performance info
//!
//!

use simple_backend::{
    create_counters,
    perf::{CounterCategory, PerfCounter},
};

use crate::api::{account::{ACCOUNT_REGISTER_COUNTERS_LIST, ACCOUNT_LOGIN_COUNTERS_LIST, ACCOUNT_SETTINGS_COUNTERS_LIST, ACCOUNT_DELETE_COUNTERS_LIST, ACCOUNT_STATE_COUNTERS_LIST}, account_internal::ACCOUNT_INTERNAL_COUNTERS_LIST, chat::{CHAT_BLOCK_COUNTERS_LIST, CHAT_LIKE_COUNTERS_LIST, CHAT_MATCH_COUNTERS_LIST, CHAT_MESSAGE_COUNTERS_LIST}, common::COMMON_COUNTERS_LIST, common_admin::COMMON_ADMIN_COUNTERS_LIST, media::{MEDIA_CONTENT_COUNTERS_LIST, MEDIA_PROFILE_CONTENT_COUNTERS_LIST, MEDIA_SECURITY_IMAGE_COUNTERS_LIST, MEDIA_MODERATION_REQUEST_COUNTERS_LIST, MEDIA_TILE_MAP_COUNTERS_LIST}, media_admin::MEDIA_ADMIN_MODERATION_COUNTERS_LIST, media_internal::MEDIA_INTERNAL_COUNTERS_LIST, profile::PROFILE_COUNTERS_LIST, profile_internal::PROFILE_INTERNAL_COUNTERS_LIST};

pub static ALL_COUNTERS: &'static [&'static CounterCategory] = &[
    &CounterCategory::new("common", COMMON_COUNTERS_LIST),
    &CounterCategory::new("common_admin", COMMON_ADMIN_COUNTERS_LIST),

    // Account
    &CounterCategory::new("account_register", ACCOUNT_REGISTER_COUNTERS_LIST),
    &CounterCategory::new("account_login", ACCOUNT_LOGIN_COUNTERS_LIST),
    &CounterCategory::new("account_delete", ACCOUNT_DELETE_COUNTERS_LIST),
    &CounterCategory::new("account_settings", ACCOUNT_SETTINGS_COUNTERS_LIST),
    &CounterCategory::new("account_state", ACCOUNT_STATE_COUNTERS_LIST),

    // Account internal
    &CounterCategory::new("account_internal", ACCOUNT_INTERNAL_COUNTERS_LIST),

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

    // Chat
    &CounterCategory::new("chat_like", CHAT_LIKE_COUNTERS_LIST),
    &CounterCategory::new("chat_block", CHAT_BLOCK_COUNTERS_LIST),
    &CounterCategory::new("chat_match", CHAT_MATCH_COUNTERS_LIST),
    &CounterCategory::new("chat_message", CHAT_MESSAGE_COUNTERS_LIST),
];
