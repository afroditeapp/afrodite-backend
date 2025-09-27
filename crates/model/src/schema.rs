// @generated automatically by Diesel CLI.

diesel::table! {
    use crate::schema_sqlite_types::*;

    account (account_id) {
        account_id -> Integer,
        email -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_app_notification_settings (account_id) {
        account_id -> Integer,
        news -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_email_sending_state (account_id) {
        account_id -> Integer,
        account_registered_state_number -> Integer,
        new_message_state_number -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_global_state (row_type) {
        row_type -> Integer,
        admin_access_granted_count -> Integer,
        next_news_publication_id -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_id (id) {
        id -> Integer,
        uuid -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_interaction (id) {
        id -> Integer,
        state_number -> Integer,
        account_id_sender -> Nullable<Integer>,
        account_id_receiver -> Nullable<Integer>,
        account_id_block_sender -> Nullable<Integer>,
        account_id_block_receiver -> Nullable<Integer>,
        two_way_block -> Bool,
        message_counter_sender -> Integer,
        message_counter_receiver -> Integer,
        received_like_id -> Nullable<Integer>,
        received_like_viewed -> Bool,
        match_id -> Nullable<Integer>,
        conversation_id_sender -> Nullable<Integer>,
        conversation_id_receiver -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_interaction_index (account_id_first, account_id_second) {
        account_id_first -> Integer,
        account_id_second -> Integer,
        interaction_id -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_permissions (account_id) {
        account_id -> Integer,
        admin_edit_permissions -> Bool,
        admin_edit_profile_name -> Bool,
        admin_edit_max_public_key_count -> Bool,
        admin_edit_media_content_face_detected_value -> Bool,
        admin_export_data -> Bool,
        admin_moderate_media_content -> Bool,
        admin_moderate_profile_names -> Bool,
        admin_moderate_profile_texts -> Bool,
        admin_process_reports -> Bool,
        admin_delete_media_content -> Bool,
        admin_delete_account -> Bool,
        admin_ban_account -> Bool,
        admin_request_account_deletion -> Bool,
        admin_view_all_profiles -> Bool,
        admin_view_private_info -> Bool,
        admin_view_profile_history -> Bool,
        admin_view_permissions -> Bool,
        admin_find_account_by_email -> Bool,
        admin_server_maintenance_view_info -> Bool,
        admin_server_maintenance_view_backend_config -> Bool,
        admin_server_maintenance_update_software -> Bool,
        admin_server_maintenance_reset_data -> Bool,
        admin_server_maintenance_restart_backend -> Bool,
        admin_server_maintenance_save_backend_config -> Bool,
        admin_server_maintenance_edit_notification -> Bool,
        admin_news_create -> Bool,
        admin_news_edit_all -> Bool,
        admin_profile_statistics -> Bool,
        admin_subscribe_admin_notifications -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_setup (account_id) {
        account_id -> Integer,
        birthdate -> Nullable<Date>,
        is_adult -> Nullable<Bool>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_state (account_id) {
        account_id -> Integer,
        next_client_id -> Integer,
        account_deletion_request_unix_time -> Nullable<Integer>,
        account_banned_reason_category -> Nullable<Integer>,
        account_banned_reason_details -> Text,
        account_banned_admin_account_id -> Nullable<Integer>,
        account_banned_until_unix_time -> Nullable<Integer>,
        account_banned_state_change_unix_time -> Nullable<Integer>,
        news_sync_version -> Integer,
        unread_news_count -> Integer,
        account_created_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    admin_notification_settings (account_id) {
        account_id -> Integer,
        weekdays -> Integer,
        daily_enabled_time_start_seconds -> Integer,
        daily_enabled_time_end_seconds -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    admin_notification_subscriptions (account_id) {
        account_id -> Integer,
        moderate_initial_media_content_bot -> Bool,
        moderate_initial_media_content_human -> Bool,
        moderate_media_content_bot -> Bool,
        moderate_media_content_human -> Bool,
        moderate_profile_texts_bot -> Bool,
        moderate_profile_texts_human -> Bool,
        moderate_profile_names_bot -> Bool,
        moderate_profile_names_human -> Bool,
        process_reports -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    api_usage_statistics_metric_name (id) {
        id -> Integer,
        metric_name -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    api_usage_statistics_metric_value (account_id, time_id, metric_id) {
        account_id -> Integer,
        time_id -> Integer,
        metric_id -> Integer,
        metric_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    api_usage_statistics_save_time (id) {
        id -> Integer,
        unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    chat_app_notification_settings (account_id) {
        account_id -> Integer,
        likes -> Bool,
        messages -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    chat_global_state (row_type) {
        row_type -> Integer,
        next_match_id -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    chat_report_chat_message (report_id) {
        report_id -> Integer,
        message_sender_account_id_uuid -> Binary,
        message_receiver_account_id_uuid -> Binary,
        message_unix_time -> Integer,
        message_id -> Integer,
        message_symmetric_key -> Binary,
        client_message_bytes -> Binary,
        backend_signed_message_bytes -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    chat_state (account_id) {
        account_id -> Integer,
        received_likes_sync_version -> Integer,
        new_received_likes_count -> Integer,
        next_received_like_id -> Integer,
        max_public_key_count -> Integer,
        next_conversation_id -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    client_features_file_hash (row_type) {
        row_type -> Integer,
        sha256_hash -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    common_report (id) {
        id -> Integer,
        creator_account_id -> Integer,
        target_account_id -> Integer,
        report_type_number -> Integer,
        creation_unix_time -> Integer,
        moderator_account_id -> Nullable<Integer>,
        processing_state -> Integer,
        processing_state_change_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    common_state (account_id) {
        account_id -> Integer,
        client_config_sync_version -> Integer,
        pending_notification -> Integer,
        pending_notification_token -> Nullable<Text>,
        push_notification_sent -> Bool,
        fcm_device_token -> Nullable<Text>,
        fcm_device_token_unix_time -> Nullable<Integer>,
        client_login_session_platform -> Nullable<Integer>,
        client_language -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    current_account_media (account_id) {
        account_id -> Integer,
        security_content_id -> Nullable<Integer>,
        profile_content_version_uuid -> Binary,
        profile_content_id_0 -> Nullable<Integer>,
        profile_content_id_1 -> Nullable<Integer>,
        profile_content_id_2 -> Nullable<Integer>,
        profile_content_id_3 -> Nullable<Integer>,
        profile_content_id_4 -> Nullable<Integer>,
        profile_content_id_5 -> Nullable<Integer>,
        grid_crop_size -> Nullable<Double>,
        grid_crop_x -> Nullable<Double>,
        grid_crop_y -> Nullable<Double>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    custom_reports_file_hash (row_type) {
        row_type -> Integer,
        sha256_hash -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    daily_likes_left (account_id) {
        account_id -> Integer,
        sync_version -> Integer,
        likes_left -> Integer,
        latest_limit_reset_unix_time -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    demo_account_owned_accounts (demo_account_id, account_id) {
        demo_account_id -> Integer,
        account_id -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    favorite_profile (account_id, favorite_account_id) {
        account_id -> Integer,
        favorite_account_id -> Integer,
        unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_client_version_statistics (time_id, version_id) {
        time_id -> Integer,
        version_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_client_version_statistics_version_number (id) {
        id -> Integer,
        major -> Integer,
        minor -> Integer,
        patch -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_common_statistics_save_time (id) {
        id -> Integer,
        unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_ip_country_statistics (time_id, country_id) {
        time_id -> Integer,
        country_id -> Integer,
        new_tcp_connections -> Integer,
        new_http_requests -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_ip_country_statistics_country_name (id) {
        id -> Integer,
        country_name -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_performance_statistics_metric_name (id) {
        id -> Integer,
        metric_name -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_performance_statistics_metric_value (time_id, metric_id) {
        time_id -> Integer,
        metric_id -> Integer,
        metric_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_all_genders (time_id, age) {
        time_id -> Integer,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_man (time_id, age) {
        time_id -> Integer,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_non_binary (time_id, age) {
        time_id -> Integer,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_woman (time_id, age) {
        time_id -> Integer,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_account (time_id) {
        time_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_all_genders (time_id) {
        time_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_man (time_id) {
        time_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_non_binary (time_id) {
        time_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_woman (time_id) {
        time_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    ip_address_usage_statistics (account_id, ip_address) {
        account_id -> Integer,
        ip_address -> Binary,
        usage_count -> Integer,
        first_usage_unix_time -> Integer,
        latest_usage_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    login_session (account_id) {
        account_id -> Integer,
        access_token -> Text,
        access_token_unix_time -> Integer,
        access_token_ip_address -> Binary,
        refresh_token -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_app_notification_settings (account_id) {
        account_id -> Integer,
        media_content_moderation -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_app_notification_state (account_id) {
        account_id -> Integer,
        media_content_accepted -> Integer,
        media_content_accepted_viewed -> Integer,
        media_content_rejected -> Integer,
        media_content_rejected_viewed -> Integer,
        media_content_deleted -> Integer,
        media_content_deleted_viewed -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_content (id) {
        id -> Integer,
        uuid -> Binary,
        account_id -> Integer,
        secure_capture -> Bool,
        face_detected -> Bool,
        content_type_number -> Integer,
        slot_number -> Integer,
        creation_unix_time -> Integer,
        initial_content -> Bool,
        moderation_state -> Integer,
        moderation_rejected_reason_category -> Nullable<Integer>,
        moderation_rejected_reason_details -> Text,
        moderation_moderator_account_id -> Nullable<Integer>,
        usage_start_unix_time -> Nullable<Integer>,
        usage_end_unix_time -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_report_profile_content (report_id) {
        report_id -> Integer,
        profile_content_uuid -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_state (account_id) {
        account_id -> Integer,
        media_content_sync_version -> Integer,
        profile_content_edited_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    news (id) {
        id -> Integer,
        account_id_creator -> Nullable<Integer>,
        first_publication_unix_time -> Nullable<Integer>,
        latest_publication_unix_time -> Nullable<Integer>,
        publication_id -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    news_translations (locale, news_id) {
        locale -> Text,
        news_id -> Integer,
        title -> Text,
        body -> Text,
        creation_unix_time -> Integer,
        version_number -> Integer,
        account_id_creator -> Nullable<Integer>,
        account_id_editor -> Nullable<Integer>,
        edit_unix_time -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    pending_messages (id) {
        id -> Integer,
        account_interaction -> Integer,
        account_id_sender -> Integer,
        account_id_receiver -> Integer,
        sender_acknowledgement -> Bool,
        receiver_acknowledgement -> Bool,
        receiver_push_notification_sent -> Bool,
        receiver_email_notification_sent -> Bool,
        message_unix_time -> Integer,
        message_id -> Integer,
        sender_client_id -> Integer,
        sender_client_local_id -> Integer,
        message_bytes -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile (account_id) {
        account_id -> Integer,
        version_uuid -> Binary,
        profile_name -> Text,
        profile_text -> Text,
        age -> Integer,
        last_seen_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_app_notification_settings (account_id) {
        account_id -> Integer,
        profile_string_moderation -> Bool,
        automatic_profile_search -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_app_notification_state (account_id) {
        account_id -> Integer,
        profile_name_accepted -> Integer,
        profile_name_accepted_viewed -> Integer,
        profile_name_rejected -> Integer,
        profile_name_rejected_viewed -> Integer,
        profile_text_accepted -> Integer,
        profile_text_accepted_viewed -> Integer,
        profile_text_rejected -> Integer,
        profile_text_rejected_viewed -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_file_hash (row_type) {
        row_type -> Integer,
        sha256_hash -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_filter_list_unwanted (account_id, attribute_id, filter_value) {
        account_id -> Integer,
        attribute_id -> Integer,
        filter_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_filter_list_wanted (account_id, attribute_id, filter_value) {
        account_id -> Integer,
        attribute_id -> Integer,
        filter_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_filter_settings (account_id, attribute_id) {
        account_id -> Integer,
        attribute_id -> Integer,
        filter_accept_missing_attribute -> Bool,
        filter_use_logical_operator_and -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_value_list (account_id, attribute_id, attribute_value) {
        account_id -> Integer,
        attribute_id -> Integer,
        attribute_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_automatic_profile_search_settings (account_id) {
        account_id -> Integer,
        new_profiles -> Bool,
        attribute_filters -> Bool,
        distance_filters -> Bool,
        weekdays -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_automatic_profile_search_state (account_id) {
        account_id -> Integer,
        last_seen_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_moderation (account_id, content_type) {
        account_id -> Integer,
        content_type -> Integer,
        state_type -> Integer,
        rejected_reason_category -> Nullable<Integer>,
        rejected_reason_details -> Text,
        moderator_account_id -> Nullable<Integer>,
        created_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_name_allowlist (profile_name) {
        profile_name -> Text,
        name_creator_account_id -> Integer,
        name_moderator_account_id -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_report_profile_name (report_id) {
        report_id -> Integer,
        profile_name -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_report_profile_text (report_id) {
        report_id -> Integer,
        profile_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_state (account_id) {
        account_id -> Integer,
        search_age_range_min -> Integer,
        search_age_range_max -> Integer,
        search_group_flags -> Integer,
        last_seen_time_filter -> Nullable<Integer>,
        unlimited_likes_filter -> Nullable<Bool>,
        min_distance_km_filter -> Nullable<Integer>,
        max_distance_km_filter -> Nullable<Integer>,
        profile_created_time_filter -> Nullable<Integer>,
        profile_edited_time_filter -> Nullable<Integer>,
        profile_text_min_characters_filter -> Nullable<Integer>,
        profile_text_max_characters_filter -> Nullable<Integer>,
        random_profile_order -> Bool,
        latitude -> Double,
        longitude -> Double,
        profile_sync_version -> Integer,
        initial_profile_age -> Nullable<Integer>,
        initial_profile_age_set_unix_time -> Nullable<Integer>,
        profile_edited_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    public_key (account_id, key_id) {
        account_id -> Integer,
        key_id -> Integer,
        key_data -> Binary,
        key_added_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    shared_state (account_id) {
        account_id -> Integer,
        account_state_initial_setup_completed -> Bool,
        account_state_banned -> Bool,
        account_state_pending_deletion -> Bool,
        profile_visibility_state_number -> Integer,
        sync_version -> Integer,
        unlimited_likes -> Bool,
        birthdate -> Nullable<Date>,
        is_bot_account -> Bool,
        initial_setup_completed_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    sign_in_with_info (account_id) {
        account_id -> Integer,
        apple_account_id -> Nullable<Text>,
        google_account_id -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    used_account_ids (id) {
        id -> Integer,
        uuid -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    used_content_ids (account_id, uuid) {
        account_id -> Integer,
        uuid -> Binary,
    }
}

diesel::joinable!(account -> account_id (account_id));
diesel::joinable!(account_app_notification_settings -> account_id (account_id));
diesel::joinable!(account_email_sending_state -> account_id (account_id));
diesel::joinable!(account_interaction_index -> account_interaction (interaction_id));
diesel::joinable!(account_permissions -> account_id (account_id));
diesel::joinable!(account_setup -> account_id (account_id));
diesel::joinable!(admin_notification_settings -> account_id (account_id));
diesel::joinable!(admin_notification_subscriptions -> account_id (account_id));
diesel::joinable!(api_usage_statistics_metric_value -> account_id (account_id));
diesel::joinable!(api_usage_statistics_metric_value -> api_usage_statistics_metric_name (metric_id));
diesel::joinable!(api_usage_statistics_metric_value -> api_usage_statistics_save_time (time_id));
diesel::joinable!(chat_app_notification_settings -> account_id (account_id));
diesel::joinable!(chat_report_chat_message -> common_report (report_id));
diesel::joinable!(chat_state -> account_id (account_id));
diesel::joinable!(common_state -> account_id (account_id));
diesel::joinable!(current_account_media -> account_id (account_id));
diesel::joinable!(daily_likes_left -> account_id (account_id));
diesel::joinable!(demo_account_owned_accounts -> account_id (account_id));
diesel::joinable!(history_client_version_statistics -> history_client_version_statistics_version_number (version_id));
diesel::joinable!(history_client_version_statistics -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_ip_country_statistics -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_ip_country_statistics -> history_ip_country_statistics_country_name (country_id));
diesel::joinable!(history_performance_statistics_metric_value -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_performance_statistics_metric_value -> history_performance_statistics_metric_name (metric_id));
diesel::joinable!(history_profile_statistics_age_changes_all_genders -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_profile_statistics_age_changes_man -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_profile_statistics_age_changes_non_binary -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_profile_statistics_age_changes_woman -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_profile_statistics_count_changes_account -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_profile_statistics_count_changes_all_genders -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_profile_statistics_count_changes_man -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_profile_statistics_count_changes_non_binary -> history_common_statistics_save_time (time_id));
diesel::joinable!(history_profile_statistics_count_changes_woman -> history_common_statistics_save_time (time_id));
diesel::joinable!(ip_address_usage_statistics -> account_id (account_id));
diesel::joinable!(login_session -> account_id (account_id));
diesel::joinable!(media_app_notification_settings -> account_id (account_id));
diesel::joinable!(media_app_notification_state -> account_id (account_id));
diesel::joinable!(media_content -> account_id (account_id));
diesel::joinable!(media_report_profile_content -> common_report (report_id));
diesel::joinable!(media_state -> account_id (account_id));
diesel::joinable!(news -> account_id (account_id_creator));
diesel::joinable!(news_translations -> news (news_id));
diesel::joinable!(pending_messages -> account_interaction (account_interaction));
diesel::joinable!(profile -> account_id (account_id));
diesel::joinable!(profile_app_notification_settings -> account_id (account_id));
diesel::joinable!(profile_app_notification_state -> account_id (account_id));
diesel::joinable!(profile_attributes_filter_list_unwanted -> account_id (account_id));
diesel::joinable!(profile_attributes_filter_list_wanted -> account_id (account_id));
diesel::joinable!(profile_attributes_filter_settings -> account_id (account_id));
diesel::joinable!(profile_attributes_value_list -> account_id (account_id));
diesel::joinable!(profile_automatic_profile_search_settings -> account_id (account_id));
diesel::joinable!(profile_automatic_profile_search_state -> account_id (account_id));
diesel::joinable!(profile_report_profile_name -> common_report (report_id));
diesel::joinable!(profile_report_profile_text -> common_report (report_id));
diesel::joinable!(profile_state -> account_id (account_id));
diesel::joinable!(public_key -> account_id (account_id));
diesel::joinable!(shared_state -> account_id (account_id));
diesel::joinable!(sign_in_with_info -> account_id (account_id));
diesel::joinable!(used_content_ids -> account_id (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    account_app_notification_settings,
    account_email_sending_state,
    account_global_state,
    account_id,
    account_interaction,
    account_interaction_index,
    account_permissions,
    account_setup,
    account_state,
    admin_notification_settings,
    admin_notification_subscriptions,
    api_usage_statistics_metric_name,
    api_usage_statistics_metric_value,
    api_usage_statistics_save_time,
    chat_app_notification_settings,
    chat_global_state,
    chat_report_chat_message,
    chat_state,
    client_features_file_hash,
    common_report,
    common_state,
    current_account_media,
    custom_reports_file_hash,
    daily_likes_left,
    demo_account_owned_accounts,
    favorite_profile,
    history_client_version_statistics,
    history_client_version_statistics_version_number,
    history_common_statistics_save_time,
    history_ip_country_statistics,
    history_ip_country_statistics_country_name,
    history_performance_statistics_metric_name,
    history_performance_statistics_metric_value,
    history_profile_statistics_age_changes_all_genders,
    history_profile_statistics_age_changes_man,
    history_profile_statistics_age_changes_non_binary,
    history_profile_statistics_age_changes_woman,
    history_profile_statistics_count_changes_account,
    history_profile_statistics_count_changes_all_genders,
    history_profile_statistics_count_changes_man,
    history_profile_statistics_count_changes_non_binary,
    history_profile_statistics_count_changes_woman,
    ip_address_usage_statistics,
    login_session,
    media_app_notification_settings,
    media_app_notification_state,
    media_content,
    media_report_profile_content,
    media_state,
    news,
    news_translations,
    pending_messages,
    profile,
    profile_app_notification_settings,
    profile_app_notification_state,
    profile_attributes_file_hash,
    profile_attributes_filter_list_unwanted,
    profile_attributes_filter_list_wanted,
    profile_attributes_filter_settings,
    profile_attributes_value_list,
    profile_automatic_profile_search_settings,
    profile_automatic_profile_search_state,
    profile_moderation,
    profile_name_allowlist,
    profile_report_profile_name,
    profile_report_profile_text,
    profile_state,
    public_key,
    shared_state,
    sign_in_with_info,
    used_account_ids,
    used_content_ids,
);
