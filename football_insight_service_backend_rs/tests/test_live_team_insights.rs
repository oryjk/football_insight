use std::sync::Arc;

use async_trait::async_trait;
use football_insight_service_backend_rs::insight::{
    application::get_live_team_insights::GetLiveTeamInsightsUseCase,
    domain::{
        match_list::MatchListView,
        overview::InsightOverview,
        rankings::RankingsView,
        round_reference::RoundReference,
        team_insight::{
            AssistContribution, OpponentContribution, PlayerContribution, TeamInsight,
            TeamInsightTeam, TeamInsightsView,
        },
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

    async fn get_live_team_insights(&self) -> anyhow::Result<TeamInsightsView> {
        Ok(TeamInsightsView {
            view_kind: "live".to_string(),
            round_number: Some(4),
            current_season: 2026,
            teams: vec![TeamInsightTeam {
                team_id: 77680,
                team_name: "成都蓉城".to_string(),
                rank_no: 1,
                avatar_storage_url: Some("chengdu.png".to_string()),
            }],
            insights: vec![TeamInsight {
                team_id: 77680,
                team_name: "成都蓉城".to_string(),
                rank_no: 1,
                avatar_storage_url: Some("chengdu.png".to_string()),
                goals_for_total: 14,
                goals_against_total: 5,
                goals_for_by_opponent: vec![OpponentContribution {
                    opponent_team_id: 77689,
                    opponent_team_name: "上海申花".to_string(),
                    opponent_avatar_storage_url: Some("shenhua.png".to_string()),
                    goals: 3,
                    share: 0.214,
                }],
                goals_for_by_player: vec![PlayerContribution {
                    player_id: Some(204211),
                    player_name: "费利佩".to_string(),
                    avatar_storage_url: Some("felipe.png".to_string()),
                    goals: 4,
                    share: 0.286,
                }],
                assists_for_by_player: vec![AssistContribution {
                    player_id: Some(7727346),
                    player_name: "席尔瓦".to_string(),
                    avatar_storage_url: Some("silva.png".to_string()),
                    assists: 3,
                    share: 0.5,
                }],
                goals_against_by_opponent: vec![OpponentContribution {
                    opponent_team_id: 77689,
                    opponent_team_name: "上海申花".to_string(),
                    opponent_avatar_storage_url: Some("shenhua.png".to_string()),
                    goals: 2,
                    share: 0.4,
                }],
            }],
        })
    }
}

#[tokio::test]
async fn execute_returns_live_team_insights() {
    let use_case = GetLiveTeamInsightsUseCase::new(Arc::new(FakeRepository));

    let result = use_case.execute().await.unwrap();

    assert_eq!(result.view_kind, "live");
    assert_eq!(result.current_season, 2026);
    assert_eq!(result.teams.len(), 1);
    assert_eq!(result.insights[0].goals_for_total, 14);
    assert_eq!(
        result.insights[0].goals_for_by_player[0].player_name,
        "费利佩"
    );
    assert_eq!(
        result.insights[0].assists_for_by_player[0].player_name,
        "席尔瓦"
    );
}
