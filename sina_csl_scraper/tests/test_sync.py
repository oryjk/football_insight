from __future__ import annotations

from datetime import UTC, datetime

import pytest

from sina_csl_scraper.models import MatchResult, PlayerProfile, RankingDataset, StandingEntry, TeamProfile
from sina_csl_scraper.sync import (
    InsightSyncService,
    SnapshotKind,
    SyncPayload,
    find_latest_completed_round,
    normalize_live_round_number,
)


class FakeRepository:
    def __init__(self, fail_on: str | None = None, finalized_rounds: set[tuple[int, int]] | None = None) -> None:
        self.fail_on = fail_on
        self.calls: list[tuple[str, object]] = []
        self.finalized_rounds = finalized_rounds or set()

    def begin_sync(self) -> None:
        self.calls.append(("begin_sync", None))

    def commit_sync(self) -> None:
        self.calls.append(("commit_sync", None))

    def rollback_sync(self) -> None:
        self.calls.append(("rollback_sync", None))

    def start_scrape_run(self, *, source: str, season: int, started_at: datetime) -> str:
        self.calls.append(("start_scrape_run", {"source": source, "season": season, "started_at": started_at}))
        return "run-1"

    def upsert_teams(self, teams: list[TeamProfile]) -> int:
        self.calls.append(("upsert_teams", teams))
        return self._maybe_fail("upsert_teams", len(teams))

    def upsert_players(self, players: list[PlayerProfile]) -> int:
        self.calls.append(("upsert_players", players))
        return self._maybe_fail("upsert_players", len(players))

    def upsert_matches(self, matches: list[MatchResult]) -> int:
        self.calls.append(("upsert_matches", matches))
        return self._maybe_fail("upsert_matches", len(matches))

    def round_final_exists(self, *, season: int, round_number: int) -> bool:
        self.calls.append(("round_final_exists", {"season": season, "round_number": round_number}))
        return (season, round_number) in self.finalized_rounds

    def insert_standings(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: SnapshotKind,
        standings: list[StandingEntry],
        snapshot_at: datetime,
    ) -> int:
        self.calls.append(
            (
                "insert_standings",
                {
                    "run_id": run_id,
                    "season": season,
                    "round_number": round_number,
                    "snapshot_kind": snapshot_kind,
                    "standings": standings,
                    "snapshot_at": snapshot_at,
                },
            )
        )
        return self._maybe_fail("insert_standings", len(standings))

    def sync_team_rankings(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: SnapshotKind,
        datasets: list[RankingDataset],
        snapshot_at: datetime,
    ) -> int:
        self.calls.append(
            (
                "sync_team_rankings",
                {
                    "run_id": run_id,
                    "season": season,
                    "round_number": round_number,
                    "snapshot_kind": snapshot_kind,
                    "datasets": datasets,
                    "snapshot_at": snapshot_at,
                },
            )
        )
        return self._maybe_fail("sync_team_rankings", len(datasets))

    def sync_player_rankings(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: SnapshotKind,
        datasets: list[RankingDataset],
        snapshot_at: datetime,
    ) -> int:
        self.calls.append(
            (
                "sync_player_rankings",
                {
                    "run_id": run_id,
                    "season": season,
                    "round_number": round_number,
                    "snapshot_kind": snapshot_kind,
                    "datasets": datasets,
                    "snapshot_at": snapshot_at,
                },
            )
        )
        return self._maybe_fail("sync_player_rankings", len(datasets))

    def sync_team_insights(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: SnapshotKind,
        payload: SyncPayload,
        snapshot_at: datetime,
    ) -> int:
        self.calls.append(
            (
                "sync_team_insights",
                {
                    "run_id": run_id,
                    "season": season,
                    "round_number": round_number,
                    "snapshot_kind": snapshot_kind,
                    "payload": payload,
                    "snapshot_at": snapshot_at,
                },
            )
        )
        return self._maybe_fail("sync_team_insights", len(payload.standings))

    def complete_scrape_run(self, *, run_id: str, finished_at: datetime, remark: str | None = None) -> None:
        self.calls.append(("complete_scrape_run", {"run_id": run_id, "finished_at": finished_at, "remark": remark}))

    def fail_scrape_run(self, *, run_id: str, finished_at: datetime, remark: str) -> None:
        self.calls.append(("fail_scrape_run", {"run_id": run_id, "finished_at": finished_at, "remark": remark}))

    def _maybe_fail(self, action: str, count: int) -> int:
        if self.fail_on == action:
            raise RuntimeError(f"{action} failed")
        return count


def _sample_payload() -> SyncPayload:
    return SyncPayload(
        season=2026,
        current_round=1,
        teams=[
            TeamProfile(team_id=136, team_name="北京国安", avatar_object_name="summary/teams/136.png"),
        ],
        players=[
            PlayerProfile(player_id=81795, player_name="王刚", team_id=136, team_name="北京国安"),
        ],
        matches=[
            MatchResult(
                match_id=1,
                season=2026,
                round_number=1,
                round_name="第1轮",
                date="2026-03-01",
                time="19:35",
                status="3",
                home_team_id=136,
                home_team_name="北京国安",
                home_score="2",
                away_team_id=500,
                away_team_name="云南玉昆",
                away_score="0",
                home_logo="home.png",
                away_logo="away.png",
            ),
            MatchResult(
                match_id=2,
                season=2026,
                round_number=2,
                round_name="第2轮",
                date="2026-03-08",
                time="19:35",
                status="1",
                home_team_id=136,
                home_team_name="北京国安",
                home_score="0",
                away_team_id=500,
                away_team_name="云南玉昆",
                away_score="0",
                home_logo="home.png",
                away_logo="away.png",
            ),
        ],
        standings=[
            StandingEntry(
                order=1,
                team_id=136,
                team_name="北京国安",
                team_logo="home.png",
                played=1,
                wins=1,
                draws=0,
                losses=0,
                goals_for=2,
                goals_against=0,
                goal_difference=2,
                points=3,
            ),
        ],
        team_rankings=[
            RankingDataset(
                slug="goals",
                label="进球",
                item_id=1,
                entries=[
                    {
                        "rank": 1,
                        "score": "2",
                        "team_id": 136,
                        "team_name": "北京国安",
                        "team_logo": "home.png",
                    }
                ],
            ),
        ],
        player_rankings=[
            RankingDataset(
                slug="goals",
                label="进球",
                item_id=2,
                entries=[
                    {
                        "rank": 1,
                        "score": "2",
                        "player_id": 81795,
                        "player_name": "王刚",
                        "player_logo": "player.png",
                        "team_id": 136,
                        "team_name": "北京国安",
                        "penalty": None,
                    }
                ],
            ),
        ],
    )


def _sample_payload_with_partial_next_round() -> SyncPayload:
    return SyncPayload(
        season=2026,
        current_round=2,
        teams=[
            TeamProfile(team_id=136, team_name="北京国安", avatar_object_name="summary/teams/136.png"),
        ],
        players=[
            PlayerProfile(player_id=81795, player_name="王刚", team_id=136, team_name="北京国安"),
        ],
        matches=[
            MatchResult(
                match_id=1,
                season=2026,
                round_number=1,
                round_name="第1轮",
                date="2026-03-01",
                time="19:35",
                status="3",
                home_team_id=136,
                home_team_name="北京国安",
                home_score="2",
                away_team_id=500,
                away_team_name="云南玉昆",
                away_score="0",
                home_logo="home.png",
                away_logo="away.png",
            ),
            MatchResult(
                match_id=2,
                season=2026,
                round_number=2,
                round_name="第2轮",
                date="2026-03-08",
                time="19:35",
                status="3",
                home_team_id=136,
                home_team_name="北京国安",
                home_score="1",
                away_team_id=500,
                away_team_name="云南玉昆",
                away_score="0",
                home_logo="home.png",
                away_logo="away.png",
            ),
            MatchResult(
                match_id=3,
                season=2026,
                round_number=2,
                round_name="第2轮",
                date="2026-03-09",
                time="19:35",
                status="1",
                home_team_id=500,
                home_team_name="云南玉昆",
                home_score="",
                away_team_id=136,
                away_team_name="北京国安",
                away_score="",
                home_logo="home.png",
                away_logo="away.png",
            ),
        ],
        standings=[
            StandingEntry(
                order=1,
                team_id=136,
                team_name="北京国安",
                team_logo="home.png",
                played=2,
                wins=2,
                draws=0,
                losses=0,
                goals_for=3,
                goals_against=0,
                goal_difference=3,
                points=6,
            ),
        ],
        team_rankings=[
            RankingDataset(
                slug="goals",
                label="进球",
                item_id=1,
                entries=[
                    {
                        "rank": 1,
                        "score": "3",
                        "team_id": 136,
                        "team_name": "北京国安",
                        "team_logo": "home.png",
                    }
                ],
            ),
        ],
        player_rankings=[
            RankingDataset(
                slug="goals",
                label="进球",
                item_id=2,
                entries=[
                    {
                        "rank": 1,
                        "score": "3",
                        "player_id": 81795,
                        "player_name": "王刚",
                        "player_logo": "player.png",
                        "team_id": 136,
                        "team_name": "北京国安",
                        "penalty": None,
                    }
                ],
            ),
        ],
    )


def test_find_latest_completed_round_returns_last_fully_finished_round() -> None:
    round_number = find_latest_completed_round(_sample_payload().matches)

    assert round_number == 1


def test_normalize_live_round_number_returns_none_for_non_positive_value() -> None:
    assert normalize_live_round_number(0) is None
    assert normalize_live_round_number(-1) is None
    assert normalize_live_round_number(3) == 3


def test_sync_service_persists_payload_and_completes_run() -> None:
    repository = FakeRepository()
    service = InsightSyncService(
        repository=repository,
        clock=lambda: datetime(2026, 4, 5, 8, 0, tzinfo=UTC),
    )

    result = service.sync(_sample_payload())

    assert result.run_id == "run-1"
    assert result.teams_upserted == 1
    assert result.players_upserted == 1
    assert result.matches_upserted == 2
    assert result.live_standings_inserted == 1
    assert result.live_team_ranking_categories == 1
    assert result.live_player_ranking_categories == 1
    assert result.round_finalized_for == 1
    assert result.round_final_standings_inserted == 1
    assert result.round_final_team_ranking_categories == 1
    assert result.round_final_player_ranking_categories == 1
    assert [name for name, _ in repository.calls] == [
        "start_scrape_run",
        "begin_sync",
        "upsert_teams",
        "upsert_players",
        "upsert_matches",
        "insert_standings",
        "sync_team_rankings",
        "sync_player_rankings",
        "sync_team_insights",
        "round_final_exists",
        "insert_standings",
        "sync_team_rankings",
        "sync_player_rankings",
        "sync_team_insights",
        "commit_sync",
        "complete_scrape_run",
    ]


def test_sync_service_skips_round_final_when_round_already_finalized() -> None:
    repository = FakeRepository(finalized_rounds={(2026, 1)})
    service = InsightSyncService(
        repository=repository,
        clock=lambda: datetime(2026, 4, 5, 8, 0, tzinfo=UTC),
    )

    result = service.sync(_sample_payload())

    assert result.round_finalized_for is None
    assert [name for name, _ in repository.calls] == [
        "start_scrape_run",
        "begin_sync",
        "upsert_teams",
        "upsert_players",
        "upsert_matches",
        "insert_standings",
        "sync_team_rankings",
        "sync_player_rankings",
        "sync_team_insights",
        "round_final_exists",
        "commit_sync",
        "complete_scrape_run",
    ]


def test_sync_service_skips_round_final_when_next_round_has_already_started() -> None:
    repository = FakeRepository()
    service = InsightSyncService(
        repository=repository,
        clock=lambda: datetime(2026, 4, 5, 8, 0, tzinfo=UTC),
    )

    result = service.sync(_sample_payload_with_partial_next_round())

    assert result.round_finalized_for is None
    assert [name for name, _ in repository.calls] == [
        "start_scrape_run",
        "begin_sync",
        "upsert_teams",
        "upsert_players",
        "upsert_matches",
        "insert_standings",
        "sync_team_rankings",
        "sync_player_rankings",
        "sync_team_insights",
        "commit_sync",
        "complete_scrape_run",
    ]


def test_sync_service_marks_run_failed_when_live_sync_errors() -> None:
    repository = FakeRepository(fail_on="upsert_matches")
    service = InsightSyncService(
        repository=repository,
        clock=lambda: datetime(2026, 4, 5, 8, 0, tzinfo=UTC),
    )

    with pytest.raises(RuntimeError, match="upsert_matches failed"):
        service.sync(_sample_payload())

    assert [name for name, _ in repository.calls] == [
        "start_scrape_run",
        "begin_sync",
        "upsert_teams",
        "upsert_players",
        "upsert_matches",
        "rollback_sync",
        "fail_scrape_run",
    ]
