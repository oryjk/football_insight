CREATE TABLE IF NOT EXISTS f_i_player_match_stats (
    player_id BIGINT NOT NULL,
    match_id BIGINT NOT NULL REFERENCES f_i_matches(match_id) ON DELETE CASCADE,
    season INT NOT NULL,
    goals INT NOT NULL CHECK (goals BETWEEN 0 AND 30),
    minutes_played INT NOT NULL CHECK (minutes_played BETWEEN 1 AND 130),
    source_player_id TEXT NOT NULL,
    source_match_id TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'cfl',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (player_id, match_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_player_match_stats_season_player
    ON f_i_player_match_stats (season, player_id);
