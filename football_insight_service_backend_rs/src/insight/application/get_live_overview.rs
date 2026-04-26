use std::sync::Arc;

use crate::insight::{
    domain::overview::InsightOverview, ports::insight_query_repository::InsightQueryRepository,
};

pub struct GetLiveOverviewUseCase {
    repository: Arc<dyn InsightQueryRepository>,
}

impl GetLiveOverviewUseCase {
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

    use crate::insight::{
        application::get_live_overview::GetLiveOverviewUseCase,
        domain::{
            match_list::MatchListView, overview::InsightOverview, rankings::RankingsView,
            round_reference::RoundReference,
        },
        ports::insight_query_repository::InsightQueryRepository,
    };

    struct FakeRepository;

    #[async_trait]
    impl InsightQueryRepository for FakeRepository {
        async fn get_live_overview(&self) -> anyhow::Result<InsightOverview> {
            Ok(InsightOverview {
                view_kind: "live".to_string(),
                round_number: None,
                current_season: 2026,
                latest_scrape_finished_at: None,
                total_matches: 240,
                total_teams: 16,
                total_players: 153,
                player_ranking_categories: 18,
                team_ranking_categories: 18,
                standings_top: Vec::new(),
                recent_matches: Vec::new(),
                top_scorers: Vec::new(),
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

        async fn list_available_rounds(&self, _season: i32) -> anyhow::Result<Vec<RoundReference>> {
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
    async fn execute_returns_live_overview() {
        let use_case = GetLiveOverviewUseCase::new(Arc::new(FakeRepository));

        let result = use_case.execute().await.unwrap();

        assert_eq!(result.current_season, 2026);
        assert_eq!(result.total_players, 153);
    }
}
