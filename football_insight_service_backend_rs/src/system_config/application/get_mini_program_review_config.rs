use std::sync::Arc;

use crate::system_config::{
    domain::mini_program_review_config::MiniProgramReviewConfigView,
    ports::mini_program_review_config_port::MiniProgramReviewConfigPort,
};

pub struct GetMiniProgramReviewConfigUseCase {
    review_config_port: Arc<dyn MiniProgramReviewConfigPort>,
}

impl GetMiniProgramReviewConfigUseCase {
    pub fn new(review_config_port: Arc<dyn MiniProgramReviewConfigPort>) -> Self {
        Self { review_config_port }
    }

    pub async fn execute(
        &self,
        mini_program_app_id: Option<String>,
        mini_program_version: String,
    ) -> anyhow::Result<MiniProgramReviewConfigView> {
        let mini_program_app_id = mini_program_app_id.unwrap_or_default().trim().to_string();
        let mini_program_version = mini_program_version.trim().to_string();

        if mini_program_version.is_empty() {
            anyhow::bail!("mini program version is required");
        }

        let config = self
            .review_config_port
            .find_review_config(&mini_program_app_id, &mini_program_version)
            .await?;

        Ok(match config {
            Some(config) => MiniProgramReviewConfigView {
                mini_program_app_id: config.mini_program_app_id,
                mini_program_version: config.mini_program_version,
                is_under_review: config.is_under_review,
                matched: true,
                created_at: Some(config.created_at),
                updated_at: Some(config.updated_at),
            },
            None => MiniProgramReviewConfigView {
                mini_program_app_id,
                mini_program_version,
                is_under_review: false,
                matched: false,
                created_at: None,
                updated_at: None,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};

    use super::GetMiniProgramReviewConfigUseCase;
    use crate::system_config::{
        domain::mini_program_review_config::MiniProgramReviewConfig,
        ports::mini_program_review_config_port::MiniProgramReviewConfigPort,
    };

    struct FakeReviewConfigPort {
        config: Option<MiniProgramReviewConfig>,
    }

    #[async_trait]
    impl MiniProgramReviewConfigPort for FakeReviewConfigPort {
        async fn find_review_config(
            &self,
            _mini_program_app_id: &str,
            _mini_program_version: &str,
        ) -> anyhow::Result<Option<MiniProgramReviewConfig>> {
            Ok(self.config.clone())
        }
    }

    #[tokio::test]
    async fn execute_returns_review_status_for_matched_version() {
        let now = Utc.with_ymd_and_hms(2026, 4, 24, 20, 0, 0).unwrap();
        let use_case = GetMiniProgramReviewConfigUseCase::new(std::sync::Arc::new(
            FakeReviewConfigPort {
                config: Some(MiniProgramReviewConfig {
                    mini_program_app_id: "".to_string(),
                    mini_program_version: "1.2.3".to_string(),
                    is_under_review: true,
                    created_at: now,
                    updated_at: now,
                }),
            },
        ));

        let view = use_case
            .execute(None, "1.2.3".to_string())
            .await
            .expect("review config");

        assert!(view.matched);
        assert!(view.is_under_review);
        assert_eq!(view.mini_program_version, "1.2.3");
        assert_eq!(view.created_at, Some(now));
    }

    #[tokio::test]
    async fn execute_defaults_to_not_under_review_when_version_is_missing() {
        let use_case =
            GetMiniProgramReviewConfigUseCase::new(std::sync::Arc::new(FakeReviewConfigPort {
                config: None,
            }));

        let view = use_case
            .execute(None, "1.2.4".to_string())
            .await
            .expect("review config");

        assert!(!view.matched);
        assert!(!view.is_under_review);
        assert_eq!(view.mini_program_version, "1.2.4");
        assert_eq!(view.created_at, None);
    }

    #[tokio::test]
    async fn execute_rejects_blank_version() {
        let use_case =
            GetMiniProgramReviewConfigUseCase::new(std::sync::Arc::new(FakeReviewConfigPort {
                config: None,
            }));

        let error = use_case
            .execute(None, " ".to_string())
            .await
            .expect_err("blank version should be rejected");

        assert!(error.to_string().contains("version is required"));
    }
}
