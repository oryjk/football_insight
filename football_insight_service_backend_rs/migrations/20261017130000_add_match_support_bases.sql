-- Operator-set base counts added on top of real votes when showing a match's support totals.
CREATE TABLE f_i_match_support_bases (
    match_id BIGINT NOT NULL REFERENCES f_i_matches(match_id) ON DELETE CASCADE,
    team_id BIGINT NOT NULL REFERENCES f_i_teams(team_id) ON DELETE CASCADE,
    base_count BIGINT NOT NULL CHECK (base_count >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (match_id, team_id)
);
