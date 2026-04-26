use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::support::domain::support::{SupportMatchDetail, SupportTeamSummary, SupportUserContext};

#[async_trait]
pub trait SupportRepository: Send + Sync {
    async fn list_teams(&self) -> anyhow::Result<Vec<SupportTeamSummary>>;
    async fn get_user_context(&self, user_id: Uuid) -> anyhow::Result<SupportUserContext>;
    async fn set_favorite_team(
        &self,
        user_id: Uuid,
        team_id: i64,
    ) -> anyhow::Result<SupportTeamSummary>;
    async fn find_matches_for_team(
        &self,
        team_id: i64,
        viewer_user_id: Option<Uuid>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<SupportMatchDetail>>;
    async fn find_match_detail(
        &self,
        match_id: i64,
        viewer_user_id: Option<Uuid>,
    ) -> anyhow::Result<Option<SupportMatchDetail>>;
    async fn create_vote(
        &self,
        user_id: Uuid,
        match_id: i64,
        supported_team_id: i64,
    ) -> anyhow::Result<()>;
}
