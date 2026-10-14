"""Strict mapping of official CSL appearance logs, never cumulative differences."""
from __future__ import annotations

from dataclasses import dataclass
from datetime import date, time
from typing import Any

from .cfl_client import CflApiError, CflCslClient, DEFAULT_CFL_CALENDARS, TeamIdentityIndex
from .models import MatchResult


@dataclass(frozen=True)
class PlayerMatchGoals:
    player_id: int
    match_id: int
    season: int
    goals: int
    minutes_played: int
    source_player_id: str
    source_match_id: str


class IncompletePlayerPage(ValueError):
    """The official endpoint counted rows that were not returned."""

    def __init__(self, message: str, rows: list[dict]) -> None:
        super().__init__(message)
        self.rows = rows.copy()


def _integer(value: Any, label: str) -> int:
    if isinstance(value, bool) or not str(value).isascii() or not str(value).isdigit():
        raise ValueError(f"invalid {label}: expected nonnegative integer")
    return int(value)


def resolve_official_player(rows: list[dict], *, name: str, team_id: int, season: int) -> str:
    index = TeamIdentityIndex.for_season(season)
    candidates = set()
    for row in rows:
        if str(row.get("name") or "").strip() != name.strip():
            continue
        try:
            team = index.resolve(str(row.get("contestant_id") or ""), "")
        except CflApiError:
            continue
        if team.team_id == team_id and row.get("id"):
            candidates.add(str(row["id"]))
    if len(candidates) != 1:
        raise ValueError(f"cannot uniquely identify official player {name} on team {team_id}")
    return candidates.pop()


def map_player_logs(*, player_id: int, official_player_id: str, season: int,
                    rows: list[dict], matches: list[MatchResult]) -> list[PlayerMatchGoals]:
    index = TeamIdentityIndex.for_season(season)
    fixtures = {(m.date, m.home_team_id, m.away_team_id): m for m in matches if m.season == season}
    result: dict[int, PlayerMatchGoals] = {}
    for row in rows:
        if (row.get("player_id") != official_player_id or row.get("competition_code") != "CSL"
                or str(row.get("tournament_calendar_name")) != str(season)):
            raise ValueError("appearance identity, competition or season mismatch")
        goals = _integer(row.get("goals"), "goals")
        minutes = _integer(row.get("minutes_played"), "minutes_played")
        if minutes > 130 or goals > 30:
            raise ValueError("appearance statistics outside valid match range")
        home = index.resolve(str(row.get("home_contestant_id") or ""), "")
        away = index.resolve(str(row.get("away_contestant_id") or ""), "")
        match_date = date.fromisoformat(str(row.get("local_date"))).isoformat()
        match_time = time.fromisoformat(str(row.get("local_time")))
        fixture = fixtures.get((match_date, home.team_id, away.team_id))
        if fixture is None:
            raise ValueError("official appearance does not match an active season fixture")
        if fixture.status != "3" or minutes == 0:
            continue
        if time.fromisoformat(fixture.time) != match_time:
            raise ValueError("official appearance kickoff differs from season fixture")
        home_score = _integer(fixture.home_score, "home score")
        away_score = _integer(fixture.away_score, "away score")
        if goals > max(home_score, away_score):
            raise ValueError("player goals exceed the match score")
        source_match_id = str(row.get("match_id") or "").strip()
        if not source_match_id:
            raise ValueError("official appearance missing match id")
        record = PlayerMatchGoals(player_id, fixture.match_id, season, goals, minutes,
                                  official_player_id, source_match_id)
        previous = result.get(fixture.match_id)
        if previous is not None and previous != record:
            raise ValueError("conflicting duplicate appearance")
        result[fixture.match_id] = record
    return list(result.values())


class CflPlayerLogClient(CflCslClient):
    def _get_rows(self, path: str, params: dict) -> list[dict]:
        """Read every page; an incomplete response must not erase stored appearances."""
        rows: list[dict] = []
        seen: set[tuple[str, str]] = set()
        for page in range(1, 21):
            response = self.session.get(f"{self.base_url}{path}",
                                        params={**params, "curPage": page, "pageSize": 100},
                                        timeout=self.timeout)
            response.raise_for_status()
            payload = response.json()
            data = payload.get("data")
            if payload.get("status") != 200 or not isinstance(data, dict):
                raise ValueError("official player log request failed")
            batch = data.get("dataList")
            count = _integer(data.get("count"), "record count")
            if not isinstance(batch, list) or any(not isinstance(row, dict) for row in batch):
                raise ValueError("official player log response missing dataList")
            for row in batch:
                is_appearance = path.endswith("/player/played")
                record_id = str(row.get("match_id" if is_appearance else "id") or "")
                identity = (record_id, "" if is_appearance else str(row.get("contestant_id") or ""))
                if not record_id or identity in seen:
                    raise ValueError("official player log pagination repeated or missing record identity")
                seen.add(identity)
            rows.extend(batch)
            if len(rows) == count:
                return rows
            if not batch or len(rows) > count:
                raise IncompletePlayerPage("official player log pagination incomplete", rows)
        raise ValueError("official player log pagination limit exceeded")

    def fetch_player_logs(self, *, name: str, team_id: int, season: int) -> tuple[str, list[dict]]:
        calendar = self.tournament_calendar_id or DEFAULT_CFL_CALENDARS.get(season)
        if not calendar:
            raise ValueError(f"no official calendar configured for {season}")
        try:
            candidates = self._get_rows("/players/find", {"competition_code": "CSL", "name_like": name})
        except IncompletePlayerPage as error:
            # Name search counts some historical records it does not return. The
            # season club roster supplies a complete, independently checked identity.
            club = TeamIdentityIndex.for_season(season).official_id_for(team_id)
            roster = self._get_rows("/players/page", {
                "competition_code": "CSL", "name_like": name,
                "tournament_calendar_id": calendar, "contestant_id": club,
            })
            candidates = [{**row, "id": row.get("player_id"), "name": row.get("player_name")}
                          for row in roster]
            verified = resolve_official_player(candidates, name=name, team_id=team_id, season=season)
            candidates.extend(row for row in error.rows if str(row.get("id")) == verified)
        identity = resolve_official_player(candidates, name=name, team_id=team_id, season=season)
        # This API requires a club. Include every club returned for the same identity.
        clubs = {str(row["contestant_id"]) for row in candidates
                 if str(row.get("id")) == identity and row.get("contestant_id")}
        rows = []
        for club in sorted(clubs):
            rows.extend(self._get_rows("/players/player/played", {
                "player_id": identity, "tournament_calendar_id": calendar, "contestant_id": club,
            }))
        return identity, rows
