
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
