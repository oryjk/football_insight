use std::sync::Arc;

use crate::insight::{
    domain::overview::InsightOverview, ports::insight_query_repository::InsightQueryRepository,
};

pub struct GetOverviewUseCase {
    repository: Arc<dyn InsightQueryRepository>,
}

impl GetOverviewUseCase {
    pub fn new(repository: Arc<dyn InsightQueryRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> anyhow::Result<InsightOverview> {
        self.repository.get_live_overview().await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};

    use crate::insight::{
        application::get_overview::GetOverviewUseCase,
        domain::{
            match_list::MatchListView,
            overview::{InsightOverview, OverviewMatch, OverviewPlayer, OverviewStanding},
            rankings::RankingsView,
        },
        ports::insight_query_repository::InsightQueryRepository,
    };

    struct FakeInsightQueryRepository;

    #[async_trait]
    impl InsightQueryRepository for FakeInsightQueryRepository {
        async fn get_live_overview(&self) -> anyhow::Result<InsightOverview> {
            Ok(InsightOverview {
                view_kind: "live".to_string(),
                round_number: None,
                current_season: 2026,
                latest_scrape_finished_at: Some(Utc.with_ymd_and_hms(2026, 4, 5, 8, 0, 0).unwrap()),
                total_matches: 240,
                total_teams: 16,
                total_players: 149,
                player_ranking_categories: 18,
                team_ranking_categories: 18,
                standings_top: vec![OverviewStanding {
                    rank_no: 1,
                    team_id: 77680,
                    team_name: "成都蓉城".to_string(),
                    points: 18,
                    avatar_storage_url: Some("https://cdn.example.com/teams/77680.png".to_string()),
                }],
                recent_matches: vec![OverviewMatch {
                    match_id: 1,
                    round_number: 6,
                    match_date: "2026-04-03".to_string(),
                    match_time: "19:35".to_string(),
                    home_team_name: "成都蓉城".to_string(),
                    away_team_name: "青岛西海岸".to_string(),
                    home_score: "5".to_string(),
                    away_score: "1".to_string(),
                }],
                top_scorers: vec![OverviewPlayer {
                    rank_no: 1,
                    player_id: 204211,
                    player_name: "费利佩".to_string(),
                    team_name: "成都蓉城".to_string(),
                    score_value: "7".to_string(),
                    avatar_storage_url: Some(
                        "https://cdn.example.com/players/204211.png".to_string(),
                    ),
                }],
                insight_summary: None,
            })
        }

        async fn get_round_overview(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<InsightOverview> {
            unreachable!()
        }

        async fn list_available_rounds(
            &self,
            _season: i32,
        ) -> anyhow::Result<Vec<crate::insight::domain::round_reference::RoundReference>> {
            unreachable!()
        }

        async fn get_live_rankings(&self) -> anyhow::Result<RankingsView> {
            unreachable!()
        }

        async fn get_live_team_insights(
            &self,
        ) -> anyhow::Result<crate::insight::domain::team_insight::TeamInsightsView> {
            unreachable!()
        }

        async fn get_round_rankings(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<RankingsView> {
            unreachable!()
        }

        async fn get_live_matches(&self) -> anyhow::Result<MatchListView> {
            unreachable!()
        }

        async fn get_round_matches(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<MatchListView> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn execute_returns_rich_overview_payload() {
        let use_case = GetOverviewUseCase::new(Arc::new(FakeInsightQueryRepository));

        let overview = use_case.execute().await.unwrap();

        assert_eq!(overview.current_season, 2026);
        assert_eq!(overview.total_players, 149);
        assert_eq!(overview.standings_top.len(), 1);
        assert_eq!(overview.recent_matches[0].home_team_name, "成都蓉城");
        assert_eq!(overview.top_scorers[0].player_name, "费利佩");
    }
}
