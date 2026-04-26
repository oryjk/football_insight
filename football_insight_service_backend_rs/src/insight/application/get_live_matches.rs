use std::sync::Arc;

use crate::insight::{
    domain::match_list::MatchListView, ports::insight_query_repository::InsightQueryRepository,
};

pub struct GetLiveMatchesUseCase {
    repository: Arc<dyn InsightQueryRepository>,
}

impl GetLiveMatchesUseCase {
    pub fn new(repository: Arc<dyn InsightQueryRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> anyhow::Result<MatchListView> {
        self.repository.get_live_matches().await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::insight::{
        application::get_live_matches::GetLiveMatchesUseCase,
        domain::{
            match_list::{MatchCard, MatchListView, MatchTechnicalStat},
            overview::InsightOverview,
            rankings::RankingsView,
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
            Ok(MatchListView {
                view_kind: "live".to_string(),
                round_number: None,
                current_season: 2026,
                matches: vec![MatchCard {
                    match_id: 1,
                    round_number: 3,
                    match_date: "2026-04-05".to_string(),
                    match_time: "19:35".to_string(),
                    status: "finished".to_string(),
                    home_team_id: 136,
                    home_team_name: "北京国安".to_string(),
                    home_score: "2".to_string(),
                    away_team_id: 93740,
                    away_team_name: "浙江队".to_string(),
                    away_score: "0".to_string(),
                    home_team_avatar: None,
                    away_team_avatar: None,
                    leisu_match_id: Some(4422785),
                    home_corners: Some(4),
                    away_corners: Some(2),
                    corner_source: Some("leisu_detail".to_string()),
                    technical_stats: vec![
                        MatchTechnicalStat {
                            slug: "attacks".to_string(),
                            label: "进攻".to_string(),
                            home_value: 92,
                            away_value: 118,
                            unit: None,
                        },
                        MatchTechnicalStat {
                            slug: "corners".to_string(),
                            label: "角球".to_string(),
                            home_value: 4,
                            away_value: 2,
                            unit: None,
                        },
                    ],
                }],
            })
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
    async fn execute_returns_live_matches() {
        let use_case = GetLiveMatchesUseCase::new(Arc::new(FakeRepository));

        let result = use_case.execute().await.unwrap();

        assert_eq!(result.view_kind, "live");
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].home_corners, Some(4));
        assert_eq!(result.matches[0].technical_stats.len(), 2);
        assert_eq!(result.matches[0].technical_stats[0].slug, "attacks");
    }
}
