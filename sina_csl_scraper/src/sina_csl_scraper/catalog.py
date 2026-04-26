from __future__ import annotations

from .models import MatchResult, PlayerProfile, RankingDataset, StandingEntry, TeamProfile


def _prefer_avatar(current: str | None, candidate: str | None, overwrite: bool = False) -> str | None:
    if overwrite and candidate:
        return candidate
    if current:
        return current
    return candidate or None


def build_team_profiles(
    matches: list[MatchResult],
    standings: list[StandingEntry],
    team_rankings: list[RankingDataset],
) -> list[TeamProfile]:
    teams: dict[int, TeamProfile] = {}

    def upsert(team_id: int, team_name: str, avatar_url: str | None, overwrite_avatar: bool = False) -> None:
        if team_id <= 0:
            return
        current = teams.get(team_id)
        if current is None:
            teams[team_id] = TeamProfile(
                team_id=team_id,
                team_name=team_name,
                avatar_source_url=avatar_url or None,
            )
            return
        teams[team_id] = TeamProfile(
            team_id=team_id,
            team_name=team_name or current.team_name,
            avatar_source_url=_prefer_avatar(current.avatar_source_url, avatar_url, overwrite=overwrite_avatar),
            avatar_object_name=current.avatar_object_name,
            avatar_storage_url=current.avatar_storage_url,
        )

    for item in matches:
        upsert(item.home_team_id, item.home_team_name, item.home_logo)
        upsert(item.away_team_id, item.away_team_name, item.away_logo)

    for item in standings:
        upsert(item.team_id, item.team_name, item.team_logo, overwrite_avatar=True)

    for dataset in team_rankings:
        for entry in dataset.entries:
            upsert(
                int(entry["team_id"]),
                str(entry["team_name"]),
                str(entry.get("team_logo") or ""),
            )

    return [teams[key] for key in sorted(teams)]


def build_player_profiles(player_rankings: list[RankingDataset]) -> list[PlayerProfile]:
    players: dict[int, PlayerProfile] = {}

    for dataset in player_rankings:
        for entry in dataset.entries:
            player_id = int(entry["player_id"])
            if player_id <= 0:
                continue

            current = players.get(player_id)
            if current is None:
                players[player_id] = PlayerProfile(
                    player_id=player_id,
                    player_name=str(entry["player_name"]),
                    team_id=int(entry["team_id"]),
                    team_name=str(entry["team_name"]),
                    avatar_source_url=str(entry.get("player_logo") or "") or None,
                )
                continue

            players[player_id] = PlayerProfile(
                player_id=player_id,
                player_name=current.player_name or str(entry["player_name"]),
                team_id=current.team_id or int(entry["team_id"]),
                team_name=current.team_name or str(entry["team_name"]),
                avatar_source_url=_prefer_avatar(current.avatar_source_url, str(entry.get("player_logo") or "")),
                avatar_object_name=current.avatar_object_name,
                avatar_storage_url=current.avatar_storage_url,
            )

    return [players[key] for key in sorted(players)]
