from __future__ import annotations

import hashlib
import os
from dataclasses import dataclass
from datetime import datetime
from typing import Any

import requests

from .models import MatchResult

DEFAULT_CFL_BASE_URL = "https://api.cfl-china.cn/frontweb/api"
DEFAULT_CFL_TIMEOUT = 20
DEFAULT_CFL_CALENDARS = {2026: "e6818x4pwankpph8awr91m1hw"}


class CflApiError(RuntimeError):
    """Raised when the official CSL schedule cannot be mapped safely."""


@dataclass(frozen=True)
class TeamIdentity:
    team_id: int
    team_name: str


_CFL_2026_TEAMS = {
    "7fqhid43axyzp47bihoqhqdbf": TeamIdentity(153, "山东泰山"),
    "3n4jrcw6w965v1qvp0azx5kwk": TeamIdentity(500, "云南玉昆"),
    "6nbq8ek8wl4scwttlmgdyveqy": TeamIdentity(110645, "武汉三镇"),
    "1eaept8dld85s894brcrkrq5p": TeamIdentity(178, "河南队"),
    "cvx4yvx5oby9dz7vm70mw04um": TeamIdentity(144, "上海申花"),
    "exq7je02eprk5fcre1uo9zwbi": TeamIdentity(136, "北京国安"),
    "a1a0zo3jliehxemg6qpa3lrva": TeamIdentity(87942, "深圳新鵬城"),
    "2ff3aaqtf2g91wbcz4w80kkr8": TeamIdentity(123380, "青岛西海岸"),
    "19g03gt1yj4oo8ak4vyf6sp9l": TeamIdentity(77680, "成都蓉城"),
    "cjnb1ayk5mruxqpud9qmy4t1w": TeamIdentity(502, "重庆铜梁龙"),
    "di8wvsww8w86i7gr80pbbhsyi": TeamIdentity(148, "天津津门虎"),
    "7u46yssnarhr1uyxsmv4q5x3q": TeamIdentity(507, "辽宁铁人"),
    "5x5hvs5xtbbh8b0zg5ehyyudv": TeamIdentity(41300, "上海海港"),
    "dicpumaiand6vk7vqbn4tqdxw": TeamIdentity(501, "大连英博"),
    "a3l87ozfq8bjwpy0nxvs5w28o": TeamIdentity(264, "浙江队"),
    "a3rgv4o76uowd2gxzx9pgf02d": TeamIdentity(143, "青岛海牛"),
}


class TeamIdentityIndex:
    def __init__(self, by_official_id: dict[str, TeamIdentity]) -> None:
        self._by_official_id = by_official_id

    @classmethod
    def for_season(cls, season: int) -> TeamIdentityIndex:
        return cls(dict(_CFL_2026_TEAMS) if season == 2026 else {})

    def resolve(self, official_id: str, official_name: str) -> TeamIdentity:
        identity = self._by_official_id.get(official_id)
        if identity is None:
            raise CflApiError(
                f"unmapped CFL contestant {official_id!r} ({official_name})"
            )
        return identity


def _official_match_id(value: str) -> int:
    digest = hashlib.blake2b(value.encode("utf-8"), digest_size=8).digest()
    safe_value = int.from_bytes(digest, "big") % (2**53 - 1)
    return -(safe_value or 1)


def _logo_url(value: Any) -> str:
    logo = str(value or "")
    return f"https:{logo}" if logo.startswith("//") else logo


def _status_code(value: Any) -> str:
    status = str(value or "")
    status_map = {
        "Fixture": "1",
        "Playing": "2",
        "FirstHalf": "2",
        "HalfTime": "2",
        "SecondHalf": "2",
        "ExtraTime": "2",
        "PenaltyShootout": "2",
        "Played": "3",
        "Postponed": "4",
    }
    try:
        return status_map[status]
    except KeyError as error:
        raise CflApiError(f"unsupported CFL match status: {status!r}") from error


class CflCslClient:
    def __init__(
        self,
        base_url: str = DEFAULT_CFL_BASE_URL,
        timeout: int = DEFAULT_CFL_TIMEOUT,
        tournament_calendar_id: str | None = None,
        session: Any | None = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self.tournament_calendar_id = tournament_calendar_id
        self.session = session or requests.Session()
        self.session.headers.update(
            {
                "User-Agent": (
                    "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) "
                    "AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148"
                )
            }
        )

    def fetch_round_matches(
        self,
        season: int,
        round_number: int,
        team_index: TeamIdentityIndex,
    ) -> list[MatchResult]:
        calendar_id = (
            self.tournament_calendar_id
            or os.getenv("FI_CFL_TOURNAMENT_CALENDAR_ID")
            or DEFAULT_CFL_CALENDARS.get(season)
        )
        if not calendar_id:
            raise CflApiError(f"no CFL tournament calendar configured for season {season}")

        response = self.session.get(
            f"{self.base_url}/matches/page",
            params={
                "tournament_calendar_id": calendar_id,
                "competition_code": "CSL",
                "contestant_id": "",
                "week": round_number,
                "stage_id": "",
                "curPage": 1,
                "pageSize": 999,
            },
            timeout=self.timeout,
        )
        response.raise_for_status()
        payload = response.json()
        if payload.get("status") != 200:
            raise CflApiError(f"CFL API failed: {payload.get('msg') or payload.get('status')}")

        data = payload.get("data")
        if not isinstance(data, dict) or not isinstance(data.get("dataList"), list):
            raise CflApiError("CFL API response is missing data.dataList")

        matches = [
            self._parse_match(item, season, round_number, team_index)
            for item in data["dataList"]
        ]
        if data.get("count") != len(matches):
            raise CflApiError(
                f"CFL round {round_number} count mismatch: "
                f"declared {data.get('count')}, parsed {len(matches)}"
            )
        return matches

    @staticmethod
    def _parse_match(
        item: Any,
        season: int,
        round_number: int,
        team_index: TeamIdentityIndex,
    ) -> MatchResult:
        if not isinstance(item, dict) or int(item.get("week") or 0) != round_number:
            raise CflApiError(f"CFL response contains a match outside round {round_number}")

        source_match_id = str(item.get("id") or "").strip()
        if not source_match_id:
            raise CflApiError("CFL match is missing id")
        try:
            kickoff = datetime.strptime(
                str(item["local_date_time"]),
                "%Y-%m-%d %H:%M:%S",
            )
        except (KeyError, TypeError, ValueError) as error:
            raise CflApiError(f"invalid CFL kickoff for match {source_match_id}") from error

        home = team_index.resolve(
            str(item.get("home_contestant_id") or ""),
            str(item.get("home_contestant_name") or ""),
        )
        away = team_index.resolve(
            str(item.get("away_contestant_id") or ""),
            str(item.get("away_contestant_name") or ""),
        )
        status = _status_code(item.get("match_status"))
        has_score = status in {"2", "3"}

        return MatchResult(
            match_id=_official_match_id(source_match_id),
            season=season,
            round_number=round_number,
            round_name=f"第{round_number}轮",
            date=kickoff.strftime("%Y-%m-%d"),
            time=kickoff.strftime("%H:%M"),
            status=status,
            home_team_id=home.team_id,
            home_team_name=home.team_name,
            home_score=str(item.get("ft_home_score", "")) if has_score else "",
            away_team_id=away.team_id,
            away_team_name=away.team_name,
            away_score=str(item.get("ft_away_score", "")) if has_score else "",
            home_logo=_logo_url(item.get("home_contestant_icon")),
            away_logo=_logo_url(item.get("away_contestant_icon")),
            schedule_source="cfl",
            source_match_id=source_match_id,
        )
