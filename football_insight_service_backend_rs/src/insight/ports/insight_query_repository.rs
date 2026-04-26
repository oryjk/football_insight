use async_trait::async_trait;

use crate::insight::domain::{
    match_list::MatchListView, overview::InsightOverview, rankings::RankingsView,
    round_reference::RoundReference, team_insight::TeamInsightsView,
};

#[async_trait]
pub trait InsightQueryRepository: Send + Sync {
    async fn get_live_overview(&self) -> anyhow::Result<InsightOverview>;
    async fn get_round_overview(
        &self,
        season: i32,
        round_number: i32,
    ) -> anyhow::Result<InsightOverview>;
    async fn list_available_rounds(&self, season: i32) -> anyhow::Result<Vec<RoundReference>>;
    async fn get_live_rankings(&self) -> anyhow::Result<RankingsView>;
    async fn get_live_team_insights(&self) -> anyhow::Result<TeamInsightsView>;
    async fn get_round_rankings(
        &self,
        season: i32,
        round_number: i32,
    ) -> anyhow::Result<RankingsView>;
    async fn get_live_matches(&self) -> anyhow::Result<MatchListView>;
    async fn get_round_matches(
        &self,
        season: i32,
        round_number: i32,
    ) -> anyhow::Result<MatchListView>;
}
