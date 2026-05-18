CREATE TABLE IF NOT EXISTS f_i_reflux_subscription_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(64) NOT NULL,
    scope VARCHAR(32) NOT NULL,
    team_code VARCHAR(64) NOT NULL DEFAULT 'global',
    season INT NULL,
    title VARCHAR(128) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    price_cents INT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INT NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_code, code)
);

CREATE INDEX IF NOT EXISTS idx_f_i_reflux_subscription_plans_enabled
    ON f_i_reflux_subscription_plans (team_code, enabled, sort_order);

CREATE TABLE IF NOT EXISTS f_i_user_notification_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    channel VARCHAR(32) NOT NULL,
    target TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, channel)
);

CREATE INDEX IF NOT EXISTS idx_f_i_user_notification_targets_user_id
    ON f_i_user_notification_targets (user_id, channel, is_active);

CREATE TABLE IF NOT EXISTS f_i_user_reflux_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    plan_code VARCHAR(64) NOT NULL,
    scope VARCHAR(32) NOT NULL,
    team_code VARCHAR(64) NOT NULL,
    season INT NULL,
    match_id BIGINT NULL,
    order_no VARCHAR(64) NOT NULL REFERENCES f_i_payment_orders(order_no),
    starts_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_user_reflux_subscriptions_lookup
    ON f_i_user_reflux_subscriptions (team_code, match_id, season, status, expires_at);

CREATE INDEX IF NOT EXISTS idx_f_i_user_reflux_subscriptions_active_scope
    ON f_i_user_reflux_subscriptions (team_code, scope, season, match_id, status);

CREATE INDEX IF NOT EXISTS idx_f_i_user_reflux_subscriptions_user_id
    ON f_i_user_reflux_subscriptions (user_id, status, created_at DESC);

CREATE TABLE IF NOT EXISTS f_i_reflux_notification_cursors (
    team_code VARCHAR(64) NOT NULL,
    match_id BIGINT NOT NULL,
    last_processed_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_code, match_id)
);

CREATE TABLE IF NOT EXISTS f_i_reflux_notification_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    target_id UUID NOT NULL REFERENCES f_i_user_notification_targets(id) ON DELETE CASCADE,
    team_code VARCHAR(64) NOT NULL,
    match_id BIGINT NULL,
    subject TEXT NOT NULL,
    body_html TEXT NOT NULL,
    payload_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    attempts INT NOT NULL DEFAULT 0,
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_error TEXT NULL,
    sent_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_reflux_notification_jobs_pending
    ON f_i_reflux_notification_jobs (status, next_attempt_at, attempts);

CREATE INDEX IF NOT EXISTS idx_f_i_reflux_notification_jobs_user_match
    ON f_i_reflux_notification_jobs (user_id, team_code, match_id, created_at DESC);

INSERT INTO f_i_reflux_subscription_plans (
    code,
    scope,
    team_code,
    season,
    title,
    description,
    price_cents,
    enabled,
    sort_order,
    expires_at
) VALUES
(
    'single_match',
    'single_match',
    'global',
    2026,
    '单场回流提醒',
    '订阅当前比赛回流邮件提醒。监控到新增回流后按分钟聚合发送。',
    500,
    TRUE,
    10,
    NULL
),
(
    'season_2026',
    'season',
    'global',
    2026,
    '2026 赛季回流提醒',
    '订阅当前队伍 2026 赛季回流邮件提醒。第一版只提醒当前正在监控的比赛。',
    5000,
    TRUE,
    20,
    '2026-12-31 15:59:59+00'
)
ON CONFLICT (team_code, code) DO UPDATE SET
    scope = EXCLUDED.scope,
    season = EXCLUDED.season,
    title = EXCLUDED.title,
    description = EXCLUDED.description,
    price_cents = EXCLUDED.price_cents,
    enabled = EXCLUDED.enabled,
    sort_order = EXCLUDED.sort_order,
    expires_at = EXCLUDED.expires_at,
    updated_at = NOW();
