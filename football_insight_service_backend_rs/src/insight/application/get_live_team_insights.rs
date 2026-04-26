use std::sync::Arc;

use crate::insight::{
    domain::team_insight::TeamInsightsView, ports::insight_query_repository::InsightQueryRepository,
};

pub struct GetLiveTeamInsightsUseCase {
    repository: Arc<dyn InsightQueryRepository>,
}

impl GetLiveTeamInsightsUseCase {
    pub fn new(repository: Arc<dyn InsightQueryRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> anyhow::Result<TeamInsightsView> {
        self.repository.get_live_team_insights().await
    }
}
