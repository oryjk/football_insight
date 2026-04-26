from __future__ import annotations

from datetime import datetime
from typing import Any
from uuid import uuid4

import psycopg
from psycopg.types.json import Jsonb

from .models import MatchResult, PlayerProfile, RankingDataset, StandingEntry, TeamProfile
from .sync import SyncPayload
from .team_insights import build_team_insights


class PostgresInsightSyncRepository:
    def __init__(self, database_url: str) -> None:
        self.status_connection = psycopg.connect(database_url, autocommit=True)
        self.data_connection = psycopg.connect(database_url, autocommit=False)
        self._match_tech_columns = self._detect_match_tech_columns()

    def close(self) -> None:
        self.data_connection.close()
        self.status_connection.close()

    def begin_sync(self) -> None:
        return None

    def commit_sync(self) -> None:
        self.data_connection.commit()

    def rollback_sync(self) -> None:
        self.data_connection.rollback()

    def start_scrape_run(self, *, source: str, season: int, started_at: datetime) -> str:
        run_id = str(uuid4())
        with self.status_connection.cursor() as cursor:
            cursor.execute(
                """
                INSERT INTO f_i_scrape_runs (id, source, season, status, started_at)
                VALUES (%s, %s, %s, 'running', %s)
                """,
                (run_id, source, season, started_at),
            )
        return run_id

    def complete_scrape_run(self, *, run_id: str, finished_at: datetime, remark: str | None = None) -> None:
        with self.status_connection.cursor() as cursor:
            cursor.execute(
                """
                UPDATE f_i_scrape_runs
                   SET status = 'completed',
                       finished_at = %s,
                       remark = %s
                 WHERE id = %s
                """,
                (finished_at, remark, run_id),
            )

    def fail_scrape_run(self, *, run_id: str, finished_at: datetime, remark: str) -> None:
        with self.status_connection.cursor() as cursor:
            cursor.execute(
                """
                UPDATE f_i_scrape_runs
                   SET status = 'failed',
                       finished_at = %s,
                       remark = %s
                 WHERE id = %s
                """,
                (finished_at, remark, run_id),
            )

    def upsert_teams(self, teams: list[TeamProfile]) -> int:
        if not teams:
            return 0

        with self.data_connection.cursor() as cursor:
            cursor.executemany(
                """
                INSERT INTO f_i_teams (
                    team_id,
                    team_name,
                    avatar_source_url,
                    avatar_object_name,
                    avatar_storage_url,
                    updated_at
                )
                VALUES (%s, %s, %s, %s, %s, NOW())
                ON CONFLICT (team_id) DO UPDATE
                    SET team_name = EXCLUDED.team_name,
                        avatar_source_url = COALESCE(EXCLUDED.avatar_source_url, f_i_teams.avatar_source_url),
                        avatar_object_name = COALESCE(EXCLUDED.avatar_object_name, f_i_teams.avatar_object_name),
                        avatar_storage_url = COALESCE(EXCLUDED.avatar_storage_url, f_i_teams.avatar_storage_url),
                        updated_at = NOW()
                """,
                [
                    (
                        team.team_id,
                        team.team_name,
                        team.avatar_source_url,
                        team.avatar_object_name,
                        team.avatar_storage_url,
                    )
                    for team in teams
                ],
            )
        return len(teams)

    def upsert_players(self, players: list[PlayerProfile]) -> int:
        if not players:
            return 0

        with self.data_connection.cursor() as cursor:
            cursor.executemany(
                """
                INSERT INTO f_i_players (
                    player_id,
                    player_name,
                    team_id,
                    team_name,
                    avatar_source_url,
                    avatar_object_name,
                    avatar_storage_url,
                    updated_at
                )
                VALUES (%s, %s, %s, %s, %s, %s, %s, NOW())
                ON CONFLICT (player_id) DO UPDATE
                    SET player_name = EXCLUDED.player_name,
                        team_id = EXCLUDED.team_id,
                        team_name = EXCLUDED.team_name,
                        avatar_source_url = COALESCE(EXCLUDED.avatar_source_url, f_i_players.avatar_source_url),
                        avatar_object_name = COALESCE(EXCLUDED.avatar_object_name, f_i_players.avatar_object_name),
                        avatar_storage_url = COALESCE(EXCLUDED.avatar_storage_url, f_i_players.avatar_storage_url),
                        updated_at = NOW()
                """,
                [
                    (
                        player.player_id,
                        player.player_name,
                        player.team_id,
                        player.team_name,
                        player.avatar_source_url,
                        player.avatar_object_name,
                        player.avatar_storage_url,
                    )
                    for player in players
                ],
            )
        return len(players)

    def upsert_matches(self, matches: list[MatchResult]) -> int:
        if not matches:
            return 0

        if self._match_tech_columns["all_available"]:
            sql = self._upsert_matches_sql_with_tech_stats()
        elif self._match_tech_columns["corner_columns_available"]:
            sql = self._upsert_matches_sql_with_corners()
        else:
            sql = self._upsert_matches_sql_legacy()
        rows = [
            self._match_record(
                match,
                include_corner_columns=self._match_tech_columns["corner_columns_available"],
                include_technical_stats=self._match_tech_columns["technical_stats_available"],
            )
            for match in matches
        ]

        with self.data_connection.cursor() as cursor:
            cursor.executemany(sql, rows)
        return len(matches)

    def _detect_match_tech_columns(self) -> dict[str, bool]:
        required_columns = {
            "leisu_match_id",
            "home_corners",
            "away_corners",
            "corner_source",
            "technical_stats",
        }
        with self.status_connection.cursor() as cursor:
            cursor.execute(
                """
                SELECT column_name
                  FROM information_schema.columns
                 WHERE table_schema = current_schema()
                   AND table_name = 'f_i_matches'
                   AND column_name = ANY(%s)
                """,
                (list(required_columns),),
            )
            existing_columns = {str(row[0]) for row in cursor.fetchall()}
        corner_columns = {
            "leisu_match_id",
            "home_corners",
            "away_corners",
            "corner_source",
        }
        return {
            "corner_columns_available": corner_columns.issubset(existing_columns),
            "technical_stats_available": "technical_stats" in existing_columns,
            "all_available": required_columns.issubset(existing_columns),
        }

    @staticmethod
    def _upsert_matches_sql_legacy() -> str:
        return """
            INSERT INTO f_i_matches (
                match_id,
                season,
                round_number,
                round_name,
                match_date,
                match_time,
                status,
                home_team_id,
                home_team_name,
                home_score,
                away_team_id,
                away_team_name,
                away_score,
                home_logo,
                away_logo,
                updated_at
            )
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, NOW())
            ON CONFLICT (match_id) DO UPDATE
                SET season = EXCLUDED.season,
                    round_number = EXCLUDED.round_number,
                    round_name = EXCLUDED.round_name,
                    match_date = EXCLUDED.match_date,
                    match_time = EXCLUDED.match_time,
                    status = EXCLUDED.status,
                    home_team_id = EXCLUDED.home_team_id,
                    home_team_name = EXCLUDED.home_team_name,
                    home_score = EXCLUDED.home_score,
                    away_team_id = EXCLUDED.away_team_id,
                    away_team_name = EXCLUDED.away_team_name,
                    away_score = EXCLUDED.away_score,
                    home_logo = EXCLUDED.home_logo,
                    away_logo = EXCLUDED.away_logo,
                    updated_at = NOW()
        """

    @staticmethod
    def _upsert_matches_sql_with_corners() -> str:
        return """
            INSERT INTO f_i_matches (
                match_id,
                season,
                round_number,
                round_name,
                match_date,
                match_time,
                status,
                home_team_id,
                home_team_name,
                home_score,
                away_team_id,
                away_team_name,
                away_score,
                home_logo,
                away_logo,
                leisu_match_id,
                home_corners,
                away_corners,
                corner_source,
                updated_at
            )
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, NOW())
            ON CONFLICT (match_id) DO UPDATE
                SET season = EXCLUDED.season,
                    round_number = EXCLUDED.round_number,
                    round_name = EXCLUDED.round_name,
                    match_date = EXCLUDED.match_date,
                    match_time = EXCLUDED.match_time,
                    status = EXCLUDED.status,
                    home_team_id = EXCLUDED.home_team_id,
                    home_team_name = EXCLUDED.home_team_name,
                    home_score = EXCLUDED.home_score,
                    away_team_id = EXCLUDED.away_team_id,
                    away_team_name = EXCLUDED.away_team_name,
                    away_score = EXCLUDED.away_score,
                    home_logo = EXCLUDED.home_logo,
                    away_logo = EXCLUDED.away_logo,
                    leisu_match_id = COALESCE(EXCLUDED.leisu_match_id, f_i_matches.leisu_match_id),
                    home_corners = COALESCE(EXCLUDED.home_corners, f_i_matches.home_corners),
                    away_corners = COALESCE(EXCLUDED.away_corners, f_i_matches.away_corners),
                    corner_source = COALESCE(EXCLUDED.corner_source, f_i_matches.corner_source),
                    updated_at = NOW()
        """

    @staticmethod
    def _upsert_matches_sql_with_tech_stats() -> str:
        return """
            INSERT INTO f_i_matches (
                match_id,
                season,
                round_number,
                round_name,
                match_date,
                match_time,
                status,
                home_team_id,
                home_team_name,
                home_score,
                away_team_id,
                away_team_name,
                away_score,
                home_logo,
                away_logo,
                leisu_match_id,
                home_corners,
                away_corners,
                corner_source,
                technical_stats,
                updated_at
            )
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, NOW())
            ON CONFLICT (match_id) DO UPDATE
                SET season = EXCLUDED.season,
                    round_number = EXCLUDED.round_number,
                    round_name = EXCLUDED.round_name,
                    match_date = EXCLUDED.match_date,
                    match_time = EXCLUDED.match_time,
                    status = EXCLUDED.status,
                    home_team_id = EXCLUDED.home_team_id,
                    home_team_name = EXCLUDED.home_team_name,
                    home_score = EXCLUDED.home_score,
                    away_team_id = EXCLUDED.away_team_id,
                    away_team_name = EXCLUDED.away_team_name,
                    away_score = EXCLUDED.away_score,
                    home_logo = EXCLUDED.home_logo,
                    away_logo = EXCLUDED.away_logo,
                    leisu_match_id = COALESCE(EXCLUDED.leisu_match_id, f_i_matches.leisu_match_id),
                    home_corners = COALESCE(EXCLUDED.home_corners, f_i_matches.home_corners),
                    away_corners = COALESCE(EXCLUDED.away_corners, f_i_matches.away_corners),
                    corner_source = COALESCE(EXCLUDED.corner_source, f_i_matches.corner_source),
                    technical_stats = COALESCE(EXCLUDED.technical_stats, f_i_matches.technical_stats),
                    updated_at = NOW()
        """

    @staticmethod
    def _match_record(
        match: MatchResult,
        *,
        include_corner_columns: bool,
        include_technical_stats: bool,
    ) -> tuple[Any, ...]:
        base_record: tuple[Any, ...] = (
            match.match_id,
            match.season,
            match.round_number,
            match.round_name,
            match.date,
            match.time,
            match.status,
            match.home_team_id,
            match.home_team_name,
            match.home_score,
            match.away_team_id,
            match.away_team_name,
            match.away_score,
            match.home_logo,
            match.away_logo,
        )
        if not include_corner_columns and not include_technical_stats:
            return base_record
        extended_record: tuple[Any, ...] = base_record
        if include_corner_columns:
            extended_record = (
                *extended_record,
                match.leisu_match_id,
                match.home_corners,
                match.away_corners,
                match.corner_source,
            )
        if include_technical_stats:
            extended_record = (
                *extended_record,
                Jsonb(match.technical_stats) if match.technical_stats is not None else None,
            )
        return extended_record

    def round_final_exists(self, *, season: int, round_number: int) -> bool:
        with self.data_connection.cursor() as cursor:
            cursor.execute(
                """
                SELECT EXISTS(
                    SELECT 1
                      FROM f_i_team_ranking_snapshots
                     WHERE season = %s
                       AND round_number = %s
                       AND snapshot_kind = 'round_final'
                )
                """,
                (season, round_number),
            )
            row = cursor.fetchone()
            return bool(row[0]) if row is not None else False

    def insert_standings(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: str,
        standings: list[StandingEntry],
        snapshot_at: datetime,
    ) -> int:
        if not standings:
            return 0

        with self.data_connection.cursor() as cursor:
            cursor.executemany(
                """
                INSERT INTO f_i_standings (
                    scrape_run_id,
                    season,
                    round_number,
                    snapshot_kind,
                    team_id,
                    team_name,
                    team_logo,
                    rank_no,
                    played,
                    wins,
                    draws,
                    losses,
                    goals_for,
                    goals_against,
                    goal_difference,
                    points,
                    snapshot_at
                )
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (season, round_number, snapshot_kind, team_id)
                    WHERE snapshot_kind = 'round_final'
                DO UPDATE
                    SET scrape_run_id = EXCLUDED.scrape_run_id,
                        team_name = EXCLUDED.team_name,
                        team_logo = EXCLUDED.team_logo,
                        rank_no = EXCLUDED.rank_no,
                        played = EXCLUDED.played,
                        wins = EXCLUDED.wins,
                        draws = EXCLUDED.draws,
                        losses = EXCLUDED.losses,
                        goals_for = EXCLUDED.goals_for,
                        goals_against = EXCLUDED.goals_against,
                        goal_difference = EXCLUDED.goal_difference,
                        points = EXCLUDED.points,
                        snapshot_at = EXCLUDED.snapshot_at
                """,
                [
                    (
                        run_id,
                        season,
                        round_number,
                        snapshot_kind,
                        standing.team_id,
                        standing.team_name,
                        standing.team_logo,
                        standing.order,
                        standing.played,
                        standing.wins,
                        standing.draws,
                        standing.losses,
                        standing.goals_for,
                        standing.goals_against,
                        standing.goal_difference,
                        standing.points,
                        snapshot_at,
                    )
                    for standing in standings
                ],
            )
        return len(standings)

    def sync_team_rankings(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: str,
        datasets: list[RankingDataset],
        snapshot_at: datetime,
    ) -> int:
        if not datasets:
            return 0

        with self.data_connection.cursor() as cursor:
            for dataset in datasets:
                category_id = self._ensure_category(
                    cursor=cursor,
                    scope="team",
                    item_id=dataset.item_id,
                    slug=dataset.slug,
                    label=dataset.label,
                )
                snapshot_id = self._insert_snapshot(
                    cursor=cursor,
                        table="f_i_team_ranking_snapshots",
                        run_id=run_id,
                        season=season,
                        round_number=round_number,
                        snapshot_kind=snapshot_kind,
                        category_id=category_id,
                        snapshot_at=snapshot_at,
                        entry_count=len(dataset.entries),
                )
                if dataset.entries:
                    cursor.executemany(
                        """
                        INSERT INTO f_i_team_ranking_entries (
                            snapshot_id,
                            team_id,
                            team_name,
                            team_logo,
                            rank_no,
                            score_value
                        )
                        VALUES (%s, %s, %s, %s, %s, %s)
                        ON CONFLICT (snapshot_id, team_id) DO UPDATE
                            SET team_name = EXCLUDED.team_name,
                                team_logo = EXCLUDED.team_logo,
                                rank_no = EXCLUDED.rank_no,
                                score_value = EXCLUDED.score_value
                        """,
                        [
                            (
                                snapshot_id,
                                int(entry["team_id"]),
                                str(entry["team_name"]),
                                str(entry["team_logo"]),
                                int(entry["rank"]),
                                str(entry["score"]),
                            )
                            for entry in dataset.entries
                        ],
                    )
        return len(datasets)

    def sync_player_rankings(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: str,
        datasets: list[RankingDataset],
        snapshot_at: datetime,
    ) -> int:
        if not datasets:
            return 0

        with self.data_connection.cursor() as cursor:
            for dataset in datasets:
                category_id = self._ensure_category(
                    cursor=cursor,
                    scope="player",
                    item_id=dataset.item_id,
                    slug=dataset.slug,
                    label=dataset.label,
                )
                snapshot_id = self._insert_snapshot(
                    cursor=cursor,
                        table="f_i_player_ranking_snapshots",
                        run_id=run_id,
                        season=season,
                        round_number=round_number,
                        snapshot_kind=snapshot_kind,
                        category_id=category_id,
                        snapshot_at=snapshot_at,
                        entry_count=len(dataset.entries),
                )
                if dataset.entries:
                    cursor.executemany(
                        """
                        INSERT INTO f_i_player_ranking_entries (
                            snapshot_id,
                            player_id,
                            player_name,
                            player_logo,
                            team_id,
                            team_name,
                            rank_no,
                            score_value,
                            penalty_value
                        )
                        VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)
                        ON CONFLICT (snapshot_id, player_id) DO UPDATE
                            SET player_name = EXCLUDED.player_name,
                                player_logo = EXCLUDED.player_logo,
                                team_id = EXCLUDED.team_id,
                                team_name = EXCLUDED.team_name,
                                rank_no = EXCLUDED.rank_no,
                                score_value = EXCLUDED.score_value,
                                penalty_value = EXCLUDED.penalty_value
                        """,
                        [
                            (
                                snapshot_id,
                                int(entry["player_id"]),
                                str(entry["player_name"]),
                                str(entry["player_logo"]),
                                int(entry["team_id"]),
                                str(entry["team_name"]),
                                int(entry["rank"]),
                                str(entry["score"]),
                                self._nullable_text(entry.get("penalty")),
                            )
                            for entry in dataset.entries
                        ],
                    )
        return len(datasets)

    def sync_team_insights(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: str,
        payload: SyncPayload,
        snapshot_at: datetime,
    ) -> int:
        team_insights = build_team_insights(
            season=season,
            round_number=round_number,
            snapshot_kind=snapshot_kind,
            teams=payload.teams,
            players=payload.players,
            matches=payload.matches,
            standings=payload.standings,
            team_rankings=payload.team_rankings,
            player_rankings=payload.player_rankings,
        )
        if not team_insights:
            return 0

        with self.data_connection.cursor() as cursor:
            if snapshot_kind == "round_final":
                cursor.executemany(
                    """
                    INSERT INTO f_i_team_insights (
                        scrape_run_id,
                        season,
                        round_number,
                        snapshot_kind,
                        team_id,
                        team_name,
                        rank_no,
                        avatar_storage_url,
                        goals_for_total,
                        goals_against_total,
                        goals_for_by_opponent,
                        goals_for_by_player,
                        assists_for_by_player,
                        goals_against_by_opponent,
                        snapshot_at
                    )
                    VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                    ON CONFLICT (season, round_number, snapshot_kind, team_id)
                        WHERE snapshot_kind = 'round_final'
                    DO UPDATE
                        SET scrape_run_id = EXCLUDED.scrape_run_id,
                            team_name = EXCLUDED.team_name,
                            rank_no = EXCLUDED.rank_no,
                            avatar_storage_url = EXCLUDED.avatar_storage_url,
                            goals_for_total = EXCLUDED.goals_for_total,
                            goals_against_total = EXCLUDED.goals_against_total,
                            goals_for_by_opponent = EXCLUDED.goals_for_by_opponent,
                            goals_for_by_player = EXCLUDED.goals_for_by_player,
                            assists_for_by_player = EXCLUDED.assists_for_by_player,
                            goals_against_by_opponent = EXCLUDED.goals_against_by_opponent,
                            snapshot_at = EXCLUDED.snapshot_at
                    """,
                    [self._team_insight_record(run_id, season, round_number, snapshot_kind, item, snapshot_at) for item in team_insights],
                )
            else:
                cursor.executemany(
                    """
                    INSERT INTO f_i_team_insights (
                        scrape_run_id,
                        season,
                        round_number,
                        snapshot_kind,
                        team_id,
                        team_name,
                        rank_no,
                        avatar_storage_url,
                        goals_for_total,
                        goals_against_total,
                        goals_for_by_opponent,
                        goals_for_by_player,
                        assists_for_by_player,
                        goals_against_by_opponent,
                        snapshot_at
                    )
                    VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                    ON CONFLICT (season, snapshot_kind, team_id)
                        WHERE snapshot_kind = 'live'
                    DO UPDATE
                        SET scrape_run_id = EXCLUDED.scrape_run_id,
                            round_number = EXCLUDED.round_number,
                            team_name = EXCLUDED.team_name,
                            rank_no = EXCLUDED.rank_no,
                            avatar_storage_url = EXCLUDED.avatar_storage_url,
                            goals_for_total = EXCLUDED.goals_for_total,
                            goals_against_total = EXCLUDED.goals_against_total,
                            goals_for_by_opponent = EXCLUDED.goals_for_by_opponent,
                            goals_for_by_player = EXCLUDED.goals_for_by_player,
                            assists_for_by_player = EXCLUDED.assists_for_by_player,
                            goals_against_by_opponent = EXCLUDED.goals_against_by_opponent,
                            snapshot_at = EXCLUDED.snapshot_at
                    """,
                    [self._team_insight_record(run_id, season, round_number, snapshot_kind, item, snapshot_at) for item in team_insights],
                )

        return len(team_insights)

    def _ensure_category(
        self,
        *,
        cursor: psycopg.Cursor[Any],
        scope: str,
        item_id: int,
        slug: str,
        label: str,
    ) -> int:
        cursor.execute(
            """
            INSERT INTO f_i_ranking_categories (scope, item_id, slug, label)
            VALUES (%s, %s, %s, %s)
            ON CONFLICT (scope, item_id) DO UPDATE
                SET slug = EXCLUDED.slug,
                    label = EXCLUDED.label
            RETURNING id
            """,
            (scope, item_id, slug, label),
        )
        category_id = cursor.fetchone()
        assert category_id is not None
        return int(category_id[0])

    def _insert_snapshot(
        self,
        *,
        cursor: psycopg.Cursor[Any],
        table: str,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: str,
        category_id: int,
        snapshot_at: datetime,
        entry_count: int,
    ) -> int:
        cursor.execute(
            f"""
            INSERT INTO {table} (
                scrape_run_id,
                season,
                round_number,
                snapshot_kind,
                category_id,
                snapshot_at,
                entry_count
            )
            VALUES (%s, %s, %s, %s, %s, %s, %s)
            ON CONFLICT (season, round_number, snapshot_kind, category_id)
                WHERE snapshot_kind = 'round_final'
            DO UPDATE
                SET scrape_run_id = EXCLUDED.scrape_run_id,
                    snapshot_at = EXCLUDED.snapshot_at,
                    entry_count = EXCLUDED.entry_count
            RETURNING id
            """,
            (run_id, season, round_number, snapshot_kind, category_id, snapshot_at, entry_count),
        )
        snapshot_id = cursor.fetchone()
        assert snapshot_id is not None
        return int(snapshot_id[0])

    @staticmethod
    def _team_insight_record(
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: str,
        insight: Any,
        snapshot_at: datetime,
    ) -> tuple[Any, ...]:
        return (
            run_id,
            season,
            round_number,
            snapshot_kind,
            insight.team_id,
            insight.team_name,
            insight.rank_no,
            insight.avatar_storage_url,
            insight.goals_for_total,
            insight.goals_against_total,
            Jsonb([item.__dict__ for item in insight.goals_for_by_opponent]),
            Jsonb([item.__dict__ for item in insight.goals_for_by_player]),
            Jsonb([item.__dict__ for item in insight.assists_for_by_player]),
            Jsonb([item.__dict__ for item in insight.goals_against_by_opponent]),
            snapshot_at,
        )

    @staticmethod
    def _nullable_text(value: Any) -> str | None:
        if value in (None, ""):
            return None
        return str(value)
