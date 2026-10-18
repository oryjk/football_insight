"""Validated historical score snapshots; never writes the existing match table."""

from datetime import datetime, timedelta, timezone
from .model import Game

BEIJING = timezone(timedelta(hours=8))


def expected_count(year: int) -> int:
    return {2020: 160, 2021: 176, 2022: 306}.get(year, 240)


def game_from_dict(row: dict) -> Game:
    try:
        if "status" in row and str(row["status"]) != "3":
            raise ValueError("unfinished historical match")
        year = int(row["season"])
        home, away = str(row["home_score"]), str(row["away_score"])
        if (
            not home.isascii()
            or not home.isdigit()
            or not away.isascii()
            or not away.isdigit()
        ):
            raise ValueError("missing or invalid score")
        return Game(
            int(row["match_id"]),
            year,
            datetime.fromisoformat(f"{row['date']}T{row['time']}").replace(
                tzinfo=BEIJING
            ),
            int(row["home_team_id"]),
            int(row["away_team_id"]),
            int(home),
            int(away),
            year in (2020, 2021),
        )
    except (KeyError, TypeError, ValueError) as error:
        raise ValueError("invalid historical score or kickoff") from error


def validate_history(rows: list[Game], years: list[int]) -> None:
    if len({g.match_id for g in rows}) != len(rows):
        raise ValueError("duplicate historical match_id")
    if any(
        g.season not in years
        or g.home_team_id <= 0
        or g.away_team_id <= 0
        or g.home_team_id == g.away_team_id
        or min(g.home_score, g.away_score) < 0
        or g.neutral != (g.season in (2020, 2021))
        for g in rows
    ):
        raise ValueError("invalid historical match")
    for year in years:
        if sum(g.season == year for g in rows) != expected_count(year):
            raise ValueError(f"{year}: incomplete historical season")
