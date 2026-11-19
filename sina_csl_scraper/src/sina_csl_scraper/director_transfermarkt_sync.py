"""Scrape player market values, birth dates, and contract end dates from Transfermarkt.

Fetches league club lists and detailed club squad pages (``/kader/verein/<id>/plus/1``),
parses player valuations/dates with stdlib ``html.parser``, and idempotently backfills
``market_value_eur``, ``birth_date``, ``contract_until``, and ``transfermarkt_synced_at``
into ``f_i_squad_players`` (for CSL) and ``f_i_director_candidates`` (for all leagues).
"""
from __future__ import annotations

import logging
import re
import time
import unicodedata
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from datetime import date
from decimal import Decimal, InvalidOperation
from html.parser import HTMLParser
from typing import Any, Protocol

import requests

logger = logging.getLogger(__name__)

DEFAULT_BASE_URL = "https://www.transfermarkt.cn"
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
    "Accept-Language": "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7",
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


@dataclass(frozen=True)
class SquadPlayerRecord:
    team_id: int
    player_id: int
    player_name: str
    team_name: str = ""
    shirt_number: int | None = None
    active: bool = True


@dataclass(frozen=True)
class DirectorCandidateRecord:
    id: int
    player_name: str
    current_club: str = ""
    league_tag: str = ""
    active: bool = True


class TransfermarktClient(Protocol):
    def fetch_league_html(self, tm_code: str) -> str: ...
    def fetch_club_squad_html(self, club_id: int) -> str: ...


class TransfermarktStore(Protocol):
    def load_squad_players(self) -> list[SquadPlayerRecord]: ...
    def load_director_candidates(self) -> list[DirectorCandidateRecord]: ...
    def update_squad_players(self, rows: list[dict[str, Any]]) -> int: ...
    def update_director_candidates(self, rows: list[dict[str, Any]]) -> int: ...


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
        return self._get(f"/startseite/wettbewerb/{tm_code}")

    def fetch_club_squad_html(self, club_id: int) -> str:
        return self._get(f"/kader/verein/{club_id}/plus/1")


class PostgresTransfermarktStore:
    """PostgreSQL store for reading player match targets and applying valuation updates."""

    def __init__(self, connection) -> None:
        self.connection = connection

    def load_squad_players(self) -> list[SquadPlayerRecord]:
        rows = self.connection.execute(
            """
            SELECT sp.team_id, sp.player_id, sp.player_name, COALESCE(t.team_name, ''), sp.shirt_number, sp.active
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
            )
            for team_id, player_id, player_name, team_name, shirt_number, active in rows
        ]

    def load_director_candidates(self) -> list[DirectorCandidateRecord]:
        rows = self.connection.execute(
            """
            SELECT id, player_name, COALESCE(current_club, ''), COALESCE(league_tag, ''), active
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
            )
            for cand_id, player_name, current_club, league_tag, active in rows
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


def _club_similarity_score(tm_club: str, db_club: str) -> int:
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
) -> SquadPlayerRecord | None:
    """Match a parsed Transfermarkt player against ``f_i_squad_players`` rows."""
    key = normalize_player_name(tm_player.player_name)
    if not key:
        return None
    candidates = squad_by_name.get(key)
    if not candidates:
        return None
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
    candidates_by_name: dict[str, list[DirectorCandidateRecord]] = {}

    if store is not None:
        for record in store.load_squad_players():
            key = normalize_player_name(record.player_name)
            if key:
                squad_by_name.setdefault(key, []).append(record)
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

                if league_key == "csl" and squad_by_name:
                    matched_squad = match_squad_player(player, squad_by_name)
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

        league_summaries.append(
            {
                "league": league_key,
                "tm_code": tm_code,
                "clubs_fetched": league_clubs_fetched,
                "clubs_failed": league_clubs_failed,
                "players_parsed": league_players_parsed,
            }
        )

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
