use std::sync::Arc;

use uuid::Uuid;

use crate::admin::{
    domain::admin_review_exempt::{AdminReviewExemptError, ReviewExemptUser},
    ports::admin_review_exempt_repository::AdminReviewExemptRepository,
};

pub struct AdminReviewExemptService {
    repository: Arc<dyn AdminReviewExemptRepository>,
}

impl AdminReviewExemptService {
    pub fn new(repository: Arc<dyn AdminReviewExemptRepository>) -> Self {
        Self { repository }
    }

    pub async fn list(&self) -> Result<Vec<ReviewExemptUser>, AdminReviewExemptError> {
        self.repository.list().await
    }

    pub async fn add(&self, user_id: Uuid, admin_id: Uuid) -> Result<ReviewExemptUser, AdminReviewExemptError> {
        self.repository.add(user_id, admin_id).await
    }

    pub async fn remove(&self, user_id: Uuid, admin_id: Uuid) -> Result<(), AdminReviewExemptError> {
        self.repository.remove(user_id, admin_id).await
    }
}
