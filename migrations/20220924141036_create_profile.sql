-- Add migration script here

CREATE TABLE IF NOT EXISTS Account(
    account_id TEXT PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS AccountState(
    state_json      TEXT     NOT NULL DEFAULT '',
    account_id  TEXT PRIMARY KEY NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES Account (account_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);


-- ApiKeys do net need SQLite storage as those are
-- loaded from git directory at startup.

CREATE TABLE IF NOT EXISTS Profile(
    profile_json  TEXT     NOT NULL DEFAULT '',
    account_id    TEXT PRIMARY KEY NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES Account (account_id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
