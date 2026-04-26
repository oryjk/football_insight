from __future__ import annotations

from dataclasses import dataclass


DEFAULT_LEAGUE_ID = 213
DEFAULT_BASE_URL = "https://goal.sports.sina.cn"
DEFAULT_TIMEOUT = 20
DEFAULT_PLAYER_PAGE_SIZE = 50


@dataclass(frozen=True)
class RankingCategory:
    slug: str
    label: str
    item_id: int


TEAM_RANKING_CATEGORIES = (
    RankingCategory("goals", "进球", 1),
    RankingCategory("penalties", "点球", 22),
    RankingCategory("assists", "助攻", 3),
    RankingCategory("key_passes", "关键传球", 19),
    RankingCategory("shots", "射门", 4),
    RankingCategory("shots_on_target", "射正", 5),
    RankingCategory("passes", "传球", 6),
    RankingCategory("successful_passes", "成功传球", 7),
    RankingCategory("pass_success_rate", "传球成功率", 101),
    RankingCategory("cross_success_rate", "传中成功率", 103),
    RankingCategory("interceptions", "拦截", 10),
    RankingCategory("tackles", "抢断", 23),
    RankingCategory("clearances", "解围", 12),
    RankingCategory("fouls", "犯规", 13),
    RankingCategory("fouled", "被侵犯", 14),
    RankingCategory("red_cards", "红牌", 15),
    RankingCategory("yellow_cards", "黄牌", 16),
    RankingCategory("shots_hit_woodwork", "击中门框", 17),
)

PLAYER_RANKING_CATEGORIES = (
    RankingCategory("goals", "进球", 2),
    RankingCategory("assists", "助攻", 3),
    RankingCategory("shots", "射门", 4),
    RankingCategory("shots_on_target", "射正", 5),
    RankingCategory("offsides", "越位", 6),
    RankingCategory("passes", "传球", 7),
    RankingCategory("successful_passes", "成功传球", 8),
    RankingCategory("interceptions", "拦截", 9),
    RankingCategory("tackles", "抢断", 21),
    RankingCategory("clearances", "解围", 11),
    RankingCategory("fouls", "犯规", 12),
    RankingCategory("fouled", "被侵犯", 13),
    RankingCategory("red_cards", "红牌", 14),
    RankingCategory("yellow_cards", "黄牌", 15),
    RankingCategory("saves", "扑救", 16),
    RankingCategory("minutes_played", "出场时间", 17),
    RankingCategory("key_passes", "关键传球", 18),
    RankingCategory("appearances", "出场", 19),
)
