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

-- API access token for account
CREATE TABLE IF NOT EXISTS access_token(
    account_id   INTEGER PRIMARY KEY NOT NULL,
    token        TEXT                          UNIQUE,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- API refresh token for account
CREATE TABLE IF NOT EXISTS refresh_token(
    account_id    INTEGER PRIMARY KEY NOT NULL,
    token         BLOB                          UNIQUE,
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
    admin_modify_permissions                     BOOLEAN NOT NULL DEFAULT 0,
    admin_moderate_profile_content               BOOLEAN NOT NULL DEFAULT 0,
    admin_moderate_profile_names                 BOOLEAN NOT NULL DEFAULT 0,
    admin_moderate_profile_texts                 BOOLEAN NOT NULL DEFAULT 0,
    admin_delete_media_content                   BOOLEAN NOT NULL DEFAULT 0,
    admin_view_all_profiles                      BOOLEAN NOT NULL DEFAULT 0,
    admin_view_private_info                      BOOLEAN NOT NULL DEFAULT 0,
    admin_view_profile_history                   BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_view_info           BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_view_backend_config BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_update_software     BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_reset_data          BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_reboot_backend      BOOLEAN NOT NULL DEFAULT 0,
    admin_server_maintenance_save_backend_config BOOLEAN NOT NULL DEFAULT 0,
    admin_news_create                            BOOLEAN NOT NULL DEFAULT 0,
    admin_news_edit_all                          BOOLEAN NOT NULL DEFAULT 0,
    admin_profile_statistics                     BOOLEAN NOT NULL DEFAULT 0,
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
    -- initial setup = 0
    -- normal = 1
    -- banned = 2
    -- pending deletion = 3
    account_state_number      INTEGER              NOT NULL DEFAULT 0,
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
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- TODO(prod): Remove queue number tables?

-- All next new queue numbers are stored here.
CREATE TABLE IF NOT EXISTS next_queue_number(
    -- Queue type number: 0 = media moderation
    -- Queue type number: 1 = initial media moderation
    queue_type_number       INTEGER PRIMARY KEY     NOT NULL,
    -- Next unused queue number
    next_number             INTEGER                 NOT NULL DEFAULT 0
);

-- Table for storing active queue entries.
-- Only active queue entries are stored here.
CREATE TABLE IF NOT EXISTS queue_entry(
    -- Queue number from next_queue_number table.
    -- The number in that table is incremented when
    -- new queue entry is created.
    queue_number      INTEGER                        NOT NULL,
    -- Queue entry type number. Check next_queue_number table for
    -- available queue type numbers.
    queue_type_number INTEGER                        NOT NULL,
    -- Associate queue entry with account.
    account_id        INTEGER                        NOT NULL,
    PRIMARY KEY (queue_number, queue_type_number),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- Tables for server component account ----------

-- Sign in with related IDs for account
CREATE TABLE IF NOT EXISTS sign_in_with_info(
    account_id         INTEGER PRIMARY KEY NOT NULL,
    google_account_id  TEXT                          UNIQUE,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Account information which can change
CREATE TABLE IF NOT EXISTS account(
    account_id   INTEGER PRIMARY KEY NOT NULL,
    email        TEXT,
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
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- State specific to account component.
CREATE TABLE IF NOT EXISTS account_state(
    account_id            INTEGER PRIMARY KEY NOT NULL,
    next_client_id        INTEGER             NOT NULL DEFAULT 0,
    -- Sync version for news.
    news_sync_version     INTEGER             NOT NULL DEFAULT 0,
    unread_news_count     INTEGER             NOT NULL DEFAULT 0,
    publication_id_at_news_iterator_reset INTEGER,
    publication_id_at_unread_news_count_incrementing INTEGER,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Demo mode user created accounts
CREATE TABLE IF NOT EXISTS demo_mode_account_ids(
    id               INTEGER PRIMARY KEY NOT NULL,
    demo_mode_id     INTEGER             NOT NULL,
    account_id_uuid  BLOB                NOT NULL,
    FOREIGN KEY (account_id_uuid)
        REFERENCES account_id (uuid)
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
    latitude                   DOUBLE               NOT NULL    DEFAULT 0.0,
    longitude                  DOUBLE               NOT NULL    DEFAULT 0.0,
    -- Sync version for profile attributes config file.
    profile_attributes_sync_version   INTEGER       NOT NULL    DEFAULT 0,
    -- Sync version for profile data for this account.
    profile_sync_version              INTEGER       NOT NULL    DEFAULT 0,
    -- Profile age when initial setup is completed
    profile_initial_age               INTEGER,
    profile_initial_age_set_unix_time INTEGER,
    -- 0 = Empty
    -- 1 = WaitingBotOrHumanModeration
    -- 2 = WaitingHumanModeration
    -- 3 = AcceptedByBot
    -- 4 = AcceptedByHuman
    -- 5 = AcceptedUsingAllowlist
    -- 6 = RejectedByBot
    -- 7 = RejectedByHuman
    profile_name_moderation_state     INTEGER        NOT NULL    DEFAULT 0,
    -- 0 = Empty
    -- 1 = WaitingBotOrHumanModeration
    -- 2 = WaitingHumanModeration
    -- 3 = AcceptedByBot
    -- 4 = AcceptedByHuman
    -- 5 = RejectedByBot
    -- 6 = RejectedByHuman
    profile_text_moderation_state     INTEGER       NOT NULL    DEFAULT 0,
    profile_text_moderation_rejected_reason_category INTEGER,
    profile_text_moderation_rejected_reason_details  TEXT,
    profile_text_moderation_moderator_account_id     INTEGER,
    profile_text_edit_time_unix_time                 INTEGER,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (profile_text_moderation_moderator_account_id)
        REFERENCES account_id (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

-- Profile information which can be sent to clients if
-- profile visibility is public.
CREATE TABLE IF NOT EXISTS profile(
    account_id      INTEGER PRIMARY KEY NOT NULL,
    version_uuid    BLOB                NOT NULL,
    name            TEXT                NOT NULL    DEFAULT '',
    profile_text    TEXT                NOT NULL    DEFAULT '',
    -- Age in years and inside inclusive range of [18,99].
    age             INTEGER             NOT NULL    DEFAULT 18,
    last_seen_unix_time  INTEGER,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Store profile attributes which config file defines.
CREATE TABLE IF NOT EXISTS profile_attributes(
    account_id      INTEGER             NOT NULL,
    attribute_id    INTEGER             NOT NULL,
    -- Bitflags value or top level attribute value
    attribute_value_part1 INTEGER,
    -- Sub level attribute value
    attribute_value_part2 INTEGER,
    -- Bitflags value or top level attribute value
    filter_value_part1    INTEGER,
    -- Sub level attribute value
    filter_value_part2    INTEGER,
    filter_accept_missing_attribute BOOLEAN,
    PRIMARY KEY (account_id, attribute_id),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Store profile attribute number list values which config file defines.
CREATE TABLE IF NOT EXISTS profile_attributes_number_list(
    account_id      INTEGER             NOT NULL,
    attribute_id    INTEGER             NOT NULL,
    attribute_value INTEGER             NOT NULL,
    PRIMARY KEY (account_id, attribute_id, attribute_value),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Store profile attribute number list filter values which config file defines.
CREATE TABLE IF NOT EXISTS profile_attributes_number_list_filters(
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
    moderation_state     INTEGER            NOT NULL    DEFAULT 0,
    moderation_rejected_reason_category INTEGER,
    moderation_rejected_reason_details  TEXT,
    moderation_moderator_account_id     INTEGER,
    usage_start_unix_time  INTEGER,
    usage_end_unix_time    INTEGER,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- Tables for server component chat ----------

-- State specific to chat component.
CREATE TABLE IF NOT EXISTS chat_state(
    account_id              INTEGER PRIMARY KEY NOT NULL,
    received_blocks_sync_version INTEGER        NOT NULL DEFAULT 0,
    received_likes_sync_version  INTEGER        NOT NULL DEFAULT 0,
    sent_blocks_sync_version     INTEGER        NOT NULL DEFAULT 0,
    sent_likes_sync_version      INTEGER        NOT NULL DEFAULT 0,
    matches_sync_version         INTEGER        NOT NULL DEFAULT 0,
    -- Bitflag value for pending notification
    pending_notification         INTEGER        NOT NULL DEFAULT 0,
    -- Access token for getting pending notifications from server.
    pending_notification_token   TEXT           UNIQUE,
    fcm_notification_sent        BOOLEAN        NOT NULL DEFAULT 0,
    fcm_device_token             TEXT           UNIQUE,
    new_received_likes_count     INTEGER        NOT NULL DEFAULT 0,
    next_received_like_id        INTEGER        NOT NULL DEFAULT 0,
    received_like_id_at_received_likes_iterator_reset           INTEGER,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS public_key(
    account_id                   INTEGER NOT NULL,
    public_key_version           INTEGER NOT NULL,
    public_key_id                INTEGER,
    public_key_data              TEXT,
    PRIMARY KEY (account_id, public_key_version)
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
    -- The account which started the interaction (e.g. sent a like).
    -- Can be null for example if a like is removed afterwards.
    account_id_sender               INTEGER,
    -- The target of the interaction (e.g. received a like).
    -- Can be null for example if a like is removed afterwards.
    account_id_receiver             INTEGER,
    -- The account which started the block.
    account_id_block_sender         INTEGER,
    -- The account which received the block.
    account_id_block_receiver       INTEGER,
    -- If this is true, then both sides have blocked each other.
    two_way_block                   BOOLEAN NOT NULL DEFAULT 0,
    -- Incrementing counter for getting order number for conversation messages.
    message_counter                 INTEGER NOT NULL DEFAULT 0,
    -- Sender's latest viewed message number in the conversation.
    -- Can be null for example if account is blocked.
    sender_latest_viewed_message    INTEGER,
    -- Receivers's latest viewed message number in the conversation.
    -- Can be null for example if account is blocked.
    receiver_latest_viewed_message  INTEGER,
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
    -- The account which sent the message.
    account_id_sender               INTEGER NOT NULL,
    -- The account which will receive the message.
    account_id_receiver             INTEGER NOT NULL,
    -- Acknowledgement from sender and receiver
    sender_acknowledgement          BOOLEAN NOT NULL DEFAULT 0,
    receiver_acknowledgement        BOOLEAN NOT NULL DEFAULT 0,
    -- Receiving time of the message.
    unix_time                       INTEGER NOT NULL,
    -- Order number for the message in the conversation.
    message_number                  INTEGER NOT NULL,
    -- Client ID and client local ID together makes
    -- an practically unique ID which client can use
    -- detecting was message sent correctly.
    sender_client_id                       INTEGER NOT NULL,
    sender_client_local_id                 INTEGER NOT NULL,
    -- Message bytes.
    message_bytes                   BLOB    NOT NULL,
    FOREIGN KEY (account_id_sender)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (account_id_receiver)
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

-- UUID for account
CREATE TABLE IF NOT EXISTS history_account_id(
    id    INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    uuid  BLOB                              NOT NULL  UNIQUE
);

---------- History tables for server component account ----------

-- All used account IDs. Account ID is not removed from here
-- when account data is removed.
CREATE TABLE IF NOT EXISTS history_used_account_ids(
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    uuid       BLOB                              NOT NULL UNIQUE
);

---------- History tables for server component profile ----------

CREATE TABLE IF NOT EXISTS history_profile_statistics_save_time(
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    unix_time  INTEGER                           NOT NULL
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_men(
    save_time_id  INTEGER NOT NULL,
    age           INTEGER NOT NULL,
    count         INTEGER NOT NULL,
    PRIMARY KEY (save_time_id, age)
    FOREIGN KEY (save_time_id)
        REFERENCES history_profile_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_woman(
    save_time_id  INTEGER NOT NULL,
    age           INTEGER NOT NULL,
    count         INTEGER NOT NULL,
    PRIMARY KEY (save_time_id, age)
    FOREIGN KEY (save_time_id)
        REFERENCES history_profile_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_non_binary(
    save_time_id  INTEGER NOT NULL,
    age           INTEGER NOT NULL,
    count         INTEGER NOT NULL,
    PRIMARY KEY (save_time_id, age)
    FOREIGN KEY (save_time_id)
        REFERENCES history_profile_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_age_changes_all_genders(
    save_time_id  INTEGER NOT NULL,
    age           INTEGER NOT NULL,
    count         INTEGER NOT NULL,
    PRIMARY KEY (save_time_id, age)
    FOREIGN KEY (save_time_id)
        REFERENCES history_profile_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_account(
    save_time_id  INTEGER PRIMARY KEY NOT NULL,
    count INTEGER             NOT NULL,
    FOREIGN KEY (save_time_id)
        REFERENCES history_profile_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_man(
    save_time_id  INTEGER PRIMARY KEY NOT NULL,
    count     INTEGER             NOT NULL,
    FOREIGN KEY (save_time_id)
        REFERENCES history_profile_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_woman(
    save_time_id  INTEGER PRIMARY KEY NOT NULL,
    count   INTEGER             NOT NULL,
    FOREIGN KEY (save_time_id)
        REFERENCES history_profile_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_non_binary(
    save_time_id     INTEGER PRIMARY KEY NOT NULL,
    count INTEGER             NOT NULL,
    FOREIGN KEY (save_time_id)
        REFERENCES history_profile_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS history_profile_statistics_count_changes_all_genders(
    save_time_id     INTEGER PRIMARY KEY NOT NULL,
    count INTEGER             NOT NULL,
    FOREIGN KEY (save_time_id)
        REFERENCES history_profile_statistics_save_time (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- History tables for server component media ----------

CREATE TABLE IF NOT EXISTS history_used_content_ids(
    account_id            INTEGER                           NOT NULL,
    uuid                  BLOB                              NOT NULL,
    PRIMARY KEY (account_id, uuid),
    FOREIGN KEY (account_id)
        REFERENCES history_account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
