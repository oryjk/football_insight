use crate::admin::{
    application::{
        admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        admin_credentials_service::AdminCredentialsService,
    },
    domain::admin_credentials::{AdminCredentialsError, AdminCredentialsInput},
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde::Deserialize;
use std::sync::Arc;
#[derive(Clone)]
pub struct AdminCredentialsWebState {
    pub auth: Arc<AdminAuthService>,
    pub credentials: Arc<AdminCredentialsService>,
}
// Do not derive Debug: this DTO contains passwords.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminCredentialsRequest {
    pub username: String,
    pub current_password: String,
    pub new_password: Option<String>,
}
impl From<AdminCredentialsRequest> for AdminCredentialsInput {
    fn from(value: AdminCredentialsRequest) -> Self {
        Self {
            username: value.username,
            current_password: value.current_password,
            new_password: value.new_password,
        }
    }
}
pub async fn update_credentials(
    State(state): State<AdminCredentialsWebState>,
    headers: HeaderMap,
    body: Result<Json<AdminCredentialsRequest>, JsonRejection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "admin login required".into()))?;
    let principal = state
        .auth
        .authenticate_checked(token)
        .await
        .map_err(|error| {
            let status = match error {
                AdminAuthorizationError::Unauthorized => StatusCode::UNAUTHORIZED,
                AdminAuthorizationError::Forbidden => StatusCode::FORBIDDEN,
                AdminAuthorizationError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, error.to_string())
        })?;
    let Json(body) = body.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "invalid credentials request".into(),
        )
    })?;
    state
        .credentials
        .change(principal, body.into())
        .await
        .map_err(|error| {
            let status = match error {
                AdminCredentialsError::InvalidInput(_) => StatusCode::BAD_REQUEST,
                AdminCredentialsError::WrongPassword => StatusCode::FORBIDDEN,
                AdminCredentialsError::Conflict => StatusCode::CONFLICT,
                AdminCredentialsError::Unauthorized => StatusCode::UNAUTHORIZED,
                AdminCredentialsError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, error.to_string())
        })?;
    Ok(StatusCode::NO_CONTENT)
}
