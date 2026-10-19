from __future__ import annotations

from sina_csl_scraper.client import SinaCslClient, map_squad_position, parse_squad_player


class FakeResponse:
    def __init__(self, payload):
        self.payload = payload

    def raise_for_status(self) -> None:
        return None

    def json(self):
        return self.payload


class FakeSession:
    def __init__(self, players):
        self.headers = {}
        self.calls = []
        self.players = players

    def get(self, url, params, timeout):
        self.calls.append((url, params))
        return FakeResponse({"result": {"status": {"code": 0, "msg": "team players"}, "data": {"players": self.players}}})


def test_maps_chinese_positions():
    assert map_squad_position("守门员") == "GK"
    assert map_squad_position("后卫") == "DF"
    assert map_squad_position("中场") == "MF"
    assert map_squad_position("前锋") == "FW"
    assert map_squad_position("教练") is None
    assert map_squad_position(None) is None


def test_parses_player_fields_and_tolerates_bad_numbers():
    player = parse_squad_player(77680, {
        "pid": "156047", "name": "Feng Zhuoyi", "name_cn": "冯卓毅", "number": "6",
        "country": "中国", "position": "中场", "logo": "http://n.sinaimg.cn/ty/opta/player150/156047.png",
    })
    assert player is not None
    assert (player.team_id, player.player_id, player.player_name, player.shirt_number, player.position) == (
        77680, 156047, "冯卓毅", 6, "MF")
    assert player.avatar_source_url == "http://n.sinaimg.cn/ty/opta/player150/156047.png"
    assert parse_squad_player(1, {"pid": "9", "name": "X", "number": "", "position": ""}).shirt_number is None
    assert parse_squad_player(1, {"pid": "9", "name": "X", "number": "1a"}).shirt_number is None
    assert parse_squad_player(1, {"pid": "9", "name_cn": "", "name": "Only English"}).player_name == "Only English"


def test_skips_items_without_pid_or_name():
    assert parse_squad_player(1, {"name_cn": "无编号"}) is None
    assert parse_squad_player(1, {"pid": "3", "name": "", "name_cn": ""}) is None


def test_fetch_team_squad_passes_team_and_season():
    session = FakeSession([{"pid": "1", "name_cn": "甲", "number": "1", "position": "守门员"}, {"name_cn": "无pid"}])
    client = SinaCslClient(session=session)
    players = client.fetch_team_squad(77680, season=2025)
    assert [p.player_id for p in players] == [1]
    url, params = session.calls[0]
    assert url.endswith("/op/api/team/player")
    assert params["tid"] == 77680 and params["season"] == 2025


def test_fetch_team_squad_without_season_omits_param():
    session = FakeSession([])
    SinaCslClient(session=session).fetch_team_squad(77680)
    assert "season" not in session.calls[0][1]
