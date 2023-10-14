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
        json_text -> Text,
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
        email -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    current_account_media (account_id) {
        account_id -> Integer,
        security_content_id -> Nullable<Integer>,
        profile_content_id -> Nullable<Integer>,
        grid_crop_size -> Double,
        grid_crop_x -> Double,
        grid_crop_y -> Double,
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
        moderation_state -> Integer,
        content_type -> Integer,
        slot_number -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_moderation (account_id, moderation_request_id) {
        account_id -> Integer,
        moderation_request_id -> Integer,
        state_number -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_moderation_queue_number (queue_number) {
        queue_number -> Integer,
        account_id -> Integer,
        sub_queue -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    media_moderation_request (id) {
        id -> Integer,
        account_id -> Integer,
        queue_number -> Integer,
        json_text -> Text,
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
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    profile_location (account_id) {
        account_id -> Integer,
        latitude -> Double,
        longitude -> Double,
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

    sign_in_with_info (account_id) {
        account_id -> Integer,
        google_account_id -> Nullable<Text>,
    }
}

diesel::joinable!(access_token -> account_id (account_id));
diesel::joinable!(account -> account_id (account_id));
diesel::joinable!(account_interaction_index -> account_interaction (interaction_id));
diesel::joinable!(account_setup -> account_id (account_id));
diesel::joinable!(current_account_media -> account_id (account_id));
diesel::joinable!(history_account -> account_id (account_id));
diesel::joinable!(history_account_setup -> account_id (account_id));
diesel::joinable!(history_media_moderation_request -> account_id (account_id));
diesel::joinable!(history_profile -> account_id (account_id));
diesel::joinable!(media_content -> account_id (account_id));
diesel::joinable!(media_moderation -> account_id (account_id));
diesel::joinable!(media_moderation -> media_moderation_request (moderation_request_id));
diesel::joinable!(media_moderation_queue_number -> account_id (account_id));
diesel::joinable!(media_moderation_request -> account_id (account_id));
diesel::joinable!(profile -> account_id (account_id));
diesel::joinable!(profile_location -> account_id (account_id));
diesel::joinable!(refresh_token -> account_id (account_id));
diesel::joinable!(sign_in_with_info -> account_id (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    access_token,
    account,
    account_id,
    account_interaction,
    account_interaction_index,
    account_setup,
    current_account_media,
    favorite_profile,
    history_account,
    history_account_setup,
    history_media_moderation_request,
    history_profile,
    media_content,
    media_moderation,
    media_moderation_queue_number,
    media_moderation_request,
    pending_messages,
    profile,
    profile_location,
    refresh_token,
    sign_in_with_info,
);
