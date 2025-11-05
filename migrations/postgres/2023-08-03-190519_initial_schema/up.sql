-- Your SQL goes here

---------- Tables for server component common ----------

-- All used account IDs. Account ID is not removed from here
-- when account data is removed.
CREATE TABLE IF NOT EXISTS used_account_ids(
    id         BIGSERIAL PRIMARY KEY             NOT NULL,
    uuid       BYTEA                             NOT NULL UNIQUE
);

-- Account IDs for currently existing accounts
CREATE TABLE IF NOT EXISTS account_id(
    id    BIGINT PRIMARY KEY                NOT NULL,
    -- Main UUID for account.
    -- This is used internally in the server, client and API level.
    -- Also this should be not used as somekind of secret as it
    -- can be seen from filesystem.
    uuid  BYTEA                             NOT NULL  UNIQUE,
    FOREIGN KEY (id)
        REFERENCES used_account_ids (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS login_session(
    account_id              BIGINT PRIMARY KEY  NOT NULL,
    -- Rust HashMap guarantees access token uniqueness, so
    -- UNIQUE constrait is not needed here.
    access_token            BYTEA               NOT NULL,
    access_token_unix_time  BIGINT              NOT NULL,
    -- 4 or 16 bytes
    access_token_ip_address BYTEA               NOT NULL,
    -- Using refresh token requires valid access token, so
    -- UNIQUE constraint is not needed here.
    refresh_token           BYTEA               NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Account permissions are shared between server components.
-- If the data is located in this table it should be set through account
-- server as it propagates the changes to other components.
CREATE TABLE IF NOT EXISTS account_permissions(
    account_id    BIGINT PRIMARY KEY NOT NULL,
    admin_edit_login                             BOOLEAN NOT NULL DEFAULT FALSE,
    admin_edit_permissions                       BOOLEAN NOT NULL DEFAULT FALSE,
    admin_edit_profile_name                      BOOLEAN NOT NULL DEFAULT FALSE,
    admin_edit_max_public_key_count              BOOLEAN NOT NULL DEFAULT FALSE,
    admin_edit_media_content_face_detected_value BOOLEAN NOT NULL DEFAULT FALSE,
    admin_export_data                            BOOLEAN NOT NULL DEFAULT FALSE,
    admin_moderate_media_content                 BOOLEAN NOT NULL DEFAULT FALSE,
    admin_moderate_profile_names                 BOOLEAN NOT NULL DEFAULT FALSE,
    admin_moderate_profile_texts                 BOOLEAN NOT NULL DEFAULT FALSE,
    admin_process_reports                        BOOLEAN NOT NULL DEFAULT FALSE,
    admin_delete_media_content                   BOOLEAN NOT NULL DEFAULT FALSE,
    admin_delete_account                         BOOLEAN NOT NULL DEFAULT FALSE,
    admin_ban_account                            BOOLEAN NOT NULL DEFAULT FALSE,
    admin_request_account_deletion               BOOLEAN NOT NULL DEFAULT FALSE,
    admin_view_all_profiles                      BOOLEAN NOT NULL DEFAULT FALSE,
    admin_view_private_info                      BOOLEAN NOT NULL DEFAULT FALSE,
    admin_view_profile_history                   BOOLEAN NOT NULL DEFAULT FALSE,
    admin_view_permissions                       BOOLEAN NOT NULL DEFAULT FALSE,
    admin_view_email_address                     BOOLEAN NOT NULL DEFAULT FALSE,
    admin_find_account_by_email                  BOOLEAN NOT NULL DEFAULT FALSE,
    admin_server_maintenance_view_info           BOOLEAN NOT NULL DEFAULT FALSE,
    admin_server_maintenance_view_backend_config BOOLEAN NOT NULL DEFAULT FALSE,
    admin_server_maintenance_update_software     BOOLEAN NOT NULL DEFAULT FALSE,
    admin_server_maintenance_reset_data          BOOLEAN NOT NULL DEFAULT FALSE,
    admin_server_maintenance_restart_backend     BOOLEAN NOT NULL DEFAULT FALSE,
    admin_server_maintenance_save_backend_config BOOLEAN NOT NULL DEFAULT FALSE,
    admin_server_maintenance_edit_notification   BOOLEAN NOT NULL DEFAULT FALSE,
    admin_news_create                            BOOLEAN NOT NULL DEFAULT FALSE,
    admin_news_edit_all                          BOOLEAN NOT NULL DEFAULT FALSE,
    admin_profile_statistics                     BOOLEAN NOT NULL DEFAULT FALSE,
    admin_subscribe_admin_notifications          BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- TODO(prod): Add subscription level to shared_state

-- Shared state between server components.
-- If the data is located in this table it should be set through account
-- server as it propagates the changes to other components.
CREATE TABLE IF NOT EXISTS shared_state(
    account_id                BIGINT PRIMARY KEY NOT NULL,
    account_state_initial_setup_completed   BOOLEAN             NOT NULL DEFAULT FALSE,
    account_state_banned                    BOOLEAN             NOT NULL DEFAULT FALSE,
    account_state_pending_deletion          BOOLEAN             NOT NULL DEFAULT FALSE,
    -- pending private = 0
    -- pending public = 1
    -- private = 2
    -- public = 3
    profile_visibility_state_number SMALLINT NOT NULL DEFAULT 0,
    -- Version number which only account server increments.
    -- Used in receiving end to avoid saving old state in case of
    -- concurrent updates.
    sync_version              SMALLINT             NOT NULL DEFAULT 0,
    unlimited_likes           BOOLEAN              NOT NULL DEFAULT FALSE,
    -- Birthdate has YYYY-MM-DD format. This is in shared state if
    -- birthdate validation using third party service is implemented
    -- someday.
    birthdate                 DATE,
    is_bot_account            BOOLEAN              NOT NULL DEFAULT FALSE,
    email_verified            BOOLEAN              NOT NULL DEFAULT FALSE,
    -- Profile component uses this info for profile filtering.
    initial_setup_completed_unix_time BIGINT       NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS common_report(
    id                      BIGSERIAL PRIMARY KEY NOT NULL,
    creator_account_id      BIGINT              NOT NULL,
    target_account_id       BIGINT              NOT NULL,
    -- 0 = profile name
    -- 1 = profile text
    -- 2 = profile content
    -- 3 = chat message
    -- Values from 64 to 127 are reserved for custom reports.
    report_type_number      SMALLINT            NOT NULL,
    creation_unix_time      BIGINT              NOT NULL,
    moderator_account_id    BIGINT,
    -- 0 = Waiting
    -- 1 = Done
    processing_state        SMALLINT            NOT NULL    DEFAULT 0,
    processing_state_change_unix_time BIGINT    NOT NULL,
    FOREIGN KEY (creator_account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (target_account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (moderator_account_id)
        REFERENCES account_id (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

-- State specific to all components.
CREATE TABLE IF NOT EXISTS common_state(
    account_id                    BIGINT PRIMARY KEY  NOT NULL,
    -- Sync version for client config.
    client_config_sync_version    SMALLINT            NOT NULL DEFAULT 0,
    -- 0 = Android
    -- 1 = iOS
    -- 2 = Web
    client_login_session_platform SMALLINT,
    -- Null or non-empty string
    client_language               TEXT,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS push_notification(
    account_id             BIGINT PRIMARY KEY  NOT NULL,
    -- Bitflag value for pending push notification
    pending_flags          BIGINT              NOT NULL DEFAULT 0,
    -- Bitflag value for sent push notifications. Used
    -- to prevent showing same notification again when WebSocket
    -- connects.
    sent_flags             BIGINT              NOT NULL DEFAULT 0,
    -- Push notification encryption key for APNs and FCM notifications
    encryption_key         TEXT,
    device_token           TEXT                         UNIQUE,
    -- Time when a token is saved. Not currently used for anything.
    -- Firebase docs recommend storing a timestamp with a token.
    device_token_unix_time BIGINT,
    sync_version           SMALLINT            NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS api_usage_statistics_save_time(
    id           BIGSERIAL PRIMARY KEY             NOT NULL,
    unix_time    BIGINT                            NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS api_usage_statistics_metric_name(
    id           BIGSERIAL PRIMARY KEY             NOT NULL,
    metric_name  TEXT                              NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS api_usage_statistics_metric_value(
    account_id   BIGINT                            NOT NULL,
    time_id      BIGINT                            NOT NULL,
    metric_id    BIGINT                            NOT NULL,
    metric_value BIGINT                            NOT NULL,
    PRIMARY KEY (account_id, time_id, metric_id),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (time_id)
        REFERENCES api_usage_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (metric_id)
        REFERENCES api_usage_statistics_metric_name (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS ip_address_usage_statistics(
    account_id             BIGINT                  NOT NULL,
    -- 4 or 16 bytes
    ip_address             BYTEA                   NOT NULL,
    usage_count            BIGINT                  NOT NULL,
    first_usage_unix_time  BIGINT                  NOT NULL,
    latest_usage_unix_time BIGINT                  NOT NULL,
    PRIMARY KEY (account_id, ip_address),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS admin_notification_settings(
    account_id                       BIGINT PRIMARY KEY NOT NULL,
    weekdays                         SMALLINT NOT NULL,
    daily_enabled_time_start_seconds INTEGER NOT NULL,
    daily_enabled_time_end_seconds   INTEGER NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS admin_notification_subscriptions(
    account_id BIGINT PRIMARY KEY NOT NULL,
    moderate_initial_media_content_bot           BOOLEAN NOT NULL DEFAULT FALSE,
    moderate_initial_media_content_human         BOOLEAN NOT NULL DEFAULT FALSE,
    moderate_media_content_bot                   BOOLEAN NOT NULL DEFAULT FALSE,
    moderate_media_content_human                 BOOLEAN NOT NULL DEFAULT FALSE,
    moderate_profile_texts_bot                   BOOLEAN NOT NULL DEFAULT FALSE,
    moderate_profile_texts_human                 BOOLEAN NOT NULL DEFAULT FALSE,
    moderate_profile_names_bot                   BOOLEAN NOT NULL DEFAULT FALSE,
    moderate_profile_names_human                 BOOLEAN NOT NULL DEFAULT FALSE,
    process_reports                              BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Store VAPID public key hash, so that changes to it can be detected
-- when server starts.
CREATE TABLE IF NOT EXISTS vapid_public_key_hash(
    -- 0 = VAPID hash
    row_type      INTEGER PRIMARY KEY NOT NULL,
    sha256_hash   TEXT                NOT NULL
);

---------- Tables for server component account ----------

-- Sign in with related IDs for account
CREATE TABLE IF NOT EXISTS sign_in_with_info(
    account_id         BIGINT PRIMARY KEY NOT NULL,
    apple_account_id   TEXT                          UNIQUE,
    google_account_id  TEXT                          UNIQUE,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Account information which can change
CREATE TABLE IF NOT EXISTS account(
    account_id   BIGINT PRIMARY KEY NOT NULL,
    email        TEXT                                UNIQUE,
    email_verification_token           BYTEA         UNIQUE,
    email_verification_token_unix_time BIGINT,
    -- Pending new email address
    email_change TEXT,
    -- Time when pending new email address is set
    email_change_unix_time BIGINT,
    -- Verification token for pending new email address.
    -- email_change_unix_time tracks token validity.
    -- Verification request email is sent when email_change is set.
    email_change_verification_token           BYTEA         UNIQUE,
    -- Verification status of email_change.
    -- This is required to be TRUE when backend logic changes the pending
    -- email to account's email address.
    email_change_verified           BOOLEAN   NOT NULL DEFAULT FALSE,
    email_login_token                         BYTEA         UNIQUE,
    email_login_token_unix_time               BIGINT,
    email_login_enabled                       BOOLEAN   NOT NULL DEFAULT TRUE,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Information which can not change after account initial setup completes
CREATE TABLE IF NOT EXISTS account_setup(
    account_id  BIGINT PRIMARY KEY NOT NULL,
    -- Birthdate has YYYY-MM-DD format. This is the birthdate from user when
    -- account initial setup is done. The birthdate in shared_state can
    -- be modified later.
    birthdate                 DATE,
    is_adult                  BOOLEAN,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Account related email sending state
-- State numbers have these values
-- 0 - Not sent
-- 1 - Sending requested
-- 2 - Sent successfully
CREATE TABLE IF NOT EXISTS account_email_sending_state(
    account_id                      BIGINT PRIMARY KEY  NOT NULL,
    email_verification_state_number SMALLINT            NOT NULL DEFAULT 0,
    new_message_state_number        SMALLINT            NOT NULL DEFAULT 0,
    new_like_state_number           SMALLINT            NOT NULL DEFAULT 0,
    account_deletion_remainder_first_state_number  SMALLINT NOT NULL DEFAULT 0,
    account_deletion_remainder_second_state_number SMALLINT NOT NULL DEFAULT 0,
    account_deletion_remainder_third_state_number  SMALLINT NOT NULL DEFAULT 0,
    email_change_verification_state_number         SMALLINT NOT NULL DEFAULT 0,
    email_change_notification_state_number         SMALLINT NOT NULL DEFAULT 0,
    email_login_state_number                       SMALLINT NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- State specific to account component.
CREATE TABLE IF NOT EXISTS account_state(
    account_id                         BIGINT PRIMARY KEY NOT NULL,
    next_client_id                     BIGINT             NOT NULL DEFAULT 0,
    account_deletion_request_unix_time BIGINT,
    account_banned_reason_category     SMALLINT,
    -- Null or non-empty string
    account_banned_reason_details      TEXT,
    account_banned_admin_account_id    BIGINT,
    account_banned_until_unix_time     BIGINT,
    account_banned_state_change_unix_time BIGINT,
    -- Sync version for news.
    news_sync_version                  SMALLINT            NOT NULL DEFAULT 0,
    unread_news_count                  BIGINT              NOT NULL DEFAULT 0,
    account_created_unix_time          BIGINT              NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_banned_admin_account_id)
        REFERENCES account_id (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS account_app_notification_settings(
    account_id                         BIGINT PRIMARY KEY NOT NULL,
    news                               BOOLEAN             NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS demo_account_owned_accounts(
    -- These are defined in config file
    demo_account_id BIGINT              NOT NULL,
    account_id      BIGINT              NOT NULL,
    PRIMARY KEY (demo_account_id, account_id),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS news(
    id                    BIGSERIAL PRIMARY KEY NOT NULL,
    account_id_creator    BIGINT,
    first_publication_unix_time  BIGINT,
    latest_publication_unix_time BIGINT,
    -- If publication ID exists the news are public.
    publication_id        BIGINT,
    FOREIGN KEY (account_id_creator)
        REFERENCES account_id (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS news_translations(
    locale                TEXT                NOT NULL,
    news_id               BIGINT              NOT NULL,
    title                 TEXT                NOT NULL,
    body                  TEXT                NOT NULL,
    creation_unix_time    BIGINT              NOT NULL,
    version_number        BIGINT              NOT NULL DEFAULT 0,
    account_id_creator    BIGINT,
    account_id_editor     BIGINT,
    edit_unix_time        BIGINT,
    PRIMARY KEY (locale, news_id),
    FOREIGN KEY (news_id)
        REFERENCES news (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_creator)
        REFERENCES account_id (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_editor)
        REFERENCES account_id (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS account_global_state(
    -- 0 = account component global state
    row_type                   INTEGER PRIMARY KEY NOT NULL,
    admin_access_granted_count BIGINT              NOT NULL DEFAULT 0,
    -- Publication ID for news which always increments.
    next_news_publication_id   BIGINT              NOT NULL DEFAULT 0
);

-- Store custom reports file hash, so that changes to it can be detected
-- when server starts.
CREATE TABLE IF NOT EXISTS custom_reports_file_hash(
    -- 0 = custom reports file hash
    row_type      INTEGER PRIMARY KEY NOT NULL,
    sha256_hash   TEXT                NOT NULL
);

-- Store client features file hash, so that changes to it can be detected
-- when server starts.
CREATE TABLE IF NOT EXISTS client_features_file_hash(
    -- 0 = client features file hash
    row_type      INTEGER PRIMARY KEY NOT NULL,
    sha256_hash   TEXT                NOT NULL
);

---------- Tables for server component profile ----------

-- Private profile related state for some account.
CREATE TABLE IF NOT EXISTS profile_state(
    account_id                 BIGINT PRIMARY KEY   NOT NULL,
    -- Min age in years and inside inclusive range of [18,99] for
    -- searching profiles.
    search_age_range_min       SMALLINT             NOT NULL    DEFAULT 18,
    -- Max age in years and inside inclusive range of [18,99] for
    -- searching profiles.
    search_age_range_max       SMALLINT             NOT NULL    DEFAULT 18,
    -- Bitflags value containing gender and genders that
    -- the profile owner searches for.
    search_group_flags         SMALLINT              NOT NULL    DEFAULT 0,
    -- Filter setting for last seen time.
    last_seen_time_filter      BIGINT,
    -- Filter setting for unlimited likes.
    unlimited_likes_filter     BOOLEAN,
    -- Filter setting for profile iterator min distance in kilometers.
    min_distance_km_filter     SMALLINT,
    -- Filter setting for profile iterator max distance in kilometers.
    max_distance_km_filter     SMALLINT,
    -- Filter setting for profile created time in seconds.
    profile_created_time_filter BIGINT,
    -- Filter setting for profile edited time in seconds.
    profile_edited_time_filter BIGINT,
    -- Filter setting for profile text min character count.
    profile_text_min_characters_filter SMALLINT,
    -- Filter setting for profile text max character count.
    profile_text_max_characters_filter SMALLINT,
    -- Profile iterator setting for random profile order.
    random_profile_order       BOOLEAN              NOT NULL    DEFAULT FALSE,
    latitude                   DOUBLE PRECISION               NOT NULL    DEFAULT 0.0,
    longitude                  DOUBLE PRECISION               NOT NULL    DEFAULT 0.0,
    -- Sync version for profile data for this account.
    profile_sync_version              SMALLINT      NOT NULL    DEFAULT 0,
    -- Profile age when initial setup is completed
    initial_profile_age               SMALLINT,
    initial_profile_age_set_unix_time BIGINT,
    -- Edit time for public profile changes. This updates from both
    -- user and admin made changes.
    profile_edited_unix_time          BIGINT        NOT NULL    DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Profile information which can be sent to clients if
-- profile visibility is public.
CREATE TABLE IF NOT EXISTS profile(
    account_id      BIGINT PRIMARY KEY  NOT NULL,
    version_uuid    BYTEA               NOT NULL,
    -- Null or non-empty string
    profile_name    TEXT,
    -- Null or non-empty string
    profile_text    TEXT,
    -- Age in years and inside inclusive range of [18,99].
    age             SMALLINT            NOT NULL    DEFAULT 18,
    last_seen_unix_time  BIGINT         NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Store profile attribute specific filter settings
CREATE TABLE IF NOT EXISTS profile_attributes_filter_settings(
    account_id      BIGINT              NOT NULL,
    attribute_id    SMALLINT            NOT NULL,
    filter_accept_missing_attribute BOOLEAN NOT NULL DEFAULT FALSE,
    filter_use_logical_operator_and BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (account_id, attribute_id),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_attributes_value_list(
    account_id      BIGINT              NOT NULL,
    attribute_id    SMALLINT            NOT NULL,
    attribute_value BIGINT              NOT NULL,
    PRIMARY KEY (account_id, attribute_id, attribute_value),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_attributes_filter_list_wanted(
    account_id      BIGINT              NOT NULL,
    attribute_id    SMALLINT            NOT NULL,
    filter_value    BIGINT              NOT NULL,
    PRIMARY KEY (account_id, attribute_id, filter_value),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_attributes_filter_list_unwanted(
    account_id      BIGINT              NOT NULL,
    attribute_id    SMALLINT            NOT NULL,
    filter_value    BIGINT              NOT NULL,
    PRIMARY KEY (account_id, attribute_id, filter_value),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Store profile attributes file hash, so that changes to it can be detected
-- when server starts.
CREATE TABLE IF NOT EXISTS profile_attributes_file_hash(
    -- 0 = profile attributes file hash
    row_type      INTEGER PRIMARY KEY NOT NULL,
    sha256_hash   TEXT                NOT NULL
);

CREATE TABLE IF NOT EXISTS favorite_profile(
    -- Account which marked the profile as a favorite.
    account_id          BIGINT                NOT NULL,
    -- Account which profile is marked as a favorite.
    favorite_account_id BIGINT                NOT NULL,
    -- Unix timestamp when favorite was added.
    unix_time           BIGINT                NOT NULL,
    PRIMARY KEY (account_id, favorite_account_id),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (favorite_account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_name_allowlist(
    profile_name              TEXT    PRIMARY KEY NOT NULL,
    name_creator_account_id   BIGINT              NOT NULL,
    name_moderator_account_id BIGINT,
    FOREIGN KEY (name_creator_account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (name_moderator_account_id)
        REFERENCES account_id (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_report_profile_name(
    report_id               BIGINT PRIMARY KEY  NOT NULL,
    -- Non-empty string
    profile_name            TEXT                NOT NULL,
    FOREIGN KEY (report_id)
        REFERENCES common_report (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_report_profile_text(
    report_id               BIGINT PRIMARY KEY NOT NULL,
    -- Non-empty string
    profile_text            TEXT                NOT NULL,
    FOREIGN KEY (report_id)
        REFERENCES common_report (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_app_notification_settings(
    account_id                                 BIGINT PRIMARY KEY  NOT NULL,
    profile_string_moderation                  BOOLEAN             NOT NULL,
    automatic_profile_search                   BOOLEAN             NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_app_notification_state(
    account_id                         BIGINT PRIMARY KEY  NOT NULL,
    profile_name_accepted              SMALLINT            NOT NULL DEFAULT 0,
    profile_name_accepted_viewed       SMALLINT            NOT NULL DEFAULT 0,
    profile_name_rejected              SMALLINT            NOT NULL DEFAULT 0,
    profile_name_rejected_viewed       SMALLINT            NOT NULL DEFAULT 0,
    profile_text_accepted              SMALLINT            NOT NULL DEFAULT 0,
    profile_text_accepted_viewed       SMALLINT            NOT NULL DEFAULT 0,
    profile_text_rejected              SMALLINT            NOT NULL DEFAULT 0,
    profile_text_rejected_viewed       SMALLINT            NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_automatic_profile_search_settings(
    account_id        BIGINT PRIMARY KEY  NOT NULL,
    new_profiles      BOOLEAN             NOT NULL,
    attribute_filters BOOLEAN             NOT NULL,
    distance_filters  BOOLEAN             NOT NULL,
    weekdays          SMALLINT            NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_automatic_profile_search_state(
    account_id                         BIGINT PRIMARY KEY  NOT NULL,
    last_seen_unix_time                BIGINT              NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_moderation(
    account_id              BIGINT              NOT NULL,
    -- 0 = ProfileName
    -- 1 = ProfileText
    content_type            SMALLINT            NOT NULL,
    -- 0 = WaitingBotOrHumanModeration
    -- 1 = WaitingHumanModeration
    -- 2 = AcceptedByBot
    -- 3 = AcceptedByHuman
    -- 4 = AcceptedByAllowlist
    -- 5 = RejectedByBot
    -- 6 = RejectedByHuman
    state_type               SMALLINT           NOT NULL,
    rejected_reason_category SMALLINT,
    -- Null or non-empty string
    rejected_reason_details  TEXT,
    moderator_account_id     BIGINT,
    -- Created or state reset time
    created_unix_time        BIGINT             NOT NULL,
    PRIMARY KEY (account_id, content_type),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (moderator_account_id)
        REFERENCES account_id (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

---------- Tables for server component media ----------

-- State specific to media component.
CREATE TABLE IF NOT EXISTS media_state(
    account_id                          BIGINT PRIMARY KEY  NOT NULL,
    -- Sync version for profile and security content data for this account.
    media_content_sync_version          SMALLINT            NOT NULL DEFAULT 0,
    -- Edit time for profile content changes. This updates from both
    -- user and admin made changes.
    profile_content_edited_unix_time    BIGINT              NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Information about uploaded media content
CREATE TABLE IF NOT EXISTS media_content(
    id                  BIGSERIAL PRIMARY KEY NOT NULL,
    uuid                BYTEA               NOT NULL,
    account_id          BIGINT              NOT NULL,
    -- Client captured this media
    secure_capture      BOOLEAN             NOT NULL,
    -- Face was detected from the content
    face_detected       BOOLEAN             NOT NULL,
    -- JpegImage = 0, Jpeg image
    content_type_number SMALLINT            NOT NULL,
    -- Numbers from 0 to 6.
    slot_number         SMALLINT            NOT NULL,
    creation_unix_time  BIGINT              NOT NULL,
    -- Content was uploaded when profile visibility is pending public or
    -- pending private.
    initial_content     BOOLEAN             NOT NULL,
    -- State groups:
    -- InSlot, If user uploads new content to slot the current will be removed.
    -- InModeration, Content is in moderation. User can not remove the content.
    -- ModeratedAsAccepted, Content is moderated as accepted. User can not remove the content until
    --                      specific time elapses.
    -- ModeratedAsRejected, Content is moderated as rejected. Content deleting
    --                      is possible.
    -- State values:
    -- 0 = Empty (InSlot),
    -- 1 = WaitingBotOrHumanModeration (InModeration)
    -- 2 = WaitingHumanModeration (InModeration)
    -- 3 = AcceptedByBot (ModeratedAsAccepted)
    -- 4 = AcceptedByHuman (ModeratedAsAccepted)
    -- 5 = RejectedByBot (ModeratedAsRejected)
    -- 6 = RejectedByHuman (ModeratedAsRejected)
    moderation_state     SMALLINT            NOT NULL    DEFAULT 0,
    moderation_rejected_reason_category SMALLINT,
    -- Null or non-empty string
    moderation_rejected_reason_details  TEXT,
    moderation_moderator_account_id     BIGINT,
    usage_start_unix_time  BIGINT,
    usage_end_unix_time    BIGINT,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (moderation_moderator_account_id)
        REFERENCES account_id (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

-- Currently selected images for account.
-- Contains profile editing related pending profile image info.
CREATE TABLE IF NOT EXISTS current_account_media(
    account_id                   BIGINT PRIMARY KEY  NOT NULL,
    security_content_id          BIGINT,
    profile_content_version_uuid BYTEA               NOT NULL,
    profile_content_id_0         BIGINT,
    profile_content_id_1         BIGINT,
    profile_content_id_2         BIGINT,
    profile_content_id_3         BIGINT,
    profile_content_id_4         BIGINT,
    profile_content_id_5         BIGINT,
    -- Image's max square size multipler.
    -- Value 1.0 is the max size and the size of the original image.
    grid_crop_size       DOUBLE PRECISION,
    -- X coordinate for square top left corner.
    -- Counted from top left corner of the original image.
    grid_crop_x          DOUBLE PRECISION,
    -- Y coordinate for square top left corner.
    -- Counted from top left corner of the original image.
    grid_crop_y          DOUBLE PRECISION,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (security_content_id)
        REFERENCES media_content (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE,
    FOREIGN KEY (profile_content_id_0)
        REFERENCES media_content (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE,
    FOREIGN KEY (profile_content_id_1)
        REFERENCES media_content (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE,
    FOREIGN KEY (profile_content_id_2)
        REFERENCES media_content (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE,
    FOREIGN KEY (profile_content_id_3)
        REFERENCES media_content (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE,
    FOREIGN KEY (profile_content_id_4)
        REFERENCES media_content (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE,
    FOREIGN KEY (profile_content_id_5)
        REFERENCES media_content (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS used_content_ids(
    account_id            BIGINT                            NOT NULL,
    uuid                  BYTEA                             NOT NULL,
    PRIMARY KEY (account_id, uuid),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS media_report_profile_content(
    report_id               BIGINT PRIMARY KEY  NOT NULL,
    -- The image UUID is stored here to avoid
    -- image changes if database image ID is reused.
    profile_content_uuid    BYTEA               NOT NULL,
    FOREIGN KEY (report_id)
        REFERENCES common_report (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS media_app_notification_settings(
    account_id                         BIGINT PRIMARY KEY  NOT NULL,
    media_content_moderation           BOOLEAN             NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS media_app_notification_state(
    account_id                         BIGINT PRIMARY KEY  NOT NULL,
    media_content_accepted             SMALLINT            NOT NULL DEFAULT 0,
    media_content_accepted_viewed      SMALLINT            NOT NULL DEFAULT 0,
    media_content_rejected             SMALLINT            NOT NULL DEFAULT 0,
    media_content_rejected_viewed      SMALLINT            NOT NULL DEFAULT 0,
    media_content_deleted              SMALLINT            NOT NULL DEFAULT 0,
    media_content_deleted_viewed       SMALLINT            NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- Tables for server component chat ----------

-- State specific to chat component.
CREATE TABLE IF NOT EXISTS chat_state(
    account_id              BIGINT PRIMARY KEY  NOT NULL,
    received_likes_sync_version  SMALLINT       NOT NULL DEFAULT 0,
    new_received_likes_count     BIGINT        NOT NULL DEFAULT 0,
    next_received_like_id        BIGINT        NOT NULL DEFAULT 0,
    max_public_key_count         BIGINT        NOT NULL DEFAULT 0,
    next_conversation_id         BIGINT        NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS daily_likes_left(
    account_id            BIGINT PRIMARY KEY  NOT NULL,
    sync_version          SMALLINT            NOT NULL DEFAULT 0,
    likes_left            SMALLINT            NOT NULL DEFAULT 0,
    latest_limit_reset_unix_time BIGINT,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS public_key(
    account_id            BIGINT  NOT NULL,
    key_id                BIGINT  NOT NULL,
    key_data              BYTEA   NOT NULL,
    key_added_unix_time   BIGINT  NOT NULL,
    PRIMARY KEY (account_id, key_id),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Current relationship between accounts
CREATE TABLE IF NOT EXISTS account_interaction(
    id                  BIGSERIAL PRIMARY KEY NOT NULL,
    -- 0 = no interaction
    -- 1 = like
    -- 2 = match
    state_number                    SMALLINT NOT NULL DEFAULT 0,
    -- The account which sent a like.
    account_id_sender               BIGINT,
    -- The account which received a like.
    account_id_receiver             BIGINT,
    -- The account which started the block.
    account_id_block_sender         BIGINT,
    -- The account which received the block.
    account_id_block_receiver       BIGINT,
    -- If this is true, then both sides have blocked each other.
    two_way_block                   BOOLEAN NOT NULL DEFAULT FALSE,
    -- Incrementing counters for tracking sent message count for both accounts.
    message_counter_sender          BIGINT  NOT NULL DEFAULT 0,
    message_counter_receiver        BIGINT  NOT NULL DEFAULT 0,
    -- Track if video call URL has been created for each side
    video_call_url_created_sender   BOOLEAN NOT NULL DEFAULT FALSE,
    video_call_url_created_receiver BOOLEAN NOT NULL DEFAULT FALSE,
    -- Received likes iterator uses received likes ID to return
    -- correct pages.
    received_like_id                BIGINT,
    received_like_viewed            BOOLEAN NOT NULL DEFAULT FALSE,
    received_like_email_notification_sent BOOLEAN NOT NULL DEFAULT FALSE,
    received_like_unix_time         BIGINT,
    -- Matches iterator uses match ID to return correct pages.
    match_id                        BIGINT,
    match_unix_time                 BIGINT,
    -- Account specific conversation ID for new message notifications.
    -- Available when accounts are a match.
    conversation_id_sender                  BIGINT,
    conversation_id_receiver                BIGINT,
    FOREIGN KEY (account_id_sender)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_receiver)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_block_sender)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_block_receiver)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Lookup table for finding interaction ID for a pair of accounts.
-- One account pair has two rows in this table, so accessing
-- with (a1, a2) and (a2, a1) is possible.
CREATE TABLE IF NOT EXISTS account_interaction_index(
    account_id_first               BIGINT NOT NULL,
    account_id_second              BIGINT NOT NULL,
    interaction_id                 BIGINT NOT NULL,
    PRIMARY KEY (account_id_first, account_id_second),
    FOREIGN KEY (account_id_first)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_second)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (interaction_id)
        REFERENCES account_interaction (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Messages received from clients which are pending for acknowledgements from
-- sender and receiver.
CREATE TABLE IF NOT EXISTS pending_messages(
    id                  BIGSERIAL PRIMARY KEY NOT NULL,
    account_interaction             BIGINT NOT NULL,
    -- The account which sent the message.
    account_id_sender               BIGINT NOT NULL,
    -- The account which will receive the message.
    account_id_receiver             BIGINT NOT NULL,
    -- Acknowledgement from sender and receiver
    sender_acknowledgement          BOOLEAN NOT NULL DEFAULT FALSE,
    receiver_acknowledgement        BOOLEAN NOT NULL DEFAULT FALSE,
    -- Track push notification sending for the message to
    -- avoid sending the same data again.
    receiver_push_notification_sent BOOLEAN NOT NULL DEFAULT FALSE,
    -- Email notification for the message.
    receiver_email_notification_sent BOOLEAN NOT NULL DEFAULT FALSE,
    message_unix_time               BIGINT NOT NULL,
    -- Conversation specific ID for the message.
    message_id                      BIGINT NOT NULL,
    -- Client ID and client local ID together makes
    -- an practically unique ID which client can use
    -- detecting was message sent correctly.
    sender_client_id                BIGINT NOT NULL,
    sender_client_local_id          BIGINT NOT NULL,
    -- Message bytes.
    message_bytes                   BYTEA  NOT NULL,
    FOREIGN KEY (account_interaction)
        REFERENCES account_interaction (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_sender)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_receiver)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS chat_report_chat_message(
    report_id                          BIGINT PRIMARY KEY  NOT NULL,
    message_sender_account_id_uuid     BYTEA               NOT NULL,
    message_receiver_account_id_uuid   BYTEA               NOT NULL,
    message_unix_time                  BIGINT              NOT NULL,
    message_id                         BIGINT              NOT NULL,
    message_symmetric_key              BYTEA               NOT NULL,
    client_message_bytes               BYTEA               NOT NULL,
    backend_signed_message_bytes       BYTEA               NOT NULL,
    FOREIGN KEY (report_id)
        REFERENCES common_report (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
    -- Reports have own deletion logic, so REFERENCES for account_id values
    -- are not needed.
);

CREATE TABLE IF NOT EXISTS chat_app_notification_settings(
    account_id                         BIGINT PRIMARY KEY  NOT NULL,
    likes                              BOOLEAN             NOT NULL,
    messages                           BOOLEAN             NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS chat_email_notification_settings(
    account_id                         BIGINT PRIMARY KEY  NOT NULL,
    likes                              BOOLEAN             NOT NULL,
    messages                           BOOLEAN             NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS chat_global_state(
    -- 0 = chat component global state
    row_type              INTEGER PRIMARY KEY NOT NULL,
    next_match_id         BIGINT              NOT NULL DEFAULT 0
);

---------- History tables for server component common ----------

CREATE TABLE IF NOT EXISTS history_common_statistics_save_time(
    id           BIGSERIAL PRIMARY KEY             NOT NULL,
    unix_time    BIGINT                            NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS history_performance_statistics_metric_name(
    id           BIGSERIAL PRIMARY KEY             NOT NULL,
    metric_name  TEXT                              NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS history_performance_statistics_metric_value(
    time_id      BIGINT                            NOT NULL,
    metric_id    BIGINT                            NOT NULL,
    metric_value BIGINT                            NOT NULL,
    PRIMARY KEY (time_id, metric_id),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (metric_id)
        REFERENCES history_performance_statistics_metric_name (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Use own table for country names to keep ID value small as possible
CREATE TABLE IF NOT EXISTS history_ip_country_statistics_country_name(
    id           BIGSERIAL PRIMARY KEY             NOT NULL,
    country_name TEXT                              NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS history_ip_country_statistics(
    time_id             BIGINT                            NOT NULL,
    country_id          BIGINT                            NOT NULL,
    new_tcp_connections BIGINT                            NOT NULL,
    new_http_requests   BIGINT                            NOT NULL,
    PRIMARY KEY (time_id, country_id),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (country_id)
        REFERENCES history_ip_country_statistics_country_name (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- History tables for server component account ----------

CREATE TABLE IF NOT EXISTS history_client_version_statistics_version_number(
    id            BIGSERIAL PRIMARY KEY                   NOT NULL,
    major         BIGINT NOT NULL,
    minor         BIGINT NOT NULL,
    patch         BIGINT NOT NULL,
    UNIQUE (major, minor, patch)
);

CREATE TABLE IF NOT EXISTS history_client_version_statistics(
    time_id       BIGINT  NOT NULL,
    version_id    BIGINT  NOT NULL,
    count         BIGINT  NOT NULL,
    PRIMARY KEY (time_id, version_id),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (version_id)
        REFERENCES history_client_version_statistics_version_number (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- History tables for server component profile ----------

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_man(
    time_id BIGINT   NOT NULL,
    age     SMALLINT NOT NULL,
    count   BIGINT   NOT NULL,
    PRIMARY KEY (time_id, age),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_woman(
    time_id BIGINT   NOT NULL,
    age     SMALLINT NOT NULL,
    count   BIGINT   NOT NULL,
    PRIMARY KEY (time_id, age),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_non_binary(
    time_id BIGINT   NOT NULL,
    age     SMALLINT NOT NULL,
    count   BIGINT   NOT NULL,
    PRIMARY KEY (time_id, age),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_all_genders(
    time_id BIGINT   NOT NULL,
    age     SMALLINT NOT NULL,
    count   BIGINT   NOT NULL,
    PRIMARY KEY (time_id, age),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_account(
    time_id BIGINT PRIMARY KEY  NOT NULL,
    count   BIGINT              NOT NULL,
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_man(
    time_id BIGINT PRIMARY KEY  NOT NULL,
    count   BIGINT              NOT NULL,
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_woman(
    time_id BIGINT PRIMARY KEY  NOT NULL,
    count   BIGINT              NOT NULL,
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_non_binary(
    time_id BIGINT PRIMARY KEY  NOT NULL,
    count   BIGINT              NOT NULL,
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_all_genders(
    time_id BIGINT PRIMARY KEY  NOT NULL,
    count   BIGINT              NOT NULL,
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- History tables for server component media ----------
