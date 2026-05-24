CREATE INDEX IF NOT EXISTS idx_f_i_matches_overview_recent_finished
    ON f_i_matches (season, status, round_number DESC, match_date DESC, match_time DESC, match_id DESC);

CREATE INDEX IF NOT EXISTS idx_f_i_player_ranking_entries_snapshot_rank
    ON f_i_player_ranking_entries (snapshot_id, rank_no ASC);

CREATE INDEX IF NOT EXISTS idx_f_i_standings_snapshot_rank
    ON f_i_standings (season, snapshot_kind, round_number, snapshot_at DESC, rank_no ASC);

CREATE INDEX IF NOT EXISTS idx_f_i_matches_home_team_schedule
    ON f_i_matches (home_team_id, match_date ASC, match_time ASC, match_id ASC);

CREATE INDEX IF NOT EXISTS idx_f_i_matches_away_team_schedule
    ON f_i_matches (away_team_id, match_date ASC, match_time ASC, match_id ASC);

CREATE INDEX IF NOT EXISTS idx_f_i_standings_team_latest
    ON f_i_standings (team_id, snapshot_at DESC);
