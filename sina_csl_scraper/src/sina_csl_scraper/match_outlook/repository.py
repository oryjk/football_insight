"""Persistence only: writer predicates use the DB wall clock for the forecast lifecycle."""

from psycopg.types.json import Jsonb
from .model import rps

PUBLISH_SQL = """
INSERT INTO f_i_match_outlooks(match_id,home_win,draw,away_win,baseline_home,baseline_draw,baseline_away,tau,model_version,params,kickoff_at)
SELECT %s,%s,%s,%s,%s,%s,%s,%s,'elo-v2',%s,%s FROM f_i_matches m
WHERE m.match_id=%s AND m.source_active AND m.status='1' AND %s > clock_timestamp()
 AND CASE WHEN m.match_time ~ '^([01][0-9]|2[0-3]):[0-5][0-9](:[0-5][0-9])?$'
     THEN ((m.match_date+m.match_time::time) AT TIME ZONE 'Asia/Shanghai')=%s ELSE false END
FOR SHARE OF m
ON CONFLICT(match_id) DO UPDATE SET
 home_win=EXCLUDED.home_win,draw=EXCLUDED.draw,away_win=EXCLUDED.away_win,
 baseline_home=EXCLUDED.baseline_home,baseline_draw=EXCLUDED.baseline_draw,baseline_away=EXCLUDED.baseline_away,
 tau=EXCLUDED.tau,model_version=EXCLUDED.model_version,params=EXCLUDED.params,
 generated_at=clock_timestamp(),kickoff_at=EXCLUDED.kickoff_at
WHERE f_i_match_outlooks.kickoff_at > clock_timestamp()
  AND EXCLUDED.kickoff_at > clock_timestamp() AND f_i_match_outlooks.scored_at IS NULL
"""


def publish(connection, match_id, p, base, tau, params, kickoff) -> int:
    return connection.execute(
        PUBLISH_SQL,
        (match_id, *p, *base, tau, Jsonb(params), kickoff, match_id, kickoff, kickoff),
    ).rowcount


def score_finished(connection) -> int:
    scored = 0
    rows = connection.execute("""SELECT o.match_id,o.home_win,o.draw,o.away_win,
        o.baseline_home,o.baseline_draw,o.baseline_away,m.home_score,m.away_score
        FROM f_i_match_outlooks o JOIN f_i_matches m USING(match_id)
        WHERE o.scored_at IS NULL AND o.kickoff_at<=clock_timestamp()
        AND m.source_active AND m.status='3' FOR UPDATE OF o""").fetchall()
    for mid, *values in rows:
        home, away = values[-2:]
        if (
            not str(home).isascii()
            or not str(home).isdigit()
            or not str(away).isascii()
            or not str(away).isdigit()
        ):
            continue
        outcome = 0 if int(home) > int(away) else 1 if int(home) == int(away) else 2
        scored += connection.execute(
            """UPDATE f_i_match_outlooks SET outcome=%s,rps_model=%s,rps_baseline=%s,scored_at=clock_timestamp()
            WHERE match_id=%s AND scored_at IS NULL AND kickoff_at<=clock_timestamp()""",
            (outcome, rps(values[:3], outcome), rps(values[3:6], outcome), mid),
        ).rowcount
    return scored


def annotate_reschedules(connection) -> int:
    return connection.execute("""UPDATE f_i_match_outlooks o SET params=o.params||'{"schedule_changed":true}'::jsonb
      FROM f_i_matches m WHERE m.match_id=o.match_id AND o.kickoff_at<=clock_timestamp()
        AND o.scored_at IS NULL AND NOT COALESCE((o.params->>'schedule_changed')::boolean,false)
        AND CASE WHEN m.match_time ~ '^([01][0-9]|2[0-3]):[0-5][0-9](:[0-5][0-9])?$'
          THEN ((m.match_date + m.match_time::time) AT TIME ZONE 'Asia/Shanghai') <> o.kickoff_at
          ELSE false END""").rowcount
