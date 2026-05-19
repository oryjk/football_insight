CREATE TABLE IF NOT EXISTS f_i_user_device_tokens (
    id           BIGSERIAL PRIMARY KEY,
    user_id      UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    device_token TEXT NOT NULL,
    platform     TEXT NOT NULL CHECK (platform IN ('jpush', 'fcm', 'apns')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(device_token)
);

CREATE INDEX idx_f_i_user_device_tokens_user_id ON f_i_user_device_tokens(user_id);
