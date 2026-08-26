CREATE TABLE IF NOT EXISTS f_i_user_match_id_unlocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    match_id BIGINT NOT NULL,
    order_no VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, match_id),
    UNIQUE (order_no)
);
