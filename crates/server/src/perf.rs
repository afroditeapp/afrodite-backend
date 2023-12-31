//! Server performance info
//!
//!

use simple_backend::{
    create_counters,
    perf::{CounterCategory, PerfCounter},
};

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_COUNTERS_LIST,
    get_version,
    get_connect_websocket,
);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_COUNTERS_LIST,
    get_system_info,
    get_software_info,
    get_latest_build_info,
    post_request_build_software,
    post_request_update_software,
    post_request_restart_or_reset_backend,
    get_backend_config,
    post_backend_config,
    get_perf_data,
);


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
    put_content_to_content_slot,
    get_content_slot_state,
    get_map_tile,
);

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_COUNTERS_LIST,
    patch_moderation_request_list,
    post_handle_moderation_request,
    get_security_image_info,
);

create_counters!(
    MediaInternalCounters,
    MEDIA_INTERNAL,
    MEDIA_INTERNAL_COUNTERS_LIST,
    internal_get_check_moderation_request_for_account,
    internal_post_update_profile_image_visibility,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_COUNTERS_LIST,
    get_profile,
    post_profile,
    get_location,
    put_location,
    post_get_next_profile_page,
    post_reset_profile_paging,
    get_favorite_profiles,
    post_favorite_profile,
    delete_favorite_profile,
    get_profile_from_database_debug_mode_benchmark,
    post_profile_to_database_debug_mode_benchmark,
);

create_counters!(
    ProfileInternalCounters,
    PROFILE_INTERNAL,
    PROFILE_INTERNAL_COUNTERS_LIST,
    internal_post_update_profile_visibility,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_COUNTERS_LIST,
    post_send_like,
    get_sent_likes,
    get_received_likes,
    delete_like,
    get_matches,
    post_block_profile,
    post_unblock_profile,
    get_sent_blocks,
    get_received_blocks,
    get_pending_messages,
    delete_pending_messages,
    get_message_number_of_latest_viewed_message,
    post_message_number_of_latest_viewed_message,
    post_send_message,
);

pub static ALL_COUNTERS: &'static [&'static CounterCategory] = &[
    &CounterCategory::new("common", COMMON_COUNTERS_LIST),
    &CounterCategory::new("common_admin", COMMON_ADMIN_COUNTERS_LIST),
    &CounterCategory::new("account_internal", ACCOUNT_INTERNAL_COUNTERS_LIST),
    &CounterCategory::new("account", ACCOUNT_COUNTERS_LIST),
    &CounterCategory::new("media", MEDIA_COUNTERS_LIST),
    &CounterCategory::new("media_internal", MEDIA_INTERNAL_COUNTERS_LIST),
    &CounterCategory::new("profile", PROFILE_COUNTERS_LIST),
    &CounterCategory::new("profile_internal", PROFILE_INTERNAL_COUNTERS_LIST),
    &CounterCategory::new("chat", CHAT_COUNTERS_LIST),
];
