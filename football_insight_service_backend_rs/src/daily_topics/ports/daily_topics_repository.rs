use async_trait::async_trait;

use crate::daily_topics::domain::edition::DailyTopicEdition;

#[derive(Debug, thiserror::Error)]
pub enum DailyTopicsRepositoryError {
    #[error("daily topics storage is unavailable")]
    Unavailable,
    #[error("published daily topic edition is invalid")]
    InvalidEdition,
}

#[async_trait]
pub trait DailyTopicsRepository: Send + Sync {
    async fn latest_for_team(
        &self,
        team_id: i64,
    ) -> Result<Option<DailyTopicEdition>, DailyTopicsRepositoryError>;
}
