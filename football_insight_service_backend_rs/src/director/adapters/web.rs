use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, Request, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{AUTHORIZATION, CACHE_CONTROL},
    },
    middleware::{self, Next},
    response::Response,
    routing::{get, post, put},
};
use uuid::Uuid;

use crate::{
    auth::ports::token_port::TokenPort,
    director::{application::DirectorService, domain::DirectorError},
};

use super::dto::{
    CandidatesQuery, CandidatesResponseDto, CircleConsensusDto, ContextQuery, CreateCommentBody,
    DirectorCommentDto, DirectorContextDto, DirectorPlanDetailDto, ReactionBody,
    ReactionSummaryDto, SavePlanBody,
};

pub struct DirectorWebState {
    pub service: Arc<DirectorService>,
    pub token_port: Arc<dyn TokenPort>,
}

type WebError = (StatusCode, String);
type Shared = State<Arc<DirectorWebState>>;

pub fn director_routes(state: Arc<DirectorWebState>) -> Router {
    Router::new()
        .route("/api/v1/director/context", get(context))
        .route("/api/v1/director/candidates", get(candidates))
        .route("/api/v1/director/plans/me", put(save_my_plan))
        .route("/api/v1/director/plans/{share_code}", get(plan_detail))
        .route(
            "/api/v1/director/circles/{circle_code}",
            get(circle_consensus),
        )
        .route(
            "/api/v1/director/plans/{share_code}/reaction",
            put(set_reaction),
        )
        .route(
            "/api/v1/director/plans/{share_code}/comments",
            post(create_comment),
        )
        .layer(middleware::from_fn(no_store))
        .with_state(state)
}

async fn no_store(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

pub(super) fn map_error(error: DirectorError) -> WebError {
    let status = match &error {
        DirectorError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        DirectorError::Unauthorized => StatusCode::UNAUTHORIZED,
        DirectorError::NotFound(_) => StatusCode::NOT_FOUND,
        DirectorError::ContentRejected => StatusCode::UNPROCESSABLE_ENTITY,
        DirectorError::ModerationUnavailable
        | DirectorError::ShareCodeTaken
        | DirectorError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    };
    let message = match error {
        DirectorError::ShareCodeTaken => DirectorError::Unavailable.to_string(),
        other => other.to_string(),
    };
    (status, message)
}

fn optional_viewer(headers: &HeaderMap, tokens: &dyn TokenPort) -> Result<Option<Uuid>, WebError> {
    let Some(header) = headers.get(AUTHORIZATION) else {
        return Ok(None);
    };
    let token = header
        .to_str()
        .ok()
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| map_error(DirectorError::Unauthorized))?;
    tokens
        .verify_token(token)
        .map(|claims| Some(claims.sub))
        .map_err(|_| map_error(DirectorError::Unauthorized))
}

fn required_viewer(headers: &HeaderMap, tokens: &dyn TokenPort) -> Result<Uuid, WebError> {
    optional_viewer(headers, tokens)?.ok_or_else(|| map_error(DirectorError::Unauthorized))
}

async fn context(
    State(state): Shared,
    headers: HeaderMap,
    Query(query): Query<ContextQuery>,
) -> Result<Json<DirectorContextDto>, WebError> {
    let viewer = optional_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .get_context(query.team_id, viewer)
        .await
        .map(|ctx| Json(ctx.into()))
        .map_err(map_error)
}

async fn candidates(
    State(state): Shared,
    Query(query): Query<CandidatesQuery>,
) -> Result<Json<CandidatesResponseDto>, WebError> {
    state
        .service
        .list_candidates(
            query.team_id,
            query.is_foreign,
            query.league.as_deref(),
            query.q.as_deref(),
        )
        .await
        .map(|items| {
            Json(CandidatesResponseDto {
                items: items.into_iter().map(Into::into).collect(),
            })
        })
        .map_err(map_error)
}

async fn save_my_plan(
    State(state): Shared,
    headers: HeaderMap,
    Json(body): Json<SavePlanBody>,
) -> Result<Json<DirectorPlanDetailDto>, WebError> {
    let user_id = required_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .save_my_plan(user_id, body.into())
        .await
        .map(|detail| Json(detail.into()))
        .map_err(map_error)
}

async fn plan_detail(
    State(state): Shared,
    headers: HeaderMap,
    Path(share_code): Path<String>,
) -> Result<Json<DirectorPlanDetailDto>, WebError> {
    let viewer = optional_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .get_plan_detail(&share_code, viewer)
        .await
        .map(|detail| Json(detail.into()))
        .map_err(map_error)
}

async fn circle_consensus(
    State(state): Shared,
    headers: HeaderMap,
    Path(circle_code): Path<String>,
) -> Result<Json<CircleConsensusDto>, WebError> {
    let viewer = optional_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .get_circle_consensus(&circle_code, viewer)
        .await
        .map(|consensus| Json(consensus.into()))
        .map_err(map_error)
}

async fn set_reaction(
    State(state): Shared,
    headers: HeaderMap,
    Path(share_code): Path<String>,
    Json(body): Json<ReactionBody>,
) -> Result<Json<ReactionSummaryDto>, WebError> {
    let user_id = required_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .set_reaction(&share_code, user_id, body.value)
        .await
        .map(|summary| Json(summary.into()))
        .map_err(map_error)
}

async fn create_comment(
    State(state): Shared,
    headers: HeaderMap,
    Path(share_code): Path<String>,
    Json(body): Json<CreateCommentBody>,
) -> Result<Json<DirectorCommentDto>, WebError> {
    let user_id = required_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .create_comment(&share_code, user_id, &body.content)
        .await
        .map(|comment| Json(comment.into()))
        .map_err(map_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::ports::token_port::{AuthTokenClaims, WechatBindTokenClaims, WechatBindTokenPayload},
        director::{
            application::tests::{FAN_A, FAN_B, MockContentSecurity, MockDirectorRepo, build_test_service},
            domain::DEFAULT_TEAM_ID,
        },
    };
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;

    struct StubTokenPort;

    impl TokenPort for StubTokenPort {
        fn issue_token(
            &self,
            _user_id: Uuid,
            _account_identifier: &str,
            _expires_at: chrono::DateTime<chrono::Utc>,
        ) -> anyhow::Result<String> {
            Ok("token-a".into())
        }

        fn verify_token(&self, token: &str) -> anyhow::Result<AuthTokenClaims> {
            let sub = match token {
                "token-a" => FAN_A,
                "token-b" => FAN_B,
                _ => anyhow::bail!("invalid token"),
            };
            Ok(AuthTokenClaims {
                sub,
                account_identifier: "fan".into(),
                exp: chrono::Utc::now() + chrono::Duration::hours(1),
            })
        }

        fn issue_wechat_bind_token(
            &self,
            _payload: WechatBindTokenPayload,
            _expires_at: chrono::DateTime<chrono::Utc>,
        ) -> anyhow::Result<String> {
            Ok("bind".into())
        }

        fn verify_wechat_bind_token(&self, _token: &str) -> anyhow::Result<WechatBindTokenClaims> {
            anyhow::bail!("unused")
        }
    }

    fn build_app(repo: Arc<MockDirectorRepo>, sec: Arc<MockContentSecurity>) -> Router {
        let service = Arc::new(build_test_service(repo, sec));
        director_routes(Arc::new(DirectorWebState {
            service,
            token_port: Arc::new(StubTokenPort),
        }))
    }

    #[tokio::test]
    async fn unauthenticated_write_endpoints_return_401_and_no_store_header() {
        let app = build_app(MockDirectorRepo::new(), MockContentSecurity::allow());

        let save_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/director/plans/me")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "team_id": DEFAULT_TEAM_ID,
                            "foreign_stay_ids": [1001]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(save_res.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            save_res.headers().get(CACHE_CONTROL).unwrap(),
            "no-store"
        );

        let react_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/director/plans/AB12CD34/reaction")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"value":1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(react_res.status(), StatusCode::UNAUTHORIZED);

        let comment_res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/director/plans/AB12CD34/comments")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"content":"不错"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(comment_res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn end_to_end_http_flow_for_context_candidates_save_reaction_and_comment() {
        let app = build_app(MockDirectorRepo::new(), MockContentSecurity::allow());

        // 1. GET context
        let ctx_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/director/context?team_id={DEFAULT_TEAM_ID}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ctx_res.status(), StatusCode::OK);
        assert_eq!(ctx_res.headers().get(CACHE_CONTROL).unwrap(), "no-store");
        let ctx_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(ctx_res.into_body(), 65536).await.unwrap()).unwrap();
        assert_eq!(ctx_body["foreign_quota"], 5);
        assert_eq!(ctx_body["squad_foreign"].as_array().unwrap().len(), 3);
        assert_eq!(ctx_body["squad_domestic"].as_array().unwrap().len(), 2);

        // 2. GET candidates
        let cand_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/director/candidates?is_foreign=true&league=K1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cand_res.status(), StatusCode::OK);
        let cand_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(cand_res.into_body(), 65536).await.unwrap()).unwrap();
        assert_eq!(cand_body["items"][0]["candidate_code"], "k1-cesinha");

        // 3. PUT plans/me
        let save_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/director/plans/me")
                    .header("authorization", "Bearer token-a")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "team_id": DEFAULT_TEAM_ID,
                            "foreign_stay_ids": [1001, 1002],
                            "foreign_leave_ids": [1003],
                            "foreign_ins": [
                                { "source": "pool", "candidate_code": "k1-cesinha" }
                            ],
                            "manifesto": "冲击亚冠！"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(save_res.status(), StatusCode::OK);
        let saved: serde_json::Value =
            serde_json::from_slice(&to_bytes(save_res.into_body(), 65536).await.unwrap()).unwrap();
        let share_code = saved["plan"]["share_code"].as_str().unwrap().to_string();
        assert_eq!(saved["plan"]["style_tag"], "K联赛淘宝流");

        // 4. PUT reaction
        let react_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/director/plans/{share_code}/reaction"))
                    .header("authorization", "Bearer token-b")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"value":1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(react_res.status(), StatusCode::OK);
        let react_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(react_res.into_body(), 65536).await.unwrap()).unwrap();
        assert_eq!(react_body["likes_count"], 1);
        assert_eq!(react_body["my_reaction"], 1);

        // 5. POST comment
        let comment_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/director/plans/{share_code}/comments"))
                    .header("authorization", "Bearer token-b")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"content":"塞西尼亚很合适！"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(comment_res.status(), StatusCode::OK);
        let comment_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(comment_res.into_body(), 65536).await.unwrap())
                .unwrap();
        assert_eq!(comment_body["content"], "塞西尼亚很合适！");
    }

    #[tokio::test]
    async fn unavailable_repository_returns_503() {
        let mut repo = MockDirectorRepo::new();
        Arc::get_mut(&mut repo).unwrap().force_unavailable = true;
        let app = build_app(repo, MockContentSecurity::allow());

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/director/context")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
