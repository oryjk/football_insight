-- 球队完整名单（含往季前队员），由 sina_csl_scraper sync-squads 写入，后端 lineup 模块只读。
CREATE TABLE IF NOT EXISTS f_i_squad_players (
    team_id BIGINT NOT NULL,
    player_id BIGINT NOT NULL,
    player_name TEXT NOT NULL,
    shirt_number INTEGER,
    position TEXT CHECK (position IN ('GK', 'DF', 'MF', 'FW')),
    country TEXT,
    avatar_source_url TEXT,
    avatar_storage_url TEXT,
    last_season INTEGER NOT NULL,
    active BOOLEAN NOT NULL DEFAULT FALSE,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_squad_players_team_active
    ON f_i_squad_players (team_id, active);
