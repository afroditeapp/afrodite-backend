
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

-- CREATE TABLE IF NOT EXISTS RefreshToken(
--     account_row_id  INTEGER PRIMARY KEY,
--     refresh_token   TEXT                UNIQUE,  -- Can be null
--     FOREIGN KEY (account_row_id)
--         REFERENCES AccountId (account_row_id)
--             ON DELETE CASCADE
--             ON UPDATE CASCADE
-- );

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
    json_text       TEXT    NOT NULL    DEFAULT '',
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
            ON UPDATE CASCADE,
    FOREIGN KEY (queue_number)
        REFERENCES MediaModerationQueueNumber (queue_number)
            ON DELETE NO ACTION
            ON UPDATE NO ACTION
);

CREATE TABLE IF NOT EXISTS MediaModeration(
    row_id          INTEGER PRIMARY KEY,
    account_row_id  INTEGER NOT NULL,       -- What admin account is moderating
    request_row_id  INTEGER NOT NULL,       -- What request is in moderation
    state_number    INTEGER NOT NULL,       -- State of the moderation
    json_text       TEXT    NOT NULL,       -- What was moderated
    FOREIGN KEY (account_row_id)
        REFERENCES AccountId (account_row_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (request_row_id)
        REFERENCES MediaModerationRequest (request_row_id)
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
