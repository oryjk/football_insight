use crate::auth::application::official_bind::{CreateBindCodeError, CreateOfficialBindCodeUseCase};
use axum::{
    Json, Router,
    http::{
        HeaderMap, StatusCode,
        header::{AUTHORIZATION, CACHE_CONTROL},
    },
    response::{IntoResponse, Response},
    routing::post,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
struct BindCodeResponse {
    code: String,
    expires_in_seconds: i64,
}

pub fn official_bind_code_routes(use_case: Arc<CreateOfficialBindCodeUseCase>) -> Router {
    Router::new().route(
        "/api/v1/auth/wechat-official-bind-code",
        post(move |headers: HeaderMap| create_official_bind_code(headers, use_case.clone())),
    )
}

async fn create_official_bind_code(
    headers: HeaderMap,
    use_case: Arc<CreateOfficialBindCodeUseCase>,
) -> Response {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let result = match token {
        Some(token) => use_case.execute(token).await,
        None => Err(CreateBindCodeError::Unauthorized),
    };
    let mut response = match result {
        Ok(result) => Json(BindCodeResponse {
            code: result.code,
            expires_in_seconds: result.expires_in_seconds,
        })
        .into_response(),
        Err(error) => {
            let status = match error {
                CreateBindCodeError::Unauthorized => StatusCode::UNAUTHORIZED,
                CreateBindCodeError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
                CreateBindCodeError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            };
            (status, error.to_string()).into_response()
        }
    };
    response
        .headers_mut()
        .insert(CACHE_CONTROL, "no-store".parse().unwrap());
    response
}
