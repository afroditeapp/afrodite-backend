// @generated automatically by Diesel CLI.

diesel::table! {
    use crate::schema_sqlite_types::*;

    access_token (account_id) {
        account_id -> Integer,
        token -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account (account_id) {
        account_id -> Integer,
        email -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_email_sending_state (account_id) {
        account_id -> Integer,
        account_registered_state_number -> Integer,
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
        sender_latest_viewed_message -> Integer,
        receiver_latest_viewed_message -> Integer,
        included_in_received_new_likes_count -> Bool,
        received_like_id -> Nullable<Integer>,
        match_id -> Nullable<Integer>,
        account_id_previous_like_deleter_slot_0 -> Nullable<Integer>,
        account_id_previous_like_deleter_slot_1 -> Nullable<Integer>,
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
        admin_modify_permissions -> Bool,
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
        admin_server_maintenance_reboot_backend -> Bool,
        admin_server_maintenance_save_backend_config -> Bool,
        admin_server_maintenance_edit_notification -> Bool,
        admin_news_create -> Bool,
        admin_news_edit_all -> Bool,
        admin_profile_statistics -> Bool,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    account_report (creator_account_id, target_account_id) {
        creator_account_id -> Integer,
        target_account_id -> Integer,
        creation_unix_time -> Integer,
        content_edit_unix_time -> Integer,
        moderator_account_id -> Nullable<Integer>,
        processing_state -> Integer,
        processing_state_change_unix_time -> Integer,
        is_bot -> Bool,
        is_scammer -> Bool,
        is_spammer -> Bool,
        is_underaged -> Bool,
        details -> Nullable<Text>,
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
        account_banned_reason_details -> Nullable<Text>,
        account_banned_admin_account_id -> Nullable<Integer>,
        account_banned_until_unix_time -> Nullable<Integer>,
        account_banned_state_change_unix_time -> Nullable<Integer>,
        news_sync_version -> Integer,
        unread_news_count -> Integer,
        publication_id_at_news_iterator_reset -> Nullable<Integer>,
        publication_id_at_unread_news_count_incrementing -> Nullable<Integer>,
        account_created_unix_time -> Integer,
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
        chat_message -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    chat_state (account_id) {
        account_id -> Integer,
        received_blocks_sync_version -> Integer,
        received_likes_sync_version -> Integer,
        sent_blocks_sync_version -> Integer,
        sent_likes_sync_version -> Integer,
        matches_sync_version -> Integer,
        pending_notification -> Integer,
        pending_notification_token -> Nullable<Text>,
        fcm_notification_sent -> Bool,
        fcm_device_token -> Nullable<Text>,
        new_received_likes_count -> Integer,
        next_received_like_id -> Integer,
        received_like_id_at_received_likes_iterator_reset -> Nullable<Integer>,
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

    demo_mode_account_ids (id) {
        id -> Integer,
        demo_mode_id -> Integer,
        account_id_uuid -> Binary,
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

    history_performance_statistics_save_time (id) {
        id -> Integer,
        unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_all_genders (save_time_id, age) {
        save_time_id -> Integer,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_men (save_time_id, age) {
        save_time_id -> Integer,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_non_binary (save_time_id, age) {
        save_time_id -> Integer,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_age_changes_woman (save_time_id, age) {
        save_time_id -> Integer,
        age -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_account (save_time_id) {
        save_time_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_all_genders (save_time_id) {
        save_time_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_man (save_time_id) {
        save_time_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_non_binary (save_time_id) {
        save_time_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_count_changes_woman (save_time_id) {
        save_time_id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile_statistics_save_time (id) {
        id -> Integer,
        unix_time -> Integer,
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
        moderation_rejected_reason_details -> Nullable<Text>,
        moderation_moderator_account_id -> Nullable<Integer>,
        usage_start_unix_time -> Nullable<Integer>,
        usage_end_unix_time -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_report_profile_content (report_id) {
        report_id -> Integer,
        profile_content_uuid -> Nullable<Binary>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_state (account_id) {
        account_id -> Integer,
        initial_moderation_request_accepted -> Bool,
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

    next_queue_number (queue_type_number) {
        queue_type_number -> Integer,
        next_number -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    pending_messages (id) {
        id -> Integer,
        account_id_sender -> Integer,
        account_id_receiver -> Integer,
        sender_acknowledgement -> Bool,
        receiver_acknowledgement -> Bool,
        unix_time -> Integer,
        message_number -> Integer,
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
        name -> Text,
        profile_text -> Text,
        age -> Integer,
        last_seen_unix_time -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes (account_id, attribute_id) {
        account_id -> Integer,
        attribute_id -> Integer,
        attribute_value_part1 -> Nullable<Integer>,
        attribute_value_part2 -> Nullable<Integer>,
        filter_value_part1 -> Nullable<Integer>,
        filter_value_part2 -> Nullable<Integer>,
        filter_accept_missing_attribute -> Nullable<Bool>,
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

    profile_attributes_number_list (account_id, attribute_id, attribute_value) {
        account_id -> Integer,
        attribute_id -> Integer,
        attribute_value -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_attributes_number_list_filters (account_id, attribute_id, filter_value) {
        account_id -> Integer,
        attribute_id -> Integer,
        filter_value -> Integer,
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
        profile_name -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_report_profile_text (report_id) {
        report_id -> Integer,
        profile_text -> Nullable<Text>,
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
        max_distance_km_filter -> Nullable<Integer>,
        profile_created_time_filter -> Nullable<Integer>,
        profile_edited_time_filter -> Nullable<Integer>,
        random_profile_order -> Bool,
        latitude -> Double,
        longitude -> Double,
        profile_sync_version -> Integer,
        profile_initial_age -> Nullable<Integer>,
        profile_initial_age_set_unix_time -> Nullable<Integer>,
        profile_name_moderation_state -> Integer,
        profile_text_moderation_state -> Integer,
        profile_text_moderation_rejected_reason_category -> Nullable<Integer>,
        profile_text_moderation_rejected_reason_details -> Nullable<Text>,
        profile_text_moderation_moderator_account_id -> Nullable<Integer>,
        profile_text_edit_time_unix_time -> Nullable<Integer>,
        profile_edited_unix_time -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    public_key (account_id, public_key_version) {
        account_id -> Integer,
        public_key_version -> Integer,
        public_key_id -> Nullable<Integer>,
        public_key_data -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    queue_entry (queue_number, queue_type_number) {
        queue_number -> Integer,
        queue_type_number -> Integer,
        account_id -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    refresh_token (account_id) {
        account_id -> Integer,
        token -> Nullable<Binary>,
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

diesel::joinable!(access_token -> account_id (account_id));
diesel::joinable!(account -> account_id (account_id));
diesel::joinable!(account_email_sending_state -> account_id (account_id));
diesel::joinable!(account_interaction_index -> account_interaction (interaction_id));
diesel::joinable!(account_permissions -> account_id (account_id));
diesel::joinable!(account_setup -> account_id (account_id));
diesel::joinable!(chat_report_chat_message -> common_report (report_id));
diesel::joinable!(chat_state -> account_id (account_id));
diesel::joinable!(common_state -> account_id (account_id));
diesel::joinable!(current_account_media -> account_id (account_id));
diesel::joinable!(history_performance_statistics_metric_value -> history_performance_statistics_metric_name (metric_id));
diesel::joinable!(history_performance_statistics_metric_value -> history_performance_statistics_save_time (time_id));
diesel::joinable!(history_profile_statistics_age_changes_all_genders -> history_profile_statistics_save_time (save_time_id));
diesel::joinable!(history_profile_statistics_age_changes_men -> history_profile_statistics_save_time (save_time_id));
diesel::joinable!(history_profile_statistics_age_changes_non_binary -> history_profile_statistics_save_time (save_time_id));
diesel::joinable!(history_profile_statistics_age_changes_woman -> history_profile_statistics_save_time (save_time_id));
diesel::joinable!(history_profile_statistics_count_changes_account -> history_profile_statistics_save_time (save_time_id));
diesel::joinable!(history_profile_statistics_count_changes_all_genders -> history_profile_statistics_save_time (save_time_id));
diesel::joinable!(history_profile_statistics_count_changes_man -> history_profile_statistics_save_time (save_time_id));
diesel::joinable!(history_profile_statistics_count_changes_non_binary -> history_profile_statistics_save_time (save_time_id));
diesel::joinable!(history_profile_statistics_count_changes_woman -> history_profile_statistics_save_time (save_time_id));
diesel::joinable!(media_content -> account_id (account_id));
diesel::joinable!(media_report_profile_content -> common_report (report_id));
diesel::joinable!(media_state -> account_id (account_id));
diesel::joinable!(news -> account_id (account_id_creator));
diesel::joinable!(news_translations -> news (news_id));
diesel::joinable!(profile -> account_id (account_id));
diesel::joinable!(profile_attributes -> account_id (account_id));
diesel::joinable!(profile_attributes_number_list -> account_id (account_id));
diesel::joinable!(profile_attributes_number_list_filters -> account_id (account_id));
diesel::joinable!(profile_report_profile_name -> common_report (report_id));
diesel::joinable!(profile_report_profile_text -> common_report (report_id));
diesel::joinable!(public_key -> account_id (account_id));
diesel::joinable!(queue_entry -> account_id (account_id));
diesel::joinable!(refresh_token -> account_id (account_id));
diesel::joinable!(shared_state -> account_id (account_id));
diesel::joinable!(sign_in_with_info -> account_id (account_id));
diesel::joinable!(used_content_ids -> account_id (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    access_token,
    account,
    account_email_sending_state,
    account_global_state,
    account_id,
    account_interaction,
    account_interaction_index,
    account_permissions,
    account_report,
    account_setup,
    account_state,
    chat_global_state,
    chat_report_chat_message,
    chat_state,
    common_report,
    common_state,
    current_account_media,
    custom_reports_file_hash,
    demo_mode_account_ids,
    favorite_profile,
    history_performance_statistics_metric_name,
    history_performance_statistics_metric_value,
    history_performance_statistics_save_time,
    history_profile_statistics_age_changes_all_genders,
    history_profile_statistics_age_changes_men,
    history_profile_statistics_age_changes_non_binary,
    history_profile_statistics_age_changes_woman,
    history_profile_statistics_count_changes_account,
    history_profile_statistics_count_changes_all_genders,
    history_profile_statistics_count_changes_man,
    history_profile_statistics_count_changes_non_binary,
    history_profile_statistics_count_changes_woman,
    history_profile_statistics_save_time,
    media_content,
    media_report_profile_content,
    media_state,
    news,
    news_translations,
    next_queue_number,
    pending_messages,
    profile,
    profile_attributes,
    profile_attributes_file_hash,
    profile_attributes_number_list,
    profile_attributes_number_list_filters,
    profile_name_allowlist,
    profile_report_profile_name,
    profile_report_profile_text,
    profile_state,
    public_key,
    queue_entry,
    refresh_token,
    shared_state,
    sign_in_with_info,
    used_account_ids,
    used_content_ids,
);
