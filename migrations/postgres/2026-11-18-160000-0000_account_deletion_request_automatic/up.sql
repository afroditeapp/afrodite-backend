ALTER TABLE account_state
    ADD COLUMN account_deletion_request_automatic BOOLEAN NOT NULL DEFAULT FALSE;
