use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::domain::{
    DirectorComment, DirectorError, DirectorPlan, DirectorTeam, DirectorWindow, PlanPlayerItem,
    PoolCandidate, ReactionSummary, SquadPlayer,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorUserProfile {
    pub active: bool,
    pub favorite_team_id: Option<i64>,
    pub wx_open_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpsertPlanRecord {
    pub window_id: i64,
    pub team_id: i64,
    pub user_id: Uuid,
    pub share_code: String,
    pub circle_code: String,
    pub parent_share_code: Option<String>,
    pub foreign_stays: Vec<PlanPlayerItem>,
    pub foreign_leaves: Vec<PlanPlayerItem>,
    pub foreign_ins: Vec<PlanPlayerItem>,
    pub domestic_ins: Vec<PlanPlayerItem>,
    pub domestic_outs: Vec<PlanPlayerItem>,
    pub style_tag: String,
    pub manifesto: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingPlanCodes {
    pub share_code: String,
    pub circle_code: String,
    pub parent_share_code: Option<String>,
}

#[async_trait]
pub trait DirectorRepository: Send + Sync {
    async fn current_window(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Option<DirectorWindow>, DirectorError>;

    async fn find_team(&self, team_id: i64) -> Result<Option<DirectorTeam>, DirectorError>;

    async fn team_active_squad(&self, team_id: i64) -> Result<Vec<SquadPlayer>, DirectorError>;

    async fn other_teams_active_squad(
        &self,
        exclude_team_id: i64,
        keyword: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SquadPlayer>, DirectorError>;

    async fn find_active_squad_player(
        &self,
        player_id: i64,
        exclude_team_id: i64,
    ) -> Result<Option<SquadPlayer>, DirectorError>;

    async fn list_pool_candidates(
        &self,
        team_id: i64,
        is_foreign: bool,
        league: &str,
        keyword: Option<&str>,
        limit: i64,
    ) -> Result<Vec<PoolCandidate>, DirectorError>;

    async fn find_pool_candidate(
        &self,
        code: &str,
    ) -> Result<Option<PoolCandidate>, DirectorError>;

    async fn count_team_plans(&self, window_id: i64, team_id: i64) -> Result<i64, DirectorError>;

    async fn find_user_plan_codes(
        &self,
        window_id: i64,
        team_id: i64,
        user_id: Uuid,
    ) -> Result<Option<ExistingPlanCodes>, DirectorError>;

    async fn circle_exists(
        &self,
        window_id: i64,
        team_id: i64,
        circle_code: &str,
    ) -> Result<bool, DirectorError>;

    async fn find_circle_code_by_share_code(
        &self,
        window_id: i64,
        team_id: i64,
        share_code: &str,
    ) -> Result<Option<String>, DirectorError>;

    async fn upsert_plan(&self, record: &UpsertPlanRecord) -> Result<String, DirectorError>;

    async fn find_plan_by_share_code(
        &self,
        share_code: &str,
        viewer_user_id: Option<Uuid>,
    ) -> Result<Option<DirectorPlan>, DirectorError>;

    async fn find_plan_by_user(
        &self,
        window_id: i64,
        team_id: i64,
        user_id: Uuid,
        viewer_user_id: Option<Uuid>,
    ) -> Result<Option<DirectorPlan>, DirectorError>;

    async fn list_plan_comments(
        &self,
        plan_id: Uuid,
        limit: i64,
    ) -> Result<Vec<DirectorComment>, DirectorError>;

    async fn list_circle_plans(
        &self,
        circle_code: &str,
        viewer_user_id: Option<Uuid>,
    ) -> Result<Vec<DirectorPlan>, DirectorError>;

    async fn set_plan_reaction(
        &self,
        share_code: &str,
        user_id: Uuid,
        value: i16,
    ) -> Result<ReactionSummary, DirectorError>;

    async fn insert_plan_comment(
        &self,
        share_code: &str,
        user_id: Uuid,
        content: &str,
    ) -> Result<DirectorComment, DirectorError>;

    async fn user_profile(
        &self,
        user_id: Uuid,
    ) -> Result<Option<DirectorUserProfile>, DirectorError>;
}
