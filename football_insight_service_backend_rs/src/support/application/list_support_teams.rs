use std::sync::Arc;

use crate::support::{
    domain::support::SupportTeamSummary, ports::support_repository::SupportRepository,
};

pub struct ListSupportTeamsUseCase {
    repository: Arc<dyn SupportRepository>,
}

impl ListSupportTeamsUseCase {
    pub fn new(repository: Arc<dyn SupportRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> anyhow::Result<Vec<SupportTeamSummary>> {
        self.repository.list_teams().await
    }
}
