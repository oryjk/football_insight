use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AdminReviewExemptError {
    #[error("user not found")]
    NotFound,
    #[error("review exemption query failed")]
    Internal,
}

#[derive(Debug, Clone)]
pub struct ReviewExemptUser {
    pub user_id: Uuid,
    pub display_name: Option<String>,
    pub account_identifier: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}
