use std::sync::Arc;

use axum::{Router, routing::get};

use crate::system_config::application::{
    get_mini_program_review_config::GetMiniProgramReviewConfigUseCase,
    get_public_system_config::GetPublicSystemConfigUseCase,
};

use super::handlers::{get_mini_program_review_config_handler, get_public_system_config_handler};

pub fn system_config_routes(
    public_config_use_case: Arc<GetPublicSystemConfigUseCase>,
    mini_program_review_config_use_case: Arc<GetMiniProgramReviewConfigUseCase>,
) -> Router {
    let system_config_review_use_case = mini_program_review_config_use_case.clone();
    let system_config_hyphen_review_use_case = mini_program_review_config_use_case.clone();

    Router::new()
        .route(
            "/api/v1/system/public-config",
            get(move || get_public_system_config_handler(public_config_use_case.clone())),
        )
        .route(
            "/api/v1/system_config",
            get(move |query| {
                get_mini_program_review_config_handler(system_config_review_use_case.clone(), query)
            }),
        )
        .route(
            "/api/v1/system-config",
            get(move |query| {
                get_mini_program_review_config_handler(
                    system_config_hyphen_review_use_case.clone(),
                    query,
                )
            }),
        )
        .route(
            "/api/v1/mini-program/review-config",
            get(move |query| {
                get_mini_program_review_config_handler(
                    mini_program_review_config_use_case.clone(),
                    query,
                )
            }),
        )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::anyhow;
    use async_trait::async_trait;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use chrono::{TimeZone, Utc};
    use tower::util::ServiceExt;

    use super::system_config_routes;
    use crate::{
        insight::{
            domain::{
                match_list::MatchListView,
                overview::InsightOverview,
                rankings::RankingsView,
                round_reference::RoundReference,
                team_insight::TeamInsightsView,
            },
            ports::insight_query_repository::InsightQueryRepository,
        },
        system_config::{
            application::{
                get_mini_program_review_config::GetMiniProgramReviewConfigUseCase,
                get_public_system_config::GetPublicSystemConfigUseCase,
            },
            domain::{
                ai_chat_config::AiChatSystemConfig,
                mini_program_review_config::MiniProgramReviewConfig,
                public_system_config::{
                    AiChatMode, HomeBriefingMarquees, MembershipTierRuleConfig, PublicSystemConfig,
                },
            },
            ports::{
                mini_program_review_config_port::MiniProgramReviewConfigPort,
                system_config_port::SystemConfigPort,
            },
        },
    };

    struct StubSystemConfigPort;

    #[async_trait]
    impl SystemConfigPort for StubSystemConfigPort {
        async fn get_public_config(&self) -> anyhow::Result<PublicSystemConfig> {
            Ok(PublicSystemConfig::new(
                true,
                AiChatMode::BackendProxy,
                HomeBriefingMarquees::default(),
                vec![MembershipTierRuleConfig::new("V3", "invite", Some(0), 300)],
            ))
        }

        async fn get_ai_chat_config(&self) -> anyhow::Result<AiChatSystemConfig> {
            Ok(AiChatSystemConfig::default())
        }

        async fn get_config_value(&self, _config_key: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }
    }

    struct StubInsightQueryRepository;

    #[async_trait]
    impl InsightQueryRepository for StubInsightQueryRepository {
        async fn get_live_overview(&self) -> anyhow::Result<InsightOverview> {
            Err(anyhow!("unused"))
        }

        async fn get_round_overview(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<InsightOverview> {
            Err(anyhow!("unused"))
        }

        async fn list_available_rounds(&self, _season: i32) -> anyhow::Result<Vec<RoundReference>> {
            Err(anyhow!("unused"))
        }

        async fn get_live_rankings(&self) -> anyhow::Result<RankingsView> {
            Err(anyhow!("unused"))
        }

        async fn get_round_rankings(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<RankingsView> {
            Err(anyhow!("unused"))
        }

        async fn get_live_matches(&self) -> anyhow::Result<MatchListView> {
            Err(anyhow!("unused"))
        }

        async fn get_round_matches(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<MatchListView> {
            Err(anyhow!("unused"))
        }

        async fn get_live_team_insights(&self) -> anyhow::Result<TeamInsightsView> {
            Err(anyhow!("unused"))
        }
    }

    struct StubMiniProgramReviewConfigPort;

    #[async_trait]
    impl MiniProgramReviewConfigPort for StubMiniProgramReviewConfigPort {
        async fn find_review_config(
            &self,
            _mini_program_app_id: &str,
            mini_program_version: &str,
        ) -> anyhow::Result<Option<MiniProgramReviewConfig>> {
            let now = Utc.with_ymd_and_hms(2026, 4, 24, 20, 0, 0).unwrap();
            Ok(Some(MiniProgramReviewConfig {
                mini_program_app_id: "".to_string(),
                mini_program_version: mini_program_version.to_string(),
                is_under_review: true,
                created_at: now,
                updated_at: now,
            }))
        }
    }

    #[tokio::test]
    async fn system_config_route_should_return_review_status() {
        let app = system_config_routes(
            Arc::new(GetPublicSystemConfigUseCase::new(
                Arc::new(StubSystemConfigPort),
                Arc::new(StubInsightQueryRepository),
            )),
            Arc::new(GetMiniProgramReviewConfigUseCase::new(Arc::new(
                StubMiniProgramReviewConfigPort,
            ))),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/system_config?version=1.2.3")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
