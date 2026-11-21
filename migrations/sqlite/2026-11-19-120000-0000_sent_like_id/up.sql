-- Column for sent likes iterator uses globally unique like ID to
-- return correct pages. Like IDs are assigned from a global counter
-- and ordering goes from smaller to larger values. NULL is used
-- for account interactions which are not in Like state.
ALTER TABLE account_interaction ADD COLUMN sent_like_id BIGINT;

-- Backfill: account_interaction.id is globally unique and matches
-- creation order, so it doubles as a monotonic like ID for likes
-- which were sent before this column existed.
UPDATE account_interaction
SET sent_like_id = id
WHERE state_number = 1 AND account_id_sender IS NOT NULL;

ALTER TABLE chat_global_state ADD COLUMN next_sent_like_id BIGINT NOT NULL DEFAULT 0;

-- Initialize the sent like ID counter to the next unused value
-- above all backfilled IDs. The counter is "next unused ID", so
-- it must be one greater than the highest already assigned ID.
INSERT INTO chat_global_state (row_type, next_match_id, next_sent_like_id)
VALUES (0, 0, (SELECT COALESCE(MAX(sent_like_id), 0) + 1 FROM account_interaction))
ON CONFLICT (row_type) DO UPDATE
SET next_sent_like_id = EXCLUDED.next_sent_like_id;
