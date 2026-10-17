-- Accounts that see the full mini program even while the current version is under review.
CREATE TABLE f_i_review_exempt_users (
    user_id UUID PRIMARY KEY REFERENCES f_i_users(id) ON DELETE CASCADE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
