use super::super::{
    application::{RecapService, RecapView},
    domain::{CodeImage, RecapError},
};
use crate::auth::ports::token_port::TokenPort;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
#[derive(Clone)]
pub struct WebState {
    pub service: Arc<RecapService>,
    pub token: Arc<dyn TokenPort>,
}
#[derive(Deserialize)]
pub struct SaveRequest {
    selected_match_ids: Vec<i64>,
    home_one_way_km: Option<f64>,
}
type WebError = (StatusCode, String);
pub fn routes(state: WebState) -> Router {
    Router::new()
        .route("/api/v1/season-recap/{season}", get(load).put(save))
        .route("/api/v1/season-recap/{season}/code", post(code))
        .layer(DefaultBodyLimit::max(8192))
        .with_state(state)
}
fn authenticate(headers: &HeaderMap, state: &WebState) -> Result<Uuid, WebError> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|v| state.token.verify_token(v).ok())
        .map(|c| c.sub)
        .ok_or((StatusCode::UNAUTHORIZED, "请先登录".into()))
}
fn map_error(error: RecapError) -> WebError {
    let status = if error == RecapError::Unavailable {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::BAD_REQUEST
    };
    (status, error.to_string())
}
async fn load(
    State(state): State<WebState>,
    headers: HeaderMap,
    Path(season): Path<i32>,
) -> Result<Json<RecapView>, WebError> {
    let user = authenticate(&headers, &state)?;
    state
        .service
        .get(user, season)
        .await
        .map(Json)
        .map_err(map_error)
}
async fn save(
    State(state): State<WebState>,
    headers: HeaderMap,
    Path(season): Path<i32>,
    Json(body): Json<SaveRequest>,
) -> Result<Json<RecapView>, WebError> {
    let user = authenticate(&headers, &state)?;
    state
        .service
        .save(user, season, &body.selected_match_ids, body.home_one_way_km)
        .await
        .map(Json)
        .map_err(map_error)
}
async fn code(
    State(state): State<WebState>,
    headers: HeaderMap,
    Path(season): Path<i32>,
) -> Result<Json<CodeImage>, WebError> {
    authenticate(&headers, &state)?;
    state
        .service
        .code(season)
        .await
        .map(Json)
        .map_err(map_error)
}
