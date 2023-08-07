// @generated automatically by Diesel CLI.

diesel::table! {
    use crate::schema_sqlite_types::*;

    Account (account_row_id) {
        account_row_id -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    AccountId (account_row_id) {
        account_row_id -> Integer,
        account_id -> Binary,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    AccountSetup (account_row_id) {
        account_row_id -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    ApiKey (account_row_id) {
        account_row_id -> Integer,
        api_key -> Nullable<Text>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    CurrentAccountMedia (account_row_id) {
        account_row_id -> Integer,
        security_content_row_id -> Nullable<Integer>,
        profile_content_row_id -> Nullable<Integer>,
        grid_crop_size -> Float,
        grid_crop_x -> Float,
        grid_crop_y -> Float,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    HistoryAccount (row_id) {
        row_id -> Integer,
        account_row_id -> Integer,
        unix_time -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    HistoryAccountSetup (row_id) {
        row_id -> Integer,
        account_row_id -> Integer,
        unix_time -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    HistoryMediaModerationRequest (row_id) {
        row_id -> Integer,
        account_row_id -> Integer,
        unix_time -> Integer,
        request_row_id -> Integer,
        state_number -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    HistoryProfile (row_id) {
        row_id -> Integer,
        account_row_id -> Integer,
        unix_time -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    MediaContent (content_row_id) {
        content_row_id -> Integer,
        content_id -> Binary,
        account_row_id -> Integer,
        moderation_state -> Integer,
        content_type -> Integer,
        slot_number -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    MediaModeration (account_row_id, request_row_id) {
        account_row_id -> Integer,
        request_row_id -> Integer,
        state_number -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    MediaModerationQueueNumber (queue_number) {
        queue_number -> Integer,
        account_row_id -> Integer,
        sub_queue -> Integer,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    MediaModerationRequest (request_row_id) {
        request_row_id -> Integer,
        account_row_id -> Integer,
        queue_number -> Integer,
        json_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    Profile (account_row_id) {
        account_row_id -> Integer,
        version_uuid -> Binary,
        location_key_x -> Integer,
        location_key_y -> Integer,
        name -> Text,
        profile_text -> Text,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    RefreshToken (account_row_id) {
        account_row_id -> Integer,
        refresh_token -> Nullable<Binary>,
    }
}

diesel::table! {
    use crate::schema_sqlite_types::*;

    SignInWithInfo (account_row_id) {
        account_row_id -> Integer,
        google_account_id -> Nullable<Text>,
    }
}

diesel::joinable!(Account -> AccountId (account_row_id));
diesel::joinable!(AccountSetup -> AccountId (account_row_id));
diesel::joinable!(ApiKey -> AccountId (account_row_id));
diesel::joinable!(CurrentAccountMedia -> AccountId (account_row_id));
diesel::joinable!(HistoryAccount -> AccountId (account_row_id));
diesel::joinable!(HistoryAccountSetup -> AccountId (account_row_id));
diesel::joinable!(HistoryMediaModerationRequest -> AccountId (account_row_id));
diesel::joinable!(HistoryProfile -> AccountId (account_row_id));
diesel::joinable!(MediaContent -> AccountId (account_row_id));
diesel::joinable!(MediaModeration -> AccountId (account_row_id));
diesel::joinable!(MediaModeration -> MediaModerationRequest (request_row_id));
diesel::joinable!(MediaModerationQueueNumber -> AccountId (account_row_id));
diesel::joinable!(MediaModerationRequest -> AccountId (account_row_id));
diesel::joinable!(Profile -> AccountId (account_row_id));
diesel::joinable!(RefreshToken -> AccountId (account_row_id));
diesel::joinable!(SignInWithInfo -> AccountId (account_row_id));

diesel::allow_tables_to_appear_in_same_query!(
    Account,
    AccountId,
    AccountSetup,
    ApiKey,
    CurrentAccountMedia,
    HistoryAccount,
    HistoryAccountSetup,
    HistoryMediaModerationRequest,
    HistoryProfile,
    MediaContent,
    MediaModeration,
    MediaModerationQueueNumber,
    MediaModerationRequest,
    Profile,
    RefreshToken,
    SignInWithInfo,
);
