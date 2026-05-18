use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};

use crate::auth::ports::token_port::TokenPort;
use crate::auth_license::adapters::web::dto::*;
use crate::auth_license::application::{
    bind_license::BindLicenseUseCase, generate_license::GenerateLicenseUseCase,
};

#[derive(Clone)]
pub struct AuthLicenseWebState {
    pub generate_license_use_case: Arc<GenerateLicenseUseCase>,
    pub bind_license_use_case: Arc<BindLicenseUseCase>,
    pub token_port: Arc<dyn TokenPort>,
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    header_value.strip_prefix("Bearer ")
}

pub async fn generate_license_handler(
    headers: HeaderMap,
    State(state): State<Arc<AuthLicenseWebState>>,
) -> Result<Json<GenerateLicenseResponse>, (StatusCode, String)> {
    let token = extract_bearer_token(&headers)
        .ok_or((StatusCode::UNAUTHORIZED, "not logged in".to_string()))?;
    let claims = state
        .token_port
        .verify_token(token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token".to_string()))?;
    let user_id = claims.sub;
    let license = state
        .generate_license_use_case
        .execute(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(GenerateLicenseResponse {
        license_key: license.license_key,
        expires_at: license.expires_at.to_rfc3339(),
    }))
}

pub async fn bind_license_handler(
    State(state): State<Arc<AuthLicenseWebState>>,
    Json(req): Json<BindLicenseRequest>,
) -> Result<Json<BindLicenseResponse>, (StatusCode, String)> {
    let result = state
        .bind_license_use_case
        .execute(&req.license_key)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
    let token = state
        .token_port
        .issue_token(result.user_id, &result.license_key, expires_at)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "token issue failed".to_string()))?;
    Ok(Json(BindLicenseResponse {
        access_token: token,
        user: serde_json::json!({"id": result.user_id.to_string()}),
    }))
}
