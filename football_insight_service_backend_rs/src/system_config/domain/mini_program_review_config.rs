use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiniProgramReviewConfig {
    pub mini_program_app_id: String,
    pub mini_program_version: String,
    pub is_under_review: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiniProgramReviewConfigView {
    pub mini_program_app_id: String,
    pub mini_program_version: String,
    pub is_under_review: bool,
    pub matched: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
