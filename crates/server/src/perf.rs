//! Server performance info
//!
//!

use server_api::{
    common::{
        COMMON_CLIENT_CONFIG_COUNTERS_LIST, COMMON_DATA_EXPORT_COUNTERS_LIST,
        COMMON_FILE_PACKAGE_COUNTERS_LIST, COMMON_PUSH_NOTIFICATION_COUNTERS_LIST,
    },
    common_admin::{
        COMMON_ADMIN_MAINTENANCE_COUNTERS_LIST, COMMON_ADMIN_NOTIFICATION_COUNTERS_LIST,
        COMMON_ADMIN_REPORT_COUNTERS_LIST,
    },
    utils::API_COUNTERS_LIST,
};
use server_api_account::{
    account::{
        ACCOUNT_BAN_COUNTERS_LIST, ACCOUNT_CLIENT_FEATURES_COUNTERS_LIST,
        ACCOUNT_EMAIL_COUNTERS_LIST, ACCOUNT_LOGOUT_COUNTERS_LIST, ACCOUNT_NEWS_COUNTERS_LIST,
        ACCOUNT_NOTIFICATION_COUNTERS_LIST, ACCOUNT_REPORT_COUNTERS_LIST,
    },
    account_admin::{
        ACCOUNT_ADMIN_BAN_COUNTERS_LIST, ACCOUNT_ADMIN_CLIENT_VERSION_PERF_COUNTERS_LIST,
        ACCOUNT_ADMIN_DELETE_COUNTERS_LIST, ACCOUNT_ADMIN_EMAIL_COUNTERS_LIST,
        ACCOUNT_ADMIN_NEWS_COUNTERS_LIST, ACCOUNT_ADMIN_PERMISSIONS_COUNTERS_LIST,
        ACCOUNT_ADMIN_SEARCH_COUNTERS_LIST, ACCOUNT_ADMIN_STATE_COUNTERS_LIST,
    },
};
use server_api_chat::{
    chat::{
        CHAT_NOTIFICATION_COUNTERS_LIST, CHAT_PUBLIC_KEY_COUNTERS_LIST, CHAT_REPORT_COUNTERS_LIST,
        CHAT_VIDEO_CALL_COUNTERS_LIST,
    },
    chat_admin::CHAT_ADMIN_PUBLIC_KEY_COUNTERS_LIST,
};
use server_api_media::{
    media::{
        MEDIA_MEDIA_CONTENT_COUNTERS_LIST, MEDIA_NOTIFICATION_COUNTERS_LIST,
        MEDIA_REPORT_COUNTERS_LIST,
    },
    media_admin::MEDIA_ADMIN_CONTENT_COUNTERS_LIST,
};
use server_api_profile::{
    profile::{
        PROFILE_NOTIFICATION_COUNTERS_LIST, PROFILE_REPORT_COUNTERS_LIST,
        PROFILE_STATISTICS_COUNTERS_LIST,
    },
    profile_admin::{
        PROFILE_ADMIN_ITERATE_PROFILES_COUNTERS_LIST, PROFILE_ADMIN_MODERATION_COUNTERS_LIST,
        PROFILE_ADMIN_PROFILE_DATA_COUNTERS_LIST, PROFILE_ADMIN_STATISTICS_COUNTERS_LIST,
    },
};
use simple_backend::{SIMPLE_CONNECTION_COUNTERS_LIST, perf::counters::CounterCategory};

use crate::api::{
    account::{
        ACCOUNT_DELETE_COUNTERS_LIST, ACCOUNT_DEMO_COUNTERS_LIST, ACCOUNT_LOGIN_COUNTERS_LIST,
        ACCOUNT_REGISTER_COUNTERS_LIST, ACCOUNT_SETTINGS_COUNTERS_LIST,
        ACCOUNT_STATE_COUNTERS_LIST,
    },
    account_bot::ACCOUNT_BOT_COUNTERS_LIST,
    chat::{
        CHAT_BLOCK_COUNTERS_LIST, CHAT_LIKE_COUNTERS_LIST, CHAT_MATCH_COUNTERS_LIST,
        CHAT_MESSAGE_COUNTERS_LIST,
    },
    common::COMMON_COUNTERS_LIST,
    common_admin::{
        COMMON_ADMIN_CONFIG_COUNTERS_LIST, COMMON_ADMIN_MANAGER_COUNTERS_LIST,
        COMMON_ADMIN_STATISTICS_COUNTERS_LIST,
    },
    media::{
        MEDIA_CONTENT_COUNTERS_LIST, MEDIA_PROFILE_CONTENT_COUNTERS_LIST,
        MEDIA_SECURITY_CONTENT_COUNTERS_LIST, MEDIA_TILE_MAP_COUNTERS_LIST,
    },
    media_admin::MEDIA_ADMIN_MODERATION_COUNTERS_LIST,
    profile::{
        PROFILE_BENCHMARK_COUNTERS_LIST, PROFILE_DATA_COUNTERS_LIST,
        PROFILE_FAVORITE_COUNTERS_LIST, PROFILE_FILTERS_COUNTERS_LIST,
        PROFILE_ITERATE_PROFILES_COUNTERS_LIST, PROFILE_LOCATION_COUNTERS_LIST,
    },
};

pub static ALL_COUNTERS: &[&CounterCategory] = &[
    // Common
    &CounterCategory::new("common", COMMON_COUNTERS_LIST),
    &CounterCategory::new("api", API_COUNTERS_LIST),
    &CounterCategory::new("common_client_config", COMMON_CLIENT_CONFIG_COUNTERS_LIST),
    &CounterCategory::new("common_data_export", COMMON_DATA_EXPORT_COUNTERS_LIST),
    &CounterCategory::new("common_file_package", COMMON_FILE_PACKAGE_COUNTERS_LIST),
    &CounterCategory::new(
        "common_push_notification",
        COMMON_PUSH_NOTIFICATION_COUNTERS_LIST,
    ),
    // Common admin
    &CounterCategory::new(
        "common_admin_maintenance",
        COMMON_ADMIN_MAINTENANCE_COUNTERS_LIST,
    ),
    &CounterCategory::new("common_admin_manager", COMMON_ADMIN_MANAGER_COUNTERS_LIST),
    &CounterCategory::new("common_admin_config", COMMON_ADMIN_CONFIG_COUNTERS_LIST),
    &CounterCategory::new(
        "common_admin_statistics",
        COMMON_ADMIN_STATISTICS_COUNTERS_LIST,
    ),
    &CounterCategory::new("common_admin_report", COMMON_ADMIN_REPORT_COUNTERS_LIST),
    &CounterCategory::new(
        "common_admin_notification",
        COMMON_ADMIN_NOTIFICATION_COUNTERS_LIST,
    ),
    // Account
    &CounterCategory::new("account_register", ACCOUNT_REGISTER_COUNTERS_LIST),
    &CounterCategory::new("account_login", ACCOUNT_LOGIN_COUNTERS_LIST),
    &CounterCategory::new("account_logout", ACCOUNT_LOGOUT_COUNTERS_LIST),
    &CounterCategory::new("account_ban", ACCOUNT_BAN_COUNTERS_LIST),
    &CounterCategory::new("account_delete", ACCOUNT_DELETE_COUNTERS_LIST),
    &CounterCategory::new("account_settings", ACCOUNT_SETTINGS_COUNTERS_LIST),
    &CounterCategory::new("account_state", ACCOUNT_STATE_COUNTERS_LIST),
    &CounterCategory::new("account_demo", ACCOUNT_DEMO_COUNTERS_LIST),
    &CounterCategory::new("account_news", ACCOUNT_NEWS_COUNTERS_LIST),
    &CounterCategory::new("account_report", ACCOUNT_REPORT_COUNTERS_LIST),
    &CounterCategory::new(
        "account_client_features",
        ACCOUNT_CLIENT_FEATURES_COUNTERS_LIST,
    ),
    &CounterCategory::new("account_notification", ACCOUNT_NOTIFICATION_COUNTERS_LIST),
    &CounterCategory::new("account_email", ACCOUNT_EMAIL_COUNTERS_LIST),
    // Account admin
    &CounterCategory::new("account_admin_ban", ACCOUNT_ADMIN_BAN_COUNTERS_LIST),
    &CounterCategory::new("account_admin_delete", ACCOUNT_ADMIN_DELETE_COUNTERS_LIST),
    &CounterCategory::new("account_admin_news", ACCOUNT_ADMIN_NEWS_COUNTERS_LIST),
    &CounterCategory::new("account_admin_search", ACCOUNT_ADMIN_SEARCH_COUNTERS_LIST),
    &CounterCategory::new(
        "account_admin_permissions",
        ACCOUNT_ADMIN_PERMISSIONS_COUNTERS_LIST,
    ),
    &CounterCategory::new("account_admin_state", ACCOUNT_ADMIN_STATE_COUNTERS_LIST),
    &CounterCategory::new(
        "account_admin_client_version",
        ACCOUNT_ADMIN_CLIENT_VERSION_PERF_COUNTERS_LIST,
    ),
    &CounterCategory::new("account_admin_email", ACCOUNT_ADMIN_EMAIL_COUNTERS_LIST),
    // Account internal
    &CounterCategory::new("account_internal", ACCOUNT_BOT_COUNTERS_LIST),
    // Media
    &CounterCategory::new("media_content", MEDIA_CONTENT_COUNTERS_LIST),
    &CounterCategory::new("media_media_content", MEDIA_MEDIA_CONTENT_COUNTERS_LIST),
    &CounterCategory::new("media_profile_content", MEDIA_PROFILE_CONTENT_COUNTERS_LIST),
    &CounterCategory::new(
        "media_security_content",
        MEDIA_SECURITY_CONTENT_COUNTERS_LIST,
    ),
    &CounterCategory::new("media_tile_map", MEDIA_TILE_MAP_COUNTERS_LIST),
    &CounterCategory::new("media_report", MEDIA_REPORT_COUNTERS_LIST),
    &CounterCategory::new("media_notification", MEDIA_NOTIFICATION_COUNTERS_LIST),
    // Media admin
    &CounterCategory::new("media_admin_content", MEDIA_ADMIN_CONTENT_COUNTERS_LIST),
    &CounterCategory::new(
        "media_admin_moderation",
        MEDIA_ADMIN_MODERATION_COUNTERS_LIST,
    ),
    // Profile
    &CounterCategory::new("profile_filters", PROFILE_FILTERS_COUNTERS_LIST),
    &CounterCategory::new("profile_iterate", PROFILE_ITERATE_PROFILES_COUNTERS_LIST),
    &CounterCategory::new("profile_location", PROFILE_LOCATION_COUNTERS_LIST),
    &CounterCategory::new("profile_favorite", PROFILE_FAVORITE_COUNTERS_LIST),
    &CounterCategory::new("profile_data", PROFILE_DATA_COUNTERS_LIST),
    &CounterCategory::new("profile_report", PROFILE_REPORT_COUNTERS_LIST),
    &CounterCategory::new("profile_benchmark", PROFILE_BENCHMARK_COUNTERS_LIST),
    &CounterCategory::new("profile_statistics", PROFILE_STATISTICS_COUNTERS_LIST),
    &CounterCategory::new("profile_notification", PROFILE_NOTIFICATION_COUNTERS_LIST),
    // Profile admin
    &CounterCategory::new(
        "profile_admin_statistics",
        PROFILE_ADMIN_STATISTICS_COUNTERS_LIST,
    ),
    &CounterCategory::new(
        "profile_admin_iterate_profiles",
        PROFILE_ADMIN_ITERATE_PROFILES_COUNTERS_LIST,
    ),
    &CounterCategory::new(
        "profile_admin_profile_data",
        PROFILE_ADMIN_PROFILE_DATA_COUNTERS_LIST,
    ),
    &CounterCategory::new(
        "profile_admin_moderation",
        PROFILE_ADMIN_MODERATION_COUNTERS_LIST,
    ),
    // Chat
    &CounterCategory::new("chat_like", CHAT_LIKE_COUNTERS_LIST),
    &CounterCategory::new("chat_block", CHAT_BLOCK_COUNTERS_LIST),
    &CounterCategory::new("chat_match", CHAT_MATCH_COUNTERS_LIST),
    &CounterCategory::new("chat_message", CHAT_MESSAGE_COUNTERS_LIST),
    &CounterCategory::new("chat_public_key", CHAT_PUBLIC_KEY_COUNTERS_LIST),
    &CounterCategory::new("chat_report", CHAT_REPORT_COUNTERS_LIST),
    &CounterCategory::new("chat_notification", CHAT_NOTIFICATION_COUNTERS_LIST),
    &CounterCategory::new("chat_video_call", CHAT_VIDEO_CALL_COUNTERS_LIST),
    // Chat admin
    &CounterCategory::new("chat_admin_public_key", CHAT_ADMIN_PUBLIC_KEY_COUNTERS_LIST),
    // Server info
    &CounterCategory::new("server_info_connection", SIMPLE_CONNECTION_COUNTERS_LIST),
];
