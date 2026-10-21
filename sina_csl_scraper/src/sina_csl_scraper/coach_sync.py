"""Sync each team's current head coach into f_i_team_coaches (for transfer wishes).

Runs inside `sync-squads`. Each team succeeds or fails on its own: a failed fetch or a missing
manager leaves that team's coach untouched, so a provider hiccup never removes a coach.
"""
from __future__ import annotations

import io
import logging
import sys
from collections.abc import Callable
from typing import Any, Protocol

from PIL import Image, ImageOps

from .assets import AssetFetcher, AssetTarget
from .avatar_thumbs import upload_thumbnails
from .coach_portraits import COACH_AVATAR_OVERRIDES, CoachPortrait
from .models import TeamCoach

logger = logging.getLogger(__name__)

# 先让本队其他现任下课，再按自然键写入/换队；一位教练始终只有一行，coach_id 不变。
DEACTIVATE_OTHERS_SQL = """
UPDATE f_i_team_coaches SET active = FALSE, updated_at = NOW()
WHERE team_id = %s AND active AND coach_key <> %s
"""
UPSERT_SQL = """
INSERT INTO f_i_team_coaches (coach_key, coach_name, team_id, active)
VALUES (%s, %s, %s, TRUE)
ON CONFLICT (coach_key) DO UPDATE SET
    coach_name = EXCLUDED.coach_name,
    team_id = EXCLUDED.team_id,
    active = TRUE,
    updated_at = CASE
        WHEN (f_i_team_coaches.coach_name, f_i_team_coaches.team_id, f_i_team_coaches.active)
             IS DISTINCT FROM (EXCLUDED.coach_name, EXCLUDED.team_id, TRUE)
        THEN NOW() ELSE f_i_team_coaches.updated_at END
RETURNING coach_id, avatar_object_name
"""


class CoachStore(Protocol):
    def team_ids(self) -> list[int]: ...
    def apply(self, coach: TeamCoach) -> tuple[int, str | None]: ...
    def set_avatar(self, coach_id: int, source_url: str, object_name: str, storage_url: str) -> None: ...


class PostgresCoachStore:
    def __init__(self, connection) -> None:
        self.connection = connection

    def team_ids(self) -> list[int]:
        """只同步有现役名单的球队（即当季联赛球队）。"""
        return [row[0] for row in self.connection.execute(
            "SELECT DISTINCT team_id FROM f_i_squad_players WHERE active ORDER BY team_id"
        ).fetchall()]

    def apply(self, coach: TeamCoach) -> tuple[int, str | None]:
        with self.connection.transaction():
            self.connection.execute(DEACTIVATE_OTHERS_SQL, (coach.team_id, coach.coach_key))
            row = self.connection.execute(
                UPSERT_SQL, (coach.coach_key, coach.coach_name, coach.team_id)
            ).fetchone()
        return int(row[0]), row[1]

    def set_avatar(self, coach_id: int, source_url: str, object_name: str, storage_url: str) -> None:
        self.connection.execute(
            """UPDATE f_i_team_coaches SET avatar_source_url = %s, avatar_object_name = %s,
                   avatar_storage_url = %s, updated_at = NOW() WHERE coach_id = %s""",
            (source_url, object_name, storage_url, coach_id),
        )


def coach_avatar_object_name(prefix: str, portrait: CoachPortrait) -> str:
    # 放在 players/ 目录：小程序按该目录规则取 96/240 小图。
    return f"{prefix.strip('/')}/players/{portrait.filename}"


def crop_portrait(content: bytes, box: tuple[int, int, int, int]) -> bytes:
    with Image.open(io.BytesIO(content)) as source:
        image = ImageOps.exif_transpose(source).convert("RGB")
        out = io.BytesIO()
        image.crop(box).save(out, "JPEG", quality=90, optimize=True)
        return out.getvalue()


def make_coach_avatar_uploader(
    fetcher: AssetFetcher, target: AssetTarget, prefix: str,
) -> Callable[[CoachPortrait], tuple[str, str, str]]:
    def upload(portrait: CoachPortrait) -> tuple[str, str, str]:
        content, content_type = fetcher.fetch(portrait.source_url)
        if portrait.crop:
            content, content_type = crop_portrait(content, portrait.crop), "image/jpeg"
        object_name = coach_avatar_object_name(prefix, portrait)
        storage_url = target.upload_bytes(object_name, content, content_type)
        try:
            upload_thumbnails(target, object_name, content)
        except Exception:  # noqa: BLE001 - 小图是增强，失败不影响原图
            logger.warning("coach avatar thumbnails failed for %s", object_name, exc_info=True)
        return portrait.source_url, object_name, storage_url

    return upload


def sync_coaches(
    *, fetch: Callable[[int], TeamCoach | None], store: CoachStore, prefix: str,
    avatars: Callable[[CoachPortrait], tuple[str, str, str]] | None = None,
    portraits: dict[str, CoachPortrait] = COACH_AVATAR_OVERRIDES,
) -> dict[str, Any]:
    summary: dict[str, Any] = {"coaches": 0, "teams_without_coach": [], "teams_failed": [], "coach_avatars_uploaded": 0}
    for team_id in store.team_ids():
        try:
            coach = fetch(team_id)
        except Exception as error:
            print(f"[coaches] team {team_id} failed: {type(error).__name__}", file=sys.stderr)
            summary["teams_failed"].append(team_id)
            continue
        if coach is None:
            summary["teams_without_coach"].append(team_id)
            continue
        coach_id, stored_object = store.apply(coach)
        summary["coaches"] += 1
        portrait = portraits.get(coach.coach_key)
        if avatars is None or portrait is None or stored_object == coach_avatar_object_name(prefix, portrait):
            continue
        try:
            store.set_avatar(coach_id, *avatars(portrait))
            summary["coach_avatars_uploaded"] += 1
        except Exception as error:  # 单个头像失败不影响教练写库
            print(f"[coaches] avatar {coach.coach_key} failed: {type(error).__name__}", file=sys.stderr)
    return summary
