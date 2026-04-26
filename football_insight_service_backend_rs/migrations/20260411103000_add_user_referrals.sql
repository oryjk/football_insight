CREATE TABLE IF NOT EXISTS f_i_user_referrals (
    id UUID PRIMARY KEY,
    referrer_user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    referred_user_id UUID NOT NULL UNIQUE REFERENCES f_i_users(id) ON DELETE CASCADE,
    referral_invite_code VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_f_i_user_referrals_not_self_referral
        CHECK (referrer_user_id <> referred_user_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_user_referrals_referrer_user_id
    ON f_i_user_referrals (referrer_user_id);
