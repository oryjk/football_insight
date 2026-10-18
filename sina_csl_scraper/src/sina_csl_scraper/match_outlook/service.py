"""Independent post-sync unit of work. No write can reach the core sync connection."""

from dataclasses import asdict
from datetime import datetime
import logging
import psycopg
from .history import game_from_dict, validate_history
from .model import Game, Replay, tune, baseline
from .repository import publish, score_finished, annotate_reschedules

HISTORY_SQL = """SELECT match_id,season,match_date::text,match_time::text,home_team_id,away_team_id,home_score::text,away_score::text
 FROM f_i_historical_matches h WHERE NOT EXISTS(SELECT 1 FROM f_i_matches m WHERE m.match_id=h.match_id)
 UNION ALL SELECT match_id,season,match_date::text,match_time,home_team_id,away_team_id,home_score,away_score
 FROM f_i_matches WHERE source_active AND status='3'"""


def parse_row(row):
    keys = (
        "match_id",
        "season",
        "date",
        "time",
        "home_team_id",
        "away_team_id",
        "home_score",
        "away_score",
    )
    return game_from_dict(dict(zip(keys, row)))


def build_predictions(history: list[Game], fixtures: list[Game], now: datetime):
    # All forecasts are for the future; never manufacture a retrospective public record.
    history = sorted(
        (g for g in history if g.kickoff < now), key=lambda g: (g.kickoff, g.match_id)
    )
    targets = sorted({g.season for g in fixtures if g.kickoff > now})
    for target in targets:
        prior = [g for g in history if g.season < target]
        validate_history(prior, list(range(2014, target)))
    if not targets:
        return []
    chosen = tune(history, targets)
    result = []
    for target in targets:
        rows = [g for g in history if g.season <= target]
        model = Replay(chosen[target])
        model.run(rows)
        base = baseline(rows)
        for game in sorted(
            (g for g in fixtures if g.season == target and g.kickoff > now),
            key=lambda g: (g.kickoff, g.match_id),
        ):
            # No fictional future results are fed back into ratings or tau history.
            result.append((game, model.predict(game), base, asdict(chosen[target])))
    return result


def refresh_match_outlooks(database_url: str, season: int) -> dict:
    with psycopg.connect(database_url, autocommit=True) as connection:
        # Snapshot reads finish before CPU work and publication; no core sync transaction held.
        with connection.transaction():
            connection.execute(
                "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY"
            )
            now = connection.execute("SELECT clock_timestamp()").fetchone()[0]
            history = [
                parse_row(row) for row in connection.execute(HISTORY_SQL).fetchall()
            ]
            upcoming = []
            for row in connection.execute(
                """SELECT match_id,season,match_date::text,match_time,home_team_id,away_team_id,'0','0'
                FROM f_i_matches WHERE source_active AND status='1' AND season >= %s""",
                (season,),
            ).fetchall():
                try:
                    upcoming.append(parse_row(row))
                except ValueError:
                    logging.getLogger(__name__).warning(
                        "match outlook skipped unknown kickoff"
                    )
        # Scoring is independent of generating new outlooks or availability of historical backfill.
        with connection.transaction():
            connection.execute("SET LOCAL lock_timeout='2s'")
            scored = score_finished(connection)
            annotated = annotate_reschedules(connection)
        predictions = build_predictions(history, upcoming, now)
        published = 0
        with connection.transaction():
            connection.execute("SET LOCAL lock_timeout='2s'")
            for game, pred, base, params in predictions:
                published += publish(
                    connection,
                    game.match_id,
                    pred.probabilities,
                    base,
                    pred.tau,
                    params,
                    game.kickoff,
                )
        return {
            "published": published,
            "scored": scored,
            "reschedules_marked": annotated,
        }
