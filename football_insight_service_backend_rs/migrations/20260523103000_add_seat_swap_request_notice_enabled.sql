ALTER TABLE f_i_seat_swap_requests
ADD COLUMN IF NOT EXISTS seat_swap_notice_enabled BOOLEAN NOT NULL DEFAULT FALSE;
