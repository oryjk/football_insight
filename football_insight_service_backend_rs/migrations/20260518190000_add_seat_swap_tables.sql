CREATE TABLE IF NOT EXISTS f_i_seat_swap_requests (
    id UUID PRIMARY KEY,
    match_id BIGINT NOT NULL,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    current_region_key TEXT NOT NULL,
    current_region_name TEXT NOT NULL,
    current_row TEXT NOT NULL,
    current_seat_no TEXT NOT NULL,
    wechat_id TEXT,
    phone_number TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    matched_request_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_f_i_seat_swap_requests_status
        CHECK (status IN ('active', 'matched', 'cancelled')),
    CONSTRAINT chk_f_i_seat_swap_requests_contact
        CHECK (
            NULLIF(BTRIM(COALESCE(wechat_id, '')), '') IS NOT NULL
            OR NULLIF(BTRIM(COALESCE(phone_number, '')), '') IS NOT NULL
        )
);

CREATE TABLE IF NOT EXISTS f_i_seat_swap_desired_seats (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES f_i_seat_swap_requests(id) ON DELETE CASCADE,
    region_key TEXT NOT NULL,
    region_name TEXT NOT NULL,
    desired_row TEXT,
    desired_seat_no TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS f_i_seat_swap_confirmations (
    id UUID PRIMARY KEY,
    match_id BIGINT NOT NULL,
    request_id UUID NOT NULL REFERENCES f_i_seat_swap_requests(id) ON DELETE CASCADE,
    target_request_id UUID NOT NULL REFERENCES f_i_seat_swap_requests(id) ON DELETE CASCADE,
    confirmed_by_user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_f_i_seat_swap_confirmations_no_self
        CHECK (request_id <> target_request_id)
);

CREATE TABLE IF NOT EXISTS f_i_seat_swap_cancellations (
    id UUID PRIMARY KEY,
    match_id BIGINT NOT NULL,
    request_id UUID NOT NULL REFERENCES f_i_seat_swap_requests(id) ON DELETE CASCADE,
    target_request_id UUID NOT NULL REFERENCES f_i_seat_swap_requests(id) ON DELETE CASCADE,
    cancelled_by_user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    reason TEXT NOT NULL,
    evidence_object_key TEXT NOT NULL,
    evidence_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_seat_swap_requests_match_user_open
    ON f_i_seat_swap_requests (match_id, user_id)
    WHERE status IN ('active', 'matched');

CREATE INDEX IF NOT EXISTS idx_f_i_seat_swap_requests_match_status
    ON f_i_seat_swap_requests (match_id, status, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_f_i_seat_swap_desired_request
    ON f_i_seat_swap_desired_seats (request_id, sort_order ASC);

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_seat_swap_confirmations_match_user
    ON f_i_seat_swap_confirmations (match_id, confirmed_by_user_id);

CREATE INDEX IF NOT EXISTS idx_f_i_seat_swap_confirmations_target
    ON f_i_seat_swap_confirmations (target_request_id);
