-- Additive only. Apply manually after review; no historical forecast backfill.
CREATE TABLE f_i_historical_matches (
 match_id BIGINT PRIMARY KEY,
 season INTEGER NOT NULL CHECK(season >= 2014),
 match_date DATE NOT NULL,
 match_time TIME NOT NULL,
 home_team_id BIGINT NOT NULL CHECK(home_team_id > 0),
 away_team_id BIGINT NOT NULL CHECK(away_team_id > 0 AND away_team_id <> home_team_id),
 home_score INTEGER NOT NULL CHECK(home_score >= 0),
 away_score INTEGER NOT NULL CHECK(away_score >= 0),
 neutral BOOLEAN NOT NULL CHECK(neutral = (season IN (2020,2021)))
);
CREATE INDEX idx_historical_matches_season_time ON f_i_historical_matches(season,match_date,match_time);
CREATE TABLE f_i_match_outlooks (
 match_id BIGINT PRIMARY KEY REFERENCES f_i_matches(match_id) ON DELETE CASCADE,
 home_win DOUBLE PRECISION NOT NULL CHECK(home_win BETWEEN 0 AND 1),
 draw DOUBLE PRECISION NOT NULL CHECK(draw BETWEEN 0 AND 1),
 away_win DOUBLE PRECISION NOT NULL CHECK(away_win BETWEEN 0 AND 1),
 baseline_home DOUBLE PRECISION NOT NULL CHECK(baseline_home BETWEEN 0 AND 1),
 baseline_draw DOUBLE PRECISION NOT NULL CHECK(baseline_draw BETWEEN 0 AND 1),
 baseline_away DOUBLE PRECISION NOT NULL CHECK(baseline_away BETWEEN 0 AND 1),
 tau DOUBLE PRECISION NOT NULL CHECK(tau BETWEEN .3 AND 1.2),
 model_version TEXT NOT NULL,
 params JSONB NOT NULL,
 generated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
 kickoff_at TIMESTAMPTZ NOT NULL,
 outcome SMALLINT NULL CHECK(outcome BETWEEN 0 AND 2),
 rps_model DOUBLE PRECISION NULL CHECK(rps_model BETWEEN 0 AND 1),
 rps_baseline DOUBLE PRECISION NULL CHECK(rps_baseline BETWEEN 0 AND 1),
 scored_at TIMESTAMPTZ NULL,
 CHECK(abs(home_win+draw+away_win-1)<0.00000001),
 CHECK(abs(baseline_home+baseline_draw+baseline_away-1)<0.00000001),
 CHECK(generated_at < kickoff_at),
 CHECK((outcome IS NULL AND rps_model IS NULL AND rps_baseline IS NULL AND scored_at IS NULL)
    OR (outcome IS NOT NULL AND rps_model IS NOT NULL AND rps_baseline IS NOT NULL AND scored_at IS NOT NULL))
);
CREATE INDEX idx_match_outlooks_scored ON f_i_match_outlooks(scored_at) WHERE scored_at IS NOT NULL;
-- Guard even an unconditional direct UPDATE, in addition to writer predicates.
CREATE FUNCTION f_i_guard_frozen_match_outlook() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF OLD.scored_at IS NOT NULL THEN RETURN NULL; END IF;
 IF NEW.scored_at IS NOT NULL AND OLD.kickoff_at > clock_timestamp() THEN RETURN NULL; END IF;
 IF OLD.kickoff_at <= clock_timestamp() THEN
  IF ROW(NEW.home_win,NEW.draw,NEW.away_win,NEW.baseline_home,NEW.baseline_draw,NEW.baseline_away,
         NEW.tau,NEW.model_version,NEW.generated_at,NEW.kickoff_at)
    IS DISTINCT FROM ROW(OLD.home_win,OLD.draw,OLD.away_win,OLD.baseline_home,OLD.baseline_draw,OLD.baseline_away,
         OLD.tau,OLD.model_version,OLD.generated_at,OLD.kickoff_at) THEN RETURN NULL; END IF;
  IF NEW.params IS DISTINCT FROM OLD.params AND NOT
     ((NEW.params - 'schedule_changed' = OLD.params - 'schedule_changed'
      AND NEW.params->'schedule_changed' = 'true'::jsonb) IS TRUE) THEN RETURN NULL; END IF;
 END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER guard_frozen_match_outlook BEFORE UPDATE ON f_i_match_outlooks
 FOR EACH ROW EXECUTE FUNCTION f_i_guard_frozen_match_outlook();
