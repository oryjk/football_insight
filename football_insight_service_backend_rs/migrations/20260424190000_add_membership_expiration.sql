ALTER TABLE f_i_users
    ADD COLUMN IF NOT EXISTS membership_expires_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_f_i_users_membership_expires_at
    ON f_i_users (membership_expires_at);
