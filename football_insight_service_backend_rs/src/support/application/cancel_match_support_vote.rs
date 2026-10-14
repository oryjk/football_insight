use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::support::{
    domain::support::{SupportMatchDetail, SupportWindowStatus, refresh_match_detail},
    ports::support_repository::SupportRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum CancelSupportVoteError {
    #[error("support match not found")]
    MatchNotFound,
    #[error("support window is not open")]
    WindowClosed,
    #[error("support repository unavailable")]
    RepositoryUnavailable,
}

pub struct CancelMatchSupportVoteUseCase {
    repository: Arc<dyn SupportRepository>,
}

impl CancelMatchSupportVoteUseCase {
    pub fn new(repository: Arc<dyn SupportRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        match_id: i64,
    ) -> Result<SupportMatchDetail, CancelSupportVoteError> {
        self.execute_at(user_id, match_id, Utc::now()).await
    }

    pub async fn execute_at(
        &self,
        user_id: Uuid,
        match_id: i64,
        now: DateTime<Utc>,
    ) -> Result<SupportMatchDetail, CancelSupportVoteError> {
        self.repository
            .get_user_context(user_id)
            .await
            .map_err(|_| CancelSupportVoteError::RepositoryUnavailable)?;
        let mut detail = self
            .repository
            .find_match_detail(match_id, Some(user_id))
            .await
            .map_err(|_| CancelSupportVoteError::RepositoryUnavailable)?
            .ok_or(CancelSupportVoteError::MatchNotFound)?;
        refresh_match_detail(&mut detail, now);
        if detail.support_window_status != SupportWindowStatus::Open {
            return Err(CancelSupportVoteError::WindowClosed);
        }
        // DELETE 按 user_id + match_id 限定，重复取消为幂等操作。
        self.repository.remove_vote(user_id, match_id).await?;
        let mut updated = self
            .repository
            .find_match_detail(match_id, Some(user_id))
            .await
            .map_err(|_| CancelSupportVoteError::RepositoryUnavailable)?
            .ok_or(CancelSupportVoteError::MatchNotFound)?;
        refresh_match_detail(&mut updated, now);
        Ok(updated)
    }
}
