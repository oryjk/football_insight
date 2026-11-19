"""Backfill player ages (as approximate Jan-1 birth_date) from Sina /op/api/player.

Sina's squad listing endpoint (``/op/api/team/player``) does not return age or
birth_date, whereas the player detail endpoint (``/op/api/player?pid=<pid>``)
returns an integer ``age`` for nearly all CSL squad players and Sina-synced
director candidates (whose ``code`` is ``sina-<league>-<pid>``).

To avoid requiring a schema change and to preserve exact Transfermarkt birth
dates, this module only populates rows where ``birth_date IS NULL`` using
``date(today.year - age, 1, 1)``. Exact birth dates synced later from
Transfermarkt cleanly overwrite this approximate date.
"""

from __future__ import annotations

from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from datetime import date
import logging
import re
import threading
from typing import Any, Protocol

from .client import SinaCslClient
from .constants import DEFAULT_BASE_URL, DEFAULT_TIMEOUT

logger = logging.getLogger(__name__)

MIN_PLAYER_AGE = 14
MAX_PLAYER_AGE = 55

_SINA_CANDIDATE_CODE_RE = re.compile(r"^sina-[a-z0-9_]+-(\d+)$", re.IGNORECASE)

_SELECT_SQUAD_MISSING_BIRTH_SQL = """
SELECT DISTINCT player_id
FROM f_i_squad_players
WHERE birth_date IS NULL
ORDER BY player_id
"""

_SELECT_CANDIDATES_MISSING_BIRTH_SQL = """
SELECT id, code
FROM f_i_director_candidates
WHERE active = TRUE AND birth_date IS NULL
ORDER BY id
"""

_UPDATE_SQUAD_BIRTH_IF_NULL_SQL = """
UPDATE f_i_squad_players
SET birth_date = %(birth_date)s
WHERE player_id = %(player_id)s
  AND birth_date IS NULL
"""

_UPDATE_CANDIDATE_BIRTH_IF_NULL_SQL = """
UPDATE f_i_director_candidates
SET birth_date = %(birth_date)s,
    updated_at = NOW()
WHERE id = %(id)s
  AND birth_date IS NULL
"""


@dataclass(frozen=True)
class CandidateAgeTarget:
    id: int
    code: str
    sina_player_id: int


class SinaAgeStore(Protocol):
    def load_squad_missing_birth_ids(self) -> list[int]: ...

    def load_candidate_missing_birth_targets(self) -> list[CandidateAgeTarget]: ...

    def update_squad_birth_date_if_null(self, player_id: int, birth_date: date) -> int: ...

    def update_candidate_birth_date_if_null(self, candidate_id: int, birth_date: date) -> int: ...


class PostgresSinaAgeStore:
    def __init__(self, connection: Any) -> None:
        self.connection = connection

    def load_squad_missing_birth_ids(self) -> list[int]:
        with self.connection.cursor() as cursor:
            cursor.execute(_SELECT_SQUAD_MISSING_BIRTH_SQL)
            return [int(row[0]) for row in cursor.fetchall()]

    def load_candidate_missing_birth_targets(self) -> list[CandidateAgeTarget]:
        targets: list[CandidateAgeTarget] = []
        with self.connection.cursor() as cursor:
            cursor.execute(_SELECT_CANDIDATES_MISSING_BIRTH_SQL)
            for row in cursor.fetchall():
                cid = int(row[0])
                code = str(row[1] or "")
                pid = extract_sina_player_id_from_candidate_code(code)
                if pid is not None:
                    targets.append(CandidateAgeTarget(id=cid, code=code, sina_player_id=pid))
        return targets

    def update_squad_birth_date_if_null(self, player_id: int, birth_date: date) -> int:
        with self.connection.cursor() as cursor:
            cursor.execute(
                _UPDATE_SQUAD_BIRTH_IF_NULL_SQL,
                {"player_id": player_id, "birth_date": birth_date},
            )
            return int(cursor.rowcount or 0)

    def update_candidate_birth_date_if_null(self, candidate_id: int, birth_date: date) -> int:
        with self.connection.cursor() as cursor:
            cursor.execute(
                _UPDATE_CANDIDATE_BIRTH_IF_NULL_SQL,
                {"id": candidate_id, "birth_date": birth_date},
            )
            return int(cursor.rowcount or 0)


def extract_sina_player_id_from_candidate_code(code: str | None) -> int | None:
    if not code:
        return None
    match = _SINA_CANDIDATE_CODE_RE.match(code.strip())
    if not match:
        return None
    pid = int(match.group(1))
    return pid if pid > 0 else None


def parse_player_age(profile: dict[str, Any] | None) -> int | None:
    if not isinstance(profile, dict):
        return None
    raw = str(profile.get("age") or "").strip()
    if not raw.isdigit():
        return None
    age = int(raw)
    if MIN_PLAYER_AGE <= age <= MAX_PLAYER_AGE:
        return age
    return None


def approximate_birth_date_from_age(age: int, today: date | None = None) -> date:
    if not (MIN_PLAYER_AGE <= age <= MAX_PLAYER_AGE):
        raise ValueError(f"Age out of valid player range: {age}")
    ref = today or date.today()
    return date(ref.year - age, 1, 1)


def _make_thread_local_profile_fetcher(
    base_url: str = DEFAULT_BASE_URL,
    timeout: int = DEFAULT_TIMEOUT,
) -> Callable[[int], dict[str, Any] | None]:
    local = threading.local()

    def _fetch(player_id: int) -> dict[str, Any] | None:
        client: SinaCslClient | None = getattr(local, "client", None)
        if client is None:
            client = SinaCslClient(base_url=base_url, timeout=timeout)
            local.client = client
        return client.fetch_player_profile(player_id)

    return _fetch


def sync_director_sina_ages(
    store: SinaAgeStore,
    *,
    profile_fetcher: Callable[[int], dict[str, Any] | None] | None = None,
    dry_run: bool = True,
    only_squad: bool = False,
    only_candidates: bool = False,
    limit: int | None = None,
    concurrency: int = 8,
    timeout: int = DEFAULT_TIMEOUT,
    today: date | None = None,
) -> dict[str, Any]:
    """Backfill missing ``birth_date`` values from Sina's ``/op/api/player`` age field."""
    squad_ids = [] if only_candidates else store.load_squad_missing_birth_ids()
    candidate_targets = [] if only_squad else store.load_candidate_missing_birth_targets()

    squad_id_set = set(squad_ids)
    candidates_by_pid: dict[int, list[CandidateAgeTarget]] = {}
    for target in candidate_targets:
        candidates_by_pid.setdefault(target.sina_player_id, []).append(target)

    # Preserve deterministic order: CSL squad players first, then director candidates.
    unique_pids: list[int] = []
    seen_pids: set[int] = set()
    for pid in squad_ids:
        if pid not in seen_pids:
            seen_pids.add(pid)
            unique_pids.append(pid)
    for target in candidate_targets:
        if target.sina_player_id not in seen_pids:
            seen_pids.add(target.sina_player_id)
            unique_pids.append(target.sina_player_id)

    if limit is not None and limit >= 0:
        unique_pids = unique_pids[:limit]

    fetcher = profile_fetcher or _make_thread_local_profile_fetcher(timeout=timeout)
    workers = max(1, min(concurrency, 16))
    ref_today = today or date.today()

    fetched_ok = 0
    fetched_failed = 0
    ages_by_pid: dict[int, int] = {}
    samples: list[dict[str, Any]] = []

    def _fetch_one(pid: int) -> tuple[int, dict[str, Any] | None, str | None]:
        try:
            return pid, fetcher(pid), None
        except Exception as error:  # noqa: BLE001
            return pid, None, str(error)

    if workers == 1 or len(unique_pids) <= 1:
        results = [_fetch_one(pid) for pid in unique_pids]
    else:
        results = []
        with ThreadPoolExecutor(max_workers=workers) as executor:
            futures = [executor.submit(_fetch_one, pid) for pid in unique_pids]
            for future in as_completed(futures):
                results.append(future.result())

    for pid, profile, error in results:
        if error is not None:
            fetched_failed += 1
            logger.warning("Failed to fetch Sina player profile pid=%s: %s", pid, error)
            continue
        fetched_ok += 1
        age = parse_player_age(profile)
        if age is not None:
            ages_by_pid[pid] = age
            if len(samples) < 10 and isinstance(profile, dict):
                samples.append(
                    {
                        "pid": pid,
                        "name": profile.get("nam_cn") or profile.get("name_cn") or profile.get("name"),
                        "age": age,
                        "birth_date": approximate_birth_date_from_age(age, today=ref_today).isoformat(),
                    }
                )

    squad_matched = 0
    squad_updated = 0
    candidate_matched = 0
    candidate_updated = 0

    for pid, age in sorted(ages_by_pid.items()):
        approx_birth = approximate_birth_date_from_age(age, today=ref_today)
        if pid in squad_id_set:
            squad_matched += 1
            if not dry_run:
                squad_updated += store.update_squad_birth_date_if_null(pid, approx_birth)
        for target in candidates_by_pid.get(pid, ()):
            candidate_matched += 1
            if not dry_run:
                candidate_updated += store.update_candidate_birth_date_if_null(target.id, approx_birth)

    return {
        "dry_run": dry_run,
        "squad_targets": len(squad_ids),
        "candidate_targets": len(candidate_targets),
        "unique_pids_to_fetch": len(unique_pids),
        "fetched_ok": fetched_ok,
        "fetched_failed": fetched_failed,
        "ages_found": len(ages_by_pid),
        "squad_matched": squad_matched,
        "squad_updated": squad_updated,
        "candidate_matched": candidate_matched,
        "candidate_updated": candidate_updated,
        "samples": samples,
    }
