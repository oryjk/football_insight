"""Scrape player market values, birth dates, and contract end dates from Transfermarkt.

Fetches league club lists and detailed club squad pages (``/kader/verein/<id>/plus/1``),
parses player valuations/dates with stdlib ``html.parser``, and idempotently backfills
``market_value_eur``, ``birth_date``, ``contract_until``, and ``transfermarkt_synced_at``
into ``f_i_squad_players`` (for CSL) and ``f_i_director_candidates`` (for all leagues).
"""
from __future__ import annotations

import csv
import gzip
import io
import logging
import re
import time
import unicodedata
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from datetime import date
from decimal import Decimal, InvalidOperation
from html.parser import HTMLParser
from pathlib import Path
from typing import Any, Protocol

import requests

logger = logging.getLogger(__name__)

DEFAULT_BASE_URL = "https://www.transfermarkt.com"
DEFAULT_DATASET_URL = "https://pub-e682421888d945d684bcae8890b0ec20.r2.dev/data/players.csv.gz"
DEFAULT_DATASET_CACHE_PATH = Path("data/transfermarkt-players.csv.gz")
DEFAULT_DELAY_SECONDS = 1.2

# Mapping from CLI league key -> Transfermarkt competition code
TRANSFERMARKT_LEAGUE_CODES: dict[str, str] = {
    "csl": "CSL",
    "k1": "RSK1",
    "j1": "JAP1",
    "epl": "GB1",
    "laliga": "ES1",
    "seriea": "IT1",
    "bundesliga": "L1",
    "ligue1": "FR1",
}

DEFAULT_TM_LEAGUES: tuple[str, ...] = tuple(TRANSFERMARKT_LEAGUE_CODES.keys())
DEFAULT_TM_LEAGUES_STR = ",".join(DEFAULT_TM_LEAGUES)

LEAGUE_KEY_TO_CANDIDATE_TAG: dict[str, str] = {
    "k1": "K1",
    "j1": "J1",
    "epl": "EUR",
    "laliga": "EUR",
    "seriea": "EUR",
    "bundesliga": "EUR",
    "ligue1": "EUR",
}

DEFAULT_HEADERS: dict[str, str] = {
    "User-Agent": (
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/131.0.0.0 Safari/537.36"
    ),
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
    "Accept-Language": "en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7",
    "Cache-Control": "no-cache",
    "Pragma": "no-cache",
    "Upgrade-Insecure-Requests": "1",
}

UPDATE_SQUAD_PLAYER_SQL = """
UPDATE f_i_squad_players
SET market_value_eur = COALESCE(%(market_value_eur)s, market_value_eur),
    birth_date = COALESCE(%(birth_date)s, birth_date),
    contract_until = COALESCE(%(contract_until)s, contract_until),
    transfermarkt_synced_at = NOW()
WHERE team_id = %(team_id)s AND player_id = %(player_id)s
"""

UPDATE_DIRECTOR_CANDIDATE_SQL = """
UPDATE f_i_director_candidates
SET market_value_eur = COALESCE(%(market_value_eur)s, market_value_eur),
    birth_date = COALESCE(%(birth_date)s, birth_date),
    contract_until = COALESCE(%(contract_until)s, contract_until),
    transfermarkt_synced_at = NOW()
WHERE id = %(id)s
"""

_CLUB_HREF_RE = re.compile(r"/(startseite|kader)/verein/(\d+)(?:/|$)")
_PLAYER_HREF_RE = re.compile(r"/profil/spieler/(\d+)(?:/|$)")
_AGE_SUFFIX_RE = re.compile(r"\s*\(\s*\d+\s*\)\s*$")
_DATE_ISO_RE = re.compile(r"^(\d{4})[-/](\d{1,2})[-/](\d{1,2})$")
_DATE_DMY_RE = re.compile(r"^(\d{1,2})[./-](\d{1,2})[./-](\d{4})$")
_DATE_ZH_RE = re.compile(r"^(\d{4})\s*年\s*(\d{1,2})\s*月\s*(\d{1,2})\s*日?$")
_DATE_EN_MDY_RE = re.compile(r"^([A-Za-z.]+)\s+(\d{1,2}),?\s+(\d{4})$")
_DATE_DMY_WORD_RE = re.compile(r"^(\d{1,2})\.?\s+([A-Za-zÄäÖöÜüß.]+)\s+(\d{4})$")
_NUMBER_TOKEN_RE = re.compile(r"(\d[\d.,]*)")
_NAME_PUNCT_RE = re.compile(r"[\s·•‧・.\-_''`\"()（）]+")
_CLUB_SUFFIX_RE = re.compile(r"(足球俱乐部|俱乐部|职业|足球队|fc|sc|cf|ac|afc|fk|sk|sv|vfb|tsg|rb)$")

_MONTH_MAP: dict[str, int] = {
    "jan": 1,
    "january": 1,
    "januar": 1,
    "feb": 2,
    "february": 2,
    "februar": 2,
    "mar": 3,
    "march": 3,
    "mär": 3,
    "märz": 3,
    "apr": 4,
    "april": 4,
    "may": 5,
    "mai": 5,
    "jun": 6,
    "june": 6,
    "juni": 6,
    "jul": 7,
    "july": 7,
    "juli": 7,
    "aug": 8,
    "august": 8,
    "sep": 9,
    "sept": 9,
    "september": 9,
    "oct": 10,
    "october": 10,
    "okt": 10,
    "oktober": 10,
    "nov": 11,
    "november": 11,
    "dec": 12,
    "december": 12,
    "dez": 12,
    "dezember": 12,
}


@dataclass(frozen=True)
class TransfermarktClub:
    club_id: int
    club_name: str
    href: str = ""


@dataclass(frozen=True)
class TransfermarktPlayer:
    player_name: str
    club_name: str
    league_key: str
    shirt_number: int | None = None
    birth_date: date | None = None
    contract_until: date | None = None
    market_value_eur: int | None = None
    tm_player_id: int | None = None
    country: str = ""
    position: str = ""


@dataclass(frozen=True)
class SquadPlayerRecord:
    team_id: int
    player_id: int
    player_name: str
    team_name: str = ""
    shirt_number: int | None = None
    active: bool = True
    country: str = ""
    position: str = ""


@dataclass(frozen=True)
class DirectorCandidateRecord:
    id: int
    player_name: str
    current_club: str = ""
    league_tag: str = ""
    active: bool = True
    country: str = ""
    position: str = ""


class TransfermarktClient(Protocol):
    def fetch_league_html(self, tm_code: str) -> str: ...
    def fetch_club_squad_html(self, club_id: int) -> str: ...


class TransfermarktStore(Protocol):
    def load_squad_players(self) -> list[SquadPlayerRecord]: ...
    def load_director_candidates(self) -> list[DirectorCandidateRecord]: ...
    def update_squad_players(self, rows: list[dict[str, Any]]) -> int: ...
    def update_director_candidates(self, rows: list[dict[str, Any]]) -> int: ...


TRANSFERMARKT_LEAGUE_SLUGS: dict[str, str] = {
    "CSL": "chinese-super-league",
    "RSK1": "k-league-1",
    "JAP1": "j1-league",
    "GB1": "premier-league",
    "ES1": "laliga",
    "IT1": "serie-a",
    "L1": "bundesliga",
    "FR1": "ligue-1",
}


class HttpTransfermarktClient:
    """HTTP client for fetching Transfermarkt league and squad HTML pages."""

    def __init__(
        self,
        *,
        base_url: str = DEFAULT_BASE_URL,
        timeout: float = 20.0,
        session: requests.Session | None = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self.session = session or requests.Session()
        self.session.headers.update(DEFAULT_HEADERS)

    def _get(self, path: str) -> str:
        url = f"{self.base_url}{path}"
        response = self.session.get(url, timeout=self.timeout)
        response.raise_for_status()
        if not response.encoding or response.encoding.lower() == "iso-8859-1":
            response.encoding = response.apparent_encoding or "utf-8"
        return response.text

    def fetch_league_html(self, tm_code: str) -> str:
        slug = TRANSFERMARKT_LEAGUE_SLUGS.get(tm_code, "jumplist")
        return self._get(f"/{slug}/startseite/wettbewerb/{tm_code}")

    def fetch_club_squad_html(self, club_id: int) -> str:
        return self._get(f"/jumplist/kader/verein/{club_id}/plus/1")


class PostgresTransfermarktStore:
    """PostgreSQL store for reading player match targets and applying valuation updates."""

    def __init__(self, connection) -> None:
        self.connection = connection

    def reset_valuations(self) -> tuple[int, int]:
        """Clear previously synced Transfermarkt fields before a clean re-import."""
        with self.connection.transaction():
            with self.connection.cursor() as cursor:
                cursor.execute(
                    """
                    UPDATE f_i_squad_players
                    SET market_value_eur = NULL,
                        birth_date = NULL,
                        contract_until = NULL,
                        transfermarkt_synced_at = NULL
                    WHERE transfermarkt_synced_at IS NOT NULL
                    """
                )
                squad_cleared = cursor.rowcount or 0
                cursor.execute(
                    """
                    UPDATE f_i_director_candidates
                    SET market_value_eur = NULL,
                        birth_date = NULL,
                        contract_until = NULL,
                        transfermarkt_synced_at = NULL
                    WHERE transfermarkt_synced_at IS NOT NULL
                    """
                )
                cand_cleared = cursor.rowcount or 0
        return (squad_cleared, cand_cleared)

    def load_squad_players(self) -> list[SquadPlayerRecord]:
        rows = self.connection.execute(
            """
            SELECT sp.team_id, sp.player_id, sp.player_name, COALESCE(t.team_name, ''),
                   sp.shirt_number, sp.active, COALESCE(sp.country, ''), COALESCE(sp.position, '')
            FROM f_i_squad_players AS sp
            LEFT JOIN f_i_teams AS t ON t.team_id = sp.team_id
            """
        ).fetchall()
        return [
            SquadPlayerRecord(
                team_id=int(team_id),
                player_id=int(player_id),
                player_name=str(player_name or "").strip(),
                team_name=str(team_name or "").strip(),
                shirt_number=int(shirt_number) if shirt_number is not None else None,
                active=bool(active),
                country=str(country or "").strip(),
                position=str(position or "").strip(),
            )
            for team_id, player_id, player_name, team_name, shirt_number, active, country, position in rows
        ]

    def load_director_candidates(self) -> list[DirectorCandidateRecord]:
        rows = self.connection.execute(
            """
            SELECT id, player_name, COALESCE(current_club, ''), COALESCE(league_tag, ''),
                   active, COALESCE(country, ''), COALESCE(position, '')
            FROM f_i_director_candidates
            """
        ).fetchall()
        return [
            DirectorCandidateRecord(
                id=int(cand_id),
                player_name=str(player_name or "").strip(),
                current_club=str(current_club or "").strip(),
                league_tag=str(league_tag or "").strip(),
                active=bool(active),
                country=str(country or "").strip(),
                position=str(position or "").strip(),
            )
            for cand_id, player_name, current_club, league_tag, active, country, position in rows
        ]

    def update_squad_players(self, rows: list[dict[str, Any]]) -> int:
        if not rows:
            return 0
        with self.connection.transaction():
            with self.connection.cursor() as cursor:
                cursor.executemany(UPDATE_SQUAD_PLAYER_SQL, rows)
        return len(rows)

    def update_director_candidates(self, rows: list[dict[str, Any]]) -> int:
        if not rows:
            return 0
        with self.connection.transaction():
            with self.connection.cursor() as cursor:
                cursor.executemany(UPDATE_DIRECTOR_CANDIDATE_SQL, rows)
        return len(rows)


def parse_tm_leagues_arg(raw: str) -> tuple[str, ...]:
    """Parse and validate a comma-separated Transfermarkt league list."""
    parts = [part.strip().lower() for part in raw.split(",") if part.strip()]
    if not parts:
        raise ValueError("leagues cannot be empty")
    result: list[str] = []
    seen: set[str] = set()
    for part in parts:
        if part not in TRANSFERMARKT_LEAGUE_CODES:
            supported = ", ".join(TRANSFERMARKT_LEAGUE_CODES.keys())
            raise ValueError(f"Unsupported league key {part!r}; supported: {supported}")
        if part not in seen:
            seen.add(part)
            result.append(part)
    return tuple(result)


def normalize_player_name(name: str) -> str:
    """Normalize a player name for cross-source matching (Chinese and Latin)."""
    if not name:
        return ""
    decomposed = unicodedata.normalize("NFKD", name.strip())
    stripped_accents = "".join(ch for ch in decomposed if not unicodedata.combining(ch))
    return _NAME_PUNCT_RE.sub("", stripped_accents).lower()


def normalize_club_name(name: str) -> str:
    """Normalize a club name for tie-breaking duplicate player names."""
    norm = normalize_player_name(name)
    while norm:
        trimmed = _CLUB_SUFFIX_RE.sub("", norm)
        if trimmed == norm or not trimmed:
            break
        norm = trimmed
    return norm


def _parse_decimal_number(token: str, *, has_unit: bool) -> Decimal | None:
    cleaned = token.strip()
    if not cleaned:
        return None
    if "." in cleaned and "," in cleaned:
        if cleaned.rfind(",") > cleaned.rfind("."):
            normalized = cleaned.replace(".", "").replace(",", ".")
        else:
            normalized = cleaned.replace(",", "")
    elif "," in cleaned:
        parts = cleaned.split(",")
        if len(parts) == 2 and len(parts[1]) in (1, 2):
            normalized = f"{parts[0]}.{parts[1]}"
        elif len(parts) == 2 and len(parts[1]) == 3 and has_unit:
            normalized = f"{parts[0]}.{parts[1]}"
        else:
            normalized = cleaned.replace(",", "")
    else:
        normalized = cleaned
    try:
        return Decimal(normalized)
    except InvalidOperation:
        return None


def parse_market_value_eur(raw: str) -> int | None:
    """Parse Transfermarkt market value strings (EN/ZH/DE) into integer euros.

    Examples:
        ``€1.50m`` -> ``1500000``
        ``€800k`` -> ``800000``
        ``150万欧元`` -> ``1500000``
        ``150.00万欧`` -> ``1500000``
        ``1,50 Mio. €`` -> ``1500000``
        ``800 Tsd. €`` -> ``800000``
        ``-`` -> ``None``
    """
    if not raw:
        return None
    text = raw.replace("\xa0", " ").strip()
    if not text or text in {"-", "—", "–", "?", "N/A", "n/a", "暂无"}:
        return None

    lowered = text.lower()
    multiplier = Decimal(1)
    has_unit = True

    if "mrd" in lowered or re.search(r"\bbn\.?\b|\d\s*bn\b", lowered):
        multiplier = Decimal(1_000_000_000)
    elif "亿" in text:
        multiplier = Decimal(100_000_000)
    elif "mio" in lowered or re.search(r"[\d.,]\s*m(?:€|\s|$)", lowered):
        multiplier = Decimal(1_000_000)
    elif "万" in text:
        multiplier = Decimal(10_000)
    elif (
        "tsd" in lowered
        or "th." in lowered
        or re.search(r"\bth\b", lowered)
        or "千" in text
        or re.search(r"[\d.,]\s*k(?:€|\s|$)", lowered)
    ):
        multiplier = Decimal(1_000)
    else:
        has_unit = False

    match = _NUMBER_TOKEN_RE.search(text)
    if not match:
        return None
    number = _parse_decimal_number(match.group(1), has_unit=has_unit)
    if number is None:
        return None
    value = int((number * multiplier).quantize(Decimal("1")))
    return value if value > 0 else None


def parse_tm_date(raw: str) -> date | None:
    """Parse Transfermarkt birth/contract dates across EN/ZH/DE/ISO formats.

    Strips embedded age suffixes such as ``(29)`` before parsing.
    """
    if not raw:
        return None
    text = raw.replace("\xa0", " ").strip()
    text = _AGE_SUFFIX_RE.sub("", text).strip().rstrip("†").strip()
    if not text or text in {"-", "—", "–", "?", "N/A", "n/a", "未知"}:
        return None
    if len(text) >= 11 and text[10] in {" ", "T"} and _DATE_ISO_RE.match(text[:10]):
        text = text[:10]

    m = _DATE_ISO_RE.match(text)
    if m:
        return _safe_date(int(m.group(1)), int(m.group(2)), int(m.group(3)))

    m = _DATE_ZH_RE.match(text)
    if m:
        return _safe_date(int(m.group(1)), int(m.group(2)), int(m.group(3)))

    m = _DATE_DMY_RE.match(text)
    if m:
        return _safe_date(int(m.group(3)), int(m.group(2)), int(m.group(1)))

    m = _DATE_EN_MDY_RE.match(text)
    if m:
        month = _MONTH_MAP.get(m.group(1).rstrip(".").lower())
        if month is not None:
            return _safe_date(int(m.group(3)), month, int(m.group(2)))

    m = _DATE_DMY_WORD_RE.match(text)
    if m:
        month = _MONTH_MAP.get(m.group(2).rstrip(".").lower())
        if month is not None:
            return _safe_date(int(m.group(3)), month, int(m.group(1)))

    return None


def _safe_date(year: int, month: int, day: int) -> date | None:
    if year < 1900 or year > 2100:
        return None
    try:
        return date(year, month, day)
    except ValueError:
        return None


def _has_class(attrs: dict[str, str], class_name: str) -> bool:
    classes = attrs.get("class", "").split()
    return class_name in classes


def _collapse_ws(text: str) -> str:
    return " ".join(text.replace("\xa0", " ").split())


class _LeagueClubsParser(HTMLParser):
    """Extract club IDs and names from Transfermarkt league table HTML."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.items_table_depth = 0
        self.in_hauptlink_td = False
        self.hauptlink_td_depth = 0
        self.current_anchor: dict[str, Any] | None = None
        self.in_items_anchors: list[dict[str, Any]] = []
        self.all_anchors: list[dict[str, Any]] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attr_map = {k.lower(): (v or "") for k, v in attrs}
        if tag == "table":
            if self.items_table_depth > 0:
                self.items_table_depth += 1
            elif _has_class(attr_map, "items"):
                self.items_table_depth = 1
        elif tag == "td" and self.items_table_depth > 0:
            if self.in_hauptlink_td:
                self.hauptlink_td_depth += 1
            elif _has_class(attr_map, "hauptlink"):
                self.in_hauptlink_td = True
                self.hauptlink_td_depth = 1
        elif tag == "a":
            href = attr_map.get("href", "")
            match = _CLUB_HREF_RE.search(href)
            if match:
                self.current_anchor = {
                    "kind": match.group(1),
                    "club_id": int(match.group(2)),
                    "href": href,
                    "title": _collapse_ws(attr_map.get("title", "")),
                    "in_items": self.items_table_depth > 0,
                    "in_hauptlink": self.in_hauptlink_td,
                    "text_parts": [],
                }

    def handle_data(self, data: str) -> None:
        if self.current_anchor is not None:
            self.current_anchor["text_parts"].append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag == "a" and self.current_anchor is not None:
            anchor = self.current_anchor
            self.current_anchor = None
            anchor["text"] = _collapse_ws("".join(anchor["text_parts"]))
            if anchor["in_items"]:
                self.in_items_anchors.append(anchor)
            self.all_anchors.append(anchor)
        elif tag == "td" and self.in_hauptlink_td:
            self.hauptlink_td_depth -= 1
            if self.hauptlink_td_depth <= 0:
                self.in_hauptlink_td = False
                self.hauptlink_td_depth = 0
        elif tag == "table" and self.items_table_depth > 0:
            self.items_table_depth -= 1


def parse_league_clubs(html: str) -> list[TransfermarktClub]:
    """Parse club IDs and names from a Transfermarkt league overview page."""
    parser = _LeagueClubsParser()
    parser.feed(html)
    parser.close()

    candidates = parser.in_items_anchors or parser.all_anchors
    order: list[int] = []
    best_by_id: dict[int, tuple[int, TransfermarktClub]] = {}

    for anchor in candidates:
        club_id = anchor["club_id"]
        text = anchor["text"]
        title = anchor["title"]

        # Skip squad-count links like <a href="/club/kader/verein/123">28</a>
        if text and text.isdigit():
            name = title if title and not title.isdigit() else ""
        else:
            name = text or title

        if not name or name.isdigit():
            continue

        priority = 0
        if anchor["in_hauptlink"]:
            priority += 4
        if anchor["kind"] == "startseite":
            priority += 2
        if text and not text.isdigit():
            priority += 1

        if club_id not in best_by_id:
            order.append(club_id)
            best_by_id[club_id] = (
                priority,
                TransfermarktClub(club_id=club_id, club_name=name, href=anchor["href"]),
            )
        elif priority > best_by_id[club_id][0]:
            best_by_id[club_id] = (
                priority,
                TransfermarktClub(club_id=club_id, club_name=name, href=anchor["href"]),
            )

    return [best_by_id[cid][1] for cid in order]


@dataclass
class _ParsedCell:
    classes: set[str]
    text: str
    links: list[dict[str, str]]
    rn_nummer: str | None = None


class _ClubSquadParser(HTMLParser):
    """Parse Transfermarkt ``<table class="items">`` squad rows, handling nested inline tables."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.found_items_table = False
        self.table_depth = 0
        self.in_thead = False
        self.headers: list[str] = []
        self._current_th_parts: list[str] | None = None
        self._current_th_colspan: int = 1

        self.rows: list[list[_ParsedCell]] = []
        self._in_outer_tr = False
        self._current_row: list[_ParsedCell] = []

        self._td_depth = 0
        self._in_outer_td = False
        self._current_td_classes: set[str] = set()
        self._current_td_text_parts: list[str] = []
        self._current_td_links: list[dict[str, str]] = []
        self._current_td_rn: str | None = None

        self._in_rn_div = False
        self._rn_div_depth = 0
        self._rn_parts: list[str] = []

        self._current_link: dict[str, Any] | None = None

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attr_map = {k.lower(): (v or "") for k, v in attrs}

        if tag == "table":
            if self.table_depth > 0:
                self.table_depth += 1
            elif not self.found_items_table and _has_class(attr_map, "items"):
                self.found_items_table = True
                self.table_depth = 1
            return

        if self.table_depth == 0:
            return

        if self.table_depth == 1:
            if tag == "thead":
                self.in_thead = True
                return
            if self.in_thead and tag == "th":
                self._current_th_parts = []
                colspan_raw = attr_map.get("colspan", "1").strip()
                self._current_th_colspan = int(colspan_raw) if colspan_raw.isdigit() else 1
                return
            if not self.in_thead and tag == "tr":
                classes = set(attr_map.get("class", "").split())
                if not classes or "odd" in classes or "even" in classes:
                    self._in_outer_tr = True
                    self._current_row = []
                return
            if self._in_outer_tr and tag == "td":
                if not self._in_outer_td:
                    self._in_outer_td = True
                    self._td_depth = 1
                    self._current_td_classes = set(attr_map.get("class", "").split())
                    self._current_td_text_parts = []
                    self._current_td_links = []
                    self._current_td_rn = None
                else:
                    self._td_depth += 1
                return

        if self._in_outer_td:
            if tag == "div":
                if self._in_rn_div:
                    self._rn_div_depth += 1
                elif _has_class(attr_map, "rn_nummer"):
                    self._in_rn_div = True
                    self._rn_div_depth = 1
                    self._rn_parts = []
            elif tag == "a":
                self._current_link = {
                    "href": attr_map.get("href", ""),
                    "title": _collapse_ws(attr_map.get("title", "")),
                    "classes": attr_map.get("class", ""),
                    "text_parts": [],
                }

    def handle_data(self, data: str) -> None:
        if self.table_depth == 0:
            return
        if self._current_th_parts is not None:
            self._current_th_parts.append(data)
            return
        if self._in_outer_td:
            self._current_td_text_parts.append(data)
            if self._in_rn_div:
                self._rn_parts.append(data)
            if self._current_link is not None:
                self._current_link["text_parts"].append(data)

    def handle_endtag(self, tag: str) -> None:
        if self.table_depth == 0:
            return

        if tag == "table":
            self.table_depth -= 1
            return

        if self.table_depth == 1:
            if tag == "thead":
                self.in_thead = False
                return
            if self.in_thead and tag == "th" and self._current_th_parts is not None:
                header_text = _collapse_ws("".join(self._current_th_parts))
                for _ in range(max(1, self._current_th_colspan)):
                    self.headers.append(header_text)
                self._current_th_parts = None
                self._current_th_colspan = 1
                return
            if self._in_outer_td and tag == "td":
                self._td_depth -= 1
                if self._td_depth <= 0:
                    self._current_row.append(
                        _ParsedCell(
                            classes=set(self._current_td_classes),
                            text=_collapse_ws("".join(self._current_td_text_parts)),
                            links=list(self._current_td_links),
                            rn_nummer=self._current_td_rn,
                        )
                    )
                    self._in_outer_td = False
                    self._td_depth = 0
                return
            if self._in_outer_tr and tag == "tr":
                if self._current_row:
                    self.rows.append(self._current_row)
                self._in_outer_tr = False
                self._current_row = []
                return

        if self._in_outer_td:
            if tag == "div" and self._in_rn_div:
                self._rn_div_depth -= 1
                if self._rn_div_depth <= 0:
                    rn_text = _collapse_ws("".join(self._rn_parts))
                    if rn_text:
                        self._current_td_rn = rn_text
                    self._in_rn_div = False
                    self._rn_div_depth = 0
            elif tag == "a" and self._current_link is not None:
                link = self._current_link
                self._current_link = None
                self._current_td_links.append(
                    {
                        "href": link["href"],
                        "title": link["title"],
                        "classes": link["classes"],
                        "text": _collapse_ws("".join(link["text_parts"])),
                    }
                )


def _detect_header_columns(headers: list[str]) -> dict[str, int]:
    """Identify key column indices from ``<thead>`` header text across ZH/EN/DE."""
    mapping: dict[str, int] = {}
    for idx, raw in enumerate(headers):
        h = raw.strip().lower()
        if not h:
            continue
        if "shirt" not in mapping and (h == "#" or "号码" in h or "背号" in h):
            mapping["shirt"] = idx
        elif "birth" not in mapping and (
            "出生" in h or "年龄" in h or "birth" in h or "age" in h or "geboren" in h or "geb." in h
        ):
            mapping["birth"] = idx
        elif "contract" not in mapping and ("合同" in h or "到期" in h or "contract" in h or "vertrag" in h):
            mapping["contract"] = idx
        elif "market_value" not in mapping and ("身价" in h or "market value" in h or "marktwert" in h):
            mapping["market_value"] = idx
    return mapping


def _parse_shirt_number(raw: str | None) -> int | None:
    if not raw:
        return None
    cleaned = raw.strip()
    if cleaned.isdigit():
        num = int(cleaned)
        return num if 0 <= num <= 99 else None
    return None


def _extract_player_identity(cells: list[_ParsedCell]) -> tuple[int | None, str | None, int | None]:
    """Return ``(cell_index, player_name, tm_player_id)`` from a squad row."""
    for idx, cell in enumerate(cells):
        for link in cell.links:
            match = _PLAYER_HREF_RE.search(link["href"])
            if match:
                name = link["text"] or link["title"]
                if name:
                    return idx, name, int(match.group(1))
    # Fallback: first cell with posrela or hauptlink (not market value)
    for idx, cell in enumerate(cells):
        if "posrela" in cell.classes or ("hauptlink" in cell.classes and "rechts" not in cell.classes):
            for link in cell.links:
                if "marktwertverlauf" in link["href"]:
                    continue
                name = link["text"] or link["title"]
                if name:
                    return idx, name, None
            if cell.text:
                return idx, cell.text, None
    return None, None, None


def parse_club_squad(
    html: str,
    club_name: str,
    league_key: str,
) -> list[TransfermarktPlayer]:
    """Parse players from a Transfermarkt club detailed squad HTML page."""
    parser = _ClubSquadParser()
    parser.feed(html)
    parser.close()

    header_map = _detect_header_columns(parser.headers)
    players: list[TransfermarktPlayer] = []

    for cells in parser.rows:
        if len(cells) < 2:
            continue

        player_col_idx, player_name, tm_player_id = _extract_player_identity(cells)
        if not player_name:
            continue

        # Shirt number
        shirt_number: int | None = None
        shirt_col_idx: int | None = None
        for idx, cell in enumerate(cells):
            if cell.rn_nummer is not None or "rueckennummer" in cell.classes:
                shirt_number = _parse_shirt_number(cell.rn_nummer or cell.text)
                shirt_col_idx = idx
                break
        if shirt_col_idx is None and "shirt" in header_map and header_map["shirt"] < len(cells):
            shirt_col_idx = header_map["shirt"]
            shirt_number = _parse_shirt_number(cells[shirt_col_idx].text)

        # Market value
        market_value_eur: int | None = None
        mv_col_idx: int | None = None
        if "market_value" in header_map and header_map["market_value"] < len(cells):
            mv_col_idx = header_map["market_value"]
            market_value_eur = parse_market_value_eur(cells[mv_col_idx].text)
        else:
            for idx in range(len(cells) - 1, -1, -1):
                cell = cells[idx]
                has_mv_link = any("marktwertverlauf" in link["href"] for link in cell.links)
                if has_mv_link or ("rechts" in cell.classes and "hauptlink" in cell.classes):
                    mv_col_idx = idx
                    market_value_eur = parse_market_value_eur(cell.text)
                    break
            if mv_col_idx is None and len(cells) >= 3:
                mv_col_idx = len(cells) - 1
                market_value_eur = parse_market_value_eur(cells[mv_col_idx].text)

        # Birth date & contract until
        birth_date: date | None = None
        contract_until: date | None = None

        if "birth" in header_map and header_map["birth"] < len(cells):
            birth_date = parse_tm_date(cells[header_map["birth"]].text)
        if "contract" in header_map and header_map["contract"] < len(cells):
            contract_until = parse_tm_date(cells[header_map["contract"]].text)

        if birth_date is None or ("contract" not in header_map and contract_until is None):
            excluded = {idx for idx in (shirt_col_idx, player_col_idx, mv_col_idx) if idx is not None}
            date_cells: list[tuple[int, date, bool]] = []
            for idx, cell in enumerate(cells):
                if idx in excluded:
                    continue
                parsed = parse_tm_date(cell.text)
                if parsed is not None:
                    has_age = bool(_AGE_SUFFIX_RE.search(cell.text.strip()))
                    date_cells.append((idx, parsed, has_age))

            if birth_date is None:
                age_dated = [item for item in date_cells if item[2]]
                if age_dated:
                    birth_date = age_dated[0][1]
                    birth_idx = age_dated[0][0]
                elif date_cells:
                    birth_date = date_cells[0][1]
                    birth_idx = date_cells[0][0]
                else:
                    birth_idx = None
            else:
                birth_idx = header_map.get("birth")

            if "contract" not in header_map and contract_until is None:
                remaining_dates = [item for item in date_cells if item[0] != birth_idx]
                if len(remaining_dates) == 1:
                    # Only treat as contract_until if it is immediately before market value
                    # or in a compact table (<= 6 columns)
                    only_idx, only_date, _ = remaining_dates[0]
                    if len(cells) <= 6 or (mv_col_idx is not None and only_idx == mv_col_idx - 1):
                        contract_until = only_date
                elif len(remaining_dates) >= 2:
                    contract_until = remaining_dates[-1][1]

        players.append(
            TransfermarktPlayer(
                player_name=player_name,
                club_name=club_name,
                league_key=league_key,
                shirt_number=shirt_number,
                birth_date=birth_date,
                contract_until=contract_until,
                market_value_eur=market_value_eur,
                tm_player_id=tm_player_id,
            )
        )

    return players


_CSL_CLUB_ALIAS_GROUPS: dict[str, tuple[str, ...]] = {
    "chengdu_rongcheng": ("成都蓉城", "chengdu rongcheng", "cd rongcheng", "chengdu better city"),
    "shanghai_port": ("上海海港", "上海上港", "shanghai port", "sh port", "shanghai sipg"),
    "shanghai_shenhua": ("上海申花", "shanghai shenhua", "sh shenhua"),
    "beijing_guoan": ("北京国安", "beijing guoan", "bj guoan"),
    "shandong_taishan": ("山东泰山", "山东鲁能", "shandong taishan", "sd taishan"),
    "zhejiang": ("浙江", "浙江队", "浙江绿城", "zhejiang fc", "zhejiang professional", "zj fc", "zhejiang"),
    "tianjin_jinmen_tiger": ("天津津门虎", "津门虎", "tianjin jinmen tiger", "tj jinmen tiger"),
    "henan": ("河南", "河南队", "河南嵩山龙门", "henan fc", "henan songshan longmen", "hn fc", "henan"),
    "wuhan_three_towns": ("武汉三镇", "wuhan three towns", "wh three towns"),
    "qingdao_west_coast": ("青岛西海岸", "qingdao west coast", "qd west coast"),
    "qingdao_hainiu": ("青岛海牛", "qingdao hainiu", "qd hainiu"),
    "shenzhen_peng_city": ("深圳新鹏城", "shenzhen peng city", "sz peng city"),
    "yunnan_yukun": ("云南玉昆", "yunnan yukun", "yn yukun"),
    "dalian_yingbo": ("大连英博", "dalian yingbo", "dl yingbo"),
    "liaoning_tieren": ("辽宁铁人", "liaoning tieren", "ln tieren"),
    "chongqing_tonglianglong": ("重庆铜梁龙", "chongqing tonglianglong", "cq tonglianglong"),
    "changchun_yatai": ("长春亚泰", "changchun yatai", "cc yatai"),
    "meizhou_hakka": ("梅州客家", "meizhou hakka", "mz hakka"),
    "cangzhou_mighty_lions": ("沧州雄狮", "cangzhou mighty lions", "cz mighty lions"),
    "nantong_zhiyun": ("南通支云", "nantong zhiyun", "nt zhiyun"),
}

_CSL_CLUB_CANONICAL_LOOKUP: dict[str, str] = {
    normalize_club_name(alias): canonical
    for canonical, aliases in _CSL_CLUB_ALIAS_GROUPS.items()
    for alias in aliases
    if normalize_club_name(alias)
}


def canonicalize_csl_club(name: str) -> str:
    """Resolve Chinese or English CSL club names to a shared canonical club key."""
    norm = normalize_club_name(name)
    if not norm:
        return ""
    direct = _CSL_CLUB_CANONICAL_LOOKUP.get(norm)
    if direct:
        return direct
    for alias_norm, canonical in _CSL_CLUB_CANONICAL_LOOKUP.items():
        if len(alias_norm) >= 4 and (alias_norm in norm or norm in alias_norm):
            return canonical
    return norm


def _club_similarity_score(tm_club: str, db_club: str) -> int:
    canon_tm = canonicalize_csl_club(tm_club)
    canon_db = canonicalize_csl_club(db_club)
    if canon_tm and canon_db and canon_tm == canon_db:
        return 2
    norm_tm = normalize_club_name(tm_club)
    norm_db = normalize_club_name(db_club)
    if not norm_tm or not norm_db:
        return 0
    if norm_tm == norm_db:
        return 2
    if len(norm_tm) >= 2 and len(norm_db) >= 2 and (norm_tm in norm_db or norm_db in norm_tm):
        return 1
    return 0


def match_squad_player(
    tm_player: TransfermarktPlayer,
    squad_by_name: dict[str, list[SquadPlayerRecord]],
    squad_by_club_shirt: dict[tuple[str, int], list[SquadPlayerRecord]] | None = None,
) -> SquadPlayerRecord | None:
    """Match a parsed Transfermarkt player against ``f_i_squad_players`` rows.

    First matches by normalized player name (handles same-script names), then falls back
    to ``(canonical_csl_club, shirt_number)`` so English Transfermarkt pages reliably
    match Chinese ``f_i_squad_players`` rosters.
    """
    key = normalize_player_name(tm_player.player_name)
    candidates = squad_by_name.get(key) if key else None
    if candidates:
        if len(candidates) == 1:
            return candidates[0]

        def _rank(record: SquadPlayerRecord) -> tuple[int, int, int]:
            club_score = _club_similarity_score(tm_player.club_name, record.team_name)
            shirt_score = (
                1
                if (
                    tm_player.shirt_number is not None
                    and record.shirt_number is not None
                    and tm_player.shirt_number == record.shirt_number
                )
                else 0
            )
            active_score = 1 if record.active else 0
            return (club_score, shirt_score, active_score)

        return max(candidates, key=_rank)

    if squad_by_club_shirt and tm_player.shirt_number is not None:
        club_key = canonicalize_csl_club(tm_player.club_name)
        if club_key:
            shirt_candidates = squad_by_club_shirt.get((club_key, tm_player.shirt_number))
            if shirt_candidates:
                active_candidates = [rec for rec in shirt_candidates if rec.active]
                if len(active_candidates) == 1:
                    return active_candidates[0]
                if not active_candidates and len(shirt_candidates) == 1:
                    return shirt_candidates[0]
                if active_candidates:
                    return active_candidates[0]
                return shirt_candidates[0]

    return None


def match_director_candidate(
    tm_player: TransfermarktPlayer,
    candidates_by_name: dict[str, list[DirectorCandidateRecord]],
) -> DirectorCandidateRecord | None:
    """Match a parsed Transfermarkt player against ``f_i_director_candidates`` rows."""
    key = normalize_player_name(tm_player.player_name)
    if not key:
        return None
    candidates = candidates_by_name.get(key)
    if not candidates:
        return None
    if len(candidates) == 1:
        return candidates[0]

    expected_tag = LEAGUE_KEY_TO_CANDIDATE_TAG.get(tm_player.league_key, "")

    def _rank(record: DirectorCandidateRecord) -> tuple[int, int, int]:
        club_score = _club_similarity_score(tm_player.club_name, record.current_club)
        tag_score = 1 if expected_tag and record.league_tag == expected_tag else 0
        active_score = 1 if record.active else 0
        return (club_score, tag_score, active_score)

    return max(candidates, key=_rank)


def _has_any_valuation_field(player: TransfermarktPlayer) -> bool:
    return (
        player.market_value_eur is not None
        or player.birth_date is not None
        or player.contract_until is not None
    )


def sync_transfermarkt_valuations(
    *,
    client: TransfermarktClient,
    store: TransfermarktStore | None = None,
    leagues: Sequence[str] = DEFAULT_TM_LEAGUES,
    dry_run: bool = True,
    delay: float = DEFAULT_DELAY_SECONDS,
    limit_clubs: int | None = None,
    sleep: Callable[[float], None] = time.sleep,
) -> dict[str, Any]:
    """Fetch Transfermarkt squads and backfill valuation/bio fields into PostgreSQL."""
    squad_by_name: dict[str, list[SquadPlayerRecord]] = {}
    squad_by_club_shirt: dict[tuple[str, int], list[SquadPlayerRecord]] = {}
    candidates_by_name: dict[str, list[DirectorCandidateRecord]] = {}

    if store is not None:
        for record in store.load_squad_players():
            key = normalize_player_name(record.player_name)
            if key:
                squad_by_name.setdefault(key, []).append(record)
            club_key = canonicalize_csl_club(record.team_name)
            if club_key and record.shirt_number is not None:
                squad_by_club_shirt.setdefault((club_key, record.shirt_number), []).append(record)
        for record in store.load_director_candidates():
            key = normalize_player_name(record.player_name)
            if key:
                candidates_by_name.setdefault(key, []).append(record)

    league_summaries: list[dict[str, Any]] = []
    clubs_fetched = 0
    clubs_failed = 0
    players_parsed = 0
    squad_updates_by_pk: dict[tuple[int, int], dict[str, Any]] = {}
    candidate_updates_by_id: dict[int, dict[str, Any]] = {}
    limit_reached = False

    def _pause() -> None:
        if delay > 0:
            sleep(delay)

    for league_key in leagues:
        if limit_reached:
            break
        tm_code = TRANSFERMARKT_LEAGUE_CODES[league_key]
        try:
            league_html = client.fetch_league_html(tm_code)
            clubs = parse_league_clubs(league_html)
        except Exception as error:  # noqa: BLE001
            logger.warning("Failed to fetch Transfermarkt league %s (%s): %s", league_key, tm_code, error)
            league_summaries.append(
                {
                    "league": league_key,
                    "tm_code": tm_code,
                    "clubs_fetched": 0,
                    "clubs_failed": 0,
                    "players_parsed": 0,
                    "error": str(error),
                }
            )
            _pause()
            continue
        _pause()

        league_clubs_fetched = 0
        league_clubs_failed = 0
        league_players_parsed = 0

        for club in clubs:
            if limit_clubs is not None and clubs_fetched >= limit_clubs:
                limit_reached = True
                break

            try:
                squad_html = client.fetch_club_squad_html(club.club_id)
                players = parse_club_squad(squad_html, club_name=club.club_name, league_key=league_key)
            except Exception as error:  # noqa: BLE001
                logger.warning(
                    "Failed to fetch Transfermarkt squad for %s (%s, club_id=%s): %s",
                    club.club_name,
                    league_key,
                    club.club_id,
                    error,
                )
                clubs_failed += 1
                league_clubs_failed += 1
                _pause()
                continue

            clubs_fetched += 1
            league_clubs_fetched += 1
            players_parsed += len(players)
            league_players_parsed += len(players)

            for player in players:
                if not _has_any_valuation_field(player):
                    continue

                if league_key == "csl" and (squad_by_name or squad_by_club_shirt):
                    matched_squad = match_squad_player(player, squad_by_name, squad_by_club_shirt)
                    if matched_squad is not None:
                        squad_updates_by_pk[(matched_squad.team_id, matched_squad.player_id)] = {
                            "team_id": matched_squad.team_id,
                            "player_id": matched_squad.player_id,
                            "market_value_eur": player.market_value_eur,
                            "birth_date": player.birth_date,
                            "contract_until": player.contract_until,
                        }

                if candidates_by_name:
                    matched_cand = match_director_candidate(player, candidates_by_name)
                    if matched_cand is not None:
                        candidate_updates_by_id[matched_cand.id] = {
                            "id": matched_cand.id,
                            "market_value_eur": player.market_value_eur,
                            "birth_date": player.birth_date,
                            "contract_until": player.contract_until,
                        }

            _pause()

        league_summary: dict[str, Any] = {
            "league": league_key,
            "tm_code": tm_code,
            "clubs_fetched": league_clubs_fetched,
            "clubs_failed": league_clubs_failed,
            "players_parsed": league_players_parsed,
        }
        if not clubs:
            title_match = re.search(r"<title[^>]*>(.*?)</title>", league_html, re.IGNORECASE | re.DOTALL)
            league_summary["html_len"] = len(league_html)
            league_summary["page_title"] = _collapse_ws(title_match.group(1)) if title_match else ""
        league_summaries.append(league_summary)

    squad_rows = list(squad_updates_by_pk.values())
    candidate_rows = list(candidate_updates_by_id.values())
    squad_updated = 0
    candidate_updated = 0

    if not dry_run and store is not None:
        squad_updated = store.update_squad_players(squad_rows)
        candidate_updated = store.update_director_candidates(candidate_rows)

    return {
        "dry_run": dry_run,
        "leagues": league_summaries,
        "clubs_fetched": clubs_fetched,
        "clubs_failed": clubs_failed,
        "players_parsed": players_parsed,
        "squad_matches": len(squad_rows),
        "squad_updated": squad_updated,
        "candidate_matches": len(candidate_rows),
        "candidate_updated": candidate_updated,
    }


TM_CODE_TO_LEAGUE_KEY: dict[str, str] = {code: key for key, code in TRANSFERMARKT_LEAGUE_CODES.items()}


def _name_match_keys(name: str) -> list[str]:
    """Generate normalized full-name and reversed-order variants (e.g. Wei Shihao <-> Shihao Wei)."""
    cleaned = _collapse_ws(name)
    if not cleaned:
        return []
    keys: list[str] = []
    full = normalize_player_name(cleaned)
    if full:
        keys.append(full)
    parts = [p for p in re.split(r"[\s\-]+", cleaned) if p]
    if len(parts) == 2 and all(part.isascii() for part in parts):
        reversed_key = normalize_player_name(f"{parts[1]} {parts[0]}")
        if reversed_key and reversed_key not in keys:
            keys.append(reversed_key)
    return keys


def _first_nonempty(row: dict[str, str], keys: Sequence[str]) -> str:
    for key in keys:
        val = row.get(key)
        if val is not None and val.strip():
            return val.strip()
    return ""


def _parse_csv_market_value(raw: str) -> int | None:
    text = raw.strip()
    if not text:
        return None
    # Plain numeric EUR values from transfermarkt-datasets (e.g. "1500000" or "1500000.0")
    try:
        dec = Decimal(text)
        val = int(dec.quantize(Decimal("1")))
        return val if val > 0 else None
    except InvalidOperation:
        return parse_market_value_eur(text)


def parse_players_csv(path: Path) -> list[TransfermarktPlayer]:
    """Load players from a local `.csv` or `.csv.gz` file.

    Supports both ``dcaribou/transfermarkt-datasets`` ``players.csv(.gz)`` and simple
    user-maintained CSVs (``player_name,club_name,market_value_eur,birth_date,contract_until``).
    """
    is_gz = path.suffix.lower() == ".gz" or path.name.lower().endswith(".csv.gz")
    opener = (lambda p: gzip.open(p, "rt", encoding="utf-8-sig", newline="")) if is_gz else (
        lambda p: open(p, "r", encoding="utf-8-sig", newline="")
    )
    players: list[TransfermarktPlayer] = []
    with opener(path) as handle:
        filtered_lines = (line for line in handle if line.strip() and not line.lstrip().startswith("#"))
        reader = csv.DictReader(filtered_lines)
        if not reader.fieldnames:
            return players
        for raw_row in reader:
            row = {str(k).strip().lower(): (v or "") for k, v in raw_row.items() if k is not None}
            name = _first_nonempty(row, ("player_name", "name", "full_name"))
            if not name:
                first = _first_nonempty(row, ("first_name",))
                last = _first_nonempty(row, ("last_name",))
                name = _collapse_ws(f"{first} {last}")
            if not name:
                continue

            club_name = _first_nonempty(row, ("club_name", "current_club_name", "club", "team_name"))
            comp_raw = _first_nonempty(
                row,
                ("league", "league_key", "current_club_domestic_competition_id", "competition_id"),
            )
            league_key = TM_CODE_TO_LEAGUE_KEY.get(comp_raw.upper(), comp_raw.lower() or "csv")

            shirt_raw = _first_nonempty(row, ("shirt_number", "number"))
            shirt_number = _parse_shirt_number(shirt_raw) if shirt_raw else None

            mv_raw = _first_nonempty(
                row,
                ("market_value_eur", "market_value_in_eur", "market_value", "value_eur", "highest_market_value_in_eur"),
            )
            market_value_eur = _parse_csv_market_value(mv_raw)
            birth_date = parse_tm_date(_first_nonempty(row, ("birth_date", "date_of_birth", "dob")))
            contract_until = parse_tm_date(
                _first_nonempty(row, ("contract_until", "contract_expiration_date", "contract_end"))
            )
            tm_id_raw = _first_nonempty(row, ("tm_player_id", "player_id"))
            tm_player_id = int(tm_id_raw) if tm_id_raw.isdigit() else None
            country = _first_nonempty(row, ("country_of_citizenship", "country", "nationality"))
            position = _first_nonempty(row, ("position", "sub_position"))

            player = TransfermarktPlayer(
                player_name=name,
                club_name=club_name,
                league_key=league_key,
                shirt_number=shirt_number,
                birth_date=birth_date,
                contract_until=contract_until,
                market_value_eur=market_value_eur,
                tm_player_id=tm_player_id,
                country=country,
                position=position,
            )
            if _has_any_valuation_field(player):
                players.append(player)
    return players


def _infer_club_name_from_html(html: str, fallback_stem: str) -> str:
    h1_match = re.search(r"<h1[^>]*>(.*?)</h1>", html, re.IGNORECASE | re.DOTALL)
    if h1_match:
        h1_text = _collapse_ws(re.sub(r"<[^>]+>", " ", h1_match.group(1)))
        if h1_text:
            return h1_text
    title_match = re.search(r"<title[^>]*>(.*?)</title>", html, re.IGNORECASE | re.DOTALL)
    if title_match:
        title_text = _collapse_ws(title_match.group(1))
        if " - " in title_text:
            return title_text.split(" - ", 1)[0].strip()
        if " | " in title_text:
            return title_text.split(" | ", 1)[0].strip()
    return fallback_stem.replace("_", " ").replace("-", " ").strip()


def parse_local_html_dir(html_dir: Path) -> list[TransfermarktPlayer]:
    """Parse browser-saved Transfermarkt club squad HTML files from a local directory."""
    players: list[TransfermarktPlayer] = []
    for path in sorted(html_dir.iterdir()):
        if not path.is_file() or path.suffix.lower() not in {".html", ".htm"}:
            continue
        html = path.read_text(encoding="utf-8", errors="replace")
        club_name = _infer_club_name_from_html(html, path.stem)
        parsed = parse_club_squad(html, club_name=club_name, league_key="csl")
        players.extend(p for p in parsed if _has_any_valuation_field(p))
    return players


def download_transfermarkt_dataset(
    *,
    dataset_url: str = DEFAULT_DATASET_URL,
    cache_path: Path = DEFAULT_DATASET_CACHE_PATH,
    refresh: bool = False,
    timeout: float = 60.0,
) -> Path:
    """Download and cache the open ``dcaribou/transfermarkt-datasets`` ``players.csv.gz``."""
    if cache_path.is_file() and not refresh:
        return cache_path
    cache_path.parent.mkdir(parents=True, exist_ok=True)
    response = requests.get(dataset_url, timeout=timeout, stream=True)
    response.raise_for_status()
    tmp_path = cache_path.with_suffix(cache_path.suffix + ".tmp")
    with open(tmp_path, "wb") as handle:
        for chunk in response.iter_content(chunk_size=65536):
            if chunk:
                handle.write(chunk)
    tmp_path.replace(cache_path)
    return cache_path


def fetch_sina_english_names(team_ids: Sequence[int]) -> dict[int, str]:
    """Fetch ``player_id -> english_name`` from Sina's squad API for CSL teams."""
    from .client import SinaCslClient

    client = SinaCslClient()
    mapping: dict[int, str] = {}
    for team_id in sorted(set(team_ids)):
        try:
            data = client._request("/op/api/team/player", {"tid": team_id})
            items = data.get("players") if isinstance(data, dict) else None
            for item in items or []:
                if not isinstance(item, dict):
                    continue
                pid_raw = str(item.get("pid") or "").strip()
                eng_name = str(item.get("name") or "").strip()
                if pid_raw.isdigit() and eng_name:
                    mapping[int(pid_raw)] = eng_name
        except Exception as error:  # noqa: BLE001
            logger.warning("Failed to fetch Sina English squad names for team_id=%s: %s", team_id, error)
    return mapping


_COUNTRY_ALIAS_GROUPS: dict[str, tuple[str, ...]] = {
    "china": ("中国", "china", "china pr", "pr china"),
    "hong_kong": ("中国香港", "香港", "hong kong", "hongkong"),
    "chinese_taipei": ("中国台北", "中国台湾", "chinese taipei", "taiwan"),
    "brazil": ("巴西", "brazil"),
    "south_korea": ("韩国", "korea, south", "south korea", "korea republic"),
    "japan": ("日本", "japan"),
    "portugal": ("葡萄牙", "portugal"),
    "spain": ("西班牙", "spain"),
    "france": ("法国", "france"),
    "germany": ("德国", "germany"),
    "netherlands": ("荷兰", "netherlands"),
    "croatia": ("克罗地亚", "croatia"),
    "serbia": ("塞尔维亚", "serbia"),
    "russia": ("俄罗斯", "russia"),
    "switzerland": ("瑞士", "switzerland"),
    "sweden": ("瑞典", "sweden"),
    "argentina": ("阿根廷", "argentina"),
    "colombia": ("哥伦比亚", "colombia"),
    "uruguay": ("乌拉圭", "uruguay"),
    "chile": ("智利", "chile"),
    "peru": ("秘鲁", "peru"),
    "ecuador": ("厄瓜多尔", "ecuador"),
    "venezuela": ("委内瑞拉", "venezuela"),
    "paraguay": ("巴拉圭", "paraguay"),
    "cameroon": ("喀麦隆", "cameroon"),
    "ghana": ("加纳", "ghana"),
    "nigeria": ("尼日利亚", "nigeria"),
    "cote_divoire": ("科特迪瓦", "cote d'ivoire", "ivory coast"),
    "senegal": ("塞内加尔", "senegal"),
    "angola": ("安哥拉", "angola"),
    "dr_congo": ("民主刚果", "刚果(金)", "刚果民主共和国", "dr congo"),
    "gabon": ("加蓬", "gabon"),
    "guinea": ("几内亚", "guinea"),
    "mali": ("马里", "mali"),
    "australia": ("澳大利亚", "australia"),
    "uzbekistan": ("乌兹别克斯坦", "uzbekistan"),
    "romania": ("罗马尼亚", "romania"),
    "poland": ("波兰", "poland"),
    "czechia": ("捷克", "czech republic", "czechia"),
    "belgium": ("比利时", "belgium"),
    "italy": ("意大利", "italy"),
    "england": ("英格兰", "england"),
    "scotland": ("苏格兰", "scotland"),
    "norway": ("挪威", "norway"),
    "denmark": ("丹麦", "denmark"),
    "montenegro": ("黑山", "montenegro"),
    "bosnia": ("波黑", "bosnia-herzegovina", "bosnia and herzegovina"),
    "slovenia": ("斯洛文尼亚", "slovenia"),
    "slovakia": ("斯洛伐克", "slovakia"),
    "hungary": ("匈牙利", "hungary"),
    "austria": ("奥地利", "austria"),
    "turkey": ("土耳其", "turkey", "turkiye", "türkiye"),
    "bulgaria": ("保加利亚", "bulgaria"),
    "georgia": ("格鲁吉亚", "georgia"),
    "armenia": ("亚美尼亚", "armenia"),
    "belarus": ("白俄罗斯", "belarus"),
    "ukraine": ("乌克兰", "ukraine"),
    "albania": ("阿尔巴尼亚", "albania"),
    "north_macedonia": ("北马其顿", "north macedonia"),
    "greece": ("希腊", "greece"),
    "tunisia": ("突尼斯", "tunisia"),
    "morocco": ("摩洛哥", "morocco"),
    "algeria": ("阿尔及利亚", "algeria"),
    "egypt": ("埃及", "egypt"),
    "south_africa": ("南非", "south africa"),
    "zimbabwe": ("津巴布韦", "zimbabwe"),
    "zambia": ("赞比亚", "zambia"),
    "equatorial_guinea": ("赤道几内亚", "equatorial guinea"),
    "guinea_bissau": ("几内亚比绍", "guinea-bissau"),
    "cape_verde": ("佛得角", "cape verde"),
    "togo": ("多哥", "togo"),
    "benin": ("贝宁", "benin"),
    "madagascar": ("马达加斯加", "madagascar"),
    "mozambique": ("莫桑比克", "mozambique"),
    "burkina_faso": ("布基纳法索", "burkina faso"),
    "united_states": ("美国", "united states", "usa"),
    "canada": ("加拿大", "canada"),
    "mexico": ("墨西哥", "mexico"),
    "jamaica": ("牙买加", "jamaica"),
    "costa_rica": ("哥斯达黎加", "costa rica"),
    "honduras": ("洪都拉斯", "honduras"),
    "panama": ("巴拿马", "panama"),
    "iran": ("伊朗", "iran"),
    "iraq": ("伊拉克", "iraq"),
    "syria": ("叙利亚", "syria"),
    "lebanon": ("黎巴嫩", "lebanon"),
    "jordan": ("约旦", "jordan"),
    "saudi_arabia": ("沙特阿拉伯", "沙特", "saudi arabia"),
    "thailand": ("泰国", "thailand"),
    "vietnam": ("越南", "vietnam"),
    "indonesia": ("印度尼西亚", "印尼", "indonesia"),
    "malaysia": ("马来西亚", "malaysia"),
    "singapore": ("新加坡", "singapore"),
    "new_zealand": ("新西兰", "new zealand"),
}

_COUNTRY_CANONICAL_LOOKUP: dict[str, str] = {
    normalize_player_name(alias): canonical
    for canonical, aliases in _COUNTRY_ALIAS_GROUPS.items()
    for alias in aliases
    if normalize_player_name(alias)
}


def canonicalize_country(raw: str) -> str:
    """Resolve Chinese or English country names to a shared canonical key."""
    norm = normalize_player_name(raw)
    if not norm:
        return ""
    return _COUNTRY_CANONICAL_LOOKUP.get(norm, norm)


def canonicalize_position(raw: str) -> str:
    """Normalize Transfermarkt / Sina / DB position labels to GK / DF / MF / FW."""
    text = raw.strip().lower()
    if not text:
        return ""
    if text in {"gk", "goalkeeper", "门将", "守门员"} or "goalkeeper" in text or "keeper" in text:
        return "GK"
    if text in {"df", "defender", "后卫"} or "back" in text or "defender" in text:
        return "DF"
    if text in {"mf", "midfield", "midfielder", "中场"} or "midfield" in text:
        return "MF"
    if text in {"fw", "attack", "forward", "前锋"} or "forward" in text or "striker" in text or "winger" in text or "attack" in text:
        return "FW"
    return ""


def _positions_contradict(tm_pos: str, db_pos: str) -> bool:
    p1 = canonicalize_position(tm_pos)
    p2 = canonicalize_position(db_pos)
    if not p1 or not p2 or p1 == p2:
        return False
    if "GK" in (p1, p2):
        return True
    return {p1, p2} == {"DF", "FW"}


def _countries_contradict(tm_country: str, db_country: str) -> bool:
    c1 = canonicalize_country(tm_country)
    c2 = canonicalize_country(db_country)
    if not c1 or not c2:
        return False
    # Allow Hong Kong / Chinese Taipei / China naturalized edge cases to be checked via club/position
    china_group = {"china", "hong_kong", "chinese_taipei"}
    if c1 in china_group and c2 in china_group:
        return False
    return c1 != c2


def _is_single_word_non_cjk(name: str) -> bool:
    cleaned = _collapse_ws(name)
    if not cleaned:
        return False
    if any("\u4e00" <= ch <= "\u9fff" for ch in cleaned):
        return False
    return len(cleaned.split()) == 1


def sync_transfermarkt_from_players(
    *,
    players: Sequence[TransfermarktPlayer],
    store: TransfermarktStore | None = None,
    dry_run: bool = True,
    source_label: str = "dataset",
    sina_english_names: dict[int, str] | None = None,
) -> dict[str, Any]:
    """Match pre-loaded Transfermarkt players (dataset/CSV/local HTML) against DB tables."""
    squad_by_name: dict[str, list[SquadPlayerRecord]] = {}
    squad_by_club_shirt: dict[tuple[str, int], list[SquadPlayerRecord]] = {}
    candidates_by_name: dict[str, list[DirectorCandidateRecord]] = {}
    english_by_player_id = sina_english_names or {}

    if store is not None:
        for record in store.load_squad_players():
            for key in _name_match_keys(record.player_name):
                squad_by_name.setdefault(key, []).append(record)
            eng_name = english_by_player_id.get(record.player_id)
            if eng_name:
                for key in _name_match_keys(eng_name):
                    squad_by_name.setdefault(key, []).append(record)
            club_key = canonicalize_csl_club(record.team_name)
            if club_key and record.shirt_number is not None:
                squad_by_club_shirt.setdefault((club_key, record.shirt_number), []).append(record)

        for record in store.load_director_candidates():
            for key in _name_match_keys(record.player_name):
                candidates_by_name.setdefault(key, []).append(record)

    squad_best: dict[tuple[int, int], tuple[tuple[int, int, int, int, int, int], dict[str, Any], str]] = {}
    candidate_best: dict[int, tuple[tuple[int, int, int, int], dict[str, Any], str]] = {}
    rejected_ambiguous = 0
    is_global_dataset = source_label.startswith("dataset:")

    for player in players:
        if not _has_any_valuation_field(player):
            continue

        # 1) Match f_i_squad_players
        matched_squad: SquadPlayerRecord | None = None
        match_via_shirt = False
        for key in _name_match_keys(player.player_name):
            records = squad_by_name.get(key)
            if records:
                valid_records = [
                    rec
                    for rec in records
                    if not _countries_contradict(player.country, rec.country)
                    and not _positions_contradict(player.position, rec.position)
                ]
                if not valid_records:
                    rejected_ambiguous += 1
                    continue
                matched_squad = max(
                    valid_records,
                    key=lambda rec: (
                        _club_similarity_score(player.club_name, rec.team_name),
                        1 if (canonicalize_country(player.country) and canonicalize_country(player.country) == canonicalize_country(rec.country)) else 0,
                        1 if (canonicalize_position(player.position) and canonicalize_position(player.position) == canonicalize_position(rec.position)) else 0,
                        1 if (player.shirt_number is not None and rec.shirt_number == player.shirt_number) else 0,
                        1 if rec.active else 0,
                    ),
                )
                break

        if matched_squad is None and player.shirt_number is not None:
            matched_squad = match_squad_player(player, {}, squad_by_club_shirt)
            match_via_shirt = matched_squad is not None

        if matched_squad is not None:
            club_score = _club_similarity_score(player.club_name, matched_squad.team_name)
            # Diacritic-aware guard against single-word global name collisions (e.g. "Rômulo", "Felipe", "Oscar")
            if (
                _is_single_word_non_cjk(player.player_name)
                and not match_via_shirt
                and club_score == 0
                and player.league_key != "csl"
                and (is_global_dataset or player.league_key != "csv")
            ):
                matched_squad = None
                rejected_ambiguous += 1

        if matched_squad is not None:
            pk = (matched_squad.team_id, matched_squad.player_id)
            country_exact = (
                1
                if (
                    canonicalize_country(player.country)
                    and canonicalize_country(player.country) == canonicalize_country(matched_squad.country)
                )
                else 0
            )
            pos_exact = (
                1
                if (
                    canonicalize_position(player.position)
                    and canonicalize_position(player.position) == canonicalize_position(matched_squad.position)
                )
                else 0
            )
            score = (
                _club_similarity_score(player.club_name, matched_squad.team_name),
                country_exact,
                pos_exact,
                1 if player.league_key == "csl" else 0,
                1 if player.market_value_eur is not None else 0,
                1 if player.contract_until is not None else 0,
            )
            row = {
                "team_id": matched_squad.team_id,
                "player_id": matched_squad.player_id,
                "market_value_eur": player.market_value_eur,
                "birth_date": player.birth_date,
                "contract_until": player.contract_until,
            }
            existing = squad_best.get(pk)
            if existing is None or score > existing[0]:
                label = f"{matched_squad.team_name} {matched_squad.player_name} <- {player.player_name} ({player.market_value_eur})"
                squad_best[pk] = (score, row, label)

        # 2) Match f_i_director_candidates
        matched_cand: DirectorCandidateRecord | None = None
        for key in _name_match_keys(player.player_name):
            records = candidates_by_name.get(key)
            if records:
                valid_cands = [
                    rec
                    for rec in records
                    if not _countries_contradict(player.country, rec.country)
                    and not _positions_contradict(player.position, rec.position)
                ]
                if not valid_cands:
                    continue
                expected_tag = LEAGUE_KEY_TO_CANDIDATE_TAG.get(player.league_key, "")
                matched_cand = max(
                    valid_cands,
                    key=lambda rec: (
                        _club_similarity_score(player.club_name, rec.current_club),
                        1 if expected_tag and rec.league_tag == expected_tag else 0,
                        1 if rec.active else 0,
                    ),
                )
                break

        if matched_cand is not None:
            club_score_c = _club_similarity_score(player.club_name, matched_cand.current_club)
            if (
                _is_single_word_non_cjk(player.player_name)
                and club_score_c == 0
                and is_global_dataset
            ):
                matched_cand = None
                rejected_ambiguous += 1

        if matched_cand is not None:
            score_c = (
                _club_similarity_score(player.club_name, matched_cand.current_club),
                1 if (canonicalize_country(player.country) and canonicalize_country(player.country) == canonicalize_country(matched_cand.country)) else 0,
                1 if player.market_value_eur is not None else 0,
                1 if player.contract_until is not None else 0,
            )
            row_c = {
                "id": matched_cand.id,
                "market_value_eur": player.market_value_eur,
                "birth_date": player.birth_date,
                "contract_until": player.contract_until,
            }
            existing_c = candidate_best.get(matched_cand.id)
            if existing_c is None or score_c > existing_c[0]:
                label_c = f"[cand] {matched_cand.player_name} <- {player.player_name} ({player.market_value_eur})"
                candidate_best[matched_cand.id] = (score_c, row_c, label_c)

    squad_rows = [item[1] for item in squad_best.values()]
    candidate_rows = [item[1] for item in candidate_best.values()]
    squad_updated = 0
    candidate_updated = 0

    if not dry_run and store is not None:
        squad_updated = store.update_squad_players(squad_rows)
        candidate_updated = store.update_director_candidates(candidate_rows)

    sample_matches = [item[2] for item in list(squad_best.values())[:5]] + [
        item[2] for item in list(candidate_best.values())[:3]
    ]
    return {
        "dry_run": dry_run,
        "source": source_label,
        "players_loaded": len(players),
        "sina_names_loaded": len(english_by_player_id),
        "rejected_ambiguous": rejected_ambiguous,
        "squad_matches": len(squad_rows),
        "squad_updated": squad_updated,
        "candidate_matches": len(candidate_rows),
        "candidate_updated": candidate_updated,
        "sample_matches": sample_matches,
    }


