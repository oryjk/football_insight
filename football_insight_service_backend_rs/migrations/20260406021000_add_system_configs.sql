CREATE TABLE IF NOT EXISTS f_i_system_configs (
    config_key VARCHAR(100) PRIMARY KEY,
    config_value TEXT NOT NULL,
    description TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO f_i_system_configs (config_key, config_value, description)
VALUES (
    'wechat_login_enabled',
    'false',
    'Whether to show wechat quick login entry on H5.'
)
ON CONFLICT (config_key) DO NOTHING;
