ALTER TABLE f_i_standings
    ADD COLUMN IF NOT EXISTS scrape_run_id UUID NULL REFERENCES f_i_scrape_runs(id),
    ADD COLUMN IF NOT EXISTS round_number INT NULL,
    ADD COLUMN IF NOT EXISTS snapshot_kind VARCHAR(16) NOT NULL DEFAULT 'live';

ALTER TABLE f_i_team_ranking_snapshots
    ADD COLUMN IF NOT EXISTS round_number INT NULL,
    ADD COLUMN IF NOT EXISTS snapshot_kind VARCHAR(16) NOT NULL DEFAULT 'live';

ALTER TABLE f_i_player_ranking_snapshots
    ADD COLUMN IF NOT EXISTS round_number INT NULL,
    ADD COLUMN IF NOT EXISTS snapshot_kind VARCHAR(16) NOT NULL DEFAULT 'live';

CREATE INDEX IF NOT EXISTS idx_f_i_standings_season_kind_round_snapshot
    ON f_i_standings (season, snapshot_kind, round_number, snapshot_at DESC);

CREATE INDEX IF NOT EXISTS idx_f_i_team_ranking_snapshots_season_kind_round
    ON f_i_team_ranking_snapshots (season, snapshot_kind, round_number, snapshot_at DESC);

CREATE INDEX IF NOT EXISTS idx_f_i_player_ranking_snapshots_season_kind_round
    ON f_i_player_ranking_snapshots (season, snapshot_kind, round_number, snapshot_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_standings_round_final_team_unique
    ON f_i_standings (season, round_number, snapshot_kind, team_id)
    WHERE snapshot_kind = 'round_final';

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_team_ranking_snapshots_round_final_unique
    ON f_i_team_ranking_snapshots (season, round_number, snapshot_kind, category_id)
    WHERE snapshot_kind = 'round_final';

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_player_ranking_snapshots_round_final_unique
    ON f_i_player_ranking_snapshots (season, round_number, snapshot_kind, category_id)
    WHERE snapshot_kind = 'round_final';
