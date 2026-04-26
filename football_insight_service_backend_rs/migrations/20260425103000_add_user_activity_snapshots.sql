CREATE TABLE IF NOT EXISTS f_i_user_activity_snapshots (
    user_id UUID PRIMARY KEY REFERENCES f_i_users(id) ON DELETE CASCADE,
    last_login_at TIMESTAMPTZ,
    last_active_at TIMESTAMPTZ,
    last_active_page_key TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS f_i_user_page_activity_snapshots (
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    page_key TEXT NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    visit_count BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, page_key)
);

CREATE INDEX IF NOT EXISTS idx_f_i_user_activity_snapshots_last_active_at
    ON f_i_user_activity_snapshots (last_active_at DESC);

CREATE INDEX IF NOT EXISTS idx_f_i_user_page_activity_snapshots_page_key_last_seen_at
    ON f_i_user_page_activity_snapshots (page_key, last_seen_at DESC);
