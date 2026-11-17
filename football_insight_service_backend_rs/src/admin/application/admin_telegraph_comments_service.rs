use std::sync::Arc;

use uuid::Uuid;

use crate::admin::{
    domain::admin_telegraph_comments::{
        AdminTelegraphCommentStatus, AdminTelegraphCommentStatusFilter,
        AdminTelegraphCommentsError, AdminTelegraphCommentsView,
    },
    ports::admin_telegraph_comments_repository::AdminTelegraphCommentsRepository,
};

pub struct AdminTelegraphCommentsService {
    repository: Arc<dyn AdminTelegraphCommentsRepository>,
}

impl AdminTelegraphCommentsService {
    pub fn new(repository: Arc<dyn AdminTelegraphCommentsRepository>) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        status: Option<&str>,
        limit: Option<usize>,
    ) -> Result<AdminTelegraphCommentsView, AdminTelegraphCommentsError> {
        let filter = AdminTelegraphCommentStatusFilter::parse(status)?;
        let limit = limit.unwrap_or(50);
        if !(1..=200).contains(&limit) {
            return Err(AdminTelegraphCommentsError::InvalidInput(
                "limit 必须在 1 到 200 之间",
            ));
        }
        self.repository.list(filter, limit).await
    }

    pub async fn set_comments_enabled(
        &self,
        enabled: bool,
        admin_id: Uuid,
    ) -> Result<bool, AdminTelegraphCommentsError> {
        self.repository.set_comments_enabled(enabled, admin_id).await
    }

    pub async fn set_comment_status(
        &self,
        comment_id: Uuid,
        status: &str,
        admin_id: Uuid,
    ) -> Result<(), AdminTelegraphCommentsError> {
        let parsed = AdminTelegraphCommentStatus::parse(status)?;
        self.repository
            .set_comment_status(comment_id, parsed, admin_id)
            .await
    }
}
