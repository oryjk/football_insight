use crate::insight::domain::rankings::{
    PlayerRankingEntry, RankingsView, StandingsTable, StandingsTableEntry,
};

use super::public_system_config::HomeBriefingMarquees;

pub fn build_home_briefing_marquees_from_rankings(rankings: &RankingsView) -> HomeBriefingMarquees {
    HomeBriefingMarquees {
        leader: build_leader_messages(rankings),
        scorer: build_player_category_messages(rankings, "goals", "射手榜", "球", "射手"),
        assist: build_player_category_messages(rankings, "assists", "助攻榜", "次助攻", "助攻手"),
    }
}

pub fn merge_home_briefing_marquees(
    generated: HomeBriefingMarquees,
    fallback: HomeBriefingMarquees,
) -> HomeBriefingMarquees {
    HomeBriefingMarquees {
        leader: if generated.leader.is_empty() {
            fallback.leader
        } else {
            generated.leader
        },
        scorer: if generated.scorer.is_empty() {
            fallback.scorer
        } else {
            generated.scorer
        },
        assist: if generated.assist.is_empty() {
            fallback.assist
        } else {
            generated.assist
        },
    }
}

fn pick_primary_standings_table(rankings: &RankingsView) -> Option<&StandingsTable> {
    rankings
        .standings_tables
        .iter()
        .find(|table| table.slug == "standings")
        .or_else(|| {
            rankings
                .standings_tables
                .iter()
                .find(|table| table.slug == "standings_with_penalty")
        })
        .or_else(|| rankings.standings_tables.first())
}

fn find_distinct_points_entry(
    entries: &[StandingsTableEntry],
    top_points: i32,
) -> Option<&StandingsTableEntry> {
    entries.iter().find(|entry| entry.points < top_points)
}

fn find_distinct_score_entry<'a>(
    entries: &'a [PlayerRankingEntry],
    top_value: i32,
) -> Option<&'a PlayerRankingEntry> {
    entries
        .iter()
        .find(|entry| parse_score_value(&entry.score_value).is_some_and(|value| value < top_value))
}

fn parse_score_value(value: &str) -> Option<i32> {
    value.trim().parse::<i32>().ok()
}

fn join_names<'a>(names: impl Iterator<Item = &'a str>) -> String {
    names.collect::<Vec<_>>().join("、")
}

trait OptionVecExt {
    fn into_messages(self) -> Vec<String>;
}

impl OptionVecExt for Option<Vec<String>> {
    fn into_messages(self) -> Vec<String> {
        self.unwrap_or_default()
    }
}

fn build_leader_messages(rankings: &RankingsView) -> Vec<String> {
    (|| {
        let table = pick_primary_standings_table(rankings)?;
        let leader = table.entries.first()?;
        let top_points = leader.points;
        let top_group = table
            .entries
            .iter()
            .take_while(|entry| entry.points == top_points)
            .collect::<Vec<_>>();

        let mut messages = Vec::new();
        if top_group.len() > 1 {
            messages.push(format!(
                "{}目前同分并列榜首",
                join_names(top_group.iter().map(|entry| entry.team_name.as_str()))
            ));
        } else {
            messages.push(format!("{}暂时领跑积分榜", leader.team_name));
        }

        if let Some(second) = find_distinct_points_entry(&table.entries, top_points) {
            let gap = top_points - second.points;
            if gap == 0 {
                messages.push("榜首竞争仍然处在并列状态".to_string());
            } else {
                messages.push(format!("榜首与第二名只差 {} 分", gap));
            }
        }

        if let Some(third) = table.entries.get(2) {
            let gap = top_points - third.points;
            messages.push(format!("前 3 名之间只差 {} 分", gap));
        }

        Some(messages)
    })()
    .into_messages()
}

fn build_player_category_messages(
    rankings: &RankingsView,
    category_slug: &str,
    board_label: &str,
    unit: &str,
    group_label: &str,
) -> Vec<String> {
    (|| {
        let category = rankings
            .player_categories
            .iter()
            .find(|item| item.slug == category_slug)?;
        let leader = category.entries.first()?;
        let top_value = parse_score_value(&leader.score_value)?;
        let top_group = category
            .entries
            .iter()
            .take_while(|entry| parse_score_value(&entry.score_value) == Some(top_value))
            .collect::<Vec<_>>();

        let mut messages = Vec::new();
        if top_group.len() > 1 {
            messages.push(format!(
                "{}暂时并列{}头名",
                join_names(top_group.iter().map(|entry| entry.player_name.as_str())),
                board_label
            ));
        } else {
            messages.push(format!("{}暂时领跑{}", leader.player_name, board_label));
        }

        if let Some(second) = find_distinct_score_entry(&category.entries, top_value) {
            let gap = top_value - parse_score_value(&second.score_value)?;
            messages.push(format!("只领先第二名 {} {}", gap, unit));
        }

        if let Some(third) = category
            .entries
            .get(2)
            .and_then(|entry| parse_score_value(&entry.score_value))
        {
            let gap = top_value - third;
            messages.push(format!("前 3 名{}只差 {} {}", group_label, gap, unit));
        }

        Some(messages)
    })()
    .into_messages()
}

#[cfg(test)]
mod tests {
    use super::{build_home_briefing_marquees_from_rankings, merge_home_briefing_marquees};
    use crate::insight::domain::rankings::{
        PlayerRankingCategory, PlayerRankingEntry, RankingsView, StandingsTable,
        StandingsTableEntry,
    };
    use crate::system_config::domain::public_system_config::HomeBriefingMarquees;

    fn create_rankings() -> RankingsView {
        RankingsView {
            view_kind: "live".to_string(),
            round_number: None,
            current_season: 2026,
            standings_tables: vec![StandingsTable {
                slug: "standings".to_string(),
                label: "积分榜".to_string(),
                note: String::new(),
                entries: vec![
                    StandingsTableEntry {
                        rank_no: 1,
                        team_id: 1,
                        team_name: "成都蓉城".to_string(),
                        played: 7,
                        wins: 5,
                        draws: 1,
                        losses: 1,
                        goals_for: 14,
                        goals_against: 6,
                        goal_difference: 8,
                        points: 16,
                        points_without_penalty: 16,
                        points_adjustment: 0,
                        avatar_storage_url: None,
                    },
                    StandingsTableEntry {
                        rank_no: 2,
                        team_id: 2,
                        team_name: "上海申花".to_string(),
                        played: 7,
                        wins: 5,
                        draws: 0,
                        losses: 2,
                        goals_for: 13,
                        goals_against: 7,
                        goal_difference: 6,
                        points: 15,
                        points_without_penalty: 15,
                        points_adjustment: 0,
                        avatar_storage_url: None,
                    },
                    StandingsTableEntry {
                        rank_no: 3,
                        team_id: 3,
                        team_name: "北京国安".to_string(),
                        played: 7,
                        wins: 4,
                        draws: 2,
                        losses: 1,
                        goals_for: 12,
                        goals_against: 8,
                        goal_difference: 4,
                        points: 14,
                        points_without_penalty: 14,
                        points_adjustment: 0,
                        avatar_storage_url: None,
                    },
                ],
            }],
            team_categories: vec![],
            player_categories: vec![
                PlayerRankingCategory {
                    slug: "goals".to_string(),
                    label: "射手榜".to_string(),
                    item_id: 1,
                    entries: vec![
                        PlayerRankingEntry {
                            rank_no: 1,
                            player_id: 11,
                            player_name: "费利佩".to_string(),
                            team_id: 1,
                            team_name: "成都蓉城".to_string(),
                            score_value: "7".to_string(),
                            penalty_value: Some("1".to_string()),
                            avatar_storage_url: None,
                        },
                        PlayerRankingEntry {
                            rank_no: 2,
                            player_id: 12,
                            player_name: "莱昂纳多".to_string(),
                            team_id: 2,
                            team_name: "上海申花".to_string(),
                            score_value: "6".to_string(),
                            penalty_value: Some("0".to_string()),
                            avatar_storage_url: None,
                        },
                        PlayerRankingEntry {
                            rank_no: 3,
                            player_id: 13,
                            player_name: "法比奥".to_string(),
                            team_id: 3,
                            team_name: "北京国安".to_string(),
                            score_value: "5".to_string(),
                            penalty_value: Some("0".to_string()),
                            avatar_storage_url: None,
                        },
                    ],
                },
                PlayerRankingCategory {
                    slug: "assists".to_string(),
                    label: "助攻榜".to_string(),
                    item_id: 2,
                    entries: vec![
                        PlayerRankingEntry {
                            rank_no: 1,
                            player_id: 21,
                            player_name: "罗慕洛".to_string(),
                            team_id: 1,
                            team_name: "成都蓉城".to_string(),
                            score_value: "7".to_string(),
                            penalty_value: None,
                            avatar_storage_url: None,
                        },
                        PlayerRankingEntry {
                            rank_no: 2,
                            player_id: 22,
                            player_name: "奥斯卡".to_string(),
                            team_id: 4,
                            team_name: "上海海港".to_string(),
                            score_value: "5".to_string(),
                            penalty_value: None,
                            avatar_storage_url: None,
                        },
                        PlayerRankingEntry {
                            rank_no: 3,
                            player_id: 23,
                            player_name: "谢文能".to_string(),
                            team_id: 5,
                            team_name: "山东泰山".to_string(),
                            score_value: "4".to_string(),
                            penalty_value: None,
                            avatar_storage_url: None,
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn builds_briefing_marquees_from_live_rankings() {
        let marquees = build_home_briefing_marquees_from_rankings(&create_rankings());

        assert_eq!(
            marquees.leader,
            vec![
                "成都蓉城暂时领跑积分榜",
                "榜首与第二名只差 1 分",
                "前 3 名之间只差 2 分",
            ]
        );
        assert_eq!(
            marquees.scorer,
            vec![
                "费利佩暂时领跑射手榜",
                "只领先第二名 1 球",
                "前 3 名射手只差 2 球",
            ]
        );
        assert_eq!(
            marquees.assist,
            vec![
                "罗慕洛暂时领跑助攻榜",
                "只领先第二名 2 次助攻",
                "前 3 名助攻手只差 3 次助攻",
            ]
        );
    }

    #[test]
    fn falls_back_per_category_when_generated_messages_are_missing() {
        let merged = merge_home_briefing_marquees(
            HomeBriefingMarquees {
                leader: vec!["实时榜首文案".to_string()],
                scorer: Vec::new(),
                assist: Vec::new(),
            },
            HomeBriefingMarquees {
                leader: vec!["旧榜首文案".to_string()],
                scorer: vec!["旧射手文案".to_string()],
                assist: vec!["旧助攻文案".to_string()],
            },
        );

        assert_eq!(merged.leader, vec!["实时榜首文案"]);
        assert_eq!(merged.scorer, vec!["旧射手文案"]);
        assert_eq!(merged.assist, vec!["旧助攻文案"]);
    }
}
