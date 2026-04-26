ALTER TABLE f_i_users
    ADD COLUMN IF NOT EXISTS favorite_team_id BIGINT NULL REFERENCES f_i_teams(team_id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_f_i_users_favorite_team_id
    ON f_i_users (favorite_team_id);

CREATE TABLE IF NOT EXISTS f_i_match_support_votes (
    id UUID PRIMARY KEY,
    match_id BIGINT NOT NULL REFERENCES f_i_matches(match_id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    supported_team_id BIGINT NOT NULL REFERENCES f_i_teams(team_id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (match_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_match_support_votes_match_team
    ON f_i_match_support_votes (match_id, supported_team_id);

CREATE INDEX IF NOT EXISTS idx_f_i_match_support_votes_supported_team_id
    ON f_i_match_support_votes (supported_team_id);
