use std::sync::Arc;

use crate::insight::{
    domain::rankings::RankingsView, ports::insight_query_repository::InsightQueryRepository,
};

pub struct GetLiveRankingsUseCase {
    repository: Arc<dyn InsightQueryRepository>,
}

impl GetLiveRankingsUseCase {
    pub fn new(repository: Arc<dyn InsightQueryRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> anyhow::Result<RankingsView> {
        self.repository.get_live_rankings().await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::insight::{
        application::get_live_rankings::GetLiveRankingsUseCase,
        domain::{
            match_list::MatchListView,
            overview::InsightOverview,
            rankings::{PlayerRankingCategory, RankingsView, TeamRankingCategory},
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
            Ok(RankingsView {
                view_kind: "live".to_string(),
                round_number: None,
                current_season: 2026,
                standings_tables: Vec::new(),
                team_categories: vec![TeamRankingCategory {
                    slug: "goals".to_string(),
                    label: "进球".to_string(),
                    item_id: 22,
                    entries: Vec::new(),
                }],
                player_categories: vec![PlayerRankingCategory {
                    slug: "goals".to_string(),
                    label: "进球".to_string(),
                    item_id: 1,
                    entries: Vec::new(),
                }],
            })
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
    async fn execute_returns_live_rankings() {
        let use_case = GetLiveRankingsUseCase::new(Arc::new(FakeRepository));

        let result = use_case.execute().await.unwrap();

        assert_eq!(result.view_kind, "live");
        assert!(result.standings_tables.is_empty());
        assert_eq!(result.team_categories.len(), 1);
        assert_eq!(result.player_categories.len(), 1);
    }
}
