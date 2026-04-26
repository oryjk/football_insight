CREATE TABLE IF NOT EXISTS f_i_team_insights (
    id BIGSERIAL PRIMARY KEY,
    scrape_run_id UUID NULL REFERENCES f_i_scrape_runs(id),
    season INT NOT NULL,
    round_number INT NULL,
    snapshot_kind VARCHAR(16) NOT NULL,
    team_id BIGINT NOT NULL,
    team_name VARCHAR(128) NOT NULL,
    rank_no INT NOT NULL,
    avatar_storage_url TEXT NULL,
    goals_for_total INT NOT NULL,
    goals_against_total INT NOT NULL,
    goals_for_by_opponent JSONB NOT NULL DEFAULT '[]'::jsonb,
    goals_for_by_player JSONB NOT NULL DEFAULT '[]'::jsonb,
    goals_against_by_opponent JSONB NOT NULL DEFAULT '[]'::jsonb,
    snapshot_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_team_insights_season_kind_round_snapshot
    ON f_i_team_insights (season, snapshot_kind, round_number, snapshot_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_team_insights_live_team_unique
    ON f_i_team_insights (season, snapshot_kind, team_id)
    WHERE snapshot_kind = 'live';

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_team_insights_round_final_team_unique
    ON f_i_team_insights (season, round_number, snapshot_kind, team_id)
    WHERE snapshot_kind = 'round_final';
