ALTER TABLE used_account_ids
    ADD COLUMN initial_setup_completed BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE used_account_ids
    ADD COLUMN account_deleted_unix_time BIGINT;

UPDATE used_account_ids
SET initial_setup_completed = TRUE
WHERE id IN (
    SELECT account_id
    FROM shared_state
    WHERE account_state_initial_setup_completed = TRUE
);

UPDATE used_account_ids
SET account_deleted_unix_time = unixepoch()
WHERE id NOT IN (
    SELECT id
    FROM account_id
);
