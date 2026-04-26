use std::sync::Arc;

use crate::insight::{
    domain::overview::InsightOverview, ports::insight_query_repository::InsightQueryRepository,
};

pub struct GetRoundOverviewUseCase {
    repository: Arc<dyn InsightQueryRepository>,
}

impl GetRoundOverviewUseCase {
    pub fn new(repository: Arc<dyn InsightQueryRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, season: i32, round_number: i32) -> anyhow::Result<InsightOverview> {
        self.repository
            .get_round_overview(season, round_number)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::insight::{
        application::get_round_overview::GetRoundOverviewUseCase,
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
            season: i32,
            round_number: i32,
        ) -> anyhow::Result<InsightOverview> {
            Ok(InsightOverview {
                view_kind: "round_final".to_string(),
                round_number: Some(round_number),
                current_season: season,
                latest_scrape_finished_at: None,
                total_matches: i64::from(round_number),
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
    async fn execute_returns_specific_round_overview() {
        let use_case = GetRoundOverviewUseCase::new(Arc::new(FakeRepository));

        let result = use_case.execute(2026, 4).await.unwrap();

        assert_eq!(result.current_season, 2026);
        assert_eq!(result.total_matches, 4);
    }
}
