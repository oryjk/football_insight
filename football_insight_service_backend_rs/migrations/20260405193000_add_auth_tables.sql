CREATE TABLE IF NOT EXISTS f_i_users (
    id UUID PRIMARY KEY,
    wx_open_id VARCHAR(128) NOT NULL UNIQUE,
    union_id VARCHAR(128) NULL,
    display_name VARCHAR(128) NULL,
    avatar_url TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS f_i_user_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    session_token VARCHAR(128) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_user_sessions_user_id
    ON f_i_user_sessions (user_id);

CREATE INDEX IF NOT EXISTS idx_f_i_user_sessions_expires_at
    ON f_i_user_sessions (expires_at);
