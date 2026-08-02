import json
from pathlib import Path

import pytest

from sina_csl_scraper.cfl_client import CflApiError, CflCslClient, TeamIdentityIndex


class FakeResponse:
    def __init__(self, payload: dict[str, object]) -> None:
        self.payload = payload

    def raise_for_status(self) -> None:
        return None

    def json(self) -> dict[str, object]:
        return self.payload


class FakeSession:
    def __init__(self, payload: dict[str, object]) -> None:
        self.payload = payload
        self.headers: dict[str, str] = {}
        self.calls: list[dict[str, object]] = []

    def get(self, url: str, *, params: dict[str, object], timeout: int) -> FakeResponse:
        self.calls.append({"url": url, "params": params, "timeout": timeout})
        return FakeResponse(self.payload)


def load_round_18() -> dict[str, object]:
    fixture = Path(__file__).parent / "fixtures" / "cfl_round_18.json"
    return json.loads(fixture.read_text(encoding="utf-8"))


def test_fetch_round_matches_parses_postponed_matches_and_known_team_aliases() -> None:
    session = FakeSession(load_round_18())
    client = CflCslClient(session=session)

    matches = client.fetch_round_matches(
        season=2026,
        round_number=18,
        team_index=TeamIdentityIndex.for_season(2026),
    )

    assert len(matches) == 8
    assert sum(match.status == "4" for match in matches) == 4
    postponed = next(match for match in matches if match.home_team_name == "武汉三镇")
    assert postponed.away_team_id == 178
    assert postponed.away_team_name == "河南队"
    assert postponed.date == "2026-07-11"
    assert postponed.time == "19:00"
    assert postponed.schedule_source == "cfl"
    assert postponed.source_match_id == "7bz86a4xl8hb0rm4fyrwu347o"
    assert 0 < abs(postponed.match_id) <= (2**53 - 1)
    assert postponed.match_id == client.fetch_round_matches(
        season=2026,
        round_number=18,
        team_index=TeamIdentityIndex.for_season(2026),
    )[1].match_id
    assert session.calls[0]["params"] == {
        "tournament_calendar_id": "e6818x4pwankpph8awr91m1hw",
        "competition_code": "CSL",
        "contestant_id": "",
        "week": 18,
        "stage_id": "",
        "curPage": 1,
        "pageSize": 999,
    }


def test_fetch_round_matches_rejects_unknown_official_status() -> None:
    payload = load_round_18()
    payload["data"]["dataList"][0]["match_status"] = "Mystery"

    with pytest.raises(CflApiError, match="unsupported CFL match status"):
        CflCslClient(session=FakeSession(payload)).fetch_round_matches(
            season=2026,
            round_number=18,
            team_index=TeamIdentityIndex.for_season(2026),
        )
