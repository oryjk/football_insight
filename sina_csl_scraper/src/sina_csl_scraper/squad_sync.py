"""Sync full team squads (current + past seasons) into f_i_squad_players.

Each team succeeds or fails on its own: a failed or empty current-season fetch leaves that
team's rows untouched, so a provider hiccup can never mark a whole squad as departed.
"""
from __future__ import annotations

import sys
from collections.abc import Callable
from typing import Any, Protocol

from .assets import AssetFetcher, AssetTarget, upload_player_avatar
from .models import SquadPlayer

UPSERT_CURRENT_SQL = """
INSERT INTO f_i_squad_players
    (team_id, player_id, player_name, shirt_number, position, country,
     avatar_source_url, avatar_storage_url, last_season, active, synced_at)
VALUES (%(team_id)s, %(player_id)s, %(player_name)s, %(shirt_number)s, %(position)s, %(country)s,
        %(avatar_source_url)s, %(avatar_storage_url)s, %(season)s, TRUE, NOW())
ON CONFLICT (team_id, player_id) DO UPDATE SET
    player_name = EXCLUDED.player_name,
    shirt_number = EXCLUDED.shirt_number,
    position = EXCLUDED.position,
    country = EXCLUDED.country,
    avatar_source_url = EXCLUDED.avatar_source_url,
    avatar_storage_url = COALESCE(EXCLUDED.avatar_storage_url, f_i_squad_players.avatar_storage_url),
    last_season = GREATEST(f_i_squad_players.last_season, EXCLUDED.last_season),
    active = TRUE,
    synced_at = NOW()
"""

UPSERT_HISTORY_SQL = """
INSERT INTO f_i_squad_players
    (team_id, player_id, player_name, shirt_number, position, country,
     avatar_source_url, avatar_storage_url, last_season, active, synced_at)
VALUES (%(team_id)s, %(player_id)s, %(player_name)s, %(shirt_number)s, %(position)s, %(country)s,
        %(avatar_source_url)s, %(avatar_storage_url)s, %(season)s, FALSE, NOW())
ON CONFLICT (team_id, player_id) DO UPDATE SET
    player_name = EXCLUDED.player_name,
    shirt_number = EXCLUDED.shirt_number,
    position = EXCLUDED.position,
    country = EXCLUDED.country,
    avatar_source_url = EXCLUDED.avatar_source_url,
    avatar_storage_url = COALESCE(EXCLUDED.avatar_storage_url, f_i_squad_players.avatar_storage_url),
    last_season = EXCLUDED.last_season,
    synced_at = NOW()
WHERE f_i_squad_players.last_season < EXCLUDED.last_season
"""

DEACTIVATE_MISSING_SQL = """
UPDATE f_i_squad_players SET active = FALSE, synced_at = NOW()
WHERE team_id = %s AND active AND NOT (player_id = ANY(%s))
"""


# 当前赛季名单的合理性下限：解析出的人数过少，或不到已知现役的一半，一律视为接口异常、不写库，
# 否则被截断或字段改名的名单会把真实现役球员批量标成离队。
MIN_CURRENT_SQUAD = 11
MIN_CURRENT_RATIO = 0.5


class SquadStore(Protocol):
    def team_ids(self) -> list[int]: ...
    def active_count(self, team_id: int) -> int: ...
    def avatar_urls(self, team_id: int) -> dict[int, str]: ...
    def apply(self, team_id: int, season: int, rows: list[dict[str, Any]], current: bool) -> None: ...


class PostgresSquadStore:
    def __init__(self, connection) -> None:
        self.connection = connection

    def team_ids(self) -> list[int]:
        return [row[0] for row in self.connection.execute("SELECT team_id FROM f_i_teams ORDER BY team_id").fetchall()]

    def active_count(self, team_id: int) -> int:
        return int(self.connection.execute(
            "SELECT COUNT(*) FROM f_i_squad_players WHERE team_id=%s AND active", (team_id,)
        ).fetchone()[0])

    def avatar_urls(self, team_id: int) -> dict[int, str]:
        """已转存到 MinIO 的头像：本表任意球队（转会球员复用）与旧头像同步写入的 f_i_players。

        对象名只由球员 id 决定（players/{pid}.*），所以同一球员在哪支队的记录都可复用。
        """
        rows = self.connection.execute(
            """
            SELECT player_id, avatar_storage_url FROM f_i_squad_players WHERE avatar_storage_url IS NOT NULL
            UNION
            SELECT player_id, avatar_storage_url FROM f_i_players WHERE avatar_storage_url IS NOT NULL
            """,
        ).fetchall()
        return {int(pid): url for pid, url in rows}

    def apply(self, team_id: int, season: int, rows: list[dict[str, Any]], current: bool) -> None:
        if not rows:
            raise ValueError("refusing to apply an empty squad")
        with self.connection.transaction():
            with self.connection.cursor() as cursor:
                cursor.executemany(UPSERT_CURRENT_SQL if current else UPSERT_HISTORY_SQL, rows)
                if current:
                    cursor.execute(DEACTIVATE_MISSING_SQL, (team_id, [row["player_id"] for row in rows]))


def build_squad_rows(players: list[SquadPlayer], season: int, storage_urls: dict[int, str]) -> list[dict[str, Any]]:
    return [
        {**player.to_dict(), "season": season, "avatar_storage_url": storage_urls.get(player.player_id)}
        for player in players
    ]


def make_avatar_uploader(fetcher: AssetFetcher, target: AssetTarget, prefix: str) -> Callable[[SquadPlayer], str | None]:
    def upload(player: SquadPlayer) -> str | None:
        uploaded = upload_player_avatar(
            fetcher, target, prefix, player.player_id, player.team_id, player.avatar_source_url,
        )
        return uploaded[2] if uploaded else None

    return upload


def _with_avatars(players: list[SquadPlayer], known: dict[int, str],
                  avatars: Callable[[SquadPlayer], str | None] | None, counter: list[int]) -> dict[int, str]:
    urls = dict(known)
    if avatars is None:
        return urls
    for player in players:
        if player.player_id in urls or not player.avatar_source_url:
            continue
        try:
            url = avatars(player)
        except Exception as error:  # 单个头像失败不影响该队写库
            print(f"[squads] avatar {player.player_id} failed: {type(error).__name__}", file=sys.stderr)
            continue
        if url:
            urls[player.player_id] = url
            counter[0] += 1
    return urls


def sync_squads(*, fetch: Callable[[int, int | None], list[SquadPlayer]], store: SquadStore,
                season: int, backfill_seasons: int,
                avatars: Callable[[SquadPlayer], str | None] | None = None) -> dict[str, Any]:
    summary: dict[str, Any] = {"teams_ok": 0, "teams_failed": [], "players_current": 0, "avatars_uploaded": 0}
    uploaded = [0]
    history = [season - offset for offset in range(max(backfill_seasons, 0), 0, -1)]
    for team_id in store.team_ids():
        known = store.avatar_urls(team_id)
        for past in history:
            try:
                players = fetch(team_id, past)
                if players:
                    known = _with_avatars(players, known, avatars, uploaded)
                    store.apply(team_id, past, build_squad_rows(players, past, known), current=False)
            except Exception as error:
                print(f"[squads] team {team_id} season {past} skipped: {type(error).__name__}", file=sys.stderr)
        try:
            players = fetch(team_id, season)
            if not players:
                raise ValueError("empty current squad")
            known_active = store.active_count(team_id)
            if len(players) < MIN_CURRENT_SQUAD or len(players) < known_active * MIN_CURRENT_RATIO:
                raise ValueError(f"suspicious current squad: {len(players)} players, {known_active} known active")
            known = _with_avatars(players, known, avatars, uploaded)
            store.apply(team_id, season, build_squad_rows(players, season, known), current=True)
            summary["teams_ok"] += 1
            summary["players_current"] += len(players)
        except Exception as error:
            print(f"[squads] team {team_id} current season failed: {type(error).__name__}", file=sys.stderr)
            summary["teams_failed"].append(team_id)
    summary["avatars_uploaded"] = uploaded[0]
    return summary
