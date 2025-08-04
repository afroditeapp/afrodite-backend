-- Your SQL goes here

-- TODO(prod): Add autoincrement where needed. News?

---------- Tables for server component common ----------

-- UUID for account
CREATE TABLE IF NOT EXISTS account_id(
    id    INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    -- Main UUID for account.
    -- This is used internally in the server, client and API level.
    -- Also this should be not used as somekind of secret as it
    -- can be seen from filesystem.
    uuid  BLOB                              NOT NULL  UNIQUE
);

-- All used account IDs. Account ID is not removed from here
-- when account data is removed.
CREATE TABLE IF NOT EXISTS used_account_ids(
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    uuid       BLOB                              NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS login_session(
    account_id              INTEGER PRIMARY KEY NOT NULL,
    -- Rust HashMap guarantees access token uniqueness, so
    -- UNIQUE constrait is not needed here.
    access_token            TEXT                NOT NULL,
    access_token_unix_time  INTEGER             NOT NULL,
    -- 4 or 16 bytes
    access_token_ip_address BLOB                NOT NULL,
    -- Using refresh token requires valid access token, so
    -- UNIQUE constraint is not needed here.
    refresh_token           BLOB                NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Account permissions are shared between server components.
-- If the data is located in this table it should be set through account
-- server as it propagates the changes to other components.
CREATE TABLE IF NOT EXISTS account_permissions(
    account_id    INTEGER PRIMARY KEY NOT NULL,
    admin_edit_permissions                       BOOLEAN NOT NULL DEFAULT 0,
    admin_edit_profile_name                      BOOLEAN NOT NULL DEFAULT 0,
    admin_edit_max_public_key_count              BOOLEAN NOT NULL DEFAULT 0,
    admin_edit_media_content_face_detected_value BOOLEAN NOT NULL DEFAULT 0,
    admin_moderate_media_content                 BOOLEAN NOT NULL DEFAULT 0,
    admin_moderate_profile_names                 BOOLEAN NOT NULL DEFAULT 0,
    admin_moderate_profile_texts                 BOOLEAN NOT NULL DEFAULT 0,
    admin_process_reports                        BOOLEAN NOT NULL DEFAULT 0,
    admin_delete_media_content                   BOOLEAN NOT NULL DEFAULT 0,
    admin_delete_account                         BOOLEAN NOT NULL DEFAULT 0,
    admin_ban_account                            BOOLEAN NOT NULL DEFAULT 0,
    admin_request_account_deletion               BOOLEAN NOT NULL DEFAULT 0,
    admin_view_all_profiles                      BOOLEAN NOT NULL DEFAULT 0,
    admin_view_private_info                      BOOLEAN NOT NULL DEFAULT 0,
    admin_view_profile_history                   BOOLEAN NOT NULL DEFAULT 0,
    admin_view_permissions                       BOOLEAN NOT NULL DEFAULT 0,
    admin_find_account_by_email                  BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_view_info           BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_view_backend_config BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_update_software     BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_reset_data          BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_restart_backend     BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_save_backend_config BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_edit_notification   BOOLEAN NOT NULL DEFAULT 0,
    admin_news_create                            BOOLEAN NOT NULL DEFAULT 0,
    admin_news_edit_all                          BOOLEAN NOT NULL DEFAULT 0,
    admin_profile_statistics                     BOOLEAN NOT NULL DEFAULT 0,
    admin_subscribe_admin_notifications          BOOLEAN NOT NULL DEFAULT 0,
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
    account_id                INTEGER PRIMARY KEY NOT NULL,
    account_state_initial_setup_completed   BOOLEAN             NOT NULL DEFAULT 0,
    account_state_banned                    BOOLEAN             NOT NULL DEFAULT 0,
    account_state_pending_deletion          BOOLEAN             NOT NULL DEFAULT 0,
    -- pending private = 0
    -- pending public = 1
    -- private = 2
    -- public = 3
    profile_visibility_state_number INTEGER NOT NULL DEFAULT 0,
    -- Version number which only account server increments.
    -- Used in receiving end to avoid saving old state in case of
    -- concurrent updates.
    sync_version              INTEGER              NOT NULL DEFAULT 0,
    unlimited_likes           BOOLEAN              NOT NULL DEFAULT 0,
    -- Birthdate has YYYY-MM-DD format. This is in shared state if
    -- birthdate validation using third party service is implemented
    -- someday.
    birthdate                 DATE,
    is_bot_account            BOOLEAN              NOT NULL DEFAULT 0,
    -- Profile component uses this info for profile filtering.
    initial_setup_completed_unix_time INTEGER      NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS common_report(
    id                      INTEGER PRIMARY KEY NOT NULL,
    creator_account_id      INTEGER             NOT NULL,
    target_account_id       INTEGER             NOT NULL,
    -- 0 = profile name
    -- 1 = profile text
    -- 2 = profile content
    -- 3 = chat message
    -- Values from 64 to 127 are reserved for custom reports.
    report_type_number      INTEGER             NOT NULL,
    creation_unix_time      INTEGER             NOT NULL,
    moderator_account_id    INTEGER,
    -- 0 = Waiting
    -- 1 = Done
    processing_state        INTEGER             NOT NULL    DEFAULT 0,
    processing_state_change_unix_time INTEGER   NOT NULL,
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
    account_id                         INTEGER PRIMARY KEY NOT NULL,
    -- Sync version for client config.
    client_config_sync_version         INTEGER             NOT NULL    DEFAULT 0,
    -- Bitflag value for pending notification
    pending_notification         INTEGER        NOT NULL DEFAULT 0,
    -- Access token for getting pending notifications from server.
    pending_notification_token   TEXT           UNIQUE,
    fcm_data_notification_sent          BOOLEAN            NOT NULL DEFAULT 0,
    fcm_visible_notification_sent       BOOLEAN            NOT NULL DEFAULT 0,
    fcm_device_token             TEXT           UNIQUE,
    -- Time when a token is saved. Not currently used for anything.
    -- Firebase docs recommend storing a timestamp with a token.
    fcm_device_token_unix_time   INTEGER,
    -- 0 = Android
    -- 1 = iOS
    -- 2 = Web
    client_login_session_platform       INTEGER,
    client_language                     TEXT    NOT NULL DEFAULT '',
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS api_usage_statistics_save_time(
    id           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    unix_time    INTEGER                           NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS api_usage_statistics_metric_name(
    id           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    metric_name  TEXT                              NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS api_usage_statistics_metric_value(
    account_id   INTEGER                           NOT NULL,
    time_id      INTEGER                           NOT NULL,
    metric_id    INTEGER                           NOT NULL,
    metric_value INTEGER                           NOT NULL,
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
    account_id             INTEGER                 NOT NULL,
    -- 4 or 16 bytes
    ip_address             BLOB                    NOT NULL,
    usage_count            INTEGER                 NOT NULL,
    first_usage_unix_time  INTEGER                 NOT NULL,
    latest_usage_unix_time INTEGER                 NOT NULL,
    PRIMARY KEY (account_id, ip_address),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS admin_notification_subscriptions(
    account_id INTEGER PRIMARY KEY NOT NULL,
    moderate_initial_media_content_bot           BOOLEAN NOT NULL DEFAULT 0,
    moderate_initial_media_content_human         BOOLEAN NOT NULL DEFAULT 0,
    moderate_media_content_bot                   BOOLEAN NOT NULL DEFAULT 0,
    moderate_media_content_human                 BOOLEAN NOT NULL DEFAULT 0,
    moderate_profile_texts_bot                   BOOLEAN NOT NULL DEFAULT 0,
    moderate_profile_texts_human                 BOOLEAN NOT NULL DEFAULT 0,
    moderate_profile_names_bot                   BOOLEAN NOT NULL DEFAULT 0,
    moderate_profile_names_human                 BOOLEAN NOT NULL DEFAULT 0,
    process_reports                              BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- Tables for server component account ----------

-- Sign in with related IDs for account
CREATE TABLE IF NOT EXISTS sign_in_with_info(
    account_id         INTEGER PRIMARY KEY NOT NULL,
    apple_account_id   TEXT                          UNIQUE,
    google_account_id  TEXT                          UNIQUE,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Account information which can change
CREATE TABLE IF NOT EXISTS account(
    account_id   INTEGER PRIMARY KEY NOT NULL,
    email        TEXT                                UNIQUE,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Information which can not change after account initial setup completes
CREATE TABLE IF NOT EXISTS account_setup(
    account_id  INTEGER PRIMARY KEY NOT NULL,
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
    account_id                      INTEGER PRIMARY KEY NOT NULL,
    account_registered_state_number INTEGER             NOT NULL DEFAULT 0,
    new_message_state_number        INTEGER             NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- State specific to account component.
CREATE TABLE IF NOT EXISTS account_state(
    account_id                         INTEGER PRIMARY KEY NOT NULL,
    next_client_id                     INTEGER             NOT NULL DEFAULT 0,
    account_deletion_request_unix_time INTEGER,
    account_banned_reason_category     INTEGER,
    account_banned_reason_details      TEXT                NOT NULL DEFAULT '',
    account_banned_admin_account_id    INTEGER,
    account_banned_until_unix_time     INTEGER,
    account_banned_state_change_unix_time INTEGER,
    -- Sync version for news.
    news_sync_version                  INTEGER             NOT NULL DEFAULT 0,
    unread_news_count                  INTEGER             NOT NULL DEFAULT 0,
    publication_id_at_news_iterator_reset INTEGER,
    publication_id_at_unread_news_count_incrementing INTEGER,
    account_created_unix_time          INTEGER             NOT NULL DEFAULT 0,
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
    account_id                         INTEGER PRIMARY KEY NOT NULL,
    news                               BOOLEAN             NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS demo_account_owned_accounts(
    demo_account_id INTEGER             NOT NULL,
    account_id      INTEGER             NOT NULL,
    PRIMARY KEY (demo_account_id, account_id),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS news(
    id                    INTEGER PRIMARY KEY NOT NULL,
    account_id_creator    INTEGER,
    first_publication_unix_time  INTEGER,
    latest_publication_unix_time INTEGER,
    -- If publication ID exists the news are public.
    publication_id        INTEGER,
    FOREIGN KEY (account_id_creator)
    REFERENCES account_id (id)
        ON DELETE SET NULL
        ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS news_translations(
    locale                TEXT                NOT NULL,
    news_id               INTEGER             NOT NULL,
    title                 TEXT                NOT NULL,
    body                  TEXT                NOT NULL,
    creation_unix_time    INTEGER             NOT NULL,
    version_number        INTEGER             NOT NULL DEFAULT 0,
    account_id_creator    INTEGER,
    account_id_editor     INTEGER,
    edit_unix_time        INTEGER,
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
    admin_access_granted_count INTEGER             NOT NULL DEFAULT 0,
    -- Publication ID for news which always increments.
    next_news_publication_id   INTEGER             NOT NULL DEFAULT 0
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
    account_id                 INTEGER PRIMARY KEY  NOT NULL,
    -- Min age in years and inside inclusive range of [18,99] for
    -- searching profiles.
    search_age_range_min       INTEGER              NOT NULL    DEFAULT 18,
    -- Max age in years and inside inclusive range of [18,99] for
    -- searching profiles.
    search_age_range_max       INTEGER              NOT NULL    DEFAULT 18,
    -- Bitflags value containing gender and genders that
    -- the profile owner searches for.
    search_group_flags         INTEGER              NOT NULL    DEFAULT 0,
    -- Filter setting for last seen time.
    last_seen_time_filter      INTEGER,
    -- Filter setting for unlimited likes.
    unlimited_likes_filter     BOOLEAN,
    -- Filter setting for profile iterator min distance in kilometers.
    min_distance_km_filter     INTEGER,
    -- Filter setting for profile iterator max distance in kilometers.
    max_distance_km_filter     INTEGER,
    -- Filter setting for profile created time in seconds.
    profile_created_time_filter INTEGER,
    -- Filter setting for profile edited time in seconds.
    profile_edited_time_filter INTEGER,
    -- Filter setting for profile text min character count.
    profile_text_min_characters_filter INTEGER,
    -- Filter setting for profile text max character count.
    profile_text_max_characters_filter INTEGER,
    -- Profile iterator setting for random profile order.
    random_profile_order       BOOLEAN              NOT NULL    DEFAULT 0,
    latitude                   DOUBLE               NOT NULL    DEFAULT 0.0,
    longitude                  DOUBLE               NOT NULL    DEFAULT 0.0,
    -- Sync version for profile data for this account.
    profile_sync_version              INTEGER       NOT NULL    DEFAULT 0,
    -- Profile age when initial setup is completed
    initial_profile_age               INTEGER,
    initial_profile_age_set_unix_time INTEGER,
    -- Edit time for public profile changes. This updates from both
    -- user and admin made changes.
    profile_edited_unix_time          INTEGER       NOT NULL    DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Profile information which can be sent to clients if
-- profile visibility is public.
CREATE TABLE IF NOT EXISTS profile(
    account_id      INTEGER PRIMARY KEY NOT NULL,
    version_uuid    BLOB                NOT NULL,
    profile_name    TEXT                NOT NULL    DEFAULT '',
    profile_text    TEXT                NOT NULL    DEFAULT '',
    -- Age in years and inside inclusive range of [18,99].
    age             INTEGER             NOT NULL    DEFAULT 18,
    last_seen_unix_time  INTEGER        NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Store profile attribute specific filter settings
CREATE TABLE IF NOT EXISTS profile_attributes_filter_settings(
    account_id      INTEGER             NOT NULL,
    attribute_id    INTEGER             NOT NULL,
    filter_accept_missing_attribute BOOLEAN NOT NULL DEFAULT 0,
    filter_use_logical_operator_and BOOLEAN NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, attribute_id),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_attributes_value_list(
    account_id      INTEGER             NOT NULL,
    attribute_id    INTEGER             NOT NULL,
    attribute_value INTEGER             NOT NULL,
    PRIMARY KEY (account_id, attribute_id, attribute_value),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_attributes_filter_list_wanted(
    account_id      INTEGER             NOT NULL,
    attribute_id    INTEGER             NOT NULL,
    filter_value    INTEGER             NOT NULL,
    PRIMARY KEY (account_id, attribute_id, filter_value),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_attributes_filter_list_unwanted(
    account_id      INTEGER             NOT NULL,
    attribute_id    INTEGER             NOT NULL,
    filter_value    INTEGER             NOT NULL,
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
    account_id          INTEGER               NOT NULL,
    -- Account which profile is marked as a favorite.
    favorite_account_id INTEGER               NOT NULL,
    -- Unix timestamp when favorite was added.
    unix_time           INTEGER               NOT NULL,
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
    name_creator_account_id   INTEGER             NOT NULL,
    name_moderator_account_id INTEGER,
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
    report_id               INTEGER PRIMARY KEY NOT NULL,
    profile_name            TEXT                NOT NULL,
    FOREIGN KEY (report_id)
        REFERENCES common_report (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_report_profile_text(
    report_id               INTEGER PRIMARY KEY NOT NULL,
    profile_text            TEXT                NOT NULL,
    FOREIGN KEY (report_id)
        REFERENCES common_report (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_app_notification_settings(
    account_id                                 INTEGER PRIMARY KEY NOT NULL,
    profile_text_moderation                    BOOLEAN             NOT NULL,
    automatic_profile_search                   BOOLEAN             NOT NULL,
    automatic_profile_search_new_profiles      BOOLEAN             NOT NULL,
    automatic_profile_search_attribute_filters BOOLEAN             NOT NULL,
    automatic_profile_search_distance_filters  BOOLEAN             NOT NULL,
    automatic_profile_search_weekdays          INTEGER             NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_app_notification_state(
    account_id                         INTEGER PRIMARY KEY NOT NULL,
    profile_name_accepted              INTEGER             NOT NULL DEFAULT 0,
    profile_name_accepted_viewed       INTEGER             NOT NULL DEFAULT 0,
    profile_name_rejected              INTEGER             NOT NULL DEFAULT 0,
    profile_name_rejected_viewed       INTEGER             NOT NULL DEFAULT 0,
    profile_text_accepted              INTEGER             NOT NULL DEFAULT 0,
    profile_text_accepted_viewed       INTEGER             NOT NULL DEFAULT 0,
    profile_text_rejected              INTEGER             NOT NULL DEFAULT 0,
    profile_text_rejected_viewed       INTEGER             NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_automatic_profile_search_state(
    account_id                         INTEGER PRIMARY KEY NOT NULL,
    last_seen_unix_time                INTEGER             NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_moderation(
    account_id              INTEGER             NOT NULL,
    -- 0 = ProfileName
    -- 1 = ProfileText
    content_type            INTEGER             NOT NULL,
    -- 0 = WaitingBotOrHumanModeration
    -- 1 = WaitingHumanModeration
    -- 2 = AcceptedByBot
    -- 3 = AcceptedByHuman
    -- 4 = AcceptedByAllowlist
    -- 5 = RejectedByBot
    -- 6 = RejectedByHuman
    state_type               INTEGER            NOT NULL,
    rejected_reason_category INTEGER,
    rejected_reason_details  TEXT               NOT NULL DEFAULT '',
    moderator_account_id     INTEGER,
    -- Created or state reset time
    created_unix_time        INTEGER            NOT NULL,
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
    account_id                          INTEGER PRIMARY KEY NOT NULL,
    -- Media component sends this to account component when
    -- this turns to true. Account component then updates
    -- the profile visibility for both profile and media.
    initial_moderation_request_accepted BOOLEAN             NOT NULL DEFAULT 0,
    -- Sync version for profile and security content data for this account.
    media_content_sync_version          INTEGER             NOT NULL DEFAULT 0,
    -- Edit time for profile content changes. This updates from both
    -- user and admin made changes.
    profile_content_edited_unix_time    INTEGER             NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Currently selected images for account.
-- Contains profile editing related pending profile image info.
CREATE TABLE IF NOT EXISTS current_account_media(
    account_id                   INTEGER PRIMARY KEY NOT NULL,
    security_content_id          INTEGER,
    profile_content_version_uuid BLOB                NOT NULL,
    profile_content_id_0         INTEGER,
    profile_content_id_1         INTEGER,
    profile_content_id_2         INTEGER,
    profile_content_id_3         INTEGER,
    profile_content_id_4         INTEGER,
    profile_content_id_5         INTEGER,
    -- Image's max square size multipler.
    -- Value 1.0 is the max size and the size of the original image.
    grid_crop_size       DOUBLE,
    -- X coordinate for square top left corner.
    -- Counted from top left corner of the original image.
    grid_crop_x          DOUBLE,
    -- Y coordinate for square top left corner.
    -- Counted from top left corner of the original image.
    grid_crop_y          DOUBLE,
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

-- Information about uploaded media content
CREATE TABLE IF NOT EXISTS media_content(
    id                  INTEGER PRIMARY KEY NOT NULL,
    uuid                BLOB                NOT NULL,
    account_id          INTEGER             NOT NULL,
    -- Client captured this media
    secure_capture      BOOLEAN             NOT NULL,
    -- Face was detected from the content
    face_detected       BOOLEAN             NOT NULL,
    -- JpegImage = 0, Jpeg image
    content_type_number INTEGER             NOT NULL,
    -- Numbers from 0 to 6.
    slot_number         INTEGER             NOT NULL,
    creation_unix_time  INTEGER             NOT NULL,
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
    moderation_state     INTEGER             NOT NULL    DEFAULT 0,
    moderation_rejected_reason_category INTEGER,
    moderation_rejected_reason_details  TEXT NOT NULL    DEFAULT '',
    moderation_moderator_account_id     INTEGER,
    usage_start_unix_time  INTEGER,
    usage_end_unix_time    INTEGER,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS used_content_ids(
    account_id            INTEGER                           NOT NULL,
    uuid                  BLOB                              NOT NULL,
    PRIMARY KEY (account_id, uuid),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS media_report_profile_content(
    report_id               INTEGER PRIMARY KEY NOT NULL,
    -- The image UUID is stored here to avoid
    -- image changes if database image ID is reused.
    profile_content_uuid    BLOB                NOT NULL,
    FOREIGN KEY (report_id)
        REFERENCES common_report (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS media_app_notification_settings(
    account_id                         INTEGER PRIMARY KEY NOT NULL,
    media_content_moderation           BOOLEAN             NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS media_app_notification_state(
    account_id                         INTEGER PRIMARY KEY NOT NULL,
    media_content_accepted             INTEGER             NOT NULL DEFAULT 0,
    media_content_accepted_viewed      INTEGER             NOT NULL DEFAULT 0,
    media_content_rejected             INTEGER             NOT NULL DEFAULT 0,
    media_content_rejected_viewed      INTEGER             NOT NULL DEFAULT 0,
    media_content_deleted              INTEGER             NOT NULL DEFAULT 0,
    media_content_deleted_viewed       INTEGER             NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- Tables for server component chat ----------

-- State specific to chat component.
CREATE TABLE IF NOT EXISTS chat_state(
    account_id              INTEGER PRIMARY KEY NOT NULL,
    received_likes_sync_version  INTEGER        NOT NULL DEFAULT 0,
    new_received_likes_count     INTEGER        NOT NULL DEFAULT 0,
    next_received_like_id        INTEGER        NOT NULL DEFAULT 0,
    received_like_id_at_received_likes_iterator_reset           INTEGER,
    max_public_key_count         INTEGER        NOT NULL DEFAULT 0,
    next_conversation_id         INTEGER        NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS daily_likes_left(
    account_id            INTEGER PRIMARY KEY NOT NULL,
    sync_version          INTEGER             NOT NULL DEFAULT 0,
    likes_left            INTEGER             NOT NULL DEFAULT 0,
    latest_limit_reset_unix_time INTEGER,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS public_key(
    account_id            INTEGER NOT NULL,
    key_id                INTEGER NOT NULL,
    key_data              BLOB    NOT NULL,
    key_added_unix_time   INTEGER NOT NULL,
    PRIMARY KEY (account_id, key_id),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Lookup table for finding interaction ID for a pair of accounts.
-- One account pair has two rows in this table, so accessing
-- with (a1, a2) and (a2, a1) is possible.
CREATE TABLE IF NOT EXISTS account_interaction_index(
    account_id_first               INTEGER NOT NULL,
    account_id_second              INTEGER NOT NULL,
    interaction_id                 INTEGER NOT NULL,
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

-- Current relationship between accounts
CREATE TABLE IF NOT EXISTS account_interaction(
    id                  INTEGER PRIMARY KEY NOT NULL,
    -- 0 = no interaction
    -- 1 = like
    -- 2 = match
    state_number                    INTEGER NOT NULL DEFAULT 0,
    -- The account which sent a like.
    -- This changes back to null when the like is removed.
    -- This can't change to null once accounts are a match.
    account_id_sender               INTEGER,
    -- The account which received a like.
    -- This changes back to null when the like is removed.
    -- This can't change to null once accounts are a match.
    account_id_receiver             INTEGER,
    -- The account which started the block.
    account_id_block_sender         INTEGER,
    -- The account which received the block.
    account_id_block_receiver       INTEGER,
    -- If this is true, then both sides have blocked each other.
    two_way_block                   BOOLEAN NOT NULL DEFAULT 0,
    -- Incrementing counters for tracking sent message count for both accounts.
    message_counter_sender          INTEGER NOT NULL DEFAULT 0,
    message_counter_receiver        INTEGER NOT NULL DEFAULT 0,
    -- Track is the received like included in the receiver's
    -- new_received_likes_count.
    included_in_received_new_likes_count  BOOLEAN NOT NULL DEFAULT 0,
    -- Received likes iterator uses received likes ID to return
    -- correct pages.
    received_like_id                INTEGER,
    -- Matches iterator uses match ID to return correct pages.
    match_id                        INTEGER,
    account_id_previous_like_deleter_slot_0 INTEGER,
    account_id_previous_like_deleter_slot_1 INTEGER,
    -- Account specific conversation ID for new message notifications.
    -- Available when accounts are a match.
    conversation_id_sender                  INTEGER,
    conversation_id_receiver                INTEGER,
    FOREIGN KEY (account_id_sender)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_receiver)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_previous_like_deleter_slot_0)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_previous_like_deleter_slot_1)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Messages received from clients which are pending for acknowledgements from
-- sender and receiver.
CREATE TABLE IF NOT EXISTS pending_messages(
    id                  INTEGER PRIMARY KEY NOT NULL,
    account_interaction             INTEGER NOT NULL,
    -- The account which sent the message.
    account_id_sender               INTEGER NOT NULL,
    -- The account which will receive the message.
    account_id_receiver             INTEGER NOT NULL,
    -- Acknowledgement from sender and receiver
    sender_acknowledgement          BOOLEAN NOT NULL DEFAULT 0,
    receiver_acknowledgement        BOOLEAN NOT NULL DEFAULT 0,
    -- Track push notification sending for the message to
    -- avoid sending the same data again.
    receiver_push_notification_sent BOOLEAN NOT NULL DEFAULT 0,
    -- Email notification for the message.
    receiver_email_notification_sent BOOLEAN NOT NULL DEFAULT 0,
    message_unix_time               INTEGER NOT NULL,
    -- Conversation specific ID for the message.
    message_id                      INTEGER NOT NULL,
    -- Client ID and client local ID together makes
    -- an practically unique ID which client can use
    -- detecting was message sent correctly.
    sender_client_id                       INTEGER NOT NULL,
    sender_client_local_id                 INTEGER NOT NULL,
    -- Message bytes.
    message_bytes                   BLOB    NOT NULL,
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
    report_id                          INTEGER PRIMARY KEY NOT NULL,
    message_sender_account_id_uuid     BLOB                NOT NULL,
    message_receiver_account_id_uuid   BLOB                NOT NULL,
    message_unix_time                  INTEGER             NOT NULL,
    message_id                         INTEGER             NOT NULL,
    message_symmetric_key              BLOB                NOT NULL,
    client_message_bytes               BLOB                NOT NULL,
    backend_signed_message_bytes       BLOB                NOT NULL,
    FOREIGN KEY (report_id)
        REFERENCES common_report (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS chat_app_notification_settings(
    account_id                         INTEGER PRIMARY KEY NOT NULL,
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
    next_match_id         INTEGER             NOT NULL DEFAULT 0
);

---------- History tables for server component common ----------

CREATE TABLE IF NOT EXISTS history_common_statistics_save_time(
    id           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    unix_time    INTEGER                           NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS history_performance_statistics_metric_name(
    id           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    metric_name  TEXT                              NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS history_performance_statistics_metric_value(
    time_id      INTEGER                           NOT NULL,
    metric_id    INTEGER                           NOT NULL,
    metric_value INTEGER                           NOT NULL,
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
    id           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    country_name TEXT                              NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS history_ip_country_statistics(
    time_id             INTEGER                           NOT NULL,
    country_id          INTEGER                           NOT NULL,
    new_tcp_connections INTEGER                           NOT NULL,
    new_http_requests   INTEGER                           NOT NULL,
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
    id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    major         INTEGER NOT NULL,
    minor         INTEGER NOT NULL,
    patch         INTEGER NOT NULL,
    UNIQUE (major, minor, patch)
);

CREATE TABLE IF NOT EXISTS history_client_version_statistics(
    time_id       INTEGER NOT NULL,
    version_id    INTEGER NOT NULL,
    count         INTEGER NOT NULL,
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
    time_id INTEGER NOT NULL,
    age     INTEGER NOT NULL,
    count   INTEGER NOT NULL,
    PRIMARY KEY (time_id, age),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_woman(
    time_id INTEGER NOT NULL,
    age     INTEGER NOT NULL,
    count   INTEGER NOT NULL,
    PRIMARY KEY (time_id, age),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_non_binary(
    time_id INTEGER NOT NULL,
    age     INTEGER NOT NULL,
    count   INTEGER NOT NULL,
    PRIMARY KEY (time_id, age),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_all_genders(
    time_id INTEGER NOT NULL,
    age     INTEGER NOT NULL,
    count   INTEGER NOT NULL,
    PRIMARY KEY (time_id, age),
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_account(
    time_id INTEGER PRIMARY KEY NOT NULL,
    count   INTEGER             NOT NULL,
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_man(
    time_id INTEGER PRIMARY KEY NOT NULL,
    count   INTEGER             NOT NULL,
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_woman(
    time_id INTEGER PRIMARY KEY NOT NULL,
    count   INTEGER             NOT NULL,
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_non_binary(
    time_id INTEGER PRIMARY KEY NOT NULL,
    count   INTEGER             NOT NULL,
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_all_genders(
    time_id INTEGER PRIMARY KEY NOT NULL,
    count   INTEGER             NOT NULL,
    FOREIGN KEY (time_id)
        REFERENCES history_common_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- History tables for server component media ----------
