from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from .models import MatchResult, PlayerProfile, RankingDataset, StandingEntry, TeamProfile

SnapshotKind = Literal["live", "round_final"]


@dataclass(frozen=True)
class OpponentContribution:
    opponent_team_id: int
    opponent_team_name: str
    opponent_avatar_storage_url: str | None
    goals: int
    share: float


@dataclass(frozen=True)
class PlayerContribution:
    player_id: int | None
    player_name: str
    avatar_storage_url: str | None
    goals: int
    share: float


@dataclass(frozen=True)
class AssistContribution:
    player_id: int | None
    player_name: str
    avatar_storage_url: str | None
    assists: int
    share: float


@dataclass(frozen=True)
class TeamInsight:
    season: int
    round_number: int | None
    snapshot_kind: SnapshotKind
    team_id: int
    team_name: str
    rank_no: int
    avatar_storage_url: str | None
    goals_for_total: int
    goals_against_total: int
    goals_for_by_opponent: list[OpponentContribution]
    goals_for_by_player: list[PlayerContribution]
    assists_for_by_player: list[AssistContribution]
    goals_against_by_opponent: list[OpponentContribution]


def build_team_insights(
    *,
    season: int,
    round_number: int | None,
    snapshot_kind: SnapshotKind,
    teams: list[TeamProfile],
    players: list[PlayerProfile],
    matches: list[MatchResult],
    standings: list[StandingEntry],
    team_rankings: list[RankingDataset],
    player_rankings: list[RankingDataset],
) -> list[TeamInsight]:
    team_profiles = {team.team_id: team for team in teams}
    player_profiles = {player.player_id: player for player in players}
    finished_matches = [match for match in matches if match.status == "3"]

    goals_for_by_team: dict[int, dict[int, int]] = {}
    goals_against_by_team: dict[int, dict[int, int]] = {}
    for match in finished_matches:
        home_goals = _to_int(match.home_score)
        away_goals = _to_int(match.away_score)

        _accumulate(goals_for_by_team, match.home_team_id, match.away_team_id, home_goals)
        _accumulate(goals_for_by_team, match.away_team_id, match.home_team_id, away_goals)
        _accumulate(goals_against_by_team, match.home_team_id, match.away_team_id, away_goals)
        _accumulate(goals_against_by_team, match.away_team_id, match.home_team_id, home_goals)

    goal_dataset = next((dataset for dataset in player_rankings if dataset.slug == "goals"), None)
    assist_dataset = next((dataset for dataset in player_rankings if dataset.slug == "assists"), None)
    team_assist_dataset = next((dataset for dataset in team_rankings if dataset.slug == "assists"), None)
    goals_by_team_player: dict[int, dict[int, int]] = {}
    assists_by_team_player: dict[int, dict[int, int]] = {}
    assists_total_by_team: dict[int, int] = {}
    if goal_dataset is not None:
        for entry in goal_dataset.entries:
            team_id = int(entry["team_id"])
            player_id = int(entry["player_id"])
            score = _to_int(entry["score"])
            goals_by_team_player.setdefault(team_id, {})
            goals_by_team_player[team_id][player_id] = score
    if assist_dataset is not None:
        for entry in assist_dataset.entries:
            team_id = int(entry["team_id"])
            player_id = int(entry["player_id"])
            score = _to_int(entry["score"])
            assists_by_team_player.setdefault(team_id, {})
            assists_by_team_player[team_id][player_id] = score
    if team_assist_dataset is not None:
        for entry in team_assist_dataset.entries:
            assists_total_by_team[int(entry["team_id"])] = _to_int(entry["score"])

    results: list[TeamInsight] = []
    for standing in standings:
        profile = team_profiles.get(standing.team_id)
        goals_for_total = standing.goals_for
        goals_against_total = standing.goals_against
        player_contributions = _build_player_contributions(
            goals_by_team_player.get(standing.team_id, {}),
            player_profiles,
            goals_for_total,
        )
        assist_contributions = _build_assist_contributions(
            assists_by_team_player.get(standing.team_id, {}),
            player_profiles,
            assists_total_by_team.get(standing.team_id, 0),
        )
        results.append(
            TeamInsight(
                season=season,
                round_number=round_number,
                snapshot_kind=snapshot_kind,
                team_id=standing.team_id,
                team_name=standing.team_name,
                rank_no=standing.order,
                avatar_storage_url=profile.avatar_storage_url if profile else None,
                goals_for_total=goals_for_total,
                goals_against_total=goals_against_total,
                goals_for_by_opponent=_build_opponent_contributions(
                    goals_for_by_team.get(standing.team_id, {}),
                    team_profiles,
                    goals_for_total,
                ),
                goals_for_by_player=player_contributions,
                assists_for_by_player=assist_contributions,
                goals_against_by_opponent=_build_opponent_contributions(
                    goals_against_by_team.get(standing.team_id, {}),
                    team_profiles,
                    goals_against_total,
                ),
            )
        )

    return results


def _build_opponent_contributions(
    goals_by_opponent: dict[int, int],
    team_profiles: dict[int, TeamProfile],
    total_goals: int,
) -> list[OpponentContribution]:
    items = sorted(goals_by_opponent.items(), key=lambda item: (-item[1], item[0]))
    results: list[OpponentContribution] = []
    for opponent_team_id, goals in items:
        profile = team_profiles.get(opponent_team_id)
        results.append(
            OpponentContribution(
                opponent_team_id=opponent_team_id,
                opponent_team_name=profile.team_name if profile else f"球队{opponent_team_id}",
                opponent_avatar_storage_url=profile.avatar_storage_url if profile else None,
                goals=goals,
                share=_ratio(goals, total_goals),
            )
        )
    return results


def _build_player_contributions(
    goals_by_player: dict[int, int],
    player_profiles: dict[int, PlayerProfile],
    total_goals: int,
) -> list[PlayerContribution]:
    items = sorted(goals_by_player.items(), key=lambda item: (-item[1], item[0]))
    results: list[PlayerContribution] = []
    attributed_goals = 0
    for player_id, goals in items:
        profile = player_profiles.get(player_id)
        attributed_goals += goals
        results.append(
            PlayerContribution(
                player_id=player_id,
                player_name=profile.player_name if profile else f"球员{player_id}",
                avatar_storage_url=profile.avatar_storage_url if profile else None,
                goals=goals,
                share=_ratio(goals, total_goals),
            )
        )

    if total_goals > attributed_goals:
        missing_goals = total_goals - attributed_goals
        results.append(
            PlayerContribution(
                player_id=None,
                player_name="其他/未归因",
                avatar_storage_url=None,
                goals=missing_goals,
                share=_ratio(missing_goals, total_goals),
            )
        )

    return results


def _build_assist_contributions(
    assists_by_player: dict[int, int],
    player_profiles: dict[int, PlayerProfile],
    total_assists: int,
) -> list[AssistContribution]:
    items = sorted(assists_by_player.items(), key=lambda item: (-item[1], item[0]))
    results: list[AssistContribution] = []
    attributed_assists = 0
    for player_id, assists in items:
        profile = player_profiles.get(player_id)
        attributed_assists += assists
        results.append(
            AssistContribution(
                player_id=player_id,
                player_name=profile.player_name if profile else f"球员{player_id}",
                avatar_storage_url=profile.avatar_storage_url if profile else None,
                assists=assists,
                share=_ratio(assists, total_assists),
            )
        )

    if total_assists > attributed_assists:
        missing_assists = total_assists - attributed_assists
        results.append(
            AssistContribution(
                player_id=None,
                player_name="其他/未归因",
                avatar_storage_url=None,
                assists=missing_assists,
                share=_ratio(missing_assists, total_assists),
            )
        )

    return results


def _accumulate(target: dict[int, dict[int, int]], team_id: int, opponent_team_id: int, goals: int) -> None:
    if goals <= 0:
        return
    target.setdefault(team_id, {})
    target[team_id][opponent_team_id] = target[team_id].get(opponent_team_id, 0) + goals


def _to_int(value: str | int | None) -> int:
    if value in (None, ""):
        return 0
    return int(value)


def _ratio(value: int, total: int) -> float:
    if total <= 0:
        return 0.0
    return value / total
