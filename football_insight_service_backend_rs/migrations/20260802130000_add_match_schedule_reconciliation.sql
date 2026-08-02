ALTER TABLE f_i_matches
    ADD COLUMN IF NOT EXISTS source_active BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS last_seen_run_id UUID NULL REFERENCES f_i_scrape_runs(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS schedule_source VARCHAR(16) NOT NULL DEFAULT 'sina',
    ADD COLUMN IF NOT EXISTS source_match_id VARCHAR(64) NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_matches_fixture_identity
    ON f_i_matches (season, round_number, home_team_id, away_team_id);

CREATE INDEX IF NOT EXISTS idx_f_i_matches_active_schedule
    ON f_i_matches (season, round_number, match_date, match_time)
    WHERE source_active = TRUE;
