CREATE TABLE f_i_wechat_official_bind_codes (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    code_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX uq_wechat_official_bind_codes_hash ON f_i_wechat_official_bind_codes (code_hash);
CREATE INDEX idx_wechat_official_bind_codes_user ON f_i_wechat_official_bind_codes (user_id, created_at DESC);
