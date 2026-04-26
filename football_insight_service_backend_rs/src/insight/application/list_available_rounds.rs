use std::sync::Arc;

use crate::insight::{
    domain::round_reference::RoundReference,
    ports::insight_query_repository::InsightQueryRepository,
};

pub struct ListAvailableRoundsUseCase {
    repository: Arc<dyn InsightQueryRepository>,
}

impl ListAvailableRoundsUseCase {
    pub fn new(repository: Arc<dyn InsightQueryRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, season: i32) -> anyhow::Result<Vec<RoundReference>> {
        self.repository.list_available_rounds(season).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::insight::{
        application::list_available_rounds::ListAvailableRoundsUseCase,
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
            unreachable!()
        }

        async fn get_round_overview(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<InsightOverview> {
            unreachable!()
        }

        async fn list_available_rounds(&self, season: i32) -> anyhow::Result<Vec<RoundReference>> {
            Ok(vec![RoundReference {
                season,
                round_number: 4,
                finalized_at: Some("2026-04-05T08:00:00Z".to_string()),
                status: "completed".to_string(),
                total_matches: 8,
                completed_matches: 8,
            }])
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
    async fn execute_returns_available_rounds() {
        let use_case = ListAvailableRoundsUseCase::new(Arc::new(FakeRepository));

        let result = use_case.execute(2026).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].round_number, 4);
    }
}
