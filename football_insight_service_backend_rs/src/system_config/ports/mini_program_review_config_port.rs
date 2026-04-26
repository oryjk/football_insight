use async_trait::async_trait;

use crate::system_config::domain::mini_program_review_config::MiniProgramReviewConfig;

#[async_trait]
pub trait MiniProgramReviewConfigPort: Send + Sync {
    async fn find_review_config(
        &self,
        mini_program_app_id: &str,
        mini_program_version: &str,
    ) -> anyhow::Result<Option<MiniProgramReviewConfig>>;
}
