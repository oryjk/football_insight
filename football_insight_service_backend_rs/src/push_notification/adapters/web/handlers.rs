use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::ports::token_port::TokenPort;
use crate::push_notification::application::register_device_token::RegisterDeviceTokenUseCase;

#[derive(Clone)]
pub struct PushNotificationWebState {
    pub register_use_case: Arc<RegisterDeviceTokenUseCase>,
    pub token_port: Arc<dyn TokenPort>,
}

#[derive(Deserialize)]
pub struct RegisterTokenRequest {
    pub device_token: String,
    pub platform: String,
}

#[derive(Deserialize)]
pub struct UnregisterTokenRequest {
    pub device_token: String,
}

fn extract_user_id_from_headers(
    headers: &HeaderMap,
    token_port: &Arc<dyn TokenPort>,
) -> Result<Uuid, (StatusCode, String)> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "not logged in".to_string()))?;
    let claims = token_port
        .verify_token(token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token".to_string()))?;
    Ok(claims.sub)
}

pub async fn register_token_handler(
    headers: HeaderMap,
    State(state): State<Arc<PushNotificationWebState>>,
    Json(req): Json<RegisterTokenRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = extract_user_id_from_headers(&headers, &state.token_port)?;
    state
        .register_use_case
        .execute(user_id, &req.device_token, &req.platform)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unregister_token_handler(
    headers: HeaderMap,
    State(state): State<Arc<PushNotificationWebState>>,
    Json(_req): Json<UnregisterTokenRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let _user_id = extract_user_id_from_headers(&headers, &state.token_port)?;
    Ok(StatusCode::NO_CONTENT)
}
