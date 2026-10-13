use crate::admin::{
    application::admin_setup_service::AdminSetupService,
    domain::admin_setup::{AdminSetupError, AdminSetupInput},
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::{HeaderValue, StatusCode, header::CACHE_CONTROL},
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Serialize)]
pub struct AdminSetupStatusDto {
    pub needs_setup: bool,
    pub username: &'static str,
}
// Contains a password: deliberately no Debug/Serialize implementation.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminSetupRequest {
    pub password: String,
}
pub async fn setup_status(
    State(service): State<Arc<AdminSetupService>>,
) -> Result<Json<AdminSetupStatusDto>, (StatusCode, String)> {
    Ok(Json(AdminSetupStatusDto {
        needs_setup: service.needs_setup().await.map_err(map_setup_error)?,
        username: "superadmin",
    }))
}
pub async fn initialize_admin(
    State(service): State<Arc<AdminSetupService>>,
    body: Result<Json<AdminSetupRequest>, JsonRejection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let Json(body) = body.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "invalid initialization request".into(),
        )
    })?;
    service
        .initialize(AdminSetupInput {
            password: body.password,
        })
        .await
        .map_err(map_setup_error)?;
    Ok(StatusCode::NO_CONTENT)
}
pub async fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
fn map_setup_error(error: AdminSetupError) -> (StatusCode, String) {
    let status = match error {
        AdminSetupError::InvalidPassword => StatusCode::BAD_REQUEST,
        AdminSetupError::AlreadyInitialized => StatusCode::CONFLICT,
        AdminSetupError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}
