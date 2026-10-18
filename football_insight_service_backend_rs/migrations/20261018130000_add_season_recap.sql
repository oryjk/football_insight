-- Additive only; run manually on peiqian after review. No production data backfill.
CREATE TABLE f_i_season_recap_profiles (
 user_id UUID PRIMARY KEY REFERENCES f_i_users(id) ON DELETE CASCADE,
 home_one_way_km DOUBLE PRECISION NULL CHECK (home_one_way_km >= 0 AND home_one_way_km <= 500)
);
CREATE TABLE f_i_season_recap_attendances (
 user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
 match_id BIGINT NOT NULL REFERENCES f_i_matches(match_id) ON DELETE CASCADE,
 created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
 PRIMARY KEY (user_id,match_id)
);
CREATE INDEX idx_season_recap_attendances_match ON f_i_season_recap_attendances(match_id);
