from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Callable, Literal, Protocol

from .models import MatchResult, PlayerProfile, RankingDataset, StandingEntry, TeamProfile

SnapshotKind = Literal["live", "round_final"]


@dataclass(frozen=True)
class SyncPayload:
    season: int
    current_round: int
    teams: list[TeamProfile]
    players: list[PlayerProfile]
    matches: list[MatchResult]
    standings: list[StandingEntry]
    team_rankings: list[RankingDataset]
    player_rankings: list[RankingDataset]


@dataclass(frozen=True)
class SyncResult:
    run_id: str
    season: int
    teams_upserted: int
    players_upserted: int
    matches_upserted: int
    live_standings_inserted: int
    live_team_ranking_categories: int
    live_player_ranking_categories: int
    round_finalized_for: int | None
    round_final_standings_inserted: int
    round_final_team_ranking_categories: int
    round_final_player_ranking_categories: int


class InsightSyncRepository(Protocol):
    def begin_sync(self) -> None: ...

    def commit_sync(self) -> None: ...

    def rollback_sync(self) -> None: ...

    def start_scrape_run(self, *, source: str, season: int, started_at: datetime) -> str: ...

    def upsert_teams(self, teams: list[TeamProfile]) -> int: ...

    def upsert_players(self, players: list[PlayerProfile]) -> int: ...

    def upsert_matches(self, matches: list[MatchResult]) -> int: ...

    def round_final_exists(self, *, season: int, round_number: int) -> bool: ...

    def insert_standings(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: SnapshotKind,
        standings: list[StandingEntry],
        snapshot_at: datetime,
    ) -> int: ...

    def sync_team_rankings(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: SnapshotKind,
        datasets: list[RankingDataset],
        snapshot_at: datetime,
    ) -> int: ...

    def sync_player_rankings(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: SnapshotKind,
        datasets: list[RankingDataset],
        snapshot_at: datetime,
    ) -> int: ...

    def sync_team_insights(
        self,
        *,
        run_id: str,
        season: int,
        round_number: int | None,
        snapshot_kind: SnapshotKind,
        payload: SyncPayload,
        snapshot_at: datetime,
    ) -> int: ...

    def complete_scrape_run(self, *, run_id: str, finished_at: datetime, remark: str | None = None) -> None: ...

    def fail_scrape_run(self, *, run_id: str, finished_at: datetime, remark: str) -> None: ...


def find_latest_completed_round(matches: list[MatchResult]) -> int | None:
    matches_by_round: dict[int, list[MatchResult]] = {}
    for match in matches:
        matches_by_round.setdefault(match.round_number, []).append(match)

    completed_rounds = [
        round_number
        for round_number, round_matches in matches_by_round.items()
        if round_matches and all(match.status == "3" for match in round_matches)
    ]
    if not completed_rounds:
        return None
    return max(completed_rounds)


def normalize_live_round_number(current_round: int) -> int | None:
    return current_round if current_round > 0 else None


def has_started_matches_beyond_round(matches: list[MatchResult], round_number: int) -> bool:
    return any(
        match.round_number > round_number and match.status != "1"
        for match in matches
    )


class InsightSyncService:
    def __init__(
        self,
        repository: InsightSyncRepository,
        source: str = "sina_csl_scraper",
        clock: Callable[[], datetime] | None = None,
    ) -> None:
        self.repository = repository
        self.source = source
        self.clock = clock or (lambda: datetime.now(UTC))

    def sync(self, payload: SyncPayload) -> SyncResult:
        started_at = self.clock()
        run_id = self.repository.start_scrape_run(
            source=self.source,
            season=payload.season,
            started_at=started_at,
        )
        self.repository.begin_sync()

        try:
            live_round_number = normalize_live_round_number(payload.current_round)
            teams_upserted = self.repository.upsert_teams(payload.teams)
            players_upserted = self.repository.upsert_players(payload.players)
            matches_upserted = self.repository.upsert_matches(payload.matches)
            live_standings_inserted = self.repository.insert_standings(
                run_id=run_id,
                season=payload.season,
                round_number=live_round_number,
                snapshot_kind="live",
                standings=payload.standings,
                snapshot_at=started_at,
            )
            live_team_ranking_categories = self.repository.sync_team_rankings(
                run_id=run_id,
                season=payload.season,
                round_number=live_round_number,
                snapshot_kind="live",
                datasets=payload.team_rankings,
                snapshot_at=started_at,
            )
            live_player_ranking_categories = self.repository.sync_player_rankings(
                run_id=run_id,
                season=payload.season,
                round_number=live_round_number,
                snapshot_kind="live",
                datasets=payload.player_rankings,
                snapshot_at=started_at,
            )
            self.repository.sync_team_insights(
                run_id=run_id,
                season=payload.season,
                round_number=live_round_number,
                snapshot_kind="live",
                payload=payload,
                snapshot_at=started_at,
            )

            latest_completed_round = find_latest_completed_round(payload.matches)
            round_finalized_for: int | None = None
            round_final_standings_inserted = 0
            round_final_team_ranking_categories = 0
            round_final_player_ranking_categories = 0

            can_finalize_round = (
                latest_completed_round is not None
                and not has_started_matches_beyond_round(payload.matches, latest_completed_round)
            )

            if can_finalize_round and not self.repository.round_final_exists(
                season=payload.season,
                round_number=latest_completed_round,
            ):
                round_finalized_for = latest_completed_round
                round_final_standings_inserted = self.repository.insert_standings(
                    run_id=run_id,
                    season=payload.season,
                    round_number=latest_completed_round,
                    snapshot_kind="round_final",
                    standings=payload.standings,
                    snapshot_at=started_at,
                )
                round_final_team_ranking_categories = self.repository.sync_team_rankings(
                    run_id=run_id,
                    season=payload.season,
                    round_number=latest_completed_round,
                    snapshot_kind="round_final",
                    datasets=payload.team_rankings,
                    snapshot_at=started_at,
                )
                round_final_player_ranking_categories = self.repository.sync_player_rankings(
                    run_id=run_id,
                    season=payload.season,
                    round_number=latest_completed_round,
                    snapshot_kind="round_final",
                    datasets=payload.player_rankings,
                    snapshot_at=started_at,
                )
                self.repository.sync_team_insights(
                    run_id=run_id,
                    season=payload.season,
                    round_number=latest_completed_round,
                    snapshot_kind="round_final",
                    payload=payload,
                    snapshot_at=started_at,
                )
        except Exception as error:
            self.repository.rollback_sync()
            self.repository.fail_scrape_run(
                run_id=run_id,
                finished_at=self.clock(),
                remark=str(error),
            )
            raise

        self.repository.commit_sync()
        self.repository.complete_scrape_run(
            run_id=run_id,
            finished_at=self.clock(),
        )

        return SyncResult(
            run_id=run_id,
            season=payload.season,
            teams_upserted=teams_upserted,
            players_upserted=players_upserted,
            matches_upserted=matches_upserted,
            live_standings_inserted=live_standings_inserted,
            live_team_ranking_categories=live_team_ranking_categories,
            live_player_ranking_categories=live_player_ranking_categories,
            round_finalized_for=round_finalized_for,
            round_final_standings_inserted=round_final_standings_inserted,
            round_final_team_ranking_categories=round_final_team_ranking_categories,
            round_final_player_ranking_categories=round_final_player_ranking_categories,
        )
