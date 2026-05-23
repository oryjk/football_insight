DROP INDEX IF EXISTS idx_f_i_seat_swap_confirmations_match_user;

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_seat_swap_confirmations_match_pair
    ON f_i_seat_swap_confirmations (match_id, request_id, target_request_id);

CREATE INDEX IF NOT EXISTS idx_f_i_seat_swap_confirmations_match_user
    ON f_i_seat_swap_confirmations (match_id, confirmed_by_user_id);
