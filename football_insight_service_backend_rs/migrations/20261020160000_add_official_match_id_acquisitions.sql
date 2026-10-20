-- Existing users have already received an ID. New accounts start with no acquisition.
ALTER TABLE f_i_users
    ADD COLUMN official_match_id_received BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE f_i_users SET official_match_id_received = TRUE;

-- The official identity persists across account binding changes and unfollow/refollow.
CREATE TABLE f_i_official_match_id_acquisitions (
    official_open_id VARCHAR(128) PRIMARY KEY,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source TEXT NOT NULL CHECK (source IN ('historical', 'acquired'))
);

-- Include old followers with no account, unfollowed users and previously issued invitations.
INSERT INTO f_i_official_match_id_acquisitions (official_open_id, source)
SELECT official_open_id, 'historical'
FROM (
    SELECT open_id AS official_open_id FROM f_i_wechat_followers
    UNION
    SELECT official_wechat_open_id FROM f_i_users
    UNION
    SELECT issued_for_wechat_open_id FROM f_i_invite_codes
) AS existing_identities
WHERE official_open_id IS NOT NULL AND BTRIM(official_open_id) <> '';
