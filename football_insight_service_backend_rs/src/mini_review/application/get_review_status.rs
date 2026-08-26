use std::sync::Arc;

use crate::mini_review::{
    domain::mini_review_status::{MiniReviewStatus, UNREGISTERED_STATUS_TEXT, Version},
    ports::mini_review_repository::MiniReviewRepository,
};

pub struct GetReviewStatusUseCase {
    repository: Arc<dyn MiniReviewRepository>,
}

impl GetReviewStatusUseCase {
    pub fn new(repository: Arc<dyn MiniReviewRepository>) -> Self {
        Self { repository }
    }

    /// 小程序运行时查询；未登记的版本视为不在审核（已过审或从未提审）。
    pub async fn execute(&self, project_code: &str, raw_version: &str) -> anyhow::Result<MiniReviewStatus> {
        let project_code = project_code.trim();
        let version = Version::parse(raw_version)
            .map_err(|_| anyhow::anyhow!("project code or version is invalid"))?;
        if project_code.is_empty() {
            anyhow::bail!("project code or version is invalid");
        }

        if let Some(existing) = self
            .repository
            .find_by_project_and_version(project_code, &version.to_string())
            .await?
        {
            return Ok(existing);
        }

        Ok(MiniReviewStatus {
            id: 0,
            project_code: project_code.to_string(),
            version: version.to_string(),
            version_code: version.code(),
            is_reviewing: false,
            status_text: UNREGISTERED_STATUS_TEXT.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};

    use super::GetReviewStatusUseCase;
    use crate::mini_review::{
        domain::mini_review_status::{MiniReviewStatus, Version},
        ports::mini_review_repository::MiniReviewRepository,
    };

    struct FakeRepository {
        record: Option<MiniReviewStatus>,
    }

    #[async_trait]
    impl MiniReviewRepository for FakeRepository {
        async fn find_latest(&self, _project_code: &str) -> anyhow::Result<Option<MiniReviewStatus>> {
            Ok(self.record.clone())
        }

        async fn find_by_project_and_version(
            &self,
            project_code: &str,
            version: &str,
        ) -> anyhow::Result<Option<MiniReviewStatus>> {
            Ok(self
                .record
                .as_ref()
                .filter(|record| record.project_code == project_code && record.version == version)
                .cloned())
        }

        async fn create(&self, _status: MiniReviewStatus) -> anyhow::Result<MiniReviewStatus> {
            unreachable!("get review status never writes")
        }

        async fn update_status(&self, _status: MiniReviewStatus) -> anyhow::Result<MiniReviewStatus> {
            unreachable!("get review status never writes")
        }
    }

    fn registered(version: &str, is_reviewing: bool) -> MiniReviewStatus {
        let mut status = MiniReviewStatus::new_reviewing(
            "football_insight_mini",
            Version::parse(version).unwrap(),
            Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap(),
        );
        status.is_reviewing = is_reviewing;
        status
    }

    #[tokio::test]
    async fn execute_returns_registered_status() {
        let use_case = GetReviewStatusUseCase::new(Arc::new(FakeRepository {
            record: Some(registered("1.0.55", true)),
        }));

        let status = use_case
            .execute("football_insight_mini", "1.0.55")
            .await
            .expect("review status");

        assert!(status.is_reviewing);
        assert_eq!(status.version, "1.0.55");
    }

    #[tokio::test]
    async fn execute_treats_unregistered_version_as_not_reviewing() {
        let use_case = GetReviewStatusUseCase::new(Arc::new(FakeRepository { record: None }));

        let status = use_case
            .execute("football_insight_mini", "1.0.55")
            .await
            .expect("review status");

        assert!(!status.is_reviewing);
        assert_eq!(status.status_text, "未登记版本");
        assert_eq!(status.version_code, 10055);
    }

    #[tokio::test]
    async fn execute_rejects_invalid_project_or_version() {
        let use_case = GetReviewStatusUseCase::new(Arc::new(FakeRepository { record: None }));

        assert!(use_case.execute("", "1.0.55").await.is_err());
        assert!(use_case.execute("football_insight_mini", "1.0").await.is_err());
    }
}
