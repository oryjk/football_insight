#[cfg(test)]
mod tests {
    use crate::insight::domain::{
        insight_summary::{RankingMovement, generate_insight_summary},
        overview::{InsightOverview, OverviewMatch, OverviewPlayer, OverviewStanding},
    };

    fn sample_overview() -> InsightOverview {
        InsightOverview {
            view_kind: "live".to_string(),
            round_number: None,
            current_season: 2026,
            latest_scrape_finished_at: None,
            total_matches: 240,
            total_teams: 16,
            total_players: 149,
            player_ranking_categories: 18,
            team_ranking_categories: 18,
            standings_top: vec![
                OverviewStanding {
                    rank_no: 1,
                    team_id: 77680,
                    team_name: "成都蓉城".to_string(),
                    points: 10,
                    avatar_storage_url: None,
                },
                OverviewStanding {
                    rank_no: 2,
                    team_id: 500,
                    team_name: "云南玉昆".to_string(),
                    points: 9,
                    avatar_storage_url: None,
                },
            ],
            recent_matches: vec![OverviewMatch {
                match_id: 288601,
                round_number: 4,
                match_date: "2026-04-05".to_string(),
                match_time: "15:30".to_string(),
                home_team_name: "浙江队".to_string(),
                away_team_name: "重庆铜梁龙".to_string(),
                home_score: "0".to_string(),
                away_score: "1".to_string(),
            }],
            top_scorers: vec![
                OverviewPlayer {
                    rank_no: 1,
                    player_id: 204211,
                    player_name: "费利佩".to_string(),
                    team_name: "成都蓉城".to_string(),
                    score_value: "4".to_string(),
                    avatar_storage_url: None,
                },
                OverviewPlayer {
                    rank_no: 2,
                    player_id: 7727346,
                    player_name: "席尔瓦".to_string(),
                    team_name: "成都蓉城".to_string(),
                    score_value: "4".to_string(),
                    avatar_storage_url: None,
                },
            ],
            insight_summary: None,
        }
    }

    #[test]
    fn generate_summary_uses_focus_match_and_movements() {
        let summary = generate_insight_summary(
            &sample_overview(),
            Some(RankingMovement {
                name: "重庆铜梁龙".to_string(),
                current_rank: 6,
                previous_rank: 9,
            }),
            Some(RankingMovement {
                name: "席尔瓦".to_string(),
                current_rank: 2,
                previous_rank: 5,
            }),
        );

        assert!(summary.headline.contains("浙江队 0:1 重庆铜梁龙"));
        assert!(summary.summary.contains("成都蓉城"));
        assert!(summary.summary.contains("费利佩"));
        assert!(summary.summary.contains("席尔瓦"));
        assert_eq!(summary.focus_match_id, Some(288601));
        assert_eq!(summary.bullets.len(), 3);
        assert!(summary.bullets[0].contains("重庆铜梁龙"));
        assert!(summary.bullets[1].contains("席尔瓦"));
    }

    #[test]
    fn generate_summary_falls_back_when_match_is_missing() {
        let mut overview = sample_overview();
        overview.recent_matches.clear();

        let summary = generate_insight_summary(&overview, None, None);

        assert!(summary.headline.contains("成都蓉城"));
        assert!(summary.summary.contains("积分榜"));
        assert_eq!(summary.focus_match_id, None);
        assert!(!summary.bullets.is_empty());
    }
}
