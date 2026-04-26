from __future__ import annotations

from sina_csl_scraper.client import SinaCslClient, serialize_datasets
from sina_csl_scraper.constants import PLAYER_RANKING_CATEGORIES, TEAM_RANKING_CATEGORIES


class FakeResponse:
    def __init__(self, payload):
        self.payload = payload

    def raise_for_status(self) -> None:
        return None

    def json(self):
        return self.payload


class FakeSession:
    def __init__(self) -> None:
        self.headers = {}
        self.calls = []

    def get(self, url, params, timeout):
        self.calls.append((url, params, timeout))
        path = url.split(".cn", 1)[1]
        if path == "/op/api/league":
            return FakeResponse(
                {
                    "result": {
                        "status": {"code": 0, "msg": "ok"},
                        "data": {
                            "league": {
                                "lid": "213",
                                "name": "中超",
                                "cur_season": "2026",
                                "cur_round": "4",
                                "max_round": "30",
                            }
                        },
                    }
                }
            )
        if path == "/op/api/ranking/team":
            return FakeResponse(
                {
                    "result": {
                        "status": {"code": 0, "msg": "ok"},
                        "data": {
                            "rank": [
                                {
                                    "rank": "1",
                                    "score": "14",
                                    "tid": "77680",
                                    "team_name": "成都蓉城",
                                    "team_logo": "logo.png",
                                }
                            ]
                        },
                    }
                }
            )
        if path == "/op/api/ranking/player":
            page = params["page"]
            rank = (
                [
                    {
                        "rank": "1",
                        "score": "4",
                        "pid": "204211",
                        "player_name": "费利佩",
                        "player_logo": "player.png",
                        "tid": "77680",
                        "team_name": "成都蓉城",
                        "penalty": "1",
                    }
                ]
                if page == 1
                else []
            )
            return FakeResponse(
                {
                    "result": {
                        "status": {"code": 0, "msg": "ok"},
                        "data": {"rank": rank},
                    }
                }
            )
        raise AssertionError(f"Unexpected path {path}")


def test_fetch_team_ranking_maps_response() -> None:
    client = SinaCslClient(session=FakeSession())
    dataset = client.fetch_team_ranking(2026, TEAM_RANKING_CATEGORIES[0])

    assert dataset.slug == "goals"
    assert dataset.entries[0]["team_name"] == "成都蓉城"
    assert dataset.entries[0]["rank"] == 1


def test_fetch_all_player_rankings_stops_on_empty_page() -> None:
    session = FakeSession()
    client = SinaCslClient(session=session)

    datasets = client.fetch_all_player_rankings(2026, limit=1)

    assert len(datasets) == len(PLAYER_RANKING_CATEGORIES)
    assert datasets[0].entries[0]["player_name"] == "费利佩"
    player_calls = [call for call in session.calls if call[0].endswith("/op/api/ranking/player")]
    assert len(player_calls) == len(PLAYER_RANKING_CATEGORIES) * 2


def test_serialize_datasets_returns_plain_dicts() -> None:
    client = SinaCslClient(session=FakeSession())
    dataset = client.fetch_team_ranking(2026, TEAM_RANKING_CATEGORIES[0])

    payload = serialize_datasets([dataset])

    assert payload == [
        {
            "slug": "goals",
            "label": "进球",
            "item_id": 1,
            "entries": [
                {
                    "rank": 1,
                    "score": "14",
                    "team_id": 77680,
                    "team_name": "成都蓉城",
                    "team_logo": "logo.png",
                }
            ],
        }
    ]


def test_fetch_team_ranking_handles_null_rank() -> None:
    class NullRankSession(FakeSession):
        def get(self, url, params, timeout):
            path = url.split(".cn", 1)[1]
            if path == "/op/api/ranking/team":
                return FakeResponse(
                    {
                        "result": {
                            "status": {"code": 0, "msg": "ok"},
                            "data": {"rank": None},
                        }
                    }
                )
            return super().get(url, params, timeout)

    client = SinaCslClient(session=NullRankSession())

    dataset = client.fetch_team_ranking(2026, TEAM_RANKING_CATEGORIES[0])

    assert dataset.entries == []
