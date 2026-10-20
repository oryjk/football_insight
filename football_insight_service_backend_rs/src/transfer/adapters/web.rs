use super::dto::{
    BoardDto, BoardQuery, CandidatesDto, CardViewDto, CurrentWindowDto, IntentQuery, NominateBody,
    NominateDto, SummaryDto, TallyDto, VoteBody, WindowQuery,
};
use crate::{
    auth::ports::token_port::TokenPort,
    transfer::{
        application::TransferService,
        domain::{TransferError, VoteValue},
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
    routing::{get, post, put},
};
use std::sync::Arc;
use uuid::Uuid;

pub struct TransferWebState {
    pub service: Arc<TransferService>,
    pub token_port: Arc<dyn TokenPort>,
}

type WebError = (StatusCode, String);
type Shared = State<Arc<TransferWebState>>;

pub fn transfer_routes(state: Arc<TransferWebState>) -> Router {
    Router::new()
        .route("/api/v1/transfers/windows/current", get(current_window))
        .route("/api/v1/transfers/teams/{team_id}/summary", get(summary))
        .route("/api/v1/transfers/teams/{team_id}/board", get(board))
        .route(
            "/api/v1/transfers/teams/{team_id}/candidates",
            get(candidates),
        )
        .route(
            "/api/v1/transfers/teams/{team_id}/nominations",
            post(nominate),
        )
        .route("/api/v1/transfers/nominations/{share_code}", get(card))
        .route(
            "/api/v1/transfers/nominations/{share_code}/vote",
            put(vote).delete(unvote),
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

pub(super) fn map_error(error: TransferError) -> WebError {
    let status = match error {
        TransferError::Invalid => StatusCode::BAD_REQUEST,
        TransferError::Unauthorized => StatusCode::UNAUTHORIZED,
        TransferError::NotFavoriteTeam => StatusCode::FORBIDDEN,
        TransferError::NotFound => StatusCode::NOT_FOUND,
        TransferError::WindowClosed => StatusCode::CONFLICT,
        TransferError::InvalidCandidate => StatusCode::UNPROCESSABLE_ENTITY,
        TransferError::NominationLimit => StatusCode::TOO_MANY_REQUESTS,
        TransferError::ShareCodeTaken | TransferError::Unavailable => {
            StatusCode::SERVICE_UNAVAILABLE
        }
    };
    let message = match error {
        TransferError::ShareCodeTaken => TransferError::Unavailable.to_string(),
        other => other.to_string(),
    };
    (status, message)
}

fn viewer(headers: &HeaderMap, tokens: &dyn TokenPort) -> Result<Option<Uuid>, WebError> {
    let Some(header) = headers.get(AUTHORIZATION) else {
        return Ok(None);
    };
    let token = header
        .to_str()
        .ok()
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| map_error(TransferError::Unauthorized))?;
    tokens
        .verify_token(token)
        .map(|claims| Some(claims.sub))
        .map_err(|_| map_error(TransferError::Unauthorized))
}

fn required_viewer(headers: &HeaderMap, tokens: &dyn TokenPort) -> Result<Uuid, WebError> {
    viewer(headers, tokens)?.ok_or_else(|| map_error(TransferError::Unauthorized))
}

async fn current_window(State(state): Shared) -> Result<Json<CurrentWindowDto>, WebError> {
    state
        .service
        .current_window()
        .await
        .map(|w| {
            Json(CurrentWindowDto {
                window: w.map(Into::into),
            })
        })
        .map_err(map_error)
}

async fn summary(
    State(state): Shared,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
    Query(query): Query<WindowQuery>,
) -> Result<Json<SummaryDto>, WebError> {
    let viewer = viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .summary(team_id, query.window.as_deref(), viewer)
        .await
        .map(|s| Json(s.into()))
        .map_err(map_error)
}

async fn board(
    State(state): Shared,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
    Query(query): Query<BoardQuery>,
) -> Result<Json<BoardDto>, WebError> {
    let viewer = viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .board(
            team_id,
            &query.intent,
            query.page,
            query.window.as_deref(),
            viewer,
        )
        .await
        .map(|b| Json(b.into()))
        .map_err(map_error)
}

async fn candidates(
    State(state): Shared,
    Path(team_id): Path<i64>,
    Query(query): Query<IntentQuery>,
) -> Result<Json<CandidatesDto>, WebError> {
    state
        .service
        .candidates(team_id, &query.intent)
        .await
        .map(|c| Json(c.into()))
        .map_err(map_error)
}

async fn card(
    State(state): Shared,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<CardViewDto>, WebError> {
    let viewer = viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .card(&code, viewer)
        .await
        .map(|v| Json(v.into()))
        .map_err(map_error)
}

async fn nominate(
    State(state): Shared,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
    Json(body): Json<NominateBody>,
) -> Result<Json<NominateDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .nominate(user, team_id, body.player_id, &body.intent)
        .await
        .map(|r| Json(r.into()))
        .map_err(map_error)
}

async fn vote(
    State(state): Shared,
    headers: HeaderMap,
    Path(code): Path<String>,
    Json(body): Json<VoteBody>,
) -> Result<Json<TallyDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    let value = VoteValue::from_i16(body.value).ok_or_else(|| map_error(TransferError::Invalid))?;
    state
        .service
        .vote(user, &code, Some(value))
        .await
        .map(|t| Json(t.into()))
        .map_err(map_error)
}

async fn unvote(
    State(state): Shared,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<TallyDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    state
        .service
        .vote(user, &code, None)
        .await
        .map(|t| Json(t.into()))
        .map_err(map_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::ports::token_port::{AuthTokenClaims, WechatBindTokenClaims, WechatBindTokenPayload},
        transfer::application::tests::{FAN, at, service_at},
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
                sub: FAN,
                account_identifier: "fan".into(),
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
        day: u32,
        method: &str,
        path: &str,
        auth: Option<&str>,
        body: Option<&str>,
    ) -> (StatusCode, String) {
        let (service, _, _) = service_at(at(day), true);
        let app = transfer_routes(Arc::new(TransferWebState {
            service: Arc::new(service),
            token_port: Arc::new(Tokens),
        }));
        let mut request = Request::builder().method(method).uri(path);
        if let Some(header) = auth {
            request = request.header(AUTHORIZATION, header);
        }
        if body.is_some() {
            request = request.header("content-type", "application/json");
        }
        let response = app
            .oneshot(
                request
                    .body(
                        body.map(|b| Body::from(b.to_owned()))
                            .unwrap_or_else(Body::empty),
                    )
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
        let status = response.status();
        (
            status,
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
    async fn reads_are_anonymous_and_shaped_for_the_mini_program() {
        let (status, body) = call(5, "GET", "/api/v1/transfers/teams/1/summary", None, None).await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["window"]["code"], "2026-winter");
        assert_eq!(json["window"]["open"], true);
        assert_eq!(json["can_vote"], false);
        assert_eq!(json["boards"][0]["intent"], "out");
        assert_eq!(
            json["boards"][0]["items"][0]["player"]["current_team"]["team_id"],
            1
        );
        let (status, body) = call(
            5,
            "GET",
            "/api/v1/transfers/teams/1/board?intent=in&page=1",
            Some("Bearer good"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap()["can_vote"],
            true
        );
        assert_eq!(
            call(5, "GET", "/api/v1/transfers/windows/current", None, None)
                .await
                .0,
            StatusCode::OK
        );
        assert_eq!(
            call(
                5,
                "GET",
                "/api/v1/transfers/nominations/CODE0001",
                None,
                None
            )
            .await
            .0,
            StatusCode::OK
        );
        assert_eq!(
            call(
                5,
                "GET",
                "/api/v1/transfers/teams/1/candidates?intent=in",
                None,
                None
            )
            .await
            .0,
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn writes_require_login_and_map_domain_errors() {
        for (method, path, body) in [
            (
                "POST",
                "/api/v1/transfers/teams/1/nominations",
                Some(r#"{"player_id":10,"intent":"out"}"#),
            ),
            (
                "PUT",
                "/api/v1/transfers/nominations/CODE0001/vote",
                Some(r#"{"value":1}"#),
            ),
            (
                "DELETE",
                "/api/v1/transfers/nominations/CODE0001/vote",
                None,
            ),
        ] {
            assert_eq!(
                call(5, method, path, None, body).await.0,
                StatusCode::UNAUTHORIZED,
                "{path}"
            );
            assert_eq!(
                call(5, method, path, Some("Bearer bad"), body).await.0,
                StatusCode::UNAUTHORIZED,
                "{path}"
            );
        }
        let (status, body) = call(
            5,
            "POST",
            "/api/v1/transfers/teams/1/nominations",
            Some("Bearer good"),
            Some(r#"{"player_id":10,"intent":"out"}"#),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap()["created"],
            true
        );
        let (status, body) = call(
            25,
            "PUT",
            "/api/v1/transfers/nominations/CODE0001/vote",
            Some("Bearer good"),
            Some(r#"{"value":-1}"#),
        )
        .await;
        assert_eq!(
            (status, body.as_str()),
            (StatusCode::CONFLICT, "本期投票已结束")
        );
        assert_eq!(
            call(
                5,
                "PUT",
                "/api/v1/transfers/nominations/CODE0001/vote",
                Some("Bearer good"),
                Some(r#"{"value":2}"#)
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            call(
                5,
                "POST",
                "/api/v1/transfers/teams/1/nominations",
                Some("Bearer good"),
                Some(r#"{"player_id":40,"intent":"in"}"#)
            )
            .await
            .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn every_error_has_a_status() {
        for (error, status) in [
            (TransferError::Invalid, 400),
            (TransferError::Unauthorized, 401),
            (TransferError::NotFavoriteTeam, 403),
            (TransferError::NotFound, 404),
            (TransferError::WindowClosed, 409),
            (TransferError::InvalidCandidate, 422),
            (TransferError::NominationLimit, 429),
            (TransferError::ShareCodeTaken, 503),
            (TransferError::Unavailable, 503),
        ] {
            assert_eq!(map_error(error).0.as_u16(), status);
        }
    }
}
