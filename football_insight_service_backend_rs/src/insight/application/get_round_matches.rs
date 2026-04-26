use std::sync::Arc;

use crate::insight::{
    domain::match_list::MatchListView, ports::insight_query_repository::InsightQueryRepository,
};

pub struct GetRoundMatchesUseCase {
    repository: Arc<dyn InsightQueryRepository>,
}

impl GetRoundMatchesUseCase {
    pub fn new(repository: Arc<dyn InsightQueryRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, season: i32, round_number: i32) -> anyhow::Result<MatchListView> {
        self.repository
            .get_round_matches(season, round_number)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::insight::{
        application::get_round_matches::GetRoundMatchesUseCase,
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
            round_number: i32,
        ) -> anyhow::Result<MatchListView> {
            Ok(MatchListView {
                view_kind: "round_final".to_string(),
                round_number: Some(round_number),
                current_season: 2026,
                matches: Vec::new(),
            })
        }
    }

    #[tokio::test]
    async fn execute_returns_round_matches() {
        let use_case = GetRoundMatchesUseCase::new(Arc::new(FakeRepository));

        let result = use_case.execute(2026, 3).await.unwrap();

        assert_eq!(result.view_kind, "round_final");
        assert_eq!(result.round_number, Some(3));
    }
}
