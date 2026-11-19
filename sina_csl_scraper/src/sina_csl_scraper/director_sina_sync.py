"""Sync real transfer candidates and official portraits from Sina Sports mobile API.

Fetches teams and squads across multiple leagues (ACL, ACL2, CL1, SPL, MLS, and
European top-5 leagues by default) via ``goal.sports.sina.cn``, uploads portraits
(plus 96/240px thumbnails) to MinIO, and upserts idempotently into
``f_i_director_candidates`` team by team for checkpoint-friendly execution.
"""
from __future__ import annotations

import logging
import re
import time
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from typing import Any, Protocol

from psycopg.types.json import Jsonb

from .assets import AssetFetcher, AssetTarget, _extension_for
from .avatar_thumbs import upload_thumbnails
from .client import SinaCslClient
from .models import SquadPlayer

logger = logging.getLogger(__name__)

# Default leagues: ACL, ACL2, CL1, SPL, MLS, Serie A, La Liga, Bundesliga, Premier League, Ligue 1
DEFAULT_LEAGUE_IDS: tuple[int, ...] = (328, 403, 214, 404, 405, 1, 2, 3, 4, 5)
DEFAULT_LEAGUES_STR = ",".join(str(lid) for lid in DEFAULT_LEAGUE_IDS)

VALID_POSITIONS: frozenset[str] = frozenset({"GK", "DF", "MF", "FW"})

# Strictly aligned with backend domestic/HK/MO/TW classification (case-insensitive)
DOMESTIC_COUNTRIES: frozenset[str] = frozenset(
    {
        "中国",
        "china",
        "chn",
        "中国香港",
        "香港",
        "hong kong",
        "中国澳门",
        "澳门",
        "macau",
        "中国台湾",
        "台湾",
        "中华台北",
        "taiwan",
        "chinese taipei",
    }
)

K1_CLUB_KEYWORDS: tuple[str, ...] = (
    "蔚山",
    "全北",
    "浦项",
    "光州",
    "首尔",
    "江原",
    "水原",
    "大邱",
    "仁川",
    "济州",
    "金泉",
    "大田",
    "安养",
    "城南",
    "釜山",
    "富川",
)

J1_CLUB_KEYWORDS: tuple[str, ...] = (
    "横滨",
    "川崎",
    "神户",
    "广岛",
    "浦和",
    "鹿岛",
    "大阪",
    "东京",
    "名古屋",
    "福冈",
    "町田",
    "柏太阳神",
    "清水",
    "京都",
    "新泻",
    "札幌",
    "仙台",
    "长崎",
)

# Fallback CSL club name keywords/exact names for filtering out CSL teams in ACL/ACL2
FALLBACK_CSL_EXACT_NAMES: frozenset[str] = frozenset(
    {
        "浙江",
        "浙江队",
        "河南",
        "河南队",
        "广州",
        "广州队",
    }
)

FALLBACK_CSL_KEYWORDS: tuple[str, ...] = (
    "上海海港",
    "上海申花",
    "山东泰山",
    "北京国安",
    "成都蓉城",
    "天津津门虎",
    "武汉三镇",
    "长春亚泰",
    "青岛海牛",
    "青岛西海岸",
    "深圳新鹏城",
    "梅州客家",
    "云南玉昆",
    "大连英博",
    "浙江职业",
    "河南俱乐部",
)

EUROPEAN_LEAGUE_TAGS: dict[int, str] = {
    1: "意甲现役",
    2: "西甲现役",
    3: "德甲现役",
    4: "英超现役",
    5: "法甲现役",
}

_SINA_CODE_PID_RE = re.compile(r"^sina-[a-z0-9]+-(\d+)$")

UPSERT_CANDIDATE_SQL = """
INSERT INTO f_i_director_candidates (
    code, player_name, position, country, is_foreign,
    league_tag, current_club, tags, target_team_id,
    avatar_url, sort_order, active, updated_at
) VALUES (
    %(code)s, %(player_name)s, %(position)s, %(country)s, %(is_foreign)s,
    %(league_tag)s, %(current_club)s, %(tags)s, %(target_team_id)s,
    %(avatar_url)s, %(sort_order)s, %(active)s, NOW()
)
ON CONFLICT (code) DO UPDATE SET
    player_name = EXCLUDED.player_name,
    position = EXCLUDED.position,
    country = EXCLUDED.country,
    is_foreign = EXCLUDED.is_foreign,
    league_tag = EXCLUDED.league_tag,
    current_club = EXCLUDED.current_club,
    tags = EXCLUDED.tags,
    target_team_id = COALESCE(f_i_director_candidates.target_team_id, EXCLUDED.target_team_id),
    avatar_url = COALESCE(EXCLUDED.avatar_url, f_i_director_candidates.avatar_url),
    sort_order = EXCLUDED.sort_order,
    active = EXCLUDED.active,
    updated_at = NOW()
"""

BACKFILL_SEED_AVATARS_SQL = """
WITH avatar_sources AS (
    SELECT player_name, avatar_url, 1 AS priority, updated_at AS ts
    FROM f_i_director_candidates
    WHERE avatar_url IS NOT NULL AND BTRIM(avatar_url) <> ''
    UNION ALL
    SELECT player_name, avatar_storage_url AS avatar_url, 2 AS priority, synced_at AS ts
    FROM f_i_squad_players
    WHERE avatar_storage_url IS NOT NULL AND BTRIM(avatar_storage_url) <> ''
),
best_avatars AS (
    SELECT DISTINCT ON (player_name) player_name, avatar_url
    FROM avatar_sources
    ORDER BY player_name, priority ASC, ts DESC NULLS LAST
)
UPDATE f_i_director_candidates AS target
SET avatar_url = best_avatars.avatar_url,
    updated_at = NOW()
FROM best_avatars
WHERE target.player_name = best_avatars.player_name
  AND (target.avatar_url IS NULL OR BTRIM(target.avatar_url) = '')
"""

DEACTIVATE_NO_AVATAR_SQL = """
UPDATE f_i_director_candidates
SET active = FALSE, updated_at = NOW()
WHERE active = TRUE
  AND (avatar_url IS NULL OR BTRIM(avatar_url) = '')
"""


@dataclass(frozen=True)
class LeagueTeam:
    team_id: int
    team_name: str


@dataclass(frozen=True)
class TeamClassification:
    league_tag: str
    sort_order: int
    tags: list[str]


class SinaDirectorClient(Protocol):
    def fetch_league_meta(self, lid: int) -> tuple[int, str]: ...
    def fetch_standings_raw(self, lid: int, season: int) -> Any: ...
    def fetch_schedule_round_raw(self, lid: int, season: int, round_number: int = 1) -> Any: ...
    def fetch_team_squad(self, team_id: int, season: int | None = None) -> list[SquadPlayer]: ...


class DefaultSinaDirectorClient:
    """Thin adapter around SinaCslClient for multi-league director candidate syncing."""

    def __init__(self, client: SinaCslClient | None = None) -> None:
        self.client = client or SinaCslClient()

    def fetch_league_meta(self, lid: int) -> tuple[int, str]:
        data = self.client._request("/op/api/league", {"lid": lid})
        league = data.get("league") if isinstance(data, dict) else None
        if not isinstance(league, dict):
            raise ValueError(f"Unexpected /op/api/league payload for lid={lid}")
        cur_season = _parse_positive_int(league.get("cur_season"))
        if cur_season is None:
            raise ValueError(f"Missing cur_season in /op/api/league payload for lid={lid}")
        name = str(league.get("name") or "").strip()
        return cur_season, name

    def fetch_standings_raw(self, lid: int, season: int) -> Any:
        return self.client._request("/op/api/standings", {"lid": lid, "season": season})

    def fetch_schedule_round_raw(self, lid: int, season: int, round_number: int = 1) -> Any:
        return self.client._request(
            "/li/api/schedule/round",
            {"lid": lid, "season": season, "round": round_number},
        )

    def fetch_team_squad(self, team_id: int, season: int | None = None) -> list[SquadPlayer]:
        return self.client.fetch_team_squad(team_id, season=season)


class DirectorSinaStore(Protocol):
    def load_csl_teams(self) -> tuple[set[int], set[str]]: ...
    def load_existing_avatars(self) -> tuple[dict[str, str], dict[int, str]]: ...
    def upsert_candidates(self, rows: list[dict[str, Any]]) -> int: ...
    def backfill_seed_avatars(self) -> int: ...
    def deactivate_no_avatar(self) -> int: ...


class PostgresDirectorSinaStore:
    def __init__(self, connection) -> None:
        self.connection = connection

    def load_csl_teams(self) -> tuple[set[int], set[str]]:
        rows = self.connection.execute(
            "SELECT team_id, team_name FROM f_i_teams"
        ).fetchall()
        team_ids: set[int] = set()
        team_names: set[str] = set()
        for team_id, team_name in rows:
            if team_id is not None:
                team_ids.add(int(team_id))
            if team_name and str(team_name).strip():
                team_names.add(str(team_name).strip())
        return team_ids, team_names

    def load_existing_avatars(self) -> tuple[dict[str, str], dict[int, str]]:
        by_code: dict[str, str] = {}
        by_player_id: dict[int, str] = {}

        cand_rows = self.connection.execute(
            """
            SELECT code, avatar_url
            FROM f_i_director_candidates
            WHERE avatar_url IS NOT NULL AND BTRIM(avatar_url) <> ''
            """
        ).fetchall()
        for code, avatar_url in cand_rows:
            url = str(avatar_url).strip()
            if not url:
                continue
            code_str = str(code).strip()
            by_code[code_str] = url
            pid = extract_player_id_from_code(code_str)
            if pid is not None and pid not in by_player_id:
                by_player_id[pid] = url

        squad_rows = self.connection.execute(
            """
            SELECT player_id, avatar_storage_url
            FROM f_i_squad_players
            WHERE avatar_storage_url IS NOT NULL AND BTRIM(avatar_storage_url) <> ''
            """
        ).fetchall()
        for player_id, avatar_url in squad_rows:
            url = str(avatar_url).strip()
            if player_id is not None and url:
                pid = int(player_id)
                if pid not in by_player_id:
                    by_player_id[pid] = url

        return by_code, by_player_id

    def upsert_candidates(self, rows: list[dict[str, Any]]) -> int:
        if not rows:
            return 0
        payload = [{**row, "tags": Jsonb(row["tags"])} for row in rows]
        with self.connection.transaction():
            with self.connection.cursor() as cursor:
                cursor.executemany(UPSERT_CANDIDATE_SQL, payload)
        return len(rows)

    def backfill_seed_avatars(self) -> int:
        with self.connection.transaction():
            with self.connection.cursor() as cursor:
                cursor.execute(BACKFILL_SEED_AVATARS_SQL)
                return int(cursor.rowcount or 0)

    def deactivate_no_avatar(self) -> int:
        with self.connection.transaction():
            with self.connection.cursor() as cursor:
                cursor.execute(DEACTIVATE_NO_AVATAR_SQL)
                return int(cursor.rowcount or 0)


def _parse_positive_int(value: Any) -> int | None:
    if isinstance(value, bool) or value is None:
        return None
    if isinstance(value, int):
        return value if value > 0 else None
    text = str(value).strip()
    if text.isdigit():
        parsed = int(text)
        return parsed if parsed > 0 else None
    return None


def extract_player_id_from_code(code: str) -> int | None:
    match = _SINA_CODE_PID_RE.match(code.strip())
    return int(match.group(1)) if match else None


def parse_leagues_arg(raw: str) -> tuple[int, ...]:
    parts = [part.strip() for part in raw.split(",") if part.strip()]
    if not parts:
        raise ValueError("leagues cannot be empty")
    result: list[int] = []
    seen: set[int] = set()
    for part in parts:
        if not part.isdigit():
            raise ValueError(f"Invalid league id: {part!r}")
        lid = int(part)
        if lid not in seen:
            seen.add(lid)
            result.append(lid)
    return tuple(result)


def extract_teams_from_standings(data: Any) -> list[LeagueTeam]:
    """Recursively traverse standings JSON tree and extract unique (tid, team_name) pairs."""
    teams: list[LeagueTeam] = []
    seen: set[int] = set()

    def _walk(node: Any) -> None:
        if isinstance(node, dict):
            tid = _parse_positive_int(node.get("tid"))
            raw_name = node.get("team_name")
            name = raw_name.strip() if isinstance(raw_name, str) else ""
            if tid is not None and name and tid not in seen:
                seen.add(tid)
                teams.append(LeagueTeam(team_id=tid, team_name=name))
            for value in node.values():
                if isinstance(value, (dict, list)):
                    _walk(value)
        elif isinstance(node, list):
            for item in node:
                if isinstance(item, (dict, list)):
                    _walk(item)

    _walk(data)
    return teams


def extract_teams_from_schedule(data: Any) -> list[LeagueTeam]:
    """Recursively extract unique teams from schedule round JSON (supports Team1Id/Team1 and home_tid/home_name)."""
    teams: list[LeagueTeam] = []
    seen: set[int] = set()

    key_pairs = (
        ("Team1Id", "Team1"),
        ("Team2Id", "Team2"),
        ("home_tid", "home_name"),
        ("away_tid", "away_name"),
    )

    def _walk(node: Any) -> None:
        if isinstance(node, dict):
            for id_key, name_key in key_pairs:
                tid = _parse_positive_int(node.get(id_key))
                raw_name = node.get(name_key)
                name = raw_name.strip() if isinstance(raw_name, str) else ""
                if tid is not None and name and tid not in seen:
                    seen.add(tid)
                    teams.append(LeagueTeam(team_id=tid, team_name=name))
            for value in node.values():
                if isinstance(value, (dict, list)):
                    _walk(value)
        elif isinstance(node, list):
            for item in node:
                if isinstance(item, (dict, list)):
                    _walk(item)

    _walk(data)
    return teams


def is_csl_team(
    team_id: int,
    team_name: str,
    csl_team_ids: set[int],
    csl_team_names: set[str],
) -> bool:
    if team_id in csl_team_ids:
        return True
    cleaned = team_name.strip()
    if cleaned in csl_team_names or cleaned in FALLBACK_CSL_EXACT_NAMES:
        return True
    return any(keyword in cleaned for keyword in FALLBACK_CSL_KEYWORDS)


def classify_team(lid: int, team_name: str, league_name: str = "") -> TeamClassification:
    cleaned_name = team_name.strip()
    if lid in (328, 403):
        if any(keyword in cleaned_name for keyword in K1_CLUB_KEYWORDS):
            return TeamClassification(league_tag="K1", sort_order=40, tags=["K1现役", "亚冠"])
        if any(keyword in cleaned_name for keyword in J1_CLUB_KEYWORDS):
            return TeamClassification(league_tag="J1", sort_order=50, tags=["J1现役", "亚冠"])
        return TeamClassification(league_tag="ACL", sort_order=70, tags=["亚冠现役"])
    if lid == 214:
        return TeamClassification(league_tag="CL1", sort_order=60, tags=["中甲现役"])
    if lid == 404:
        return TeamClassification(league_tag="SPL", sort_order=90, tags=["沙特联现役"])
    if lid == 405:
        return TeamClassification(league_tag="MLS", sort_order=100, tags=["美职联现役"])
    if lid in EUROPEAN_LEAGUE_TAGS:
        return TeamClassification(league_tag="EUR", sort_order=110, tags=[EUROPEAN_LEAGUE_TAGS[lid]])
    cleaned_league = league_name.strip()
    tag = f"{cleaned_league}现役" if cleaned_league else "海外现役"
    return TeamClassification(league_tag="OTHER", sort_order=120, tags=[tag])


def is_foreign_country(country: str | None) -> bool:
    cleaned = (country or "").strip()
    if not cleaned:
        return False
    return cleaned.lower() not in DOMESTIC_COUNTRIES


def _is_reusable_minio_url(url: str | None, public_base_url: str | None) -> bool:
    if not url or not url.strip():
        return False
    if not public_base_url:
        return True
    prefix = public_base_url.rstrip("/") + "/"
    return url.strip().startswith(prefix)


def _upload_director_avatar(
    fetcher: AssetFetcher,
    target: AssetTarget,
    prefix: str,
    player_id: int,
    source_url: str,
) -> str:
    content, content_type = fetcher.fetch(source_url)
    ext = _extension_for(source_url, content_type)
    object_name = f"{prefix.strip('/')}/players/sina-{player_id}{ext}"
    storage_url = target.upload_bytes(object_name, content, content_type)
    try:
        upload_thumbnails(target, object_name, content)
    except Exception:  # noqa: BLE001 - 缩略图为增强能力，失败不影响原图入库
        logger.warning("avatar thumbnails failed for %s", object_name, exc_info=True)
    return storage_url


def sync_director_sina(
    *,
    client: SinaDirectorClient,
    store: DirectorSinaStore,
    leagues: Sequence[int] = DEFAULT_LEAGUE_IDS,
    season: int | None = None,
    include_acl: bool = True,
    include_no_avatar: bool = False,
    deactivate_no_avatar: bool = False,
    force_avatars: bool = False,
    dry_run: bool = False,
    limit_teams: int | None = None,
    interval: float = 0.2,
    fetcher: AssetFetcher | None = None,
    target: AssetTarget | None = None,
    prefix: str = "summary",
    public_base_url: str | None = None,
    sleep: Callable[[float], None] = time.sleep,
) -> dict[str, Any]:
    """Sync multi-league squads and portraits from Sina Sports into f_i_director_candidates."""
    csl_team_ids, csl_team_names = store.load_csl_teams()
    existing_by_code, existing_by_player_id = store.load_existing_avatars()

    resolved_public_base = public_base_url or getattr(target, "public_base_url", None)

    league_summaries: list[dict[str, Any]] = []
    teams_scanned = 0
    teams_failed = 0
    players_upserted = 0
    avatars_uploaded = 0
    avatars_reused = 0
    limit_reached = False

    def _pause() -> None:
        if interval > 0:
            sleep(interval)

    for lid in leagues:
        if limit_reached:
            break

        try:
            cur_season, league_name = client.fetch_league_meta(lid)
        except Exception as error:  # noqa: BLE001
            logger.warning("Failed to fetch league meta for lid=%s: %s", lid, error)
            league_summaries.append({"lid": lid, "season": season, "teams": 0, "error": str(error)})
            _pause()
            continue

        target_season = season if season is not None else cur_season
        _pause()

        teams: list[LeagueTeam] = []
        try:
            standings_data = client.fetch_standings_raw(lid, target_season)
            teams = extract_teams_from_standings(standings_data)
        except Exception as error:  # noqa: BLE001
            logger.warning("Standings fetch failed for lid=%s season=%s: %s", lid, target_season, error)
        _pause()

        if not teams:
            try:
                schedule_data = client.fetch_schedule_round_raw(lid, target_season, round_number=1)
                teams = extract_teams_from_schedule(schedule_data)
            except Exception as error:  # noqa: BLE001
                logger.warning("Schedule fallback failed for lid=%s season=%s: %s", lid, target_season, error)
            _pause()

        league_teams_processed = 0
        for team in teams:
            if limit_teams is not None and teams_scanned >= limit_teams:
                limit_reached = True
                break

            if is_csl_team(team.team_id, team.team_name, csl_team_ids, csl_team_names):
                continue

            classification = classify_team(lid, team.team_name, league_name=league_name)
            if not include_acl and lid in (328, 403) and classification.league_tag == "ACL":
                continue

            teams_scanned += 1
            league_teams_processed += 1

            try:
                squad = client.fetch_team_squad(team.team_id, season=target_season)
            except Exception as error:  # noqa: BLE001
                logger.warning(
                    "Squad fetch failed for lid=%s team_id=%s (%s): %s",
                    lid,
                    team.team_id,
                    team.team_name,
                    error,
                )
                teams_failed += 1
                _pause()
                continue
            _pause()

            team_rows: list[dict[str, Any]] = []
            seen_codes_in_team: set[str] = set()
            for player in squad:
                player_name = (player.player_name or "").strip()
                if not player_name or player.position not in VALID_POSITIONS:
                    continue

                code = f"sina-{classification.league_tag.lower()}-{player.player_id}"
                if code in seen_codes_in_team:
                    continue
                seen_codes_in_team.add(code)

                avatar_url: str | None = None
                reused_url: str | None = None
                if not force_avatars:
                    candidate_url = existing_by_code.get(code) or existing_by_player_id.get(player.player_id)
                    if _is_reusable_minio_url(candidate_url, resolved_public_base):
                        reused_url = candidate_url.strip() if candidate_url else None

                if reused_url:
                    avatar_url = reused_url
                    avatars_reused += 1
                    existing_by_code[code] = reused_url
                    existing_by_player_id[player.player_id] = reused_url
                elif player.avatar_source_url:
                    if dry_run:
                        avatar_url = player.avatar_source_url
                    elif fetcher is not None and target is not None:
                        try:
                            uploaded_url = _upload_director_avatar(
                                fetcher=fetcher,
                                target=target,
                                prefix=prefix,
                                player_id=player.player_id,
                                source_url=player.avatar_source_url,
                            )
                            avatar_url = uploaded_url
                            avatars_uploaded += 1
                            existing_by_code[code] = uploaded_url
                            existing_by_player_id[player.player_id] = uploaded_url
                        except Exception as error:  # noqa: BLE001
                            logger.warning(
                                "Avatar upload failed for player_id=%s (%s): %s",
                                player.player_id,
                                player_name,
                                error,
                            )
                            avatar_url = None

                if not include_no_avatar and not avatar_url:
                    continue

                country = (player.country or "").strip()
                team_rows.append(
                    {
                        "code": code,
                        "player_name": player_name,
                        "position": player.position,
                        "country": country,
                        "is_foreign": is_foreign_country(country),
                        "league_tag": classification.league_tag,
                        "current_club": team.team_name,
                        "tags": list(classification.tags),
                        "target_team_id": None,
                        "avatar_url": avatar_url,
                        "sort_order": classification.sort_order,
                        "active": True,
                    }
                )

            if team_rows:
                if dry_run:
                    players_upserted += len(team_rows)
                else:
                    players_upserted += store.upsert_candidates(team_rows)

        league_summaries.append(
            {
                "lid": lid,
                "season": target_season,
                "teams": league_teams_processed,
            }
        )

    seed_backfilled = 0
    deactivated_count = 0
    if not dry_run:
        seed_backfilled = store.backfill_seed_avatars()
        if deactivate_no_avatar:
            deactivated_count = store.deactivate_no_avatar()

    return {
        "leagues": league_summaries,
        "teams_scanned": teams_scanned,
        "teams_failed": teams_failed,
        "players_upserted": players_upserted,
        "avatars_uploaded": avatars_uploaded,
        "avatars_reused": avatars_reused,
        "seed_backfilled": seed_backfilled,
        "deactivated_no_avatar": deactivated_count,
    }
