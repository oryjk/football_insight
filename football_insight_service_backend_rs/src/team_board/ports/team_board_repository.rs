use async_trait::async_trait;
use uuid::Uuid;

use crate::team_board::domain::team_board::{
    NewTeamBoardComment, NewTeamBoardPost, TeamBoardComment, TeamBoardLikeSummary, TeamBoardPost,
};

#[async_trait]
pub trait TeamBoardRepository: Send + Sync {
    async fn list_posts(
        &self,
        team_id: i64,
        viewer_user_id: Uuid,
    ) -> anyhow::Result<Vec<TeamBoardPost>>;
    async fn create_post(&self, input: NewTeamBoardPost) -> anyhow::Result<TeamBoardPost>;
    async fn add_comment(&self, input: NewTeamBoardComment) -> anyhow::Result<TeamBoardComment>;
    async fn toggle_like(
        &self,
        post_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<TeamBoardLikeSummary>;
}
