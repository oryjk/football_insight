CREATE TABLE IF NOT EXISTS f_i_admin_users (
    id UUID PRIMARY KEY,
    username VARCHAR(64) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name VARCHAR(128) NOT NULL,
    role VARCHAR(32) NOT NULL DEFAULT 'owner',
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_f_i_admin_users_role CHECK (role IN ('owner', 'admin')),
    CONSTRAINT chk_f_i_admin_users_status CHECK (status IN ('active', 'disabled'))
);

CREATE TABLE IF NOT EXISTS f_i_admin_sessions (
    id UUID PRIMARY KEY,
    admin_user_id UUID NOT NULL REFERENCES f_i_admin_users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_admin_sessions_active
    ON f_i_admin_sessions (admin_user_id, expires_at)
    WHERE revoked_at IS NULL;

CREATE TABLE IF NOT EXISTS f_i_admin_audit_logs (
    id UUID PRIMARY KEY,
    admin_user_id UUID NOT NULL REFERENCES f_i_admin_users(id),
    action VARCHAR(64) NOT NULL,
    target_type VARCHAR(32) NOT NULL,
    target_id TEXT,
    reason TEXT,
    before_json JSONB,
    after_json JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_admin_audit_logs_created_at
    ON f_i_admin_audit_logs (created_at DESC);

CREATE INDEX IF NOT EXISTS idx_f_i_admin_audit_logs_target
    ON f_i_admin_audit_logs (target_type, target_id, created_at DESC);
