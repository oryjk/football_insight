use async_trait::async_trait;
use uuid::Uuid;

use crate::admin::domain::admin_review_exempt::{AdminReviewExemptError, ReviewExemptUser};

#[async_trait]
pub trait AdminReviewExemptRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<ReviewExemptUser>, AdminReviewExemptError>;
    /// Idempotent; NotFound when the user does not exist. Writes an audit row.
    async fn add(
        &self,
        user_id: Uuid,
        admin_id: Uuid,
    ) -> Result<ReviewExemptUser, AdminReviewExemptError>;
    /// Idempotent; removing a non-member is not an error. Writes an audit row only when a row was removed.
    async fn remove(&self, user_id: Uuid, admin_id: Uuid) -> Result<(), AdminReviewExemptError>;
}
