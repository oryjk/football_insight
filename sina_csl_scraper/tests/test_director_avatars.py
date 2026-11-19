from __future__ import annotations

import io
import re
from pathlib import Path
from typing import Any

from PIL import Image
from typer.testing import CliRunner

from sina_csl_scraper.avatar_thumbs import THUMB_CACHE_CONTROL
from sina_csl_scraper.cli import app
from sina_csl_scraper.director_avatars import (
    DIRECTOR_SEARCH_HINTS,
    THESPORTSDB_SEARCH_URL,
    AvatarDiscoverer,
    DirectorCandidate,
    sync_director_avatars,
)

SEED_SQL_PATH = (
    Path(__file__).resolve().parents[2]
    / "football_insight_service_backend_rs"
    / "migrations"
    / "20261119130000_seed_director_candidates.sql"
)


def make_image_bytes(size: tuple[int, int] = (200, 300), fmt: str = "PNG") -> bytes:
    out = io.BytesIO()
    Image.new("RGB", size, (30, 120, 200)).save(out, fmt)
    return out.getvalue()


class FakeResponse:
    def __init__(self, status_code: int = 200, payload: dict[str, Any] | None = None) -> None:
        self.status_code = status_code
        self._payload = payload or {}

    def raise_for_status(self) -> None:
        if self.status_code >= 400:
            raise RuntimeError(f"HTTP {self.status_code}")

    def json(self) -> dict[str, Any]:
        return self._payload


class FakeSession:
    def __init__(
        self,
        sportsdb: dict[str, dict[str, Any]] | None = None,
        wiki: dict[str, tuple[int, dict[str, Any]]] | None = None,
    ) -> None:
        self.sportsdb = sportsdb or {}
        self.wiki = wiki or {}
        self.calls: list[tuple[str, dict[str, Any] | None, dict[str, str] | None]] = []

    def get(
        self,
        url: str,
        params: dict[str, Any] | None = None,
        headers: dict[str, str] | None = None,
        timeout: int = 15,
    ) -> FakeResponse:
        self.calls.append((url, params, headers))
        if url == THESPORTSDB_SEARCH_URL:
            query = (params or {}).get("p", "")
            return FakeResponse(200, self.sportsdb.get(query, {"player": None}))
        if "/api/rest_v1/page/summary/" in url:
            title = url.rsplit("/", 1)[-1]
            status, payload = self.wiki.get(title, (404, {}))
            return FakeResponse(status, payload)
        return FakeResponse(404, {})


class FakeStore:
    def __init__(
        self,
        candidates: list[DirectorCandidate],
        squad_avatars: dict[str, str] | None = None,
    ) -> None:
        self._candidates = candidates
        self._squad_avatars = squad_avatars or {}
        self.updates: list[tuple[str, str]] = []

    def list_candidates(self) -> list[DirectorCandidate]:
        return list(self._candidates)

    def find_squad_avatar(self, player_name: str) -> str | None:
        return self._squad_avatars.get(player_name)

    def update_avatar_url(self, code: str, avatar_url: str) -> None:
        self.updates.append((code, avatar_url))


class FakeFetcher:
    def __init__(self, responses: dict[str, tuple[bytes, str] | Exception]) -> None:
        self.responses = responses
        self.calls: list[str] = []

    def fetch(self, url: str) -> tuple[bytes, str]:
        self.calls.append(url)
        item = self.responses.get(url)
        if item is None:
            raise RuntimeError(f"unexpected url: {url}")
        if isinstance(item, Exception):
            raise item
        return item


class FakeTarget:
    def __init__(self) -> None:
        self.uploads: list[tuple[str, bytes, str, str | None]] = []

    def upload_bytes(
        self,
        object_name: str,
        content: bytes,
        content_type: str,
        cache_control: str | None = None,
    ) -> str:
        self.uploads.append((object_name, content, content_type, cache_control))
        return f"https://cdn.example.com/football-insight/{object_name}"


def test_all_58_seed_codes_are_covered_by_search_hints() -> None:
    sql = SEED_SQL_PATH.read_text(encoding="utf-8")
    seed_codes = re.findall(r"^\s*\('([a-z0-9-]+)',\s*'", sql, flags=re.MULTILINE)
    assert len(seed_codes) == 58
    assert len(set(seed_codes)) == 58

    missing = [code for code in seed_codes if code not in DIRECTOR_SEARCH_HINTS]
    assert missing == []

    for code in seed_codes:
        hint = DIRECTOR_SEARCH_HINTS[code]
        assert hint.search_names and all(name.strip() for name in hint.search_names)
        assert hint.wiki_titles and all(title.strip() for title in hint.wiki_titles)


def test_reuses_squad_avatar_without_external_requests_or_upload() -> None:
    candidate = DirectorCandidate(
        code="csl-lin-liangming",
        player_name="林良铭",
        country="中国",
        league_tag="CSL",
        current_club="北京国安",
    )
    squad_url = "https://oryjk.cn:82/minio/football-insight/summary/players/12345.png"
    store = FakeStore([candidate], squad_avatars={"林良铭": squad_url})
    session = FakeSession()
    fetcher = FakeFetcher({})
    target = FakeTarget()

    summary = sync_director_avatars(
        store=store,
        prefix="summary",
        fetcher=fetcher,
        target=target,
        discoverer=AvatarDiscoverer(session=session),
        interval=0,
    )

    assert summary == {
        "candidates": 1,
        "skipped": 0,
        "reused_from_squad": 1,
        "uploaded": 0,
        "not_found": 0,
        "failed": 0,
    }
    assert store.updates == [("csl-lin-liangming", squad_url)]
    assert session.calls == []
    assert fetcher.calls == []
    assert target.uploads == []


def test_thesportsdb_filters_soccer_and_prefers_cutout_over_thumb() -> None:
    session = FakeSession(
        sportsdb={
            "Jesse Lingard": {
                "player": [
                    {
                        "strPlayer": "Jesse Lingard",
                        "strSport": "Basketball",
                        "strCutout": "https://img.example.com/wrong-sport.png",
                    },
                    {
                        "strPlayer": "Jesse Lingard",
                        "strSport": "Soccer",
                        "strThumb": "https://img.example.com/lingard-thumb.jpg",
                        "strCutout": "https://img.example.com/lingard-cutout.png",
                    },
                ]
            },
            "Stefan Mugoša": {
                "player": [
                    {
                        "strPlayer": "Stefan Mugosa",
                        "strSport": "Soccer",
                        "strThumb": "https://img.example.com/mugosa-thumb.jpg",
                        "strCutout": None,
                    }
                ]
            },
        }
    )
    discoverer = AvatarDiscoverer(session=session)

    lingard = DirectorCandidate("k1-jesse-lingard", "林加德", "英格兰", "K1", "首尔FC")
    mugosa = DirectorCandidate("k1-stefan-mugosa", "斯特凡·穆戈沙", "黑山", "K1", "仁川联")

    assert discoverer.discover(lingard) == "https://img.example.com/lingard-cutout.png"
    assert discoverer.discover(mugosa) == "https://img.example.com/mugosa-thumb.jpg"


def test_wikipedia_fallback_accepts_football_and_rejects_non_football() -> None:
    session = FakeSession(
        sportsdb={},
        wiki={
            "Jasir_Asani": (
                200,
                {
                    "type": "standard",
                    "description": "Albanian professional footballer",
                    "extract": "Jasir Asani plays as a winger and forward.",
                    "thumbnail": {"source": "https://upload.wikimedia.org/asani-thumb.jpg"},
                    "originalimage": {"source": "https://upload.wikimedia.org/asani-orig.jpg"},
                },
            ),
            "Cesinha_%28footballer%2C_born_1989%29": (
                200,
                {
                    "type": "standard",
                    "description": "Brazilian football midfielder",
                    "extract": "Cesar Fernando Silva dos Santos.",
                    "originalimage": {"source": "https://upload.wikimedia.org/cesinha-orig.jpg"},
                },
            ),
            "Oberdan_%28footballer%2C_born_1995%29": (
                200,
                {
                    "type": "standard",
                    "description": "Italian political activist",
                    "extract": "Guglielmo Oberdan was an irredentist.",
                    "thumbnail": {"source": "https://upload.wikimedia.org/wrong-oberdan.jpg"},
                },
            ),
        },
    )
    discoverer = AvatarDiscoverer(session=session)

    asani = DirectorCandidate("k1-jasir-asani", "亚西尔·阿萨尼", "阿尔巴尼亚", "K1", "光州FC")
    cesinha = DirectorCandidate("k1-cesinha", "塞西尼亚", "巴西", "K1", "大邱FC")
    oberdan = DirectorCandidate("k1-oberdan", "奥贝尔丹", "巴西", "K1", "浦项铁人")

    assert discoverer.discover(asani) == "https://upload.wikimedia.org/asani-thumb.jpg"
    assert discoverer.discover(cesinha) == "https://upload.wikimedia.org/cesinha-orig.jpg"
    assert discoverer.discover(oberdan) is None

    wiki_calls = [call for call in session.calls if "/api/rest_v1/page/summary/" in call[0]]
    assert wiki_calls and all(call[2] and "User-Agent" in call[2] for call in wiki_calls)


def test_upload_writes_original_and_96_240_thumbnails_and_updates_db() -> None:
    png_bytes = make_image_bytes((200, 300), "PNG")
    cutout_url = "https://img.example.com/lingard-cutout.png"
    candidate = DirectorCandidate("k1-jesse-lingard", "林加德", "英格兰", "K1", "首尔FC")
    store = FakeStore([candidate])
    session = FakeSession(
        sportsdb={
            "Jesse Lingard": {
                "player": [
                    {
                        "strPlayer": "Jesse Lingard",
                        "strSport": "Soccer",
                        "strCutout": cutout_url,
                    }
                ]
            }
        }
    )
    fetcher = FakeFetcher({cutout_url: (png_bytes, "image/png; charset=binary")})
    target = FakeTarget()

    summary = sync_director_avatars(
        store=store,
        prefix="summary",
        fetcher=fetcher,
        target=target,
        discoverer=AvatarDiscoverer(session=session),
        interval=0,
    )

    assert summary == {
        "candidates": 1,
        "skipped": 0,
        "reused_from_squad": 0,
        "uploaded": 1,
        "not_found": 0,
        "failed": 0,
    }
    uploaded_names = [item[0] for item in target.uploads]
    assert uploaded_names == [
        "summary/players/director-k1-jesse-lingard.png",
        "summary/players/thumbs/96/director-k1-jesse-lingard.png.jpg",
        "summary/players/thumbs/240/director-k1-jesse-lingard.png.jpg",
    ]
    assert target.uploads[0][2] == "image/png"
    assert target.uploads[1][2:] == ("image/jpeg", THUMB_CACHE_CONTROL)
    assert target.uploads[2][2:] == ("image/jpeg", THUMB_CACHE_CONTROL)
    assert store.updates == [
        ("k1-jesse-lingard", "https://cdn.example.com/football-insight/summary/players/director-k1-jesse-lingard.png")
    ]


def test_single_failure_does_not_abort_and_respects_skip_force_dry_run_limit_interval() -> None:
    jpg_bytes = make_image_bytes((240, 240), "JPEG")
    candidates = [
        DirectorCandidate(
            "k1-cesinha",
            "塞西尼亚",
            "巴西",
            "K1",
            "大邱FC",
            avatar_url="https://cdn.example.com/existing.png",
        ),
        DirectorCandidate("k1-jesse-lingard", "林加德", "英格兰", "K1", "首尔FC"),
        DirectorCandidate("k1-jasir-asani", "亚西尔·阿萨尼", "阿尔巴尼亚", "K1", "光州FC"),
        DirectorCandidate("k1-oberdan", "奥贝尔丹", "巴西", "K1", "浦项铁人"),
    ]
    session = FakeSession(
        sportsdb={
            "Cesinha": {
                "player": [{"strPlayer": "Cesinha", "strSport": "Soccer", "strThumb": "https://img/cesinha.jpg"}]
            },
            "Jesse Lingard": {
                "player": [{"strPlayer": "Jesse Lingard", "strSport": "Soccer", "strThumb": "https://img/bad.html"}]
            },
            "Jasir Asani": {
                "player": [{"strPlayer": "Jasir Asani", "strSport": "Soccer", "strThumb": "https://img/asani.jpg"}]
            },
        }
    )
    fetcher = FakeFetcher(
        {
            "https://img/cesinha.jpg": (jpg_bytes, "image/jpeg"),
            "https://img/bad.html": (b"<html>error</html>", "text/html"),
            "https://img/asani.jpg": (jpg_bytes, "image/jpeg"),
        }
    )
    target = FakeTarget()
    store = FakeStore(candidates)
    sleeps: list[float] = []

    summary = sync_director_avatars(
        store=store,
        prefix="summary",
        fetcher=fetcher,
        target=target,
        discoverer=AvatarDiscoverer(session=session),
        interval=0.25,
        sleep_fn=sleeps.append,
    )

    assert summary == {
        "candidates": 4,
        "skipped": 1,
        "reused_from_squad": 0,
        "uploaded": 1,
        "not_found": 1,
        "failed": 1,
    }
    assert sleeps == [0.25, 0.25]
    assert store.updates == [
        ("k1-jasir-asani", "https://cdn.example.com/football-insight/summary/players/director-k1-jasir-asani.jpg")
    ]

    # --dry-run + --force + --limit
    dry_store = FakeStore(candidates)
    dry_target = FakeTarget()
    dry_summary = sync_director_avatars(
        store=dry_store,
        prefix="summary",
        fetcher=FakeFetcher({}),
        target=dry_target,
        discoverer=AvatarDiscoverer(session=session),
        force=True,
        dry_run=True,
        limit=2,
        interval=0,
    )
    assert dry_summary["candidates"] == 2
    assert dry_summary["skipped"] == 0
    assert dry_summary["uploaded"] == 0
    assert dry_target.uploads == []
    assert dry_store.updates == []


def test_cli_registers_sync_director_avatars_command() -> None:
    runner = CliRunner()
    result = runner.invoke(app, ["sync-director-avatars", "--help"])
    assert result.exit_code == 0
    assert "--force" in result.output
    assert "--dry-run" in result.output
    assert "--limit" in result.output
    assert "--interval" in result.output
