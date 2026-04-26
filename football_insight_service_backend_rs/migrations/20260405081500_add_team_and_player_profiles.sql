CREATE TABLE IF NOT EXISTS f_i_teams (
    id BIGSERIAL PRIMARY KEY,
    team_id BIGINT NOT NULL UNIQUE,
    team_name VARCHAR(128) NOT NULL,
    avatar_source_url TEXT NULL,
    avatar_object_name TEXT NULL,
    avatar_storage_url TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS f_i_players (
    id BIGSERIAL PRIMARY KEY,
    player_id BIGINT NOT NULL UNIQUE,
    player_name VARCHAR(128) NOT NULL,
    team_id BIGINT NOT NULL,
    team_name VARCHAR(128) NOT NULL,
    avatar_source_url TEXT NULL,
    avatar_object_name TEXT NULL,
    avatar_storage_url TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
