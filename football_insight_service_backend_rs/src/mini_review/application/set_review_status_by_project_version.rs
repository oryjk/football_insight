use std::sync::Arc;

use crate::mini_review::{
    domain::mini_review_status::{MiniReviewStatus, Version, DEFAULT_REVIEWING_STATUS_TEXT},
    ports::mini_review_repository::MiniReviewRepository,
};

pub struct SetReviewStatusCommand {
    pub project_code: String,
    pub version: String,
    pub is_reviewing: bool,
    pub status_text: Option<String>,
}

/// 过审/重新提审时未显式给文案的默认值。
fn default_status_text(is_reviewing: bool) -> &'static str {
    if is_reviewing {
        DEFAULT_REVIEWING_STATUS_TEXT
    } else {
        "已过审"
    }
}

pub struct SetReviewStatusByProjectVersionUseCase {
    repository: Arc<dyn MiniReviewRepository>,
}

impl SetReviewStatusByProjectVersionUseCase {
    pub fn new(repository: Arc<dyn MiniReviewRepository>) -> Self {
        Self { repository }
    }

    /// 按 项目编码 + 版本号 切换审核结论（过审后标记 false，重新提审标记 true）。
    pub async fn execute(&self, command: SetReviewStatusCommand) -> anyhow::Result<MiniReviewStatus> {
        let project_code = command.project_code.trim().to_string();
        let version = Version::parse(&command.version)
            .map_err(|_| anyhow::anyhow!("project code or version is invalid"))?;
        if project_code.is_empty() {
            anyhow::bail!("project code or version is invalid");
        }

        let mut existing = self
            .repository
            .find_by_project_and_version(&project_code, &version.to_string())
            .await?
            .ok_or_else(|| anyhow::anyhow!("review status not registered for this version"))?;

        let status_text = command
            .status_text
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| default_status_text(command.is_reviewing).to_string());

        existing.set_status(command.is_reviewing, &status_text, chrono::Utc::now())?;
        self.repository.update_status(existing).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};

    use super::{SetReviewStatusByProjectVersionUseCase, SetReviewStatusCommand};
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
            unreachable!("set review status never creates")
        }

        async fn update_status(&self, status: MiniReviewStatus) -> anyhow::Result<MiniReviewStatus> {
            Ok(status)
        }
    }

    fn command(is_reviewing: bool, status_text: Option<&str>) -> SetReviewStatusCommand {
        SetReviewStatusCommand {
            project_code: "football_insight_mini".to_string(),
            version: "1.0.55".to_string(),
            is_reviewing,
            status_text: status_text.map(str::to_string),
        }
    }

    #[tokio::test]
    async fn execute_marks_version_passed_with_default_text() {
        let mut record = MiniReviewStatus::new_reviewing(
            "football_insight_mini",
            Version::parse("1.0.55").unwrap(),
            Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap(),
        );
        record.id = 7;
        let use_case = SetReviewStatusByProjectVersionUseCase::new(Arc::new(FakeRepository {
            record: Some(record),
        }));

        let status = use_case.execute(command(false, None)).await.expect("update");

        assert!(!status.is_reviewing);
        assert_eq!(status.status_text, "已过审");
    }

    #[tokio::test]
    async fn execute_honors_custom_status_text() {
        let mut record = MiniReviewStatus::new_reviewing(
            "football_insight_mini",
            Version::parse("1.0.55").unwrap(),
            Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap(),
        );
        record.id = 7;
        let use_case = SetReviewStatusByProjectVersionUseCase::new(Arc::new(FakeRepository {
            record: Some(record),
        }));

        let status = use_case
            .execute(command(true, Some("  审核中（复检） ")))
            .await
            .expect("update");

        assert!(status.is_reviewing);
        assert_eq!(status.status_text, "审核中（复检）");
    }

    #[tokio::test]
    async fn execute_rejects_unregistered_version() {
        let use_case = SetReviewStatusByProjectVersionUseCase::new(Arc::new(FakeRepository {
            record: None,
        }));

        let error = use_case
            .execute(command(false, None))
            .await
            .expect_err("unregistered version should be rejected");

        assert!(error.to_string().contains("not registered"));
    }
}
