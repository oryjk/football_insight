use super::public_dto::{CodeDto, LikeDto, ListDto, ListQuery, SharedDto};
use crate::{
    auth::ports::token_port::TokenPort,
    lineup::{
        application::public_lineup::PublicLineupService,
        domain::{LineupPublicError, LineupReaction},
    },
};
use axum::{
    Json, Router,
    extract::{Path, Query, Request, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{AUTHORIZATION, CACHE_CONTROL},
    },
    middleware::{self, Next},
    response::Response,
    routing::{get, put},
};
use std::sync::Arc;
use uuid::Uuid;

pub struct PublicLineupWebState {
    pub service: Arc<PublicLineupService>,
    pub token_port: Arc<dyn TokenPort>,
}

type WebError = (StatusCode, String);

pub fn public_routes(state: Arc<PublicLineupWebState>) -> Router {
    Router::new()
        .route("/api/v1/lineups/teams/{team_id}", get(list))
        .route("/api/v1/lineups/shared/{share_code}", get(shared))
        .route(
            "/api/v1/lineups/shared/{share_code}/like",
            put(like).delete(unlike),
        )
        .route("/api/v1/lineups/shared/{share_code}/wxacode", get(image))
        .route(
            "/api/v1/lineups/shared/{share_code}/dislike",
            put(dislike).delete(undislike),
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

fn map_error(error: LineupPublicError) -> WebError {
    let status = match error {
        LineupPublicError::Invalid => StatusCode::BAD_REQUEST,
        LineupPublicError::Unauthorized => StatusCode::UNAUTHORIZED,
        LineupPublicError::NotFound => StatusCode::NOT_FOUND,
        LineupPublicError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, error.to_string())
}

fn viewer(headers: &HeaderMap, tokens: &dyn TokenPort) -> Result<Option<Uuid>, WebError> {
    let Some(header) = headers.get(AUTHORIZATION) else {
        return Ok(None);
    };
    let token = header
        .to_str()
        .ok()
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| map_error(LineupPublicError::Unauthorized))?;
    tokens
        .verify_token(token)
        .map(|claims| Some(claims.sub))
        .map_err(|_| map_error(LineupPublicError::Unauthorized))
}

fn required_viewer(headers: &HeaderMap, tokens: &dyn TokenPort) -> Result<Uuid, WebError> {
    viewer(headers, tokens)?.ok_or_else(|| map_error(LineupPublicError::Unauthorized))
}

async fn list(
    State(state): State<Arc<PublicLineupWebState>>,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListDto>, WebError> {
    let viewer = viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .list(team_id, &query.sort, query.page, query.page_size, viewer)
        .await
        .map(|page| Json(page.into()))
        .map_err(map_error)
}
async fn shared(
    State(state): State<Arc<PublicLineupWebState>>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<SharedDto>, WebError> {
    let viewer = viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .shared(&code, viewer)
        .await
        .map(|lineup| Json(lineup.into()))
        .map_err(map_error)
}
async fn like(
    State(state): State<Arc<PublicLineupWebState>>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<LikeDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .set_like(&code, user, true)
        .await
        .map(|likes| Json(likes.into()))
        .map_err(map_error)
}
async fn unlike(
    State(state): State<Arc<PublicLineupWebState>>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<LikeDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .set_like(&code, user, false)
        .await
        .map(|likes| Json(likes.into()))
        .map_err(map_error)
}
async fn image(
    State(state): State<Arc<PublicLineupWebState>>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<CodeDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .image(&code, user)
        .await
        .map(|image| Json(image.into()))
        .map_err(map_error)
}

async fn dislike(
    State(state): State<Arc<PublicLineupWebState>>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<LikeDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .set_reaction(&code, user, Some(LineupReaction::Dislike))
        .await
        .map(|state| Json(state.into()))
        .map_err(map_error)
}

async fn undislike(
    State(state): State<Arc<PublicLineupWebState>>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<LikeDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .set_reaction(&code, user, None)
        .await
        .map(|state| Json(state.into()))
        .map_err(map_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::ports::token_port::{AuthTokenClaims, WechatBindTokenClaims, WechatBindTokenPayload},
        lineup::application::public_lineup::tests::service,
    };
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use chrono::{DateTime, Utc};
    use tower::ServiceExt;

    struct Tokens;
    impl TokenPort for Tokens {
        fn issue_token(&self, _: Uuid, _: &str, _: DateTime<Utc>) -> anyhow::Result<String> {
            unimplemented!()
        }
        fn verify_token(&self, token: &str) -> anyhow::Result<AuthTokenClaims> {
            anyhow::ensure!(token == "good", "invalid");
            Ok(AuthTokenClaims {
                sub: Uuid::from_u128(1),
                account_identifier: "private".into(),
                exp: Utc::now() + chrono::Duration::hours(1),
            })
        }
        fn issue_wechat_bind_token(
            &self,
            _: WechatBindTokenPayload,
            _: DateTime<Utc>,
        ) -> anyhow::Result<String> {
            unimplemented!()
        }
        fn verify_wechat_bind_token(&self, _: &str) -> anyhow::Result<WechatBindTokenClaims> {
            unimplemented!()
        }
    }
    async fn call(
        active: bool,
        method: &str,
        path: &str,
        authorization: Option<&str>,
    ) -> (StatusCode, String) {
        let app = public_routes(Arc::new(PublicLineupWebState {
            service: Arc::new(service(active)),
            token_port: Arc::new(Tokens),
        }));
        let mut request = Request::builder().method(method).uri(path);
        if let Some(header) = authorization {
            request = request.header(AUTHORIZATION, header);
        }
        let response = app
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
        (
            response.status(),
            String::from_utf8(
                to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap()
                    .to_vec(),
            )
            .unwrap(),
        )
    }

    #[tokio::test]
    async fn list_contract_is_anonymous_and_parameters_are_validated() {
        let (status, body) = call(true, "GET", "/api/v1/lineups/teams/77680", None).await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["page"], 1);
        assert_eq!(json["page_size"], 20);
        assert_eq!(json["team"]["team_id"], 77680);
        for query in ["sort=x", "page=0", "page=-1", "page_size=51", "page_size=x"] {
            assert_eq!(
                call(
                    true,
                    "GET",
                    &format!("/api/v1/lineups/teams/77680?{query}"),
                    None
                )
                .await
                .0,
                StatusCode::BAD_REQUEST
            );
        }
    }
    #[tokio::test]
    async fn optional_jwt_rejects_invalid_or_disabled_accounts() {
        for header in ["Bearer bad", "Basic good", ""] {
            assert_eq!(
                call(true, "GET", "/api/v1/lineups/teams/77680", Some(header))
                    .await
                    .0,
                StatusCode::UNAUTHORIZED
            );
        }
        assert_eq!(
            call(
                false,
                "GET",
                "/api/v1/lineups/teams/77680",
                Some("Bearer good")
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
    }
    #[tokio::test]
    async fn like_and_image_require_login_and_hidden_targets_are_404() {
        for (method, suffix) in [
            ("PUT", "like"),
            ("DELETE", "like"),
            ("PUT", "dislike"),
            ("DELETE", "dislike"),
            ("GET", "wxacode"),
        ] {
            let path = format!("/api/v1/lineups/shared/ABCD1234/{suffix}");
            assert_eq!(
                call(true, method, &path, None).await.0,
                StatusCode::UNAUTHORIZED
            );
            assert_eq!(
                call(true, method, &path, Some("Bearer good")).await.0,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(
            call(true, "GET", "/api/v1/lineups/shared/ABCD1234", None)
                .await
                .0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            call(true, "GET", "/api/v1/lineups/shared/bad", None)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
    }
    #[test]
    fn typed_error_mapping_covers_all_variants() {
        for (error, status) in [
            (LineupPublicError::Invalid, 400),
            (LineupPublicError::Unauthorized, 401),
            (LineupPublicError::NotFound, 404),
            (LineupPublicError::Unavailable, 503),
        ] {
            assert_eq!(map_error(error).0.as_u16(), status);
        }
    }
}
