// @generated automatically by Diesel CLI.

diesel::table! {
    account_app_notification_settings (account_id) {
        account_id -> Int8,
        news -> Bool,
    }
}

diesel::table! {
    account_email_address_state (account_id) {
        account_id -> Int8,
        email -> Nullable<Text>,
        email_verification_token -> Nullable<Bytea>,
        email_verification_token_unix_time -> Nullable<Int8>,
        email_change -> Nullable<Text>,
        email_change_unix_time -> Nullable<Int8>,
        email_change_verification_token -> Nullable<Bytea>,
        email_change_verified -> Bool,
        email_login_token -> Nullable<Bytea>,
        email_login_token_unix_time -> Nullable<Int8>,
        email_login_enabled -> Bool,
    }
}

diesel::table! {
    account_email_sending_state (account_id) {
        account_id -> Int8,
        email_verification_state_number -> Int2,
        new_message_state_number -> Int2,
        new_like_state_number -> Int2,
        account_deletion_remainder_first_state_number -> Int2,
        account_deletion_remainder_second_state_number -> Int2,
        account_deletion_remainder_third_state_number -> Int2,
        email_change_verification_state_number -> Int2,
        email_change_notification_state_number -> Int2,
        email_login_state_number -> Int2,
    }
}

diesel::table! {
    account_global_state (row_type) {
        row_type -> Int4,
        admin_access_granted_count -> Int8,
        next_news_publication_id -> Int8,
    }
}

diesel::table! {
    account_id (id) {
        id -> Int8,
        uuid -> Bytea,
    }
}

diesel::table! {
    account_interaction (id) {
        id -> Int8,
        state_number -> Int2,
        account_id_sender -> Nullable<Int8>,
        account_id_receiver -> Nullable<Int8>,
        account_id_block_sender -> Nullable<Int8>,
        account_id_block_receiver -> Nullable<Int8>,
        two_way_block -> Bool,
        message_counter_sender -> Int8,
        message_counter_receiver -> Int8,
        video_call_url_created_sender -> Bool,
        video_call_url_created_receiver -> Bool,
        received_like_id -> Nullable<Int8>,
        received_like_viewed -> Bool,
        received_like_email_notification_sent -> Bool,
        received_like_unix_time -> Nullable<Int8>,
        match_id -> Nullable<Int8>,
        match_unix_time -> Nullable<Int8>,
        conversation_id_sender -> Nullable<Int8>,
        conversation_id_receiver -> Nullable<Int8>,
    }
}

diesel::table! {
    account_interaction_index (account_id_first, account_id_second) {
        account_id_first -> Int8,
        account_id_second -> Int8,
        interaction_id -> Int8,
    }
}

diesel::table! {
    account_permissions (account_id) {
        account_id -> Int8,
        admin_change_email_address -> Bool,
        admin_edit_login -> Bool,
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
        admin_view_account_state -> Bool,
        admin_view_account_api_usage -> Bool,
        admin_view_account_ip_address_usage -> Bool,
        admin_view_profile_history -> Bool,
        admin_view_permissions -> Bool,
        admin_view_email_address -> Bool,
        admin_find_account_by_email_address -> Bool,
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
    account_setup (account_id) {
        account_id -> Int8,
        birthdate -> Nullable<Date>,
        is_adult -> Nullable<Bool>,
    }
}

diesel::table! {
    account_state (account_id) {
        account_id -> Int8,
        account_deletion_request_unix_time -> Nullable<Int8>,
        account_banned_reason_category -> Nullable<Int2>,
        account_banned_reason_details -> Nullable<Text>,
        account_banned_admin_account_id -> Nullable<Int8>,
        account_banned_until_unix_time -> Nullable<Int8>,
        account_banned_state_change_unix_time -> Nullable<Int8>,
        news_sync_version -> Int2,
        unread_news_count -> Int8,
        account_created_unix_time -> Int8,
        account_locked -> Bool,
    }
}

diesel::table! {
    admin_notification_settings (account_id) {
        account_id -> Int8,
        weekdays -> Int2,
        daily_enabled_time_start_seconds -> Int4,
        daily_enabled_time_end_seconds -> Int4,
    }
}

diesel::table! {
    admin_notification_subscriptions (account_id) {
        account_id -> Int8,
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
    api_usage_statistics_metric_name (id) {
        id -> Int8,
        metric_name -> Text,
    }
}

diesel::table! {
    api_usage_statistics_metric_value (account_id, time_id, metric_id) {
        account_id -> Int8,
        time_id -> Int8,
        metric_id -> Int8,
        metric_value -> Int8,
    }
}

diesel::table! {
    api_usage_statistics_save_time (id) {
        id -> Int8,
        unix_time -> Int8,
    }
}

diesel::table! {
    chat_app_notification_settings (account_id) {
        account_id -> Int8,
        likes -> Bool,
        messages -> Bool,
    }
}

diesel::table! {
    chat_email_notification_settings (account_id) {
        account_id -> Int8,
        likes -> Bool,
        messages -> Bool,
    }
}

diesel::table! {
    chat_global_state (row_type) {
        row_type -> Int4,
        next_match_id -> Int8,
    }
}

diesel::table! {
    chat_privacy_settings (account_id) {
        account_id -> Int8,
        message_state_delivered -> Bool,
        message_state_sent -> Bool,
        typing_indicator -> Bool,
    }
}

diesel::table! {
    chat_report_chat_message (report_id) {
        report_id -> Int8,
        message_sender_account_id_uuid -> Bytea,
        message_receiver_account_id_uuid -> Bytea,
        message_unix_time -> Int8,
        message_number -> Int8,
        message_symmetric_key -> Bytea,
        client_message_bytes -> Bytea,
        backend_signed_message_bytes -> Bytea,
    }
}

diesel::table! {
    chat_state (account_id) {
        account_id -> Int8,
        received_likes_sync_version -> Int2,
        new_received_likes_count -> Int8,
        next_received_like_id -> Int8,
        max_public_key_count -> Int8,
        next_conversation_id -> Int8,
    }
}

diesel::table! {
    client_features_file_hash (row_type) {
        row_type -> Int4,
        sha256_hash -> Text,
    }
}

diesel::table! {
    common_report (id) {
        id -> Int8,
        creator_account_id -> Int8,
        target_account_id -> Int8,
        report_type_number -> Int2,
        creation_unix_time -> Int8,
        moderator_account_id -> Nullable<Int8>,
        processing_state -> Int2,
        processing_state_change_unix_time -> Int8,
    }
}

diesel::table! {
    common_state (account_id) {
        account_id -> Int8,
        client_config_sync_version -> Int2,
        client_login_session_platform -> Nullable<Int2>,
        client_language -> Nullable<Text>,
    }
}

diesel::table! {
    current_account_media (account_id) {
        account_id -> Int8,
        security_content_id -> Nullable<Int8>,
        profile_content_version_uuid -> Bytea,
        profile_content_id_0 -> Nullable<Int8>,
        profile_content_id_1 -> Nullable<Int8>,
        profile_content_id_2 -> Nullable<Int8>,
        profile_content_id_3 -> Nullable<Int8>,
        profile_content_id_4 -> Nullable<Int8>,
        profile_content_id_5 -> Nullable<Int8>,
        grid_crop_size -> Nullable<Float8>,
        grid_crop_x -> Nullable<Float8>,
        grid_crop_y -> Nullable<Float8>,
    }
}

diesel::table! {
    custom_reports_file_hash (row_type) {
        row_type -> Int4,
        sha256_hash -> Text,
    }
}

diesel::table! {
    daily_likes_left (account_id) {
        account_id -> Int8,
        sync_version -> Int2,
        likes_left -> Int2,
        latest_limit_reset_unix_time -> Nullable<Int8>,
    }
}

diesel::table! {
    demo_account_owned_accounts (demo_account_id, account_id) {
        demo_account_id -> Int8,
        account_id -> Int8,
    }
}

diesel::table! {
    favorite_profile (account_id, favorite_account_id) {
        account_id -> Int8,
        favorite_account_id -> Int8,
        unix_time -> Int8,
    }
}

diesel::table! {
    history_client_version_statistics (time_id, version_id) {
        time_id -> Int8,
        version_id -> Int8,
        count -> Int8,
    }
}

diesel::table! {
    history_client_version_statistics_version_number (id) {
        id -> Int8,
        major -> Int8,
        minor -> Int8,
        patch -> Int8,
    }
}

diesel::table! {
    history_common_statistics_save_time (id) {
        id -> Int8,
        unix_time -> Int8,
    }
}

diesel::table! {
    history_ip_country_statistics (time_id, country_id) {
        time_id -> Int8,
        country_id -> Int8,
        new_tcp_connections -> Int8,
        new_http_requests -> Int8,
    }
}

diesel::table! {
    history_ip_country_statistics_country_name (id) {
        id -> Int8,
        country_name -> Text,
    }
}

diesel::table! {
    history_performance_statistics_metric_name (id) {
        id -> Int8,
        metric_name -> Text,
    }
}

diesel::table! {
    history_performance_statistics_metric_value (time_id, metric_id) {
        time_id -> Int8,
        metric_id -> Int8,
        metric_value -> Int8,
    }
}

diesel::table! {
    history_profile_statistics_age_changes_all_genders (time_id, age) {
        time_id -> Int8,
        age -> Int2,
        count -> Int8,
    }
}

diesel::table! {
    history_profile_statistics_age_changes_man (time_id, age) {
        time_id -> Int8,
        age -> Int2,
        count -> Int8,
    }
}

diesel::table! {
    history_profile_statistics_age_changes_non_binary (time_id, age) {
        time_id -> Int8,
        age -> Int2,
        count -> Int8,
    }
}

diesel::table! {
    history_profile_statistics_age_changes_woman (time_id, age) {
        time_id -> Int8,
        age -> Int2,
        count -> Int8,
    }
}

diesel::table! {
    history_profile_statistics_count_changes_account (time_id) {
        time_id -> Int8,
        count -> Int8,
    }
}

diesel::table! {
    history_profile_statistics_count_changes_all_genders (time_id) {
        time_id -> Int8,
        count -> Int8,
    }
}

diesel::table! {
    history_profile_statistics_count_changes_man (time_id) {
        time_id -> Int8,
        count -> Int8,
    }
}

diesel::table! {
    history_profile_statistics_count_changes_non_binary (time_id) {
        time_id -> Int8,
        count -> Int8,
    }
}

diesel::table! {
    history_profile_statistics_count_changes_woman (time_id) {
        time_id -> Int8,
        count -> Int8,
    }
}

diesel::table! {
    ip_address_usage_statistics (account_id, ip_address) {
        account_id -> Int8,
        ip_address -> Bytea,
        usage_count -> Int8,
        first_usage_unix_time -> Int8,
        latest_usage_unix_time -> Int8,
    }
}

diesel::table! {
    latest_seen_message (account_id_viewer, account_id_sender) {
        account_id_viewer -> Int8,
        account_id_sender -> Int8,
        message_number -> Int8,
    }
}

diesel::table! {
    login_session (account_id) {
        account_id -> Int8,
        access_token -> Bytea,
        access_token_unix_time -> Int8,
        access_token_ip_address -> Bytea,
        refresh_token -> Bytea,
    }
}

diesel::table! {
    media_app_notification_settings (account_id) {
        account_id -> Int8,
        media_content_moderation -> Bool,
    }
}

diesel::table! {
    media_app_notification_state (account_id) {
        account_id -> Int8,
        media_content_accepted -> Int2,
        media_content_accepted_viewed -> Int2,
        media_content_rejected -> Int2,
        media_content_rejected_viewed -> Int2,
        media_content_deleted -> Int2,
        media_content_deleted_viewed -> Int2,
    }
}

diesel::table! {
    media_content (id) {
        id -> Int8,
        uuid -> Bytea,
        account_id -> Int8,
        secure_capture -> Bool,
        face_detected -> Bool,
        content_type_number -> Int2,
        slot_number -> Int2,
        creation_unix_time -> Int8,
        initial_content -> Bool,
        moderation_state -> Int2,
        moderation_rejected_reason_category -> Nullable<Int2>,
        moderation_rejected_reason_details -> Nullable<Text>,
        moderation_moderator_account_id -> Nullable<Int8>,
        usage_start_unix_time -> Nullable<Int8>,
        usage_end_unix_time -> Nullable<Int8>,
    }
}

diesel::table! {
    media_report_profile_content (report_id) {
        report_id -> Int8,
        profile_content_uuid -> Bytea,
    }
}

diesel::table! {
    media_state (account_id) {
        account_id -> Int8,
        media_content_sync_version -> Int2,
        profile_content_edited_unix_time -> Int8,
    }
}

diesel::table! {
    message_delivery_info (id) {
        id -> Int8,
        account_id_sender -> Int8,
        account_id_receiver -> Int8,
        message_id -> Bytea,
        delivery_info_type -> Int2,
        unix_time -> Int8,
    }
}

diesel::table! {
    news (id) {
        id -> Int8,
        account_id_creator -> Nullable<Int8>,
        first_publication_unix_time -> Nullable<Int8>,
        latest_publication_unix_time -> Nullable<Int8>,
        publication_id -> Nullable<Int8>,
    }
}

diesel::table! {
    news_translations (locale, news_id) {
        locale -> Text,
        news_id -> Int8,
        title -> Text,
        body -> Text,
        creation_unix_time -> Int8,
        version_number -> Int8,
        account_id_creator -> Nullable<Int8>,
        account_id_editor -> Nullable<Int8>,
        edit_unix_time -> Nullable<Int8>,
    }
}

diesel::table! {
    pending_messages (id) {
        id -> Int8,
        account_interaction -> Int8,
        account_id_sender -> Int8,
        account_id_receiver -> Int8,
        sender_acknowledgement -> Bool,
        receiver_acknowledgement -> Bool,
        receiver_push_notification_sent -> Bool,
        receiver_email_notification_sent -> Bool,
        message_unix_time -> Int8,
        message_number -> Int8,
        message_id -> Bytea,
        message_bytes -> Bytea,
    }
}

diesel::table! {
    profile (account_id) {
        account_id -> Int8,
        version_uuid -> Bytea,
        profile_name -> Nullable<Text>,
        profile_text -> Nullable<Text>,
        age -> Int2,
        last_seen_unix_time -> Int8,
    }
}

diesel::table! {
    profile_app_notification_settings (account_id) {
        account_id -> Int8,
        profile_string_moderation -> Bool,
        automatic_profile_search -> Bool,
    }
}

diesel::table! {
    profile_privacy_settings (account_id) {
        account_id -> Int8,
        online_status -> Bool,
        last_seen_time -> Bool,
    }
}

diesel::table! {
    profile_app_notification_state (account_id) {
        account_id -> Int8,
        profile_name_accepted -> Int2,
        profile_name_accepted_viewed -> Int2,
        profile_name_rejected -> Int2,
        profile_name_rejected_viewed -> Int2,
        profile_text_accepted -> Int2,
        profile_text_accepted_viewed -> Int2,
        profile_text_rejected -> Int2,
        profile_text_rejected_viewed -> Int2,
    }
}

diesel::table! {
    profile_attributes_file_hash (row_type) {
        row_type -> Int4,
        sha256_hash -> Text,
    }
}

diesel::table! {
    profile_attributes_filter_list_unwanted (account_id, attribute_id, filter_value) {
        account_id -> Int8,
        attribute_id -> Int2,
        filter_value -> Int8,
    }
}

diesel::table! {
    profile_attributes_filter_list_wanted (account_id, attribute_id, filter_value) {
        account_id -> Int8,
        attribute_id -> Int2,
        filter_value -> Int8,
    }
}

diesel::table! {
    profile_attributes_filter_settings (account_id, attribute_id) {
        account_id -> Int8,
        attribute_id -> Int2,
        filter_accept_missing_attribute -> Bool,
        filter_use_logical_operator_and -> Bool,
    }
}

diesel::table! {
    profile_attributes_value_list (account_id, attribute_id, attribute_value) {
        account_id -> Int8,
        attribute_id -> Int2,
        attribute_value -> Int8,
    }
}

diesel::table! {
    profile_automatic_profile_search_settings (account_id) {
        account_id -> Int8,
        new_profiles -> Bool,
        attribute_filters -> Bool,
        distance_filters -> Bool,
        weekdays -> Int2,
    }
}

diesel::table! {
    profile_automatic_profile_search_state (account_id) {
        account_id -> Int8,
        last_seen_unix_time -> Int8,
    }
}

diesel::table! {
    profile_moderation (account_id, content_type) {
        account_id -> Int8,
        content_type -> Int2,
        state_type -> Int2,
        rejected_reason_category -> Nullable<Int2>,
        rejected_reason_details -> Nullable<Text>,
        moderator_account_id -> Nullable<Int8>,
        created_unix_time -> Int8,
    }
}

diesel::table! {
    profile_name_allowlist (profile_name) {
        profile_name -> Text,
        name_creator_account_id -> Int8,
        name_moderator_account_id -> Nullable<Int8>,
    }
}

diesel::table! {
    profile_report_profile_name (report_id) {
        report_id -> Int8,
        profile_name -> Text,
    }
}

diesel::table! {
    profile_report_profile_text (report_id) {
        report_id -> Int8,
        profile_text -> Text,
    }
}

diesel::table! {
    profile_state (account_id) {
        account_id -> Int8,
        search_age_range_min -> Int2,
        search_age_range_max -> Int2,
        search_group_flags -> Int2,
        last_seen_time_filter -> Nullable<Int8>,
        unlimited_likes_filter -> Nullable<Bool>,
        min_distance_km_filter -> Nullable<Int2>,
        max_distance_km_filter -> Nullable<Int2>,
        profile_created_time_filter -> Nullable<Int8>,
        profile_edited_time_filter -> Nullable<Int8>,
        profile_text_min_characters_filter -> Nullable<Int2>,
        profile_text_max_characters_filter -> Nullable<Int2>,
        random_profile_order -> Bool,
        latitude -> Float8,
        longitude -> Float8,
        profile_sync_version -> Int2,
        initial_profile_age -> Nullable<Int2>,
        initial_profile_age_set_unix_time -> Nullable<Int8>,
        profile_edited_unix_time -> Int8,
    }
}

diesel::table! {
    public_key (account_id, key_id) {
        account_id -> Int8,
        key_id -> Int8,
        key_data -> Bytea,
        key_added_unix_time -> Int8,
    }
}

diesel::table! {
    push_notification (account_id) {
        account_id -> Int8,
        pending_flags -> Int8,
        sent_flags -> Int8,
        encryption_key -> Nullable<Text>,
        device_token -> Nullable<Text>,
        device_token_unix_time -> Nullable<Int8>,
        sync_version -> Int2,
    }
}

diesel::table! {
    shared_state (account_id) {
        account_id -> Int8,
        account_state_initial_setup_completed -> Bool,
        account_state_banned -> Bool,
        account_state_pending_deletion -> Bool,
        profile_visibility_state_number -> Int2,
        sync_version -> Int2,
        unlimited_likes -> Bool,
        birthdate -> Nullable<Date>,
        is_bot_account -> Bool,
        email_verified -> Bool,
        initial_setup_completed_unix_time -> Int8,
    }
}

diesel::table! {
    sign_in_with_info (account_id) {
        account_id -> Int8,
        apple_account_id -> Nullable<Text>,
        google_account_id -> Nullable<Text>,
    }
}

diesel::table! {
    used_account_ids (id) {
        id -> Int8,
        uuid -> Bytea,
    }
}

diesel::table! {
    used_content_ids (account_id, uuid) {
        account_id -> Int8,
        uuid -> Bytea,
    }
}

diesel::table! {
    vapid_public_key_hash (row_type) {
        row_type -> Int4,
        sha256_hash -> Text,
    }
}

diesel::joinable!(account_app_notification_settings -> account_id (account_id));
diesel::joinable!(account_email_address_state -> account_id (account_id));
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
diesel::joinable!(chat_privacy_settings -> account_id (account_id));
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
diesel::joinable!(profile_privacy_settings -> account_id (account_id));
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
    account_app_notification_settings,
    account_email_address_state,
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
    chat_privacy_settings,
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
    latest_seen_message,
    login_session,
    media_app_notification_settings,
    media_app_notification_state,
    media_content,
    media_report_profile_content,
    media_state,
    message_delivery_info,
    news,
    news_translations,
    pending_messages,
    profile,
    profile_app_notification_settings,
    profile_app_notification_state,
    profile_privacy_settings,
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
