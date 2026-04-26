from __future__ import annotations

from sina_csl_scraper.models import MatchResult, PlayerProfile, RankingDataset, StandingEntry, TeamProfile
from sina_csl_scraper.team_insights import build_team_insights


def test_build_team_insights_computes_opponent_and_player_contributions() -> None:
    results = build_team_insights(
        season=2026,
        round_number=4,
        snapshot_kind="live",
        teams=[
            TeamProfile(team_id=1, team_name="成都蓉城", avatar_storage_url="chengdu.png"),
            TeamProfile(team_id=2, team_name="上海申花", avatar_storage_url="shenhua.png"),
            TeamProfile(team_id=3, team_name="北京国安", avatar_storage_url="guoan.png"),
        ],
        players=[
            PlayerProfile(
                player_id=11,
                player_name="费利佩",
                team_id=1,
                team_name="成都蓉城",
                avatar_storage_url="felipe.png",
            ),
            PlayerProfile(
                player_id=12,
                player_name="席尔瓦",
                team_id=1,
                team_name="成都蓉城",
                avatar_storage_url="silva.png",
            ),
            PlayerProfile(
                player_id=21,
                player_name="拉唐",
                team_id=2,
                team_name="上海申花",
                avatar_storage_url="latang.png",
            ),
        ],
        matches=[
            MatchResult(
                match_id=1001,
                season=2026,
                round_number=1,
                round_name="第1轮",
                date="2026-03-01",
                time="19:35",
                status="3",
                home_team_id=1,
                home_team_name="成都蓉城",
                home_score="3",
                away_team_id=2,
                away_team_name="上海申花",
                away_score="1",
                home_logo="chengdu.png",
                away_logo="shenhua.png",
            ),
            MatchResult(
                match_id=1002,
                season=2026,
                round_number=2,
                round_name="第2轮",
                date="2026-03-08",
                time="19:35",
                status="3",
                home_team_id=3,
                home_team_name="北京国安",
                home_score="0",
                away_team_id=1,
                away_team_name="成都蓉城",
                away_score="2",
                home_logo="guoan.png",
                away_logo="chengdu.png",
            ),
        ],
        standings=[
            StandingEntry(
                order=1,
                team_id=1,
                team_name="成都蓉城",
                team_logo="chengdu.png",
                played=2,
                wins=2,
                draws=0,
                losses=0,
                goals_for=5,
                goals_against=1,
                goal_difference=4,
                points=6,
            ),
            StandingEntry(
                order=2,
                team_id=2,
                team_name="上海申花",
                team_logo="shenhua.png",
                played=1,
                wins=0,
                draws=0,
                losses=1,
                goals_for=1,
                goals_against=3,
                goal_difference=-2,
                points=0,
            ),
            StandingEntry(
                order=3,
                team_id=3,
                team_name="北京国安",
                team_logo="guoan.png",
                played=1,
                wins=0,
                draws=0,
                losses=1,
                goals_for=0,
                goals_against=2,
                goal_difference=-2,
                points=0,
            ),
        ],
        team_rankings=[
            RankingDataset(
                slug="assists",
                label="助攻",
                item_id=3,
                entries=[
                    {"rank": 1, "score": "4", "team_id": 1, "team_name": "成都蓉城", "team_logo": "chengdu.png"},
                    {"rank": 2, "score": "1", "team_id": 2, "team_name": "上海申花", "team_logo": "shenhua.png"},
                ],
            ),
        ],
        player_rankings=[
            RankingDataset(
                slug="goals",
                label="进球",
                item_id=1,
                entries=[
                    {
                        "rank": 1,
                        "score": "4",
                        "player_id": 11,
                        "player_name": "费利佩",
                        "player_logo": "felipe.png",
                        "team_id": 1,
                        "team_name": "成都蓉城",
                        "penalty": None,
                    },
                    {
                        "rank": 2,
                        "score": "1",
                        "player_id": 12,
                        "player_name": "席尔瓦",
                        "player_logo": "silva.png",
                        "team_id": 1,
                        "team_name": "成都蓉城",
                        "penalty": None,
                    },
                    {
                        "rank": 3,
                        "score": "1",
                        "player_id": 21,
                        "player_name": "拉唐",
                        "player_logo": "latang.png",
                        "team_id": 2,
                        "team_name": "上海申花",
                        "penalty": None,
                    },
                ],
            ),
            RankingDataset(
                slug="assists",
                label="助攻",
                item_id=3,
                entries=[
                    {
                        "rank": 1,
                        "score": "3",
                        "player_id": 12,
                        "player_name": "席尔瓦",
                        "player_logo": "silva.png",
                        "team_id": 1,
                        "team_name": "成都蓉城",
                        "penalty": None,
                    },
                    {
                        "rank": 2,
                        "score": "1",
                        "player_id": 11,
                        "player_name": "费利佩",
                        "player_logo": "felipe.png",
                        "team_id": 1,
                        "team_name": "成都蓉城",
                        "penalty": None,
                    },
                ],
            ),
        ],
    )

    chengdu = next(item for item in results if item.team_id == 1)

    assert chengdu.goals_for_total == 5
    assert chengdu.goals_against_total == 1
    assert chengdu.rank_no == 1
    assert [item.opponent_team_name for item in chengdu.goals_for_by_opponent] == ["上海申花", "北京国安"]
    assert [item.goals for item in chengdu.goals_for_by_opponent] == [3, 2]
    assert [item.player_name for item in chengdu.goals_for_by_player] == ["费利佩", "席尔瓦"]
    assert [item.goals for item in chengdu.goals_for_by_player] == [4, 1]
    assert [item.player_name for item in chengdu.assists_for_by_player] == ["席尔瓦", "费利佩"]
    assert [item.assists for item in chengdu.assists_for_by_player] == [3, 1]
    assert [item.opponent_team_name for item in chengdu.goals_against_by_opponent] == ["上海申花"]
    assert [item.goals for item in chengdu.goals_against_by_opponent] == [1]


def test_build_team_insights_appends_other_bucket_for_unattributed_goals() -> None:
    results = build_team_insights(
        season=2026,
        round_number=4,
        snapshot_kind="live",
        teams=[
            TeamProfile(team_id=1, team_name="成都蓉城", avatar_storage_url="chengdu.png"),
            TeamProfile(team_id=2, team_name="上海申花", avatar_storage_url="shenhua.png"),
        ],
        players=[
            PlayerProfile(
                player_id=11,
                player_name="费利佩",
                team_id=1,
                team_name="成都蓉城",
                avatar_storage_url="felipe.png",
            ),
        ],
        matches=[
            MatchResult(
                match_id=1001,
                season=2026,
                round_number=1,
                round_name="第1轮",
                date="2026-03-01",
                time="19:35",
                status="3",
                home_team_id=1,
                home_team_name="成都蓉城",
                home_score="3",
                away_team_id=2,
                away_team_name="上海申花",
                away_score="0",
                home_logo="chengdu.png",
                away_logo="shenhua.png",
            ),
        ],
        standings=[
            StandingEntry(
                order=1,
                team_id=1,
                team_name="成都蓉城",
                team_logo="chengdu.png",
                played=1,
                wins=1,
                draws=0,
                losses=0,
                goals_for=3,
                goals_against=0,
                goal_difference=3,
                points=3,
            ),
            StandingEntry(
                order=2,
                team_id=2,
                team_name="上海申花",
                team_logo="shenhua.png",
                played=1,
                wins=0,
                draws=0,
                losses=1,
                goals_for=0,
                goals_against=3,
                goal_difference=-3,
                points=0,
            ),
        ],
        team_rankings=[
            RankingDataset(
                slug="assists",
                label="助攻",
                item_id=3,
                entries=[
                    {"rank": 1, "score": "2", "team_id": 1, "team_name": "成都蓉城", "team_logo": "chengdu.png"},
                ],
            ),
        ],
        player_rankings=[
            RankingDataset(
                slug="goals",
                label="进球",
                item_id=1,
                entries=[
                    {
                        "rank": 1,
                        "score": "2",
                        "player_id": 11,
                        "player_name": "费利佩",
                        "player_logo": "felipe.png",
                        "team_id": 1,
                        "team_name": "成都蓉城",
                        "penalty": None,
                    },
                ],
            ),
        ],
    )

    chengdu = next(item for item in results if item.team_id == 1)

    assert [item.player_name for item in chengdu.goals_for_by_player] == ["费利佩", "其他/未归因"]
    assert [item.goals for item in chengdu.goals_for_by_player] == [2, 1]


def test_build_team_insights_keeps_player_shares_against_team_total_goals() -> None:
    results = build_team_insights(
        season=2026,
        round_number=4,
        snapshot_kind="live",
        teams=[
            TeamProfile(team_id=1, team_name="成都蓉城", avatar_storage_url="chengdu.png"),
            TeamProfile(team_id=2, team_name="上海申花", avatar_storage_url="shenhua.png"),
        ],
        players=[
            PlayerProfile(
                player_id=11,
                player_name="费利佩",
                team_id=1,
                team_name="成都蓉城",
                avatar_storage_url="felipe.png",
            ),
            PlayerProfile(
                player_id=12,
                player_name="席尔瓦",
                team_id=1,
                team_name="成都蓉城",
                avatar_storage_url="silva.png",
            ),
        ],
        matches=[],
        standings=[
            StandingEntry(
                order=1,
                team_id=1,
                team_name="成都蓉城",
                team_logo="chengdu.png",
                played=1,
                wins=1,
                draws=0,
                losses=0,
                goals_for=6,
                goals_against=0,
                goal_difference=6,
                points=3,
            ),
            StandingEntry(
                order=2,
                team_id=2,
                team_name="上海申花",
                team_logo="shenhua.png",
                played=1,
                wins=0,
                draws=0,
                losses=1,
                goals_for=0,
                goals_against=6,
                goal_difference=-6,
                points=0,
            ),
        ],
        team_rankings=[
            RankingDataset(
                slug="assists",
                label="助攻",
                item_id=3,
                entries=[
                    {"rank": 1, "score": "2", "team_id": 1, "team_name": "成都蓉城", "team_logo": "chengdu.png"},
                ],
            ),
        ],
        player_rankings=[
            RankingDataset(
                slug="goals",
                label="进球",
                item_id=1,
                entries=[
                    {
                        "rank": 1,
                        "score": "4",
                        "player_id": 11,
                        "player_name": "费利佩",
                        "player_logo": "felipe.png",
                        "team_id": 1,
                        "team_name": "成都蓉城",
                        "penalty": None,
                    },
                    {
                        "rank": 2,
                        "score": "2",
                        "player_id": 12,
                        "player_name": "席尔瓦",
                        "player_logo": "silva.png",
                        "team_id": 1,
                        "team_name": "成都蓉城",
                        "penalty": None,
                    },
                ],
            ),
        ],
    )

    chengdu = next(item for item in results if item.team_id == 1)

    assert [round(item.share, 4) for item in chengdu.goals_for_by_player] == [0.6667, 0.3333]
