from __future__ import annotations

from collections.abc import Iterable
from typing import Any

import requests

from .constants import (
    DEFAULT_BASE_URL,
    DEFAULT_LEAGUE_ID,
    DEFAULT_PLAYER_PAGE_SIZE,
    DEFAULT_TIMEOUT,
    PLAYER_RANKING_CATEGORIES,
    TEAM_RANKING_CATEGORIES,
    RankingCategory,
)
from .models import (
    LeagueInfo,
    MatchResult,
    PlayerRankingEntry,
    RankingDataset,
    StandingEntry,
    TeamRankingEntry,
)


class SinaApiError(RuntimeError):
    """Raised when Sina APIs return a non-success status."""


def _to_int(value: str | int | None) -> int:
    if value in (None, ""):
        return 0
    return int(value)


class SinaCslClient:
    def __init__(
        self,
        base_url: str = DEFAULT_BASE_URL,
        timeout: int = DEFAULT_TIMEOUT,
        league_id: int = DEFAULT_LEAGUE_ID,
        session: requests.Session | None = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self.league_id = league_id
        self.session = session or requests.Session()
        self.session.headers.update(
            {
                "User-Agent": (
                    "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) "
                    "AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 "
                    "Mobile/15E148 Safari/604.1"
                )
            }
        )

    def _request(self, path: str, params: dict[str, Any]) -> dict[str, Any]:
        response = self.session.get(
            f"{self.base_url}{path}",
            params={**params, "dpc": 1},
            timeout=self.timeout,
        )
        response.raise_for_status()
        payload = response.json()
        status = payload["result"]["status"]
        if status["code"] != 0:
            raise SinaApiError(status["msg"])
        return payload["result"]["data"]

    def fetch_league_info(self) -> LeagueInfo:
        data = self._request("/op/api/league", {"lid": self.league_id})
        league = data["league"]
        return LeagueInfo(
            lid=_to_int(league["lid"]),
            name=league["name"],
            current_season=_to_int(league["cur_season"]),
            current_round=_to_int(league["cur_round"]),
            max_round=_to_int(league["max_round"]),
        )

    def fetch_standings(self, season: int) -> list[StandingEntry]:
        data = self._request("/op/api/standings", {"lid": self.league_id, "season": season})
        teams = data.get("teams") or []
        standings: list[StandingEntry] = []
        for item in teams:
            standings.append(
                StandingEntry(
                    order=_to_int(item["order"]),
                    team_id=_to_int(item["tid"]),
                    team_name=item["team_name"],
                    team_logo=item["team_logo"],
                    played=_to_int(item["played"]),
                    wins=_to_int(item["win"]),
                    draws=_to_int(item["draw"]),
                    losses=_to_int(item["lose"]),
                    goals_for=_to_int(item["goal"]),
                    goals_against=_to_int(item["losegoal"]),
                    goal_difference=_to_int(item["truegoal"]),
                    points=_to_int(item["score"]),
                )
            )
        return standings

    def fetch_round_matches(self, season: int, round_number: int) -> list[MatchResult]:
        data = self._request(
            "/li/api/schedule/round",
            {"lid": self.league_id, "season": season, "rnd": round_number},
        )
        matches: list[MatchResult] = []
        for item in data["matchs"]:
            matches.append(
                MatchResult(
                    match_id=_to_int(item["mid"]),
                    season=_to_int(item["season"]),
                    round_number=_to_int(item["round"]),
                    round_name=item["round_cn"],
                    date=item["date"],
                    time=item["time"],
                    status=item["status"],
                    home_team_id=_to_int(item["home_tid"]),
                    home_team_name=item["home_name"],
                    home_score=item.get("home_score", "") or "",
                    away_team_id=_to_int(item["away_tid"]),
                    away_team_name=item["away_name"],
                    away_score=item.get("away_score", "") or "",
                    home_logo=item["home_logo"],
                    away_logo=item["away_logo"],
                )
            )
        return matches

    def fetch_all_matches(self, season: int, max_round: int | None = None) -> list[MatchResult]:
        final_round = max_round if max_round is not None else self.fetch_league_info().max_round
        matches: list[MatchResult] = []
        for round_number in range(1, final_round + 1):
            matches.extend(self.fetch_round_matches(season, round_number))
        return matches

    def fetch_team_ranking(self, season: int, category: RankingCategory) -> RankingDataset:
        data = self._request(
            "/op/api/ranking/team",
            {"lid": self.league_id, "season": season, "item": category.item_id},
        )
        ranking = (data or {}).get("rank") or []
        entries = [
            TeamRankingEntry(
                rank=_to_int(item["rank"]),
                score=item["score"],
                team_id=_to_int(item["tid"]),
                team_name=item["team_name"],
                team_logo=item["team_logo"],
            ).to_dict()
            for item in ranking
        ]
        return RankingDataset(
            slug=category.slug,
            label=category.label,
            item_id=category.item_id,
            entries=entries,
        )

    def fetch_all_team_rankings(self, season: int) -> list[RankingDataset]:
        return [self.fetch_team_ranking(season, category) for category in TEAM_RANKING_CATEGORIES]

    def fetch_player_ranking_page(
        self,
        season: int,
        category: RankingCategory,
        page: int,
        limit: int = DEFAULT_PLAYER_PAGE_SIZE,
    ) -> list[PlayerRankingEntry]:
        data = self._request(
            "/op/api/ranking/player",
            {
                "lid": self.league_id,
                "season": season,
                "item": category.item_id,
                "page": page,
                "limit": limit,
            },
        )
        ranking = (data or {}).get("rank") or []
        entries: list[PlayerRankingEntry] = []
        for item in ranking:
            entries.append(
                PlayerRankingEntry(
                    rank=_to_int(item["rank"]),
                    score=item["score"],
                    player_id=_to_int(item["pid"]),
                    player_name=item["player_name"],
                    player_logo=item["player_logo"],
                    team_id=_to_int(item["tid"]),
                    team_name=item["team_name"],
                    penalty=item.get("penalty") or None,
                )
            )
        return entries

    def fetch_all_player_rankings(
        self,
        season: int,
        limit: int = DEFAULT_PLAYER_PAGE_SIZE,
    ) -> list[RankingDataset]:
        datasets: list[RankingDataset] = []
        for category in PLAYER_RANKING_CATEGORIES:
            entries: list[dict[str, Any]] = []
            page = 1
            while True:
                current_page = self.fetch_player_ranking_page(
                    season=season,
                    category=category,
                    page=page,
                    limit=limit,
                )
                if not current_page:
                    break
                entries.extend(entry.to_dict() for entry in current_page)
                if len(current_page) < limit:
                    break
                page += 1
            datasets.append(
                RankingDataset(
                    slug=category.slug,
                    label=category.label,
                    item_id=category.item_id,
                    entries=entries,
                )
            )
        return datasets


def serialize_datasets(datasets: Iterable[RankingDataset]) -> list[dict[str, Any]]:
    return [dataset.to_dict() for dataset in datasets]
