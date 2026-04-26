from __future__ import annotations

from sina_csl_scraper.models import MatchResult, PlayerProfile, TeamProfile
from sina_csl_scraper.postgres_repository import PostgresInsightSyncRepository


class FakeCursor:
    def __init__(self) -> None:
        self.executemany_calls: list[tuple[str, list[tuple[object, ...]]]] = []

    def executemany(self, sql: str, params: list[tuple[object, ...]]) -> None:
        self.executemany_calls.append((sql, params))

    def __enter__(self) -> FakeCursor:
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        return None


class FakeConnection:
    def __init__(self) -> None:
        self.cursor_instance = FakeCursor()

    def cursor(self) -> FakeCursor:
        return self.cursor_instance


def build_repository() -> tuple[PostgresInsightSyncRepository, FakeConnection]:
    repository = PostgresInsightSyncRepository.__new__(PostgresInsightSyncRepository)
    connection = FakeConnection()
    repository.data_connection = connection
    repository.status_connection = connection
    repository._match_tech_columns = {
        "home_corners": True,
        "away_corners": True,
        "corner_source": True,
        "corner_columns_available": True,
        "technical_stats_available": True,
        "all_available": True,
    }
    return repository, connection


def test_upsert_teams_preserves_existing_avatar_fields_when_new_values_are_null() -> None:
    repository, connection = build_repository()

    repository.upsert_teams([
        TeamProfile(team_id=1, team_name="成都蓉城"),
    ])

    sql, params = connection.cursor_instance.executemany_calls[0]

    assert "avatar_source_url = COALESCE(EXCLUDED.avatar_source_url, f_i_teams.avatar_source_url)" in sql
    assert "avatar_object_name = COALESCE(EXCLUDED.avatar_object_name, f_i_teams.avatar_object_name)" in sql
    assert "avatar_storage_url = COALESCE(EXCLUDED.avatar_storage_url, f_i_teams.avatar_storage_url)" in sql
    assert params == [(1, "成都蓉城", None, None, None)]


def test_upsert_players_preserves_existing_avatar_fields_when_new_values_are_null() -> None:
    repository, connection = build_repository()

    repository.upsert_players([
        PlayerProfile(player_id=10, player_name="费利佩", team_id=1, team_name="成都蓉城"),
    ])

    sql, params = connection.cursor_instance.executemany_calls[0]

    assert "avatar_source_url = COALESCE(EXCLUDED.avatar_source_url, f_i_players.avatar_source_url)" in sql
    assert "avatar_object_name = COALESCE(EXCLUDED.avatar_object_name, f_i_players.avatar_object_name)" in sql
    assert "avatar_storage_url = COALESCE(EXCLUDED.avatar_storage_url, f_i_players.avatar_storage_url)" in sql
    assert params == [(10, "费利佩", 1, "成都蓉城", None, None, None)]


def test_upsert_matches_preserves_existing_leisu_fields_when_new_values_are_null() -> None:
    repository, connection = build_repository()

    repository.upsert_matches([
        MatchResult(
            match_id=288620,
            season=2026,
            round_number=7,
            round_name="第7轮",
            date="2026-04-21",
            time="19:00",
            status="3",
            home_team_id=1,
            home_team_name="深圳新鵬城",
            home_score="0",
            away_team_id=2,
            away_team_name="北京国安",
            away_score="1",
            home_logo="home.png",
            away_logo="away.png",
            leisu_match_id=None,
            home_corners=None,
            away_corners=None,
            corner_source=None,
            technical_stats=None,
        ),
    ])

    sql, params = connection.cursor_instance.executemany_calls[0]

    assert "leisu_match_id = COALESCE(EXCLUDED.leisu_match_id, f_i_matches.leisu_match_id)" in sql
    assert "home_corners = COALESCE(EXCLUDED.home_corners, f_i_matches.home_corners)" in sql
    assert "away_corners = COALESCE(EXCLUDED.away_corners, f_i_matches.away_corners)" in sql
    assert "corner_source = COALESCE(EXCLUDED.corner_source, f_i_matches.corner_source)" in sql
    assert "technical_stats = COALESCE(EXCLUDED.technical_stats, f_i_matches.technical_stats)" in sql
    assert params[0][0] == 288620
    assert params[0][-1] is None
