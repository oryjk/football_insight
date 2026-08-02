from __future__ import annotations

from typing import Any

from .cfl_client import TeamIdentityIndex
from .models import MatchResult


class ScheduleSnapshotError(RuntimeError):
    """Raised when a complete, unique season schedule cannot be assembled."""


def _fixture_key(match: MatchResult) -> tuple[int, int, int, int]:
    return (
        match.season,
        match.round_number,
        match.home_team_id,
        match.away_team_id,
    )


def _validate_unique_round(
    matches: list[MatchResult],
    *,
    season: int,
    round_number: int,
) -> None:
    invalid = [
        match
        for match in matches
        if match.season != season or match.round_number != round_number
    ]
    if invalid:
        raise ScheduleSnapshotError(
            f"round {round_number} contains matches for another season or round"
        )
    keys = [_fixture_key(match) for match in matches]
    if len(keys) != len(set(keys)):
        raise ScheduleSnapshotError(f"round {round_number} contains duplicate fixtures")


def fetch_reconciled_schedule(
    sina_client: Any,
    cfl_client: Any,
    season: int,
    max_round: int,
    expected_matches_per_round: int = 8,
) -> list[MatchResult]:
    sina_rounds: dict[int, list[MatchResult]] = {}
    sina_errors: dict[int, Exception] = {}

    for round_number in range(1, max_round + 1):
        try:
            round_matches = sina_client.fetch_round_matches(season, round_number)
        except Exception as error:
            sina_rounds[round_number] = []
            sina_errors[round_number] = error
            continue

        _validate_unique_round(
            round_matches,
            season=season,
            round_number=round_number,
        )
        if len(round_matches) > expected_matches_per_round:
            raise ScheduleSnapshotError(
                f"round {round_number} expected {expected_matches_per_round} matches, "
                f"got {len(round_matches)}"
            )
        sina_rounds[round_number] = round_matches

    team_index = TeamIdentityIndex.for_season(season)
    merged_schedule: list[MatchResult] = []

    for round_number in range(1, max_round + 1):
        sina_matches = sina_rounds[round_number]
        if len(sina_matches) == expected_matches_per_round:
            merged_matches = sina_matches
        else:
            try:
                cfl_matches = cfl_client.fetch_round_matches(
                    season,
                    round_number,
                    team_index,
                )
            except Exception as error:
                sina_error = sina_errors.get(round_number)
                detail = f"; Sina error: {sina_error}" if sina_error else ""
                raise ScheduleSnapshotError(
                    f"round {round_number} fallback failed: {error}{detail}"
                ) from error

            _validate_unique_round(
                cfl_matches,
                season=season,
                round_number=round_number,
            )
            merged_by_fixture = {_fixture_key(match): match for match in cfl_matches}
            merged_by_fixture.update({_fixture_key(match): match for match in sina_matches})
            merged_matches = list(merged_by_fixture.values())

        if len(merged_matches) != expected_matches_per_round:
            raise ScheduleSnapshotError(
                f"round {round_number} expected {expected_matches_per_round} matches, "
                f"got {len(merged_matches)}"
            )
        merged_schedule.extend(
            sorted(
                merged_matches,
                key=lambda match: (match.date, match.time, match.match_id),
            )
        )

    expected_total = max_round * expected_matches_per_round
    if len(merged_schedule) != expected_total:
        raise ScheduleSnapshotError(
            f"season {season} expected {expected_total} matches, got {len(merged_schedule)}"
        )
    return merged_schedule
