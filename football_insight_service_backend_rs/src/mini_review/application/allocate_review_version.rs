use std::sync::Arc;

use crate::mini_review::{
    domain::mini_review_status::{
        AllocationInput, MiniReviewStatus, Version, decide_next_version,
    },
    ports::mini_review_repository::{MiniReviewRepository, VersionConflictError},
};

pub struct AllocateCommand {
    pub project_code: String,
    pub current_version: String,
    pub explicit_version: Option<String>,
}

pub struct AllocateReviewVersionUseCase {
    repository: Arc<dyn MiniReviewRepository>,
}

impl AllocateReviewVersionUseCase {
    pub fn new(repository: Arc<dyn MiniReviewRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: AllocateCommand) -> anyhow::Result<MiniReviewStatus> {
        let project_code = command.project_code.trim().to_string();
        if project_code.is_empty() {
            anyhow::bail!("project code is required");
        }

        if let Some(explicit) = command
            .explicit_version
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return self.allocate_explicit(&project_code, explicit).await;
        }

        let seed = Version::parse(&command.current_version)
            .map_err(|_| anyhow::anyhow!("current version is required to allocate"))?;

        let latest = self.repository.find_latest(&project_code).await?;
        let next = decide_next_version(&AllocationInput {
            latest: latest.clone(),
            seed,
        });

        if let Some(latest) = latest {
            if latest.is_reviewing && latest.version == next.to_string() {
                return Ok(latest);
            }
        }

        let created = MiniReviewStatus::new_reviewing(&project_code, next, chrono::Utc::now());
        match self.repository.create(created).await {
            Ok(status) => Ok(status),
            Err(error) => {
                if error.downcast_ref::<VersionConflictError>().is_some() {
                    // 并发构建撞唯一约束：以先落库的记录为准，保证幂等。
                    self.repository
                        .find_by_project_and_version(&project_code, &next.to_string())
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("conflicted version record missing"))
                } else {
                    Err(error)
                }
            }
        }
    }

    async fn allocate_explicit(&self, project_code: &str, raw_version: &str) -> anyhow::Result<MiniReviewStatus> {
        let version = Version::parse(raw_version)?;
        if let Some(existing) = self
            .repository
            .find_by_project_and_version(project_code, &version.to_string())
            .await?
        {
            if existing.is_reviewing {
                return Ok(existing);
            }
            // 已过审的版本被再次显式构建：视为重新提审。
            let mut reopened = existing;
            reopened.restart_reviewing(chrono::Utc::now());
            return self.repository.update_status(reopened).await;
        }

        let created = MiniReviewStatus::new_reviewing(project_code, version, chrono::Utc::now());
        match self.repository.create(created).await {
            Ok(status) => Ok(status),
            Err(error) => {
                if error.downcast_ref::<VersionConflictError>().is_some() {
                    self.repository
                        .find_by_project_and_version(project_code, &version.to_string())
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("conflicted version record missing"))
                } else {
                    Err(error)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};

    use super::AllocateCommand;
    use crate::mini_review::{
        application::allocate_review_version::AllocateReviewVersionUseCase,
        domain::mini_review_status::{MiniReviewStatus, Version},
        ports::mini_review_repository::{MiniReviewRepository, VersionConflictError},
    };

    #[derive(Default)]
    struct FakeRepository {
        records: std::sync::Mutex<Vec<MiniReviewStatus>>,
    }

    impl FakeRepository {
        fn with_records(records: Vec<MiniReviewStatus>) -> Self {
            Self {
                records: std::sync::Mutex::new(records),
            }
        }

        fn latest_record_version(&self) -> String {
            self.records
                .lock()
                .unwrap()
                .last()
                .map(|record| record.version.clone())
                .unwrap_or_default()
        }
    }

    fn status(project_code: &str, version: &str, is_reviewing: bool) -> MiniReviewStatus {
        let mut status = MiniReviewStatus::new_reviewing(
            project_code,
            Version::parse(version).unwrap(),
            Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap(),
        );
        status.is_reviewing = is_reviewing;
        status
    }

    #[async_trait]
    impl MiniReviewRepository for FakeRepository {
        async fn find_latest(&self, project_code: &str) -> anyhow::Result<Option<MiniReviewStatus>> {
            let records = self.records.lock().unwrap();
            Ok(records
                .iter()
                .filter(|record| record.project_code == project_code)
                .max_by_key(|record| record.version_code)
                .cloned())
        }

        async fn find_by_project_and_version(
            &self,
            project_code: &str,
            version: &str,
        ) -> anyhow::Result<Option<MiniReviewStatus>> {
            let records = self.records.lock().unwrap();
            Ok(records
                .iter()
                .find(|record| record.project_code == project_code && record.version == version)
                .cloned())
        }

        async fn create(&self, mut status: MiniReviewStatus) -> anyhow::Result<MiniReviewStatus> {
            let mut records = self.records.lock().unwrap();
            if records
                .iter()
                .any(|record| record.project_code == status.project_code && record.version == status.version)
            {
                return Err(anyhow::Error::new(VersionConflictError));
            }
            status.id = records.len() as i64 + 1;
            records.push(status.clone());
            Ok(status)
        }

        async fn update_status(&self, status: MiniReviewStatus) -> anyhow::Result<MiniReviewStatus> {
            let mut records = self.records.lock().unwrap();
            let record = records
                .iter_mut()
                .find(|record| record.id == status.id)
                .ok_or_else(|| anyhow::anyhow!("record not found"))?;
            *record = status.clone();
            Ok(status)
        }
    }

    fn command(project_code: &str, current_version: &str, explicit: Option<&str>) -> AllocateCommand {
        AllocateCommand {
            project_code: project_code.to_string(),
            current_version: current_version.to_string(),
            explicit_version: explicit.map(str::to_string),
        }
    }

    #[tokio::test]
    async fn allocate_seeds_from_manifest_when_registry_empty() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = AllocateReviewVersionUseCase::new(repository.clone());

        let status = use_case
            .execute(command("football_insight_mini", "1.0.54", None))
            .await
            .expect("allocate");

        assert_eq!(status.version, "1.0.55");
        assert!(status.is_reviewing);
    }

    #[tokio::test]
    async fn allocate_reuses_latest_reviewing_version() {
        let repository = Arc::new(FakeRepository::with_records(vec![status(
            "football_insight_mini",
            "1.0.55",
            true,
        )]));
        let use_case = AllocateReviewVersionUseCase::new(repository.clone());

        let status = use_case
            .execute(command("football_insight_mini", "1.0.54", None))
            .await
            .expect("allocate");

        assert_eq!(status.version, "1.0.55");
        assert_eq!(repository.latest_record_version(), "1.0.55");
    }

    #[tokio::test]
    async fn allocate_increments_after_latest_review_finished() {
        let repository = Arc::new(FakeRepository::with_records(vec![status(
            "football_insight_mini",
            "1.0.55",
            false,
        )]));
        let use_case = AllocateReviewVersionUseCase::new(repository.clone());

        let status = use_case
            .execute(command("football_insight_mini", "1.0.54", None))
            .await
            .expect("allocate");

        assert_eq!(status.version, "1.0.56");
        assert_eq!(repository.records.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn allocate_explicit_reopens_review_for_passed_version() {
        let repository = Arc::new(FakeRepository::with_records(vec![status(
            "football_insight_mini",
            "1.2.0",
            false,
        )]));
        let use_case = AllocateReviewVersionUseCase::new(repository.clone());

        let status = use_case
            .execute(command("football_insight_mini", "1.0.54", Some("1.2.0")))
            .await
            .expect("allocate");

        assert!(status.is_reviewing);
        assert_eq!(repository.records.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn allocate_explicit_creates_missing_version() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = AllocateReviewVersionUseCase::new(repository.clone());

        let status = use_case
            .execute(command("football_insight_mini", "1.0.54", Some("1.1.0")))
            .await
            .expect("allocate");

        assert_eq!(status.version, "1.1.0");
        assert!(status.is_reviewing);
    }

    #[tokio::test]
    async fn allocate_rejects_blank_project_code() {
        let use_case = AllocateReviewVersionUseCase::new(Arc::new(FakeRepository::default()));

        let error = use_case
            .execute(command("  ", "1.0.54", None))
            .await
            .expect_err("blank project code should be rejected");

        assert!(error.to_string().contains("project code is required"));
    }

    #[tokio::test]
    async fn allocate_rejects_missing_current_version() {
        let use_case = AllocateReviewVersionUseCase::new(Arc::new(FakeRepository::default()));

        let error = use_case
            .execute(command("football_insight_mini", "", None))
            .await
            .expect_err("missing current version should be rejected");

        assert!(error.to_string().contains("current version is required"));
    }
}
