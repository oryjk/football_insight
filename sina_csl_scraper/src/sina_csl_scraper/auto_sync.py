from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from datetime import datetime, timedelta
from pathlib import Path
from typing import Iterable

from zoneinfo import ZoneInfo

from .models import MatchResult

SHANGHAI_TZ = ZoneInfo("Asia/Shanghai")


@dataclass(frozen=True)
class AutoSyncState:
    last_processed_due_at: datetime | None = None
    last_run_at: datetime | None = None
    last_run_match_ids: tuple[int, ...] = ()


@dataclass(frozen=True)
class AutoSyncDecision:
    should_run: bool
    latest_due_at: datetime | None
    newly_due_match_ids: tuple[int, ...]
    active_match_ids: tuple[int, ...] = ()


def resolve_match_kickoff(match: MatchResult) -> datetime | None:
    date = match.date.strip()
    time = match.time.strip()
    if not date or not time:
        return None

    normalized_time = f"{time}:00" if len(time) == 5 else time
    try:
        return datetime.fromisoformat(f"{date}T{normalized_time}+08:00").astimezone(SHANGHAI_TZ)
    except ValueError:
        return None


def resolve_match_due_at(
    match: MatchResult,
    *,
    match_duration_minutes: int = 120,
    post_finish_delay_minutes: int = 10,
) -> datetime | None:
    kickoff = resolve_match_kickoff(match)
    if kickoff is None:
        return None

    return kickoff + timedelta(minutes=match_duration_minutes + post_finish_delay_minutes)


def build_auto_sync_decision(
    matches: Iterable[MatchResult],
    *,
    now: datetime,
    state: AutoSyncState,
    match_duration_minutes: int = 120,
    post_finish_delay_minutes: int = 10,
    active_sync_interval_minutes: int = 1,
) -> AutoSyncDecision:
    now_at = now.astimezone(SHANGHAI_TZ)
    due_matches: list[tuple[int, datetime]] = []
    active_matches: list[int] = []

    for match in matches:
        due_at = resolve_match_due_at(
            match,
            match_duration_minutes=match_duration_minutes,
            post_finish_delay_minutes=post_finish_delay_minutes,
        )
        kickoff = resolve_match_kickoff(match)
        if kickoff is not None and due_at is not None and kickoff <= now_at < due_at:
            active_matches.append(match.match_id)

        if due_at is None or due_at > now_at:
            continue
        due_matches.append((match.match_id, due_at))

    active_match_ids = tuple(sorted(set(active_matches)))
    last_run_at = state.last_run_at.astimezone(SHANGHAI_TZ) if state.last_run_at else None
    active_sync_due = bool(active_match_ids) and (
        last_run_at is None
        or now_at - last_run_at >= timedelta(minutes=active_sync_interval_minutes)
    )

    if not due_matches and not active_sync_due:
        return AutoSyncDecision(
            should_run=False,
            latest_due_at=None,
            newly_due_match_ids=(),
            active_match_ids=active_match_ids,
        )

    latest_due_at = max((due_at for _, due_at in due_matches), default=None)
    last_processed_due_at = state.last_processed_due_at.astimezone(SHANGHAI_TZ) if state.last_processed_due_at else None
    newly_due_match_ids = tuple(
        match_id
        for match_id, due_at in sorted(due_matches, key=lambda item: (item[1], item[0]))
        if last_processed_due_at is None or due_at > last_processed_due_at
    )

    return AutoSyncDecision(
        should_run=bool(newly_due_match_ids) or active_sync_due,
        latest_due_at=latest_due_at,
        newly_due_match_ids=newly_due_match_ids,
        active_match_ids=active_match_ids,
    )


def load_auto_sync_state(path: Path) -> AutoSyncState:
    if not path.exists():
        return AutoSyncState()

    payload = json.loads(path.read_text(encoding="utf-8"))

    def parse_datetime(value: str | None) -> datetime | None:
        if not value:
            return None
        return datetime.fromisoformat(value)

    last_run_match_ids = payload.get("last_run_match_ids", [])
    return AutoSyncState(
        last_processed_due_at=parse_datetime(payload.get("last_processed_due_at")),
        last_run_at=parse_datetime(payload.get("last_run_at")),
        last_run_match_ids=tuple(int(item) for item in last_run_match_ids),
    )


def save_auto_sync_state(path: Path, state: AutoSyncState) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = asdict(state)
    payload["last_processed_due_at"] = state.last_processed_due_at.isoformat() if state.last_processed_due_at else None
    payload["last_run_at"] = state.last_run_at.isoformat() if state.last_run_at else None
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")
