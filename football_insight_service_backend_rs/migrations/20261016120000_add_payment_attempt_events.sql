-- Observability only. Retain 180 days; cleanup automation is outside this release.
-- Apply explicitly on peiqian only after operator approval; never during application startup.
CREATE TABLE f_i_payment_attempt_events (
    id BIGSERIAL PRIMARY KEY,
    attempt_id UUID NOT NULL,
    order_no TEXT,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    scenario TEXT NOT NULL CHECK (scenario IN ('membership','match_id_unlock','reflux_subscription')),
    product_ref TEXT,
    event TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('client','server')),
    error_code TEXT,
    error_msg TEXT,
    detail JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_payment_events_order ON f_i_payment_attempt_events (order_no, occurred_at);
CREATE INDEX idx_payment_events_attempt ON f_i_payment_attempt_events (attempt_id, occurred_at);
CREATE INDEX idx_payment_events_time ON f_i_payment_attempt_events (occurred_at DESC, event);
