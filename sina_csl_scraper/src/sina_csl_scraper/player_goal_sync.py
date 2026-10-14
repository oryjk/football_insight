"""Refresh the scorer ranking independently so provider outages cannot break core sync."""
from __future__ import annotations

import sys
from dataclasses import asdict

import psycopg

from .models import MatchResult
from .player_match_goals import CflPlayerLogClient, PlayerMatchGoals, map_player_logs


def validate_complete_player_logs(records: list[PlayerMatchGoals], *, expected_goals: str,
                                  existing_match_ids: set[int]) -> None:
    if not expected_goals.isascii() or not expected_goals.isdigit():
        raise ValueError("cumulative goals are not a nonnegative integer")
    if sum(record.goals for record in records) != int(expected_goals):
        raise ValueError("verified season goals differ from cumulative scorer total")
    if not existing_match_ids.issubset({record.match_id for record in records}):
        raise ValueError("official log lost previously verified completed appearances")


def replace_player_logs(connection, *, season: int, player_id: int,
                        records: list[PlayerMatchGoals]) -> None:
    if any(r.season != season or r.player_id != player_id for r in records):
        raise ValueError("cannot replace another player's season logs")
    with connection.transaction():
        with connection.cursor() as cursor:
            cursor.execute("DELETE FROM f_i_player_match_stats WHERE season=%s AND player_id=%s",
                           (season, player_id))
            cursor.executemany(
                """INSERT INTO f_i_player_match_stats
                   (player_id,match_id,season,goals,minutes_played,source_player_id,source_match_id)
                   VALUES (%(player_id)s,%(match_id)s,%(season)s,%(goals)s,%(minutes_played)s,
                           %(source_player_id)s,%(source_match_id)s)""",
                [asdict(r) for r in records],
            )


def refresh_scorer_goals(database_url: str, season: int,
                            client: CflPlayerLogClient | None = None) -> dict:
    client = client or CflPlayerLogClient()
    summary: dict = {"players_updated": 0, "appearances": 0, "failed_players": []}
    with psycopg.connect(database_url, autocommit=True) as connection:
        with connection.cursor() as cursor:
            cursor.execute("SELECT to_regclass('f_i_player_match_stats') IS NOT NULL")
            if not cursor.fetchone()[0]:
                raise ValueError("player match stats migration has not been applied")
            cursor.execute("""
                SELECT e.player_id,e.player_name,e.team_id,e.score_value
                FROM f_i_player_ranking_entries e
                WHERE e.snapshot_id = (
                    SELECT s.id FROM f_i_player_ranking_snapshots s
                    JOIN f_i_ranking_categories c ON c.id=s.category_id
                    WHERE s.season=%s AND s.snapshot_kind='live'
                      AND c.scope='player' AND c.slug='goals'
                    ORDER BY s.snapshot_at DESC,s.id DESC LIMIT 1)
                ORDER BY e.rank_no,e.player_id
            """, (season,))
            targets = cursor.fetchall()
            cursor.execute("""
                SELECT match_id,round_number,round_name,match_date::text,match_time,status,
                       home_team_id,home_team_name,home_score,away_team_id,away_team_name,away_score
                FROM f_i_matches WHERE season=%s AND source_active=TRUE
            """, (season,))
            matches = [MatchResult(row[0], season, *row[1:], "", "") for row in cursor.fetchall()]

        for player_id, name, team_id, expected_goals in targets:
            try:
                identity, rows = client.fetch_player_logs(name=name, team_id=team_id, season=season)
                # A successful but empty provider response should not erase known appearances.
                if not rows:
                    raise ValueError("official appearance log is empty")
                records = map_player_logs(player_id=player_id, official_player_id=identity,
                                          season=season, rows=rows, matches=matches)
                if not records:
                    raise ValueError("no verified completed appearances")
                with connection.cursor() as cursor:
                    cursor.execute("""SELECT s.match_id FROM f_i_player_match_stats s
                        JOIN f_i_matches m ON m.match_id=s.match_id AND m.season=s.season
                        WHERE s.season=%s AND s.player_id=%s
                          AND m.source_active=TRUE AND m.status='3'""", (season, player_id))
                    existing_ids = {row[0] for row in cursor.fetchall()}
                validate_complete_player_logs(records, expected_goals=expected_goals,
                                              existing_match_ids=existing_ids)
                replace_player_logs(connection, season=season, player_id=player_id, records=records)
                summary["players_updated"] += 1
                summary["appearances"] += len(records)
            except Exception as error:
                summary["failed_players"].append(player_id)
                # Provider/network errors can embed URLs; only safe validation text is logged.
                reason = str(error) if isinstance(error, ValueError) else type(error).__name__
                print(f"[player-goals] {name}: kept previous data ({reason})", file=sys.stderr)
    return summary
