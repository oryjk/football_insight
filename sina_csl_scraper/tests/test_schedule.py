from dataclasses import replace

import pytest

from sina_csl_scraper.models import MatchResult
from sina_csl_scraper.schedule import ScheduleSnapshotError, fetch_reconciled_schedule


def build_match(match_id: int, *, round_number: int = 1, status: str = "1") -> MatchResult:
    return MatchResult(
        match_id=match_id,
        season=2026,
        round_number=round_number,
        round_name=f"第{round_number}轮",
        date="2026-03-01",
        time="19:35",
        status=status,
        home_team_id=match_id * 2,
        home_team_name=f"主队{match_id}",
        home_score="",
        away_team_id=match_id * 2 + 1,
        away_team_name=f"客队{match_id}",
        away_score="",
        home_logo="",
        away_logo="",
    )


class FakeSinaClient:
    def __init__(self, rounds: dict[int, list[MatchResult] | Exception]) -> None:
        self.rounds = rounds
        self.calls: list[int] = []

    def fetch_round_matches(self, season: int, round_number: int) -> list[MatchResult]:
        assert season == 2026
        self.calls.append(round_number)
        result = self.rounds[round_number]
        if isinstance(result, Exception):
            raise result
        return result


class FakeCflClient:
    def __init__(self, rounds: dict[int, list[MatchResult]]) -> None:
        self.rounds = rounds
        self.calls: list[int] = []

    def fetch_round_matches(self, season: int, round_number: int, team_index: object) -> list[MatchResult]:
        assert season == 2026
        assert team_index is not None
        self.calls.append(round_number)
        return self.rounds[round_number]


def test_complete_sina_round_does_not_call_cfl() -> None:
    sina_matches = [build_match(item) for item in range(1, 9)]
    cfl = FakeCflClient({})

    matches = fetch_reconciled_schedule(
        sina_client=FakeSinaClient({1: sina_matches}),
        cfl_client=cfl,
        season=2026,
        max_round=1,
    )

    assert matches == sina_matches
    assert cfl.calls == []


@pytest.mark.parametrize("sina_round", [[build_match(item) for item in range(1, 5)], RuntimeError("Sina unavailable")])
def test_incomplete_or_failed_sina_round_is_filled_from_cfl(
    sina_round: list[MatchResult] | Exception,
) -> None:
    official_matches = [
        replace(build_match(item), schedule_source="cfl", source_match_id=f"official-{item}")
        for item in range(1, 9)
    ]
    official_matches[-1] = replace(official_matches[-1], status="4")
    cfl = FakeCflClient({1: official_matches})

    matches = fetch_reconciled_schedule(
        sina_client=FakeSinaClient({1: sina_round}),
        cfl_client=cfl,
        season=2026,
        max_round=1,
    )

    assert len(matches) == 8
    assert cfl.calls == [1]
    if isinstance(sina_round, list):
        assert matches[0] is sina_round[0]
    assert matches[-1].status == "4"


def test_incomplete_merged_round_is_rejected() -> None:
    with pytest.raises(ScheduleSnapshotError, match="round 1 expected 8 matches, got 7"):
        fetch_reconciled_schedule(
            sina_client=FakeSinaClient({1: [build_match(item) for item in range(1, 5)]}),
            cfl_client=FakeCflClient({1: [build_match(item) for item in range(1, 8)]}),
            season=2026,
            max_round=1,
        )


def test_overfull_sina_round_is_rejected_without_calling_cfl() -> None:
    cfl = FakeCflClient({1: []})

    with pytest.raises(ScheduleSnapshotError, match="round 1 expected 8 matches, got 9"):
        fetch_reconciled_schedule(
            sina_client=FakeSinaClient({1: [build_match(item) for item in range(1, 10)]}),
            cfl_client=cfl,
            season=2026,
            max_round=1,
        )

    assert cfl.calls == []
