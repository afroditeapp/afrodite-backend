-- Your SQL goes here

---------- Tables for server component common ----------

-- UUID for account
CREATE TABLE IF NOT EXISTS account_id(
    id    INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
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
    account_id INTEGER PRIMARY KEY NOT NULL,
    json_text  TEXT                NOT NULL  DEFAULT '',
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Account information which can not change after account initial setup completes
CREATE TABLE IF NOT EXISTS account_setup(
    account_id  INTEGER PRIMARY KEY NOT NULL,
    name        TEXT                NOT NULL  DEFAULT '',
    email       TEXT                NOT NULL  DEFAULT '',
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- Tables for server component profile ----------

CREATE TABLE IF NOT EXISTS profile(
    account_id      INTEGER PRIMARY KEY NOT NULL,
    version_uuid    BLOB                NOT NULL,
    name            TEXT                NOT NULL    DEFAULT '',
    profile_text    TEXT                NOT NULL    DEFAULT '',
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS profile_location(
    account_id      INTEGER PRIMARY KEY NOT NULL,
    latitude        DOUBLE              NOT NULL    DEFAULT 0.0,
    longitude       DOUBLE              NOT NULL    DEFAULT 0.0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- Tables for server component media ----------

-- Currently selected images for account
CREATE TABLE IF NOT EXISTS current_account_media(
    account_id           INTEGER PRIMARY KEY NOT NULL,
    security_content_id  INTEGER,
    profile_content_id   INTEGER,
    -- Image's max square size multipler
    grid_crop_size       DOUBLE              NOT NULL DEFAULT 1.0,
    -- X coordinate for square top left corner.
    -- Counted from top left corner of the original image.
    grid_crop_x          DOUBLE              NOT NULL DEFAULT 0.0,
    -- Y coordinate for square top left corner.
    -- Counted from top left corner of the original image.
    grid_crop_y          DOUBLE              NOT NULL DEFAULT 0.0,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (security_content_id)
        REFERENCES media_content (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE,
    FOREIGN KEY (profile_content_id)
        REFERENCES media_content (id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

-- Information about uploaded media content
CREATE TABLE IF NOT EXISTS media_content(
    id               INTEGER PRIMARY KEY NOT NULL,
    uuid             BLOB                NOT NULL   UNIQUE,
    account_id       INTEGER             NOT NULL,
    moderation_state INTEGER             NOT NULL,
    -- Moderator sets this. 0 not set, 1 normal, 2 security
    content_type     INTEGER             NOT NULL   DEFAULT 0,
    slot_number      INTEGER             NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Table for crating moderation queue numbers using the
-- automatically incrementing queue_number column.
-- Only active queue numbers are stored here.
CREATE TABLE IF NOT EXISTS media_moderation_queue_number(
    queue_number INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    -- Associate queue number with account. Only one queue
    -- number per account is allowed.
    account_id   INTEGER                           NOT NULL UNIQUE,
    -- Priority number for the queue number.
    sub_queue    INTEGER                           NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- User made moderation request
CREATE TABLE IF NOT EXISTS media_moderation_request(
    id                  INTEGER PRIMARY KEY NOT NULL,
    -- Request owner Account ID. One request per account.
    account_id          INTEGER             NOT NULL  UNIQUE,
    -- Queue number which this media_moderation_request has.
    queue_number        INTEGER             NOT NULL,
    json_text           TEXT                NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
    -- TODO: Disabled foregin key contraint for queue_number to make deletion
    -- possible. Figure out could queue_number be deleted here or constraint SET
    -- NULL? Or just use current version?
    -- Update: modified this when added diesel support.
    -- FOREIGN KEY (queue_number)
    --    REFERENCES MediaModerationQueueNumber (queue_number)
    --        ON DELETE SET NULL
    --        ON UPDATE RESTRICT
);

-- Admin made moderation
CREATE TABLE IF NOT EXISTS media_moderation(
    -- What admin account is moderating
    account_id                INTEGER NOT NULL,
    -- What request is in moderation
    moderation_request_id   INTEGER NOT NULL,
    -- State of the moderation
    state_number            INTEGER NOT NULL,
    -- What was moderated
    json_text               TEXT    NOT NULL,
    PRIMARY KEY (account_id, moderation_request_id),
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (moderation_request_id)
        REFERENCES media_moderation_request (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- History tables for server component account ----------

CREATE TABLE IF NOT EXISTS history_account(
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    account_id INTEGER                           NOT NULL,
    unix_time  INTEGER                           NOT NULL,
    json_text  TEXT                              NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- TODO: Can be removed as account_setup state does not change after
-- initial setup?
CREATE TABLE IF NOT EXISTS history_account_setup(
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    account_id INTEGER                           NOT NULL,
    unix_time  INTEGER                           NOT NULL,
    json_text  TEXT                              NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- History tables for server component profile ----------

CREATE TABLE IF NOT EXISTS history_profile(
    id     INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    account_id INTEGER                           NOT NULL,
    unix_time  INTEGER                           NOT NULL,
    json_text  TEXT                              NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

---------- History tables for server component media ----------

-- TODO: History for new media tables.

-- Deletion is just ignored as it happens automatically when new
-- request is created.
CREATE TABLE IF NOT EXISTS history_media_moderation_request(
    id                    INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    account_id            INTEGER                           NOT NULL,
    unix_time             INTEGER                           NOT NULL,
    moderation_request_id INTEGER                           NOT NULL,
    state_number          INTEGER                           NOT NULL,
    json_text             TEXT                              NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES account_id (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
