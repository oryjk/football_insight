CREATE TABLE IF NOT EXISTS f_i_scrape_runs (
    id UUID PRIMARY KEY,
    source VARCHAR(50) NOT NULL,
    season INT NOT NULL,
    status VARCHAR(20) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ NULL,
    remark TEXT NULL
);

CREATE TABLE IF NOT EXISTS f_i_ranking_categories (
    id BIGSERIAL PRIMARY KEY,
    scope VARCHAR(20) NOT NULL,
    item_id INT NOT NULL,
    slug VARCHAR(64) NOT NULL,
    label VARCHAR(64) NOT NULL,
    UNIQUE (scope, item_id)
);

CREATE TABLE IF NOT EXISTS f_i_matches (
    id BIGSERIAL PRIMARY KEY,
    match_id BIGINT NOT NULL UNIQUE,
    season INT NOT NULL,
    round_number INT NOT NULL,
    round_name VARCHAR(32) NOT NULL,
    match_date DATE NOT NULL,
    match_time VARCHAR(16) NOT NULL,
    status VARCHAR(16) NOT NULL,
    home_team_id BIGINT NOT NULL,
    home_team_name VARCHAR(128) NOT NULL,
    home_score VARCHAR(16) NOT NULL,
    away_team_id BIGINT NOT NULL,
    away_team_name VARCHAR(128) NOT NULL,
    away_score VARCHAR(16) NOT NULL,
    home_logo TEXT NOT NULL,
    away_logo TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS f_i_standings (
    id BIGSERIAL PRIMARY KEY,
    season INT NOT NULL,
    team_id BIGINT NOT NULL,
    team_name VARCHAR(128) NOT NULL,
    team_logo TEXT NOT NULL,
    rank_no INT NOT NULL,
    played INT NOT NULL,
    wins INT NOT NULL,
    draws INT NOT NULL,
    losses INT NOT NULL,
    goals_for INT NOT NULL,
    goals_against INT NOT NULL,
    goal_difference INT NOT NULL,
    points INT NOT NULL,
    snapshot_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS f_i_team_ranking_snapshots (
    id BIGSERIAL PRIMARY KEY,
    scrape_run_id UUID NOT NULL REFERENCES f_i_scrape_runs(id),
    season INT NOT NULL,
    category_id BIGINT NOT NULL REFERENCES f_i_ranking_categories(id),
    snapshot_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    entry_count INT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS f_i_team_ranking_entries (
    id BIGSERIAL PRIMARY KEY,
    snapshot_id BIGINT NOT NULL REFERENCES f_i_team_ranking_snapshots(id) ON DELETE CASCADE,
    team_id BIGINT NOT NULL,
    team_name VARCHAR(128) NOT NULL,
    team_logo TEXT NOT NULL,
    rank_no INT NOT NULL,
    score_value VARCHAR(64) NOT NULL,
    UNIQUE (snapshot_id, team_id)
);

CREATE TABLE IF NOT EXISTS f_i_player_ranking_snapshots (
    id BIGSERIAL PRIMARY KEY,
    scrape_run_id UUID NOT NULL REFERENCES f_i_scrape_runs(id),
    season INT NOT NULL,
    category_id BIGINT NOT NULL REFERENCES f_i_ranking_categories(id),
    snapshot_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    entry_count INT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS f_i_player_ranking_entries (
    id BIGSERIAL PRIMARY KEY,
    snapshot_id BIGINT NOT NULL REFERENCES f_i_player_ranking_snapshots(id) ON DELETE CASCADE,
    player_id BIGINT NOT NULL,
    player_name VARCHAR(128) NOT NULL,
    player_logo TEXT NOT NULL,
    team_id BIGINT NOT NULL,
    team_name VARCHAR(128) NOT NULL,
    rank_no INT NOT NULL,
    score_value VARCHAR(64) NOT NULL,
    penalty_value VARCHAR(64) NULL,
    UNIQUE (snapshot_id, player_id)
);
