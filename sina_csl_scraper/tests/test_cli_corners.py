from __future__ import annotations

import json

from sina_csl_scraper.cli import run_scrape
from sina_csl_scraper.models import LeagueInfo, MatchResult, RankingDataset, StandingEntry


class FakeSinaClient:
    def fetch_league_info(self) -> LeagueInfo:
        return LeagueInfo(
            lid=213,
            name="中超",
            current_season=2026,
            current_round=1,
            max_round=1,
        )

    def fetch_standings(self, season: int) -> list[StandingEntry]:
        return [
            StandingEntry(
                order=1,
                team_id=136,
                team_name="北京国安",
                team_logo="guoan.png",
                played=1,
                wins=1,
                draws=0,
                losses=0,
                goals_for=2,
                goals_against=0,
                goal_difference=2,
                points=3,
            )
        ]

    def fetch_all_matches(self, season: int, max_round: int | None = None) -> list[MatchResult]:
        return [
            MatchResult(
                match_id=288579,
                season=2026,
                round_number=1,
                round_name="第1轮",
                date="2026-03-08",
                time="19:00",
                status="3",
                home_team_id=110645,
                home_team_name="武汉三镇",
                home_score="0",
                away_team_id=136,
                away_team_name="北京国安",
                away_score="2",
                home_logo="home.png",
                away_logo="away.png",
            )
        ]

    def fetch_all_team_rankings(self, season: int) -> list[RankingDataset]:
        return []

    def fetch_all_player_rankings(self, season: int, limit: int = 50) -> list[RankingDataset]:
        return []


class FakeCornerEnricher:
    def enrich_matches(self, matches: list[MatchResult]) -> list[MatchResult]:
        return [
            MatchResult(
                match_id=matches[0].match_id,
                season=matches[0].season,
                round_number=matches[0].round_number,
                round_name=matches[0].round_name,
                date=matches[0].date,
                time=matches[0].time,
                status=matches[0].status,
                home_team_id=matches[0].home_team_id,
                home_team_name=matches[0].home_team_name,
                home_score=matches[0].home_score,
                away_team_id=matches[0].away_team_id,
                away_team_name=matches[0].away_team_name,
                away_score=matches[0].away_score,
                home_logo=matches[0].home_logo,
                away_logo=matches[0].away_logo,
                leisu_match_id=4422785,
                home_corners=4,
                away_corners=7,
                corner_source="leisu_detail",
                technical_stats=[
                    {
                        "slug": "attacks",
                        "label": "进攻",
                        "home_value": 92,
                        "away_value": 118,
                        "unit": None,
                    },
                    {
                        "slug": "corners",
                        "label": "角球",
                        "home_value": 4,
                        "away_value": 7,
                        "unit": None,
                    },
                ],
            )
        ]


def test_run_scrape_writes_enriched_corner_fields_into_matches_json(tmp_path) -> None:
    run_scrape(
        season=2026,
        output_dir=tmp_path,
        client=FakeSinaClient(),
        corner_enricher=FakeCornerEnricher(),
        enrich_corners=True,
    )

    matches_payload = json.loads((tmp_path / "2026" / "matches.json").read_text(encoding="utf-8"))

    assert matches_payload == [
        {
            "match_id": 288579,
            "season": 2026,
            "round_number": 1,
            "round_name": "第1轮",
            "date": "2026-03-08",
            "time": "19:00",
            "status": "3",
            "home_team_id": 110645,
            "home_team_name": "武汉三镇",
            "home_score": "0",
            "away_team_id": 136,
            "away_team_name": "北京国安",
            "away_score": "2",
            "home_logo": "home.png",
            "away_logo": "away.png",
            "leisu_match_id": 4422785,
            "home_corners": 4,
            "away_corners": 7,
            "corner_source": "leisu_detail",
            "technical_stats": [
                {
                    "slug": "attacks",
                    "label": "进攻",
                    "home_value": 92,
                    "away_value": 118,
                    "unit": None,
                },
                {
                    "slug": "corners",
                    "label": "角球",
                    "home_value": 4,
                    "away_value": 7,
                    "unit": None,
                },
            ],
        }
    ]
