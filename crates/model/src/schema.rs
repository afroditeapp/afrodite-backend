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

    account_capabilities (account_id) {
        account_id -> Integer,
        admin_modify_capabilities -> Bool,
        admin_moderate_profiles -> Bool,
        admin_moderate_images -> Bool,
        admin_view_all_profiles -> Bool,
        admin_view_private_info -> Bool,
        admin_view_profile_history -> Bool,
        admin_server_maintenance_view_info -> Bool,
        admin_server_maintenance_view_backend_config -> Bool,
        admin_server_maintenance_update_software -> Bool,
        admin_server_maintenance_reset_data -> Bool,
        admin_server_maintenance_reboot_backend -> Bool,
        admin_server_maintenance_save_backend_config -> Bool,
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
        message_counter -> Integer,
        sender_latest_viewed_message -> Nullable<Integer>,
        receiver_latest_viewed_message -> Nullable<Integer>,
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

    account_setup (account_id) {
        account_id -> Integer,
        name -> Text,
        birthdate -> Text,
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
        pending_security_content_id -> Nullable<Integer>,
        pending_profile_content_id_0 -> Nullable<Integer>,
        pending_profile_content_id_1 -> Nullable<Integer>,
        pending_profile_content_id_2 -> Nullable<Integer>,
        pending_profile_content_id_3 -> Nullable<Integer>,
        pending_profile_content_id_4 -> Nullable<Integer>,
        pending_profile_content_id_5 -> Nullable<Integer>,
        pending_grid_crop_size -> Nullable<Double>,
        pending_grid_crop_x -> Nullable<Double>,
        pending_grid_crop_y -> Nullable<Double>,
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

    history_account (id) {
        id -> Integer,
        account_id -> Integer,
        unix_time -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_account_setup (id) {
        id -> Integer,
        account_id -> Integer,
        unix_time -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_media_moderation_request (id) {
        id -> Integer,
        account_id -> Integer,
        unix_time -> Integer,
        moderation_request_id -> Integer,
        state_number -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    history_profile (id) {
        id -> Integer,
        account_id -> Integer,
        unix_time -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_content (id) {
        id -> Integer,
        uuid -> Binary,
        account_id -> Integer,
        content_state -> Integer,
        secure_capture -> Bool,
        content_type_number -> Integer,
        slot_number -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_moderation (account_id, moderation_request_id) {
        account_id -> Integer,
        moderation_request_id -> Integer,
        state_number -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_moderation_request (id) {
        id -> Integer,
        account_id -> Integer,
        queue_number -> Integer,
        queue_number_type -> Integer,
        content_id_0 -> Binary,
        content_id_1 -> Nullable<Binary>,
        content_id_2 -> Nullable<Binary>,
        content_id_3 -> Nullable<Binary>,
        content_id_4 -> Nullable<Binary>,
        content_id_5 -> Nullable<Binary>,
        content_id_6 -> Nullable<Binary>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_state (account_id) {
        account_id -> Integer,
        initial_moderation_request_accepted -> Bool,
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
        unix_time -> Integer,
        message_number -> Integer,
        message_text -> Text,
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

    profile_state (account_id) {
        account_id -> Integer,
        search_age_range_min -> Integer,
        search_age_range_max -> Integer,
        search_group_flags -> Integer,
        latitude -> Double,
        longitude -> Double,
        profile_attributes_sync_version -> Integer,
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
        account_state_number -> Integer,
        profile_visibility_state_number -> Integer,
        sync_version -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    sign_in_with_info (account_id) {
        account_id -> Integer,
        google_account_id -> Nullable<Text>,
        is_bot_account -> Bool,
    }
}

diesel::joinable!(access_token -> account_id (account_id));
diesel::joinable!(account -> account_id (account_id));
diesel::joinable!(account_capabilities -> account_id (account_id));
diesel::joinable!(account_email_sending_state -> account_id (account_id));
diesel::joinable!(account_interaction_index -> account_interaction (interaction_id));
diesel::joinable!(account_setup -> account_id (account_id));
diesel::joinable!(chat_state -> account_id (account_id));
diesel::joinable!(current_account_media -> account_id (account_id));
diesel::joinable!(history_account -> account_id (account_id));
diesel::joinable!(history_account_setup -> account_id (account_id));
diesel::joinable!(history_media_moderation_request -> account_id (account_id));
diesel::joinable!(history_profile -> account_id (account_id));
diesel::joinable!(media_content -> account_id (account_id));
diesel::joinable!(media_moderation -> account_id (account_id));
diesel::joinable!(media_moderation -> media_moderation_request (moderation_request_id));
diesel::joinable!(media_moderation_request -> account_id (account_id));
diesel::joinable!(media_state -> account_id (account_id));
diesel::joinable!(profile -> account_id (account_id));
diesel::joinable!(profile_attributes -> account_id (account_id));
diesel::joinable!(profile_attributes_number_list -> account_id (account_id));
diesel::joinable!(profile_attributes_number_list_filters -> account_id (account_id));
diesel::joinable!(profile_state -> account_id (account_id));
diesel::joinable!(queue_entry -> account_id (account_id));
diesel::joinable!(refresh_token -> account_id (account_id));
diesel::joinable!(shared_state -> account_id (account_id));
diesel::joinable!(sign_in_with_info -> account_id (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    access_token,
    account,
    account_capabilities,
    account_email_sending_state,
    account_global_state,
    account_id,
    account_interaction,
    account_interaction_index,
    account_setup,
    chat_state,
    current_account_media,
    demo_mode_account_ids,
    favorite_profile,
    history_account,
    history_account_setup,
    history_media_moderation_request,
    history_profile,
    media_content,
    media_moderation,
    media_moderation_request,
    media_state,
    next_queue_number,
    pending_messages,
    profile,
    profile_attributes,
    profile_attributes_file_hash,
    profile_attributes_number_list,
    profile_attributes_number_list_filters,
    profile_state,
    queue_entry,
    refresh_token,
    shared_state,
    sign_in_with_info,
);
