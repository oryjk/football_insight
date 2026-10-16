-- Paywall impression/tap measurement. Observability only; retain 180 days (cleanup is outside this release).
-- Apply explicitly on peiqian only after operator approval; never during application startup.
CREATE TABLE f_i_paywall_events (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    surface TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('impression','tap')),
    membership_tier TEXT,
    detail JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_paywall_events_time ON f_i_paywall_events (occurred_at DESC, surface, action);
CREATE INDEX idx_paywall_events_user ON f_i_paywall_events (user_id, occurred_at DESC);
