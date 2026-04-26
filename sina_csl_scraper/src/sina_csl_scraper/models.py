from __future__ import annotations

from dataclasses import asdict, dataclass, is_dataclass
from typing import Any


@dataclass(frozen=True)
class LeagueInfo:
    lid: int
    name: str
    current_season: int
    current_round: int
    max_round: int

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class MatchResult:
    match_id: int
    season: int
    round_number: int
    round_name: str
    date: str
    time: str
    status: str
    home_team_id: int
    home_team_name: str
    home_score: str
    away_team_id: int
    away_team_name: str
    away_score: str
    home_logo: str
    away_logo: str
    leisu_match_id: int | None = None
    home_corners: int | None = None
    away_corners: int | None = None
    corner_source: str | None = None
    technical_stats: list[Any] | None = None

    def to_dict(self) -> dict[str, Any]:
        payload = asdict(self)
        if self.technical_stats is not None:
            payload["technical_stats"] = [
                asdict(item) if is_dataclass(item) else item
                for item in self.technical_stats
            ]
        return payload


@dataclass(frozen=True)
class MatchTechnicalStat:
    slug: str
    label: str
    home_value: int
    away_value: int
    unit: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class StandingEntry:
    order: int
    team_id: int
    team_name: str
    team_logo: str
    played: int
    wins: int
    draws: int
    losses: int
    goals_for: int
    goals_against: int
    goal_difference: int
    points: int

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class TeamRankingEntry:
    rank: int
    score: str
    team_id: int
    team_name: str
    team_logo: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class PlayerRankingEntry:
    rank: int
    score: str
    player_id: int
    player_name: str
    player_logo: str
    team_id: int
    team_name: str
    penalty: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class TeamProfile:
    team_id: int
    team_name: str
    avatar_source_url: str | None = None
    avatar_object_name: str | None = None
    avatar_storage_url: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class PlayerProfile:
    player_id: int
    player_name: str
    team_id: int
    team_name: str
    avatar_source_url: str | None = None
    avatar_object_name: str | None = None
    avatar_storage_url: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class RankingDataset:
    slug: str
    label: str
    item_id: int
    entries: list[dict[str, Any]]

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)
