-- Add migration script here

CREATE TABLE IF NOT EXISTS Profile(
    id     TEXT PRIMARY KEY NOT NULL,
    name   TEXT             NOT NULL DEFAULT ''
);
