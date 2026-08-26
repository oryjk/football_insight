use async_trait::async_trait;

use crate::mini_review::domain::mini_review_status::MiniReviewStatus;

/// create 撞 (project_code, version) 唯一约束时由持久化适配器返回，
/// 应用层据此按先落库的记录返回，保证并发构建幂等。
#[derive(Debug)]
pub struct VersionConflictError;

impl std::fmt::Display for VersionConflictError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "version already registered")
    }
}

impl std::error::Error for VersionConflictError {}

#[async_trait]
pub trait MiniReviewRepository: Send + Sync {
    /// 项目当前版本号最大的记录；项目首次登记时为 None。
    async fn find_latest(&self, project_code: &str) -> anyhow::Result<Option<MiniReviewStatus>>;

    async fn find_by_project_and_version(
        &self,
        project_code: &str,
        version: &str,
    ) -> anyhow::Result<Option<MiniReviewStatus>>;

    async fn create(&self, status: MiniReviewStatus) -> anyhow::Result<MiniReviewStatus>;

    /// 只更新 is_reviewing / status_text / updated_at，按 id 定位。
    async fn update_status(&self, status: MiniReviewStatus) -> anyhow::Result<MiniReviewStatus>;
}
