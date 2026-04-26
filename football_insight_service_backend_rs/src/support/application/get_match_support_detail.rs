use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::support::{
    domain::support::{SupportMatchDetail, refresh_match_detail},
    ports::support_repository::SupportRepository,
};

pub struct GetMatchSupportDetailUseCase {
    repository: Arc<dyn SupportRepository>,
}

impl GetMatchSupportDetailUseCase {
    pub fn new(repository: Arc<dyn SupportRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        match_id: i64,
        viewer_user_id: Option<Uuid>,
    ) -> anyhow::Result<Option<SupportMatchDetail>> {
        self.execute_at(match_id, viewer_user_id, Utc::now()).await
    }

    pub async fn execute_at(
        &self,
        match_id: i64,
        viewer_user_id: Option<Uuid>,
        now: chrono::DateTime<Utc>,
    ) -> anyhow::Result<Option<SupportMatchDetail>> {
        let mut detail = self
            .repository
            .find_match_detail(match_id, viewer_user_id)
            .await?;

        if let Some(detail) = detail.as_mut() {
            refresh_match_detail(detail, now);
        }

        Ok(detail)
    }
}
