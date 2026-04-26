CREATE TABLE IF NOT EXISTS f_i_wechat_followers (
    id UUID PRIMARY KEY,
    open_id VARCHAR(128) NOT NULL UNIQUE,
    latest_invite_code_id UUID NULL REFERENCES f_i_invite_codes(id) ON DELETE SET NULL,
    subscribe_count INT NOT NULL DEFAULT 1,
    subscribed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    unsubscribed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_wechat_followers_latest_invite_code_id
    ON f_i_wechat_followers (latest_invite_code_id);
