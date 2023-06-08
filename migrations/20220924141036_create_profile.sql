
-- Tables used with current and history data

CREATE TABLE IF NOT EXISTS AccountId(
    account_row_id  INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id      BLOB    NOT NULL    UNIQUE
);

-- Tables for current data

CREATE TABLE IF NOT EXISTS ApiKey(
    account_row_id  INTEGER PRIMARY KEY,
    api_key         TEXT                UNIQUE,  -- Can be null
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS RefreshToken(
    account_row_id  INTEGER PRIMARY KEY,
    refresh_token   BLOB                UNIQUE,  -- Can be null
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS SignInWithInfo(
    account_row_id  INTEGER PRIMARY KEY,
    google_account_id   TEXT            UNIQUE,  -- Can be null
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS Account(
    account_row_id  INTEGER PRIMARY KEY,
    json_text       TEXT    NOT NULL    DEFAULT '',
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS AccountSetup(
    account_row_id  INTEGER PRIMARY KEY,
    json_text       TEXT    NOT NULL    DEFAULT '',
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS Profile(
    account_row_id  INTEGER PRIMARY KEY,
    version_uuid    BLOB    NOT NULL,
    location_key_x  INTEGER NOT NULL    DEFAULT 0,
    location_key_y  INTEGER NOT NULL    DEFAULT 0,
    name            TEXT    NOT NULL    DEFAULT '',
    profile_text    TEXT    NOT NULL    DEFAULT '',
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Tables for media features

CREATE TABLE IF NOT EXISTS CurrentAccountMedia(        -- Currently selected images for account
    account_row_id            INTEGER PRIMARY KEY,
    security_content_row_id   INTEGER,
    profile_content_row_id    INTEGER,
    grid_crop_size            REAL NOT NULL DEFAULT 1.0, -- image's max square size multipler
    grid_crop_x               REAL NOT NULL DEFAULT 0.0, -- X coordinate for square top left corner. Counted from top left corner of the original image.
    grid_crop_y               REAL NOT NULL DEFAULT 0.0, -- Y coordinate for square top left corner. Counted from top left corner of the original image.
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (security_content_row_id)
        REFERENCES MediaContent (content_row_id)
            ON DELETE SET NULL
            ON UPDATE CASCADE,
    FOREIGN KEY (profile_content_row_id)
        REFERENCES MediaContent (content_row_id)
            ON DELETE SET NULL
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS MediaContent(
    content_row_id   INTEGER PRIMARY KEY,
    content_id       BLOB    NOT NULL       UNIQUE,  -- Content UUID should be unique
    account_row_id   INTEGER NOT NULL,
    moderation_state INTEGER NOT NULL,
    content_type     INTEGER NOT NULL       DEFAULT 0, -- Moderator sets this. 0 not set, 1 normal, 2 security
    slot_number      INTEGER NOT NULL,
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Active queue numbers only
CREATE TABLE IF NOT EXISTS MediaModerationQueueNumber(
    queue_number    INTEGER PRIMARY KEY AUTOINCREMENT,
    account_row_id  INTEGER NOT NULL    UNIQUE,  -- One number per account
    sub_queue       INTEGER NOT NULL,
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS MediaModerationRequest(
    request_row_id  INTEGER PRIMARY KEY  NOT NULL, -- Not null needed here for sqlx
    account_row_id  INTEGER NOT NULL     UNIQUE,   -- One request per account
    queue_number    INTEGER NOT NULL,
    json_text       TEXT    NOT NULL,
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
    -- TODO: Disabled foregin key contraint for queue_number to make deletion
    -- possible. Figure out could queue_number be deleted here or constraint SET
    -- NULL? Or just use current version?
    -- FOREIGN KEY (queue_number)
    --    REFERENCES MediaModerationQueueNumber (queue_number)
    --        ON DELETE NO ACTION
    --        ON UPDATE NO ACTION
);

CREATE TABLE IF NOT EXISTS MediaModeration(
    account_row_id  INTEGER NOT NULL,       -- What admin account is moderating
    request_row_id  INTEGER NOT NULL,       -- What request is in moderation
    state_number    INTEGER NOT NULL,       -- State of the moderation
    json_text       TEXT    NOT NULL,       -- What was moderated
    PRIMARY KEY (account_row_id, request_row_id),
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (request_row_id)
        REFERENCES MediaModerationRequest (request_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS Profile(
    account_row_id  INTEGER PRIMARY KEY,
    version_uuid    BLOB    NOT NULL,
    location_key_x  INTEGER NOT NULL    DEFAULT 0,
    location_key_y  INTEGER NOT NULL    DEFAULT 0,
    name            TEXT    NOT NULL    DEFAULT '',
    profile_text    TEXT    NOT NULL    DEFAULT '',
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- Tables for history

CREATE TABLE IF NOT EXISTS HistoryAccount(
    row_id          INTEGER PRIMARY KEY AUTOINCREMENT,
    account_row_id  INTEGER NOT NULL,
    unix_time       INTEGER NOT NULL,
    json_text       TEXT    NOT NULL,
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS HistoryAccountSetup(
    row_id          INTEGER PRIMARY KEY AUTOINCREMENT,
    account_row_id  INTEGER NOT NULL,
    unix_time       INTEGER NOT NULL,
    json_text       TEXT    NOT NULL,
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS HistoryProfile(
    row_id          INTEGER PRIMARY KEY AUTOINCREMENT,
    account_row_id  INTEGER NOT NULL,
    unix_time       INTEGER NOT NULL,
    json_text       TEXT    NOT NULL,
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

-- TODO: History for new media tables.

-- Deletion is just ignored as it happens automatically when new
-- request is created.
CREATE TABLE IF NOT EXISTS HistoryMediaModerationRequest(
    row_id          INTEGER PRIMARY KEY AUTOINCREMENT,
    account_row_id  INTEGER NOT NULL,
    unix_time       INTEGER NOT NULL,

    request_row_id  INTEGER NOT NULL,
    state_number    INTEGER NOT NULL,
    json_text       TEXT    NOT NULL,
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
