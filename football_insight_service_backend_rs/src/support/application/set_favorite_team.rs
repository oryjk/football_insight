use std::sync::Arc;

use uuid::Uuid;

use crate::support::{
    domain::support::SupportTeamSummary, ports::support_repository::SupportRepository,
};

pub struct SetFavoriteTeamUseCase {
    repository: Arc<dyn SupportRepository>,
}

impl SetFavoriteTeamUseCase {
    pub fn new(repository: Arc<dyn SupportRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid, team_id: i64) -> anyhow::Result<SupportTeamSummary> {
        self.repository.set_favorite_team(user_id, team_id).await
    }
}
