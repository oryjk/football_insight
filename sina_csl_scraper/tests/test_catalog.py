from __future__ import annotations

from sina_csl_scraper.catalog import build_player_profiles, build_team_profiles
from sina_csl_scraper.models import MatchResult, RankingDataset, StandingEntry


def test_build_team_profiles_deduplicates_multiple_sources() -> None:
    matches = [
        MatchResult(
            match_id=1,
            season=2026,
            round_number=1,
            round_name="第1轮",
            date="2026-03-01",
            time="19:35",
            status="3",
            home_team_id=77680,
            home_team_name="成都蓉城",
            home_score="2",
            away_team_id=500,
            away_team_name="云南玉昆",
            away_score="1",
            home_logo="https://cdn.example.com/teams/77680.png",
            away_logo="https://cdn.example.com/teams/500.png",
        )
    ]
    standings = [
        StandingEntry(
            order=1,
            team_id=77680,
            team_name="成都蓉城",
            team_logo="https://cdn.example.com/teams/77680-standing.png",
            played=4,
            wins=3,
            draws=1,
            losses=0,
            goals_for=14,
            goals_against=5,
            goal_difference=9,
            points=10,
        )
    ]
    team_rankings = [
        RankingDataset(
            slug="goals",
            label="进球",
            item_id=1,
            entries=[
                {
                    "rank": 1,
                    "score": "14",
                    "team_id": 77680,
                    "team_name": "成都蓉城",
                    "team_logo": "https://cdn.example.com/teams/77680-rank.png",
                }
            ],
        )
    ]

    profiles = build_team_profiles(matches, standings, team_rankings)

    assert len(profiles) == 2
    assert profiles[0].team_id == 500
    assert profiles[1].team_id == 77680
    assert profiles[1].avatar_source_url == "https://cdn.example.com/teams/77680-standing.png"


def test_build_player_profiles_deduplicates_players() -> None:
    player_rankings = [
        RankingDataset(
            slug="goals",
            label="进球",
            item_id=2,
            entries=[
                {
                    "rank": 1,
                    "score": "4",
                    "player_id": 204211,
                    "player_name": "费利佩",
                    "player_logo": "https://cdn.example.com/players/204211.png",
                    "team_id": 77680,
                    "team_name": "成都蓉城",
                    "penalty": "1",
                }
            ],
        ),
        RankingDataset(
            slug="shots",
            label="射门",
            item_id=4,
            entries=[
                {
                    "rank": 2,
                    "score": "11",
                    "player_id": 204211,
                    "player_name": "费利佩",
                    "player_logo": "https://cdn.example.com/players/204211-shot.png",
                    "team_id": 77680,
                    "team_name": "成都蓉城",
                    "penalty": None,
                },
                {
                    "rank": 3,
                    "score": "9",
                    "player_id": 7727346,
                    "player_name": "席尔瓦",
                    "player_logo": "https://cdn.example.com/players/7727346.png",
                    "team_id": 77680,
                    "team_name": "成都蓉城",
                    "penalty": None,
                },
            ],
        ),
    ]

    profiles = build_player_profiles(player_rankings)

    assert [item.player_id for item in profiles] == [204211, 7727346]
    assert profiles[0].avatar_source_url == "https://cdn.example.com/players/204211.png"
