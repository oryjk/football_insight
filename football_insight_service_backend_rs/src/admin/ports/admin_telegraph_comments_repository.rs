use async_trait::async_trait;
use uuid::Uuid;

use crate::admin::domain::admin_telegraph_comments::{
    AdminTelegraphCommentStatus, AdminTelegraphCommentStatusFilter, AdminTelegraphCommentsError,
    AdminTelegraphCommentsView,
};

#[async_trait]
pub trait AdminTelegraphCommentsRepository: Send + Sync {
    async fn list(
        &self,
        filter: AdminTelegraphCommentStatusFilter,
        limit: usize,
    ) -> Result<AdminTelegraphCommentsView, AdminTelegraphCommentsError>;

    async fn set_comments_enabled(
        &self,
        enabled: bool,
        admin_id: Uuid,
    ) -> Result<bool, AdminTelegraphCommentsError>;

    async fn set_comment_status(
        &self,
        comment_id: Uuid,
        status: AdminTelegraphCommentStatus,
        admin_id: Uuid,
    ) -> Result<(), AdminTelegraphCommentsError>;
}
