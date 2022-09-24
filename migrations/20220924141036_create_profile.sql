-- Add migration script here

CREATE TABLE IF NOT EXISTS User(
    id     TEXT PRIMARY KEY NOT NULL,
    name   TEXT             NOT NULL DEFAULT ''
);
