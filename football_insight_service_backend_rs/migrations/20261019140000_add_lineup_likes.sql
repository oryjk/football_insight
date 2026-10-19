CREATE TABLE IF NOT EXISTS f_i_lineup_likes (
    lineup_id UUID NOT NULL REFERENCES f_i_lineups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (lineup_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_lineup_likes_user_id ON f_i_lineup_likes(user_id);
CREATE INDEX IF NOT EXISTS idx_f_i_lineups_team_created
    ON f_i_lineups(team_id, created_at DESC, id DESC);
