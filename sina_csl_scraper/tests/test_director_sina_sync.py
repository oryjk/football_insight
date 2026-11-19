from __future__ import annotations

import io
import json
from typing import Any

import pytest
from PIL import Image
from typer.testing import CliRunner

from sina_csl_scraper.cli import app
from sina_csl_scraper.director_sina_sync import (
    DEFAULT_LEAGUE_IDS,
    DEFAULT_LEAGUES_STR,
    LeagueTeam,
    classify_team,
    extract_player_id_from_code,
    extract_teams_from_schedule,
    extract_teams_from_standings,
    is_csl_team,
    is_foreign_country,
    parse_leagues_arg,
    sync_director_sina,
)
from sina_csl_scraper.models import SquadPlayer


def _make_png_bytes(width: int = 16, height: int = 16) -> bytes:
    buf = io.BytesIO()
    Image.new("RGB", (width, height), color=(12, 88, 200)).save(buf, format="PNG")
    return buf.getvalue()


class FakeSinaDirectorClient:
    def __init__(
        self,
        *,
        league_meta: dict[int, tuple[int, str]],
        standings: dict[tuple[int, int], Any] | None = None,
        schedules: dict[tuple[int, int], Any] | None = None,
        squads: dict[int, list[SquadPlayer] | Exception] | None = None,
    ) -> None:
        self.league_meta = league_meta
        self.standings = standings or {}
        self.schedules = schedules or {}
        self.squads = squads or {}
        self.schedule_calls: list[tuple[int, int, int]] = []

    def fetch_league_meta(self, lid: int) -> tuple[int, str]:
        return self.league_meta[lid]

    def fetch_standings_raw(self, lid: int, season: int) -> Any:
        val = self.standings.get((lid, season), {})
        if isinstance(val, Exception):
            raise val
        return val

    def fetch_schedule_round_raw(self, lid: int, season: int, round_number: int = 1) -> Any:
        self.schedule_calls.append((lid, season, round_number))
        return self.schedules.get((lid, season), {})

    def fetch_team_squad(self, team_id: int, season: int | None = None) -> list[SquadPlayer]:
        val = self.squads.get(team_id, [])
        if isinstance(val, Exception):
            raise val
        return val


class FakeDirectorSinaStore:
    def __init__(
        self,
        *,
        csl_team_ids: set[int] | None = None,
        csl_team_names: set[str] | None = None,
        existing_by_code: dict[str, str] | None = None,
        existing_by_player_id: dict[int, str] | None = None,
        seed_backfilled: int = 0,
        deactivated: int = 0,
    ) -> None:
        self._csl_team_ids = set(csl_team_ids or ())
        self._csl_team_names = set(csl_team_names or ())
        self._existing_by_code = dict(existing_by_code or {})
        self._existing_by_player_id = dict(existing_by_player_id or {})
        self._seed_backfilled = seed_backfilled
        self._deactivated = deactivated
        self.batches: list[list[dict[str, Any]]] = []
        self.rows_by_code: dict[str, dict[str, Any]] = {}
        self.backfill_calls = 0
        self.deactivate_calls = 0

    def load_csl_teams(self) -> tuple[set[int], set[str]]:
        return set(self._csl_team_ids), set(self._csl_team_names)

    def load_existing_avatars(self) -> tuple[dict[str, str], dict[int, str]]:
        return dict(self._existing_by_code), dict(self._existing_by_player_id)

    def upsert_candidates(self, rows: list[dict[str, Any]]) -> int:
        self.batches.append(list(rows))
        for row in rows:
            self.rows_by_code[row["code"]] = row
        return len(rows)

    def backfill_seed_avatars(self) -> int:
        self.backfill_calls += 1
        return self._seed_backfilled

    def deactivate_no_avatar(self) -> int:
        self.deactivate_calls += 1
        return self._deactivated


class FakeAssetFetcher:
    def __init__(self, responses: dict[str, tuple[bytes, str] | Exception]) -> None:
        self.responses = responses
        self.fetched_urls: list[str] = []

    def fetch(self, url: str) -> tuple[bytes, str]:
        self.fetched_urls.append(url)
        result = self.responses[url]
        if isinstance(result, Exception):
            raise result
        return result


class FakeAssetTarget:
    def __init__(self, public_base_url: str = "https://oryjk.cn:82/minio/football-insight") -> None:
        self.public_base_url = public_base_url.rstrip("/")
        self.uploads: dict[str, tuple[bytes, str]] = {}

    def upload_bytes(
        self,
        object_name: str,
        content: bytes,
        content_type: str,
        cache_control: str | None = None,
    ) -> str:
        self.uploads[object_name] = (content, content_type)
        return f"{self.public_base_url}/{object_name.lstrip('/')}"


def test_parse_leagues_arg() -> None:
    assert parse_leagues_arg("328, 403, 214, 328") == (328, 403, 214)
    assert parse_leagues_arg(DEFAULT_LEAGUES_STR) == DEFAULT_LEAGUE_IDS
    with pytest.raises(ValueError):
        parse_leagues_arg("")
    with pytest.raises(ValueError):
        parse_leagues_arg("328,abc")


def test_extract_teams_from_standings_nested_and_deduped() -> None:
    payload = {
        "regions": {
            "east": [
                {
                    "group": "A",
                    "teams": [
                        {"tid": "1001", "team_name": "蔚山HD"},
                        {"tid": 1002, "team_name": "横滨水手"},
                    ],
                },
                {
                    "group": "B",
                    "teams": [
                        {"tid": 1001, "team_name": "蔚山HD"},  # duplicate
                        {"tid": "bad", "team_name": "无效队"},
                        {"tid": 1003, "team_name": "   "},
                    ],
                },
            ],
            "west": {
                "table": [
                    {"tid": 2001, "team_name": "利雅得新月"},
                ]
            },
        }
    }
    assert extract_teams_from_standings(payload) == [
        LeagueTeam(1001, "蔚山HD"),
        LeagueTeam(1002, "横滨水手"),
        LeagueTeam(2001, "利雅得新月"),
    ]


def test_extract_teams_from_schedule_supports_both_formats() -> None:
    payload = {
        "matches": [
            {"Team1Id": "3001", "Team1": "全北现代", "Team2Id": 3002, "Team2": "川崎前锋"},
            {"home_tid": 3002, "home_name": "川崎前锋", "away_tid": "3003", "away_name": "浦项铁人"},
        ]
    }
    assert extract_teams_from_schedule(payload) == [
        LeagueTeam(3001, "全北现代"),
        LeagueTeam(3002, "川崎前锋"),
        LeagueTeam(3003, "浦项铁人"),
    ]


def test_is_csl_team() -> None:
    assert is_csl_team(77680, "任意名", {77680}, set()) is True
    assert is_csl_team(999, "上海海港", set(), {"上海海港"}) is True
    assert is_csl_team(999, "山东泰山", set(), set()) is True
    assert is_csl_team(999, "浙江", set(), set()) is True
    assert is_csl_team(999, "蔚山HD", {77680}, {"上海海港"}) is False
    assert is_csl_team(999, "辽宁铁人", {77680}, {"上海海港"}) is False


def test_classify_team_all_leagues() -> None:
    assert classify_team(328, "蔚山HD") == classify_team(403, "全北现代")
    k1 = classify_team(328, "蔚山HD")
    assert (k1.league_tag, k1.sort_order, k1.tags) == ("K1", 40, ["K1现役", "亚冠"])

    j1 = classify_team(328, "横滨水手")
    assert (j1.league_tag, j1.sort_order, j1.tags) == ("J1", 50, ["J1现役", "亚冠"])

    acl = classify_team(328, "利雅得新月")
    assert (acl.league_tag, acl.sort_order, acl.tags) == ("ACL", 70, ["亚冠现役"])

    cl1 = classify_team(214, "辽宁铁人")
    assert (cl1.league_tag, cl1.sort_order, cl1.tags) == ("CL1", 60, ["中甲现役"])

    spl = classify_team(404, "利雅得胜利")
    assert (spl.league_tag, spl.sort_order, spl.tags) == ("SPL", 90, ["沙特联现役"])

    mls = classify_team(405, "迈阿密国际")
    assert (mls.league_tag, mls.sort_order, mls.tags) == ("MLS", 100, ["美职联现役"])

    eur_cases = [
        (1, "意甲现役"),
        (2, "西甲现役"),
        (3, "德甲现役"),
        (4, "英超现役"),
        (5, "法甲现役"),
    ]
    for lid, expected_tag in eur_cases:
        c = classify_team(lid, "测试俱乐部")
        assert (c.league_tag, c.sort_order, c.tags) == ("EUR", 110, [expected_tag])

    other = classify_team(999, "阿贾克斯", league_name="荷甲")
    assert (other.league_tag, other.sort_order, other.tags) == ("OTHER", 120, ["荷甲现役"])


def test_is_foreign_country() -> None:
    for domestic in (
        None,
        "",
        "  ",
        "中国",
        "China",
        "CHINA",
        "chn",
        "中国香港",
        "香港",
        "Hong Kong",
        "中国澳门",
        "澳门",
        "Macau",
        "中国台湾",
        "台湾",
        "中华台北",
        "Taiwan",
        "Chinese Taipei",
    ):
        assert is_foreign_country(domestic) is False, f"expected domestic for {domestic!r}"

    for foreign in ("巴西", "韩国", "日本", "Brazil", "Portugal", "阿根廷"):
        assert is_foreign_country(foreign) is True, f"expected foreign for {foreign!r}"


def test_extract_player_id_from_code() -> None:
    assert extract_player_id_from_code("sina-k1-123456") == 123456
    assert extract_player_id_from_code("sina-eur-987") == 987
    assert extract_player_id_from_code("k1-cesinha") is None


def test_sync_director_sina_end_to_end_with_thumbnails_reuse_fallback_and_fault_isolation() -> None:
    png_bytes = _make_png_bytes()
    public_base = "https://oryjk.cn:82/minio/football-insight"

    client = FakeSinaDirectorClient(
        league_meta={
            328: (2025, "亚冠"),
            214: (2025, "中甲"),
            4: (2025, "英超"),
        },
        standings={
            # ACL nested groups: CSL team (skipped), K1 team, ACL non-K1/J1 team, and failing team
            (328, 2025): {
                "east": [
                    {"tid": 77680, "team_name": "上海海港"},
                    {"tid": 101, "team_name": "蔚山HD"},
                    {"tid": 102, "team_name": "利雅得新月"},
                    {"tid": 103, "team_name": "广岛三箭"},
                ]
            },
            # CL1 empty standings -> triggers schedule round 1 fallback
            (214, 2025): {},
            # Premier League flat standings
            (4, 2025): {
                "teams": [
                    {"tid": 401, "team_name": "阿森纳"},
                ]
            },
        },
        schedules={
            (214, 2025): {
                "list": [
                    {"Team1Id": 201, "Team1": "辽宁铁人", "Team2Id": 77680, "Team2": "上海海港"},
                ]
            }
        },
        squads={
            101: [
                # New portrait -> should download + upload + generate 96/240px thumbnails
                SquadPlayer(101, 5001, "周敏圭", 9, "FW", "韩国", "https://cdn.sina.cn/p/5001.png"),
                # Missing avatar -> skipped by default
                SquadPlayer(101, 5002, "无头像球员", 10, "MF", "巴西", None),
                # Invalid position -> skipped
                SquadPlayer(101, 5003, "未知位置", 11, None, "巴西", "https://cdn.sina.cn/p/5003.png"),
                # Avatar download error -> skipped (since include_no_avatar=False), does not break team
                SquadPlayer(101, 5004, "头像失败球员", 12, "DF", "韩国", "https://cdn.sina.cn/p/fail.png"),
            ],
            102: [
                # Non-K1/J1 ACL club -> included by default as league_tag="ACL"
                SquadPlayer(102, 6001, "米特罗维奇", 9, "FW", "塞尔维亚", "https://cdn.sina.cn/p/6001.jpg"),
            ],
            # Team 103 raises error -> isolated, recorded in teams_failed
            103: RuntimeError("sina timeout"),
            201: [
                # Domestic player with existing MinIO avatar by player_id -> reused without fetch!
                SquadPlayer(201, 7001, "邦本宜裕", 10, "MF", "日本", "https://cdn.sina.cn/p/7001.png"),
                SquadPlayer(201, 7002, "安以恩", 7, "FW", "中国台湾", "https://cdn.sina.cn/p/7002.png"),
            ],
            401: [
                # Same player_id 6001 appears in another league -> reuses in-run uploaded avatar!
                SquadPlayer(401, 6001, "米特罗维奇", 9, "FW", "塞尔维亚", "https://cdn.sina.cn/p/6001.jpg"),
            ],
        },
    )

    store = FakeDirectorSinaStore(
        csl_team_ids={77680},
        existing_by_player_id={
            7001: f"{public_base}/summary/players/sina-7001.png",
        },
        seed_backfilled=3,
        deactivated=2,
    )
    fetcher = FakeAssetFetcher(
        {
            "https://cdn.sina.cn/p/5001.png": (png_bytes, "image/png"),
            "https://cdn.sina.cn/p/fail.png": RuntimeError("404 not found"),
            "https://cdn.sina.cn/p/6001.jpg": (png_bytes, "image/jpeg"),
            "https://cdn.sina.cn/p/7002.png": (png_bytes, "image/png"),
        }
    )
    target = FakeAssetTarget(public_base_url=public_base)

    summary = sync_director_sina(
        client=client,
        store=store,
        leagues=(328, 214, 4),
        deactivate_no_avatar=True,
        interval=0.0,
        fetcher=fetcher,
        target=target,
        prefix="summary",
        public_base_url=public_base,
    )

    assert summary["teams_scanned"] == 5
    assert summary["teams_failed"] == 1
    assert summary["players_upserted"] == 5
    assert summary["avatars_uploaded"] == 3
    assert summary["avatars_reused"] == 2
    assert summary["seed_backfilled"] == 3
    assert summary["deactivated_no_avatar"] == 2
    assert len(summary["leagues"]) == 3
    assert client.schedule_calls == [(214, 2025, 1)]

    # Per-team DB commit: 4 successful teams -> 4 separate upsert_candidates batches
    assert len(store.batches) == 4
    assert store.backfill_calls == 1
    assert store.deactivate_calls == 1

    # Verify uploaded original + 96/240px thumbnails
    assert "summary/players/sina-5001.png" in target.uploads
    assert "summary/players/thumbs/96/sina-5001.png.jpg" in target.uploads
    assert "summary/players/thumbs/240/sina-5001.png.jpg" in target.uploads
    assert "summary/players/sina-6001.jpg" in target.uploads
    assert "summary/players/sina-7002.png" in target.uploads
    # Reused player 7001 was never fetched
    assert "https://cdn.sina.cn/p/7001.png" not in fetcher.fetched_urls

    # Verify row contents
    row_k1 = store.rows_by_code["sina-k1-5001"]
    assert row_k1["league_tag"] == "K1"
    assert row_k1["sort_order"] == 40
    assert row_k1["tags"] == ["K1现役", "亚冠"]
    assert row_k1["is_foreign"] is True
    assert row_k1["avatar_url"] == f"{public_base}/summary/players/sina-5001.png"

    row_acl = store.rows_by_code["sina-acl-6001"]
    assert row_acl["league_tag"] == "ACL"
    assert row_acl["sort_order"] == 70
    assert row_acl["tags"] == ["亚冠现役"]

    row_cl1_tw = store.rows_by_code["sina-cl1-7002"]
    assert row_cl1_tw["league_tag"] == "CL1"
    assert row_cl1_tw["sort_order"] == 60
    assert row_cl1_tw["is_foreign"] is False
    assert row_cl1_tw["tags"] == ["中甲现役"]

    row_eur = store.rows_by_code["sina-eur-6001"]
    assert row_eur["league_tag"] == "EUR"
    assert row_eur["sort_order"] == 110
    assert row_eur["tags"] == ["英超现役"]
    assert row_eur["avatar_url"] == f"{public_base}/summary/players/sina-6001.jpg"


def test_sync_director_sina_no_include_acl_and_include_no_avatar_and_force_avatars() -> None:
    png_bytes = _make_png_bytes()
    public_base = "https://oryjk.cn:82/minio/football-insight"

    client = FakeSinaDirectorClient(
        league_meta={328: (2025, "亚冠")},
        standings={
            (328, 2025): {
                "teams": [
                    {"tid": 101, "team_name": "蔚山HD"},
                    {"tid": 102, "team_name": "利雅得新月"},
                ]
            }
        },
        squads={
            101: [
                SquadPlayer(101, 5001, "周敏圭", 9, "FW", "韩国", "https://cdn.sina.cn/p/5001.png"),
                SquadPlayer(101, 5002, "无头像球员", 10, "MF", None, None),
            ],
            102: [
                SquadPlayer(102, 6001, "米特罗维奇", 9, "FW", "塞尔维亚", "https://cdn.sina.cn/p/6001.jpg"),
            ],
        },
    )
    store = FakeDirectorSinaStore(
        existing_by_code={"sina-k1-5001": f"{public_base}/summary/players/sina-5001.png"}
    )
    fetcher = FakeAssetFetcher({"https://cdn.sina.cn/p/5001.png": (png_bytes, "image/png")})
    target = FakeAssetTarget(public_base_url=public_base)

    summary = sync_director_sina(
        client=client,
        store=store,
        leagues=(328,),
        include_acl=False,
        include_no_avatar=True,
        force_avatars=True,
        interval=0.0,
        fetcher=fetcher,
        target=target,
        public_base_url=public_base,
    )

    # Team 102 (ACL non-K1/J1) was skipped because include_acl=False
    assert summary["teams_scanned"] == 1
    assert summary["players_upserted"] == 2
    assert summary["avatars_uploaded"] == 1
    assert summary["avatars_reused"] == 0
    assert "sina-k1-5002" in store.rows_by_code
    assert store.rows_by_code["sina-k1-5002"]["country"] == ""
    assert store.rows_by_code["sina-k1-5002"]["is_foreign"] is False
    assert store.rows_by_code["sina-k1-5002"]["avatar_url"] is None


def test_sync_director_sina_dry_run_and_limit_teams() -> None:
    client = FakeSinaDirectorClient(
        league_meta={404: (2025, "沙特联"), 405: (2025, "美职联")},
        standings={
            (404, 2025): {
                "teams": [
                    {"tid": 1, "team_name": "利雅得胜利"},
                    {"tid": 2, "team_name": "利雅得新月"},
                ]
            },
            (405, 2025): {"teams": [{"tid": 3, "team_name": "迈阿密国际"}]},
        },
        squads={
            1: [SquadPlayer(1, 9001, "C罗", 7, "FW", "葡萄牙", "https://cdn.sina.cn/p/9001.png")],
            2: [SquadPlayer(2, 9002, "内马尔", 10, "FW", "巴西", "https://cdn.sina.cn/p/9002.png")],
            3: [SquadPlayer(3, 9003, "梅西", 10, "FW", "阿根廷", "https://cdn.sina.cn/p/9003.png")],
        },
    )
    store = FakeDirectorSinaStore(seed_backfilled=5, deactivated=4)
    target = FakeAssetTarget()

    summary = sync_director_sina(
        client=client,
        store=store,
        leagues=(404, 405),
        dry_run=True,
        limit_teams=1,
        deactivate_no_avatar=True,
        interval=0.0,
        target=target,
    )

    assert summary["teams_scanned"] == 1
    assert summary["players_upserted"] == 1
    assert summary["avatars_uploaded"] == 0
    assert summary["seed_backfilled"] == 0
    assert summary["deactivated_no_avatar"] == 0
    assert store.batches == []
    assert store.backfill_calls == 0
    assert store.deactivate_calls == 0
    assert target.uploads == {}


def test_cli_sync_director_sina_help_and_defaults() -> None:
    runner = CliRunner(env={"COLUMNS": "200"})
    result = runner.invoke(app, ["sync-director-sina", "--help"])
    assert result.exit_code == 0
    assert DEFAULT_LEAGUES_STR in result.output
    assert "--include-acl" in result.output
    assert "--deactivate-no-avatar" in result.output
    assert "--force-avatars" in result.output
    assert "--limit-teams" in result.output
    assert "--interval" in result.output
