use std::sync::Arc;

use axum::{Router, routing::get, routing::post, routing::put};

use crate::{
    auth::ports::token_port::TokenPort,
    mini_review::{
        adapters::web::handlers::{
            allocate_handler, get_review_status_handler, set_review_status_handler,
        },
        application::{
            allocate_review_version::AllocateReviewVersionUseCase,
            get_review_status::GetReviewStatusUseCase,
            set_review_status_by_project_version::SetReviewStatusByProjectVersionUseCase,
        },
        ports::mini_review_repository::MiniReviewRepository,
    },
};

/// GET review-status 为小程序运行时公开查询；
/// POST allocate 为构建脚本入口（静态 API key 鉴权）；
/// PUT review-status 支持静态 API key 或白名单用户 JWT（小程序「设置」入口）。
pub fn mini_review_routes(
    repository: Arc<dyn MiniReviewRepository>,
    api_key: Option<String>,
    token_port: Arc<dyn TokenPort>,
    control_user_ids: Vec<uuid::Uuid>,
) -> Router {
    let get_use_case = Arc::new(GetReviewStatusUseCase::new(repository.clone()));
    let allocate_use_case = Arc::new(AllocateReviewVersionUseCase::new(repository.clone()));
    let set_use_case = Arc::new(SetReviewStatusByProjectVersionUseCase::new(repository));
    let set_api_key = api_key.clone();
    let set_token_port = token_port.clone();
    let set_control_user_ids = control_user_ids.clone();

    Router::new()
        .route(
            "/api/v1/mini-review/review-status",
            get(move |query| get_review_status_handler(get_use_case.clone(), query)),
        )
        .route(
            "/api/v1/mini-review/review-status",
            put(move |headers, body| {
                set_review_status_handler(
                    set_use_case.clone(),
                    set_api_key.clone(),
                    set_token_port.clone(),
                    set_control_user_ids.clone(),
                    headers,
                    body,
                )
            }),
        )
        .route(
            "/api/v1/mini-review/allocate",
            post(move |headers, body| {
                allocate_handler(allocate_use_case.clone(), api_key.clone(), headers, body)
            }),
        )
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use chrono::{Duration, TimeZone, Utc};
    use tower::util::ServiceExt;
    use uuid::Uuid;

    use crate::auth::adapters::security::jwt_token_port::JwtTokenPort;
    use crate::auth::ports::token_port::TokenPort;

    use super::mini_review_routes;
    use crate::mini_review::{
        domain::mini_review_status::{MiniReviewStatus, Version},
        ports::mini_review_repository::MiniReviewRepository,
    };

    #[derive(Default)]
    struct FakeRepository {
        records: Mutex<Vec<MiniReviewStatus>>,
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
            status.id = records.len() as i64 + 1;
            records.push(status.clone());
            Ok(status)
        }

        async fn update_status(&self, status: MiniReviewStatus) -> anyhow::Result<MiniReviewStatus> {
            let mut records = self.records.lock().unwrap();
            let record = records
                .iter_mut()
                .find(|record| record.version == status.version)
                .expect("record exists");
            *record = status.clone();
            Ok(status)
        }
    }

    fn fixed_now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap()
    }

    fn test_token_port() -> Arc<JwtTokenPort> {
        Arc::new(JwtTokenPort::new("test-secret-for-mini-review-routes".to_string()))
    }

    fn issue_user_token(port: &JwtTokenPort, user_id: Uuid) -> String {
        port.issue_token(user_id, "user@example.com", Utc::now() + Duration::hours(1))
            .expect("issue token")
    }

    fn app(api_key: Option<&str>) -> axum::Router {
        app_with_whitelist(api_key, Vec::new())
    }

    fn app_with_whitelist(api_key: Option<&str>, control_user_ids: Vec<Uuid>) -> axum::Router {
        mini_review_routes(
            Arc::new(FakeRepository::default()),
            api_key.map(str::to_string),
            test_token_port(),
            control_user_ids,
        )
    }

    #[tokio::test]
    async fn review_status_get_is_public_and_defaults_to_not_reviewing() {
        let response = app(Some("secret"))
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/mini-review/review-status?project_code=football_insight_mini&version=1.0.55")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload: serde_json::Value =
            serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 1024 * 64).await.unwrap()).unwrap();
        assert_eq!(payload["is_reviewing"], false);
        assert_eq!(payload["status_text"], "未登记版本");
    }

    #[tokio::test]
    async fn review_status_get_rejects_invalid_version() {
        let response = app(None)
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/mini-review/review-status?project_code=football_insight_mini&version=1.0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn allocate_requires_api_key() {
        let response = app(Some("secret"))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/mini-review/allocate")
                    .header("content-type", "application/json")
                    .header("x-api-key", "wrong")
                    .body(Body::from(
                        r#"{"project_code":"football_insight_mini","current_version":"1.0.54"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn allocate_registers_next_version_with_valid_key() {
        let response = app(Some("secret"))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/mini-review/allocate")
                    .header("content-type", "application/json")
                    .header("x-api-key", "secret")
                    .body(Body::from(
                        r#"{"project_code":"football_insight_mini","current_version":"1.0.54"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload: serde_json::Value =
            serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 1024 * 64).await.unwrap()).unwrap();
        assert_eq!(payload["version"], "1.0.55");
        assert_eq!(payload["is_reviewing"], true);
    }

    #[tokio::test]
    async fn allocate_is_disabled_without_configured_key() {
        let response = app(None)
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/mini-review/allocate")
                    .header("content-type", "application/json")
                    .header("x-api-key", "anything")
                    .body(Body::from(
                        r#"{"project_code":"football_insight_mini","current_version":"1.0.54"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn set_review_status_round_trips_with_allocated_version() {
        let repository = Arc::new(FakeRepository::default());
        repository
            .create(MiniReviewStatus::new_reviewing(
                "football_insight_mini",
                Version::parse("1.0.55").unwrap(),
                fixed_now(),
            ))
            .await
            .expect("seed record");
        let app = mini_review_routes(repository, Some("secret".to_string()), test_token_port(), Vec::new());

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/api/v1/mini-review/review-status")
                    .header("content-type", "application/json")
                    .header("x-api-key", "secret")
                    .body(Body::from(
                        r#"{"project_code":"football_insight_mini","version":"1.0.55","is_reviewing":false}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload: serde_json::Value =
            serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 1024 * 64).await.unwrap()).unwrap();
        assert_eq!(payload["is_reviewing"], false);
        assert_eq!(payload["status_text"], "已过审");
    }

    #[tokio::test]
    async fn set_review_status_allows_whitelisted_user_via_bearer_token() {
        let user_id = Uuid::new_v4();
        let repository = Arc::new(FakeRepository::default());
        repository
            .create(MiniReviewStatus::new_reviewing(
                "football_insight_mini",
                Version::parse("1.0.55").unwrap(),
                fixed_now(),
            ))
            .await
            .expect("seed record");
        let token_port = test_token_port();
        let token = issue_user_token(&token_port, user_id);
        let app = mini_review_routes(
            repository,
            Some("secret".to_string()),
            token_port,
            vec![user_id],
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/api/v1/mini-review/review-status")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        r#"{"project_code":"football_insight_mini","version":"1.0.55","is_reviewing":false}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload: serde_json::Value =
            serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 1024 * 64).await.unwrap()).unwrap();
        assert_eq!(payload["is_reviewing"], false);
        assert_eq!(payload["status_text"], "已过审（小程序端切换）");
    }

    #[tokio::test]
    async fn set_review_status_rejects_user_outside_whitelist() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let token_port = test_token_port();
        let token = issue_user_token(&token_port, other_user_id);
        let app = mini_review_routes(
            Arc::new(FakeRepository::default()),
            Some("secret".to_string()),
            token_port,
            vec![user_id],
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/api/v1/mini-review/review-status")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        r#"{"project_code":"football_insight_mini","version":"1.0.55","is_reviewing":false}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn set_review_status_rejects_invalid_bearer_token() {
        let token_port = test_token_port();
        let app = mini_review_routes(
            Arc::new(FakeRepository::default()),
            Some("secret".to_string()),
            token_port,
            vec![Uuid::new_v4()],
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/api/v1/mini-review/review-status")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer not-a-real-token")
                    .body(Body::from(
                        r#"{"project_code":"football_insight_mini","version":"1.0.55","is_reviewing":false}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn set_review_status_returns_not_found_for_unknown_version() {
        let response = app(Some("secret"))
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/api/v1/mini-review/review-status")
                    .header("content-type", "application/json")
                    .header("x-api-key", "secret")
                    .body(Body::from(
                        r#"{"project_code":"football_insight_mini","version":"9.9.9","is_reviewing":false}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
