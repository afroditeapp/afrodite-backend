// @generated automatically by Diesel CLI.

diesel::table! {
    use crate::schema_sqlite_types::*;

    account (account_id) {
        account_id -> BigInt,
        email -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_app_notification_settings (account_id) {
        account_id -> BigInt,
        news -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_email_sending_state (account_id) {
        account_id -> BigInt,
        account_registered_state_number -> SmallInt,
        new_message_state_number -> SmallInt,
        new_like_state_number -> SmallInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_global_state (row_type) {
        row_type -> BigInt,
        admin_access_granted_count -> Integer,
        next_news_publication_id -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_id (id) {
        id -> BigInt,
        uuid -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_interaction (id) {
        id -> BigInt,
        state_number -> SmallInt,
        account_id_sender -> Nullable<BigInt>,
        account_id_receiver -> Nullable<BigInt>,
        account_id_block_sender -> Nullable<BigInt>,
        account_id_block_receiver -> Nullable<BigInt>,
        two_way_block -> Bool,
        message_counter_sender -> Integer,
        message_counter_receiver -> Integer,
        video_call_url_created_sender -> Bool,
        video_call_url_created_receiver -> Bool,
        received_like_id -> Nullable<Integer>,
        received_like_viewed -> Bool,
        received_like_email_notification_sent -> Bool,
        received_like_unix_time -> Nullable<BigInt>,
        match_id -> Nullable<Integer>,
        match_unix_time -> Nullable<BigInt>,
        conversation_id_sender -> Nullable<Integer>,
        conversation_id_receiver -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_interaction_index (account_id_first, account_id_second) {
        account_id_first -> BigInt,
        account_id_second -> BigInt,
        interaction_id -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_permissions (account_id) {
        account_id -> BigInt,
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
        account_id -> BigInt,
        birthdate -> Nullable<Date>,
        is_adult -> Nullable<Bool>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_state (account_id) {
        account_id -> BigInt,
        next_client_id -> Integer,
        account_deletion_request_unix_time -> Nullable<BigInt>,
        account_banned_reason_category -> Nullable<Integer>,
        account_banned_reason_details -> Nullable<Text>,
        account_banned_admin_account_id -> Nullable<BigInt>,
        account_banned_until_unix_time -> Nullable<BigInt>,
        account_banned_state_change_unix_time -> Nullable<BigInt>,
        news_sync_version -> SmallInt,
        unread_news_count -> Integer,
        account_created_unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    admin_notification_settings (account_id) {
        account_id -> BigInt,
        weekdays -> Integer,
        daily_enabled_time_start_seconds -> Integer,
        daily_enabled_time_end_seconds -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    admin_notification_subscriptions (account_id) {
        account_id -> BigInt,
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
        id -> BigInt,
        metric_name -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    api_usage_statistics_metric_value (account_id, time_id, metric_id) {
        account_id -> BigInt,
        time_id -> BigInt,
        metric_id -> BigInt,
        metric_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    api_usage_statistics_save_time (id) {
        id -> BigInt,
        unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    chat_app_notification_settings (account_id) {
        account_id -> BigInt,
        likes -> Bool,
        messages -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    chat_email_notification_settings (account_id) {
        account_id -> BigInt,
        likes -> Bool,
        messages -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    chat_global_state (row_type) {
        row_type -> BigInt,
        next_match_id -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    chat_report_chat_message (report_id) {
        report_id -> BigInt,
        message_sender_account_id_uuid -> Binary,
        message_receiver_account_id_uuid -> Binary,
        message_unix_time -> BigInt,
        message_id -> Integer,
        message_symmetric_key -> Binary,
        client_message_bytes -> Binary,
        backend_signed_message_bytes -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    chat_state (account_id) {
        account_id -> BigInt,
        received_likes_sync_version -> SmallInt,
        new_received_likes_count -> Integer,
        next_received_like_id -> Integer,
        max_public_key_count -> Integer,
        next_conversation_id -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    client_features_file_hash (row_type) {
        row_type -> BigInt,
        sha256_hash -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    common_report (id) {
        id -> BigInt,
        creator_account_id -> BigInt,
        target_account_id -> BigInt,
        report_type_number -> SmallInt,
        creation_unix_time -> BigInt,
        moderator_account_id -> Nullable<BigInt>,
        processing_state -> SmallInt,
        processing_state_change_unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    common_state (account_id) {
        account_id -> BigInt,
        client_config_sync_version -> SmallInt,
        client_login_session_platform -> Nullable<SmallInt>,
        client_language -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    current_account_media (account_id) {
        account_id -> BigInt,
        security_content_id -> Nullable<BigInt>,
        profile_content_version_uuid -> Binary,
        profile_content_id_0 -> Nullable<BigInt>,
        profile_content_id_1 -> Nullable<BigInt>,
        profile_content_id_2 -> Nullable<BigInt>,
        profile_content_id_3 -> Nullable<BigInt>,
        profile_content_id_4 -> Nullable<BigInt>,
        profile_content_id_5 -> Nullable<BigInt>,
        grid_crop_size -> Nullable<Double>,
        grid_crop_x -> Nullable<Double>,
        grid_crop_y -> Nullable<Double>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    custom_reports_file_hash (row_type) {
        row_type -> BigInt,
        sha256_hash -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    daily_likes_left (account_id) {
        account_id -> BigInt,
        sync_version -> SmallInt,
        likes_left -> Integer,
        latest_limit_reset_unix_time -> Nullable<BigInt>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    demo_account_owned_accounts (demo_account_id, account_id) {
        demo_account_id -> Integer,
        account_id -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    favorite_profile (account_id, favorite_account_id) {
        account_id -> BigInt,
        favorite_account_id -> BigInt,
        unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_client_version_statistics (time_id, version_id) {
        time_id -> BigInt,
        version_id -> BigInt,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_client_version_statistics_version_number (id) {
        id -> BigInt,
        major -> Integer,
        minor -> Integer,
        patch -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_common_statistics_save_time (id) {
        id -> BigInt,
        unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_ip_country_statistics (time_id, country_id) {
        time_id -> BigInt,
        country_id -> BigInt,
        new_tcp_connections -> Integer,
        new_http_requests -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_ip_country_statistics_country_name (id) {
        id -> BigInt,
        country_name -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_performance_statistics_metric_name (id) {
        id -> BigInt,
        metric_name -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_performance_statistics_metric_value (time_id, metric_id) {
        time_id -> BigInt,
        metric_id -> BigInt,
        metric_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_all_genders (time_id, age) {
        time_id -> BigInt,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_man (time_id, age) {
        time_id -> BigInt,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_non_binary (time_id, age) {
        time_id -> BigInt,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_woman (time_id, age) {
        time_id -> BigInt,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_account (time_id) {
        time_id -> BigInt,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_all_genders (time_id) {
        time_id -> BigInt,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_man (time_id) {
        time_id -> BigInt,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_non_binary (time_id) {
        time_id -> BigInt,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_woman (time_id) {
        time_id -> BigInt,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    ip_address_usage_statistics (account_id, ip_address) {
        account_id -> BigInt,
        ip_address -> Binary,
        usage_count -> Integer,
        first_usage_unix_time -> BigInt,
        latest_usage_unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    login_session (account_id) {
        account_id -> BigInt,
        access_token -> Binary,
        access_token_unix_time -> BigInt,
        access_token_ip_address -> Binary,
        refresh_token -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_app_notification_settings (account_id) {
        account_id -> BigInt,
        media_content_moderation -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_app_notification_state (account_id) {
        account_id -> BigInt,
        media_content_accepted -> SmallInt,
        media_content_accepted_viewed -> SmallInt,
        media_content_rejected -> SmallInt,
        media_content_rejected_viewed -> SmallInt,
        media_content_deleted -> SmallInt,
        media_content_deleted_viewed -> SmallInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_content (id) {
        id -> BigInt,
        uuid -> Binary,
        account_id -> BigInt,
        secure_capture -> Bool,
        face_detected -> Bool,
        content_type_number -> SmallInt,
        slot_number -> SmallInt,
        creation_unix_time -> BigInt,
        initial_content -> Bool,
        moderation_state -> SmallInt,
        moderation_rejected_reason_category -> Nullable<Integer>,
        moderation_rejected_reason_details -> Nullable<Text>,
        moderation_moderator_account_id -> Nullable<BigInt>,
        usage_start_unix_time -> Nullable<BigInt>,
        usage_end_unix_time -> Nullable<BigInt>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_report_profile_content (report_id) {
        report_id -> BigInt,
        profile_content_uuid -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_state (account_id) {
        account_id -> BigInt,
        media_content_sync_version -> SmallInt,
        profile_content_edited_unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    news (id) {
        id -> BigInt,
        account_id_creator -> Nullable<BigInt>,
        first_publication_unix_time -> Nullable<BigInt>,
        latest_publication_unix_time -> Nullable<BigInt>,
        publication_id -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    news_translations (locale, news_id) {
        locale -> Text,
        news_id -> BigInt,
        title -> Text,
        body -> Text,
        creation_unix_time -> BigInt,
        version_number -> Integer,
        account_id_creator -> Nullable<BigInt>,
        account_id_editor -> Nullable<BigInt>,
        edit_unix_time -> Nullable<BigInt>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    pending_messages (id) {
        id -> BigInt,
        account_interaction -> BigInt,
        account_id_sender -> BigInt,
        account_id_receiver -> BigInt,
        sender_acknowledgement -> Bool,
        receiver_acknowledgement -> Bool,
        receiver_push_notification_sent -> Bool,
        receiver_email_notification_sent -> Bool,
        message_unix_time -> BigInt,
        message_id -> Integer,
        sender_client_id -> Integer,
        sender_client_local_id -> Integer,
        message_bytes -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile (account_id) {
        account_id -> BigInt,
        version_uuid -> Binary,
        profile_name -> Nullable<Text>,
        profile_text -> Nullable<Text>,
        age -> Integer,
        last_seen_unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_app_notification_settings (account_id) {
        account_id -> BigInt,
        profile_string_moderation -> Bool,
        automatic_profile_search -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_app_notification_state (account_id) {
        account_id -> BigInt,
        profile_name_accepted -> SmallInt,
        profile_name_accepted_viewed -> SmallInt,
        profile_name_rejected -> SmallInt,
        profile_name_rejected_viewed -> SmallInt,
        profile_text_accepted -> SmallInt,
        profile_text_accepted_viewed -> SmallInt,
        profile_text_rejected -> SmallInt,
        profile_text_rejected_viewed -> SmallInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_file_hash (row_type) {
        row_type -> BigInt,
        sha256_hash -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_filter_list_unwanted (account_id, attribute_id, filter_value) {
        account_id -> BigInt,
        attribute_id -> Integer,
        filter_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_filter_list_wanted (account_id, attribute_id, filter_value) {
        account_id -> BigInt,
        attribute_id -> Integer,
        filter_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_filter_settings (account_id, attribute_id) {
        account_id -> BigInt,
        attribute_id -> Integer,
        filter_accept_missing_attribute -> Bool,
        filter_use_logical_operator_and -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_value_list (account_id, attribute_id, attribute_value) {
        account_id -> BigInt,
        attribute_id -> Integer,
        attribute_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_automatic_profile_search_settings (account_id) {
        account_id -> BigInt,
        new_profiles -> Bool,
        attribute_filters -> Bool,
        distance_filters -> Bool,
        weekdays -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_automatic_profile_search_state (account_id) {
        account_id -> BigInt,
        last_seen_unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_moderation (account_id, content_type) {
        account_id -> BigInt,
        content_type -> SmallInt,
        state_type -> SmallInt,
        rejected_reason_category -> Nullable<Integer>,
        rejected_reason_details -> Nullable<Text>,
        moderator_account_id -> Nullable<BigInt>,
        created_unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_name_allowlist (profile_name) {
        profile_name -> Text,
        name_creator_account_id -> BigInt,
        name_moderator_account_id -> Nullable<BigInt>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_report_profile_name (report_id) {
        report_id -> BigInt,
        profile_name -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_report_profile_text (report_id) {
        report_id -> BigInt,
        profile_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_state (account_id) {
        account_id -> BigInt,
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
        profile_sync_version -> SmallInt,
        initial_profile_age -> Nullable<Integer>,
        initial_profile_age_set_unix_time -> Nullable<BigInt>,
        profile_edited_unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    public_key (account_id, key_id) {
        account_id -> BigInt,
        key_id -> BigInt,
        key_data -> Binary,
        key_added_unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    push_notification (account_id) {
        account_id -> BigInt,
        pending_flags -> Integer,
        sent_flags -> Integer,
        encryption_key -> Nullable<Text>,
        device_token -> Nullable<Text>,
        device_token_unix_time -> Nullable<BigInt>,
        sync_version -> SmallInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    shared_state (account_id) {
        account_id -> BigInt,
        account_state_initial_setup_completed -> Bool,
        account_state_banned -> Bool,
        account_state_pending_deletion -> Bool,
        profile_visibility_state_number -> SmallInt,
        sync_version -> SmallInt,
        unlimited_likes -> Bool,
        birthdate -> Nullable<Date>,
        is_bot_account -> Bool,
        initial_setup_completed_unix_time -> BigInt,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    sign_in_with_info (account_id) {
        account_id -> BigInt,
        apple_account_id -> Nullable<Text>,
        google_account_id -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    used_account_ids (id) {
        id -> BigInt,
        uuid -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    used_content_ids (account_id, uuid) {
        account_id -> BigInt,
        uuid -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    vapid_public_key_hash (row_type) {
        row_type -> BigInt,
        sha256_hash -> Text,
    }
}

diesel::joinable!(account -> account_id (account_id));
diesel::joinable!(account_app_notification_settings -> account_id (account_id));
diesel::joinable!(account_email_sending_state -> account_id (account_id));
diesel::joinable!(account_id -> used_account_ids (id));
diesel::joinable!(account_interaction_index -> account_interaction (interaction_id));
diesel::joinable!(account_permissions -> account_id (account_id));
diesel::joinable!(account_setup -> account_id (account_id));
diesel::joinable!(admin_notification_settings -> account_id (account_id));
diesel::joinable!(admin_notification_subscriptions -> account_id (account_id));
diesel::joinable!(api_usage_statistics_metric_value -> account_id (account_id));
diesel::joinable!(api_usage_statistics_metric_value -> api_usage_statistics_metric_name (metric_id));
diesel::joinable!(api_usage_statistics_metric_value -> api_usage_statistics_save_time (time_id));
diesel::joinable!(chat_app_notification_settings -> account_id (account_id));
diesel::joinable!(chat_email_notification_settings -> account_id (account_id));
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
diesel::joinable!(push_notification -> account_id (account_id));
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
    chat_email_notification_settings,
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
    push_notification,
    shared_state,
    sign_in_with_info,
    used_account_ids,
    used_content_ids,
    vapid_public_key_hash,
);
