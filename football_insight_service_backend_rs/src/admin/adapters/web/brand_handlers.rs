use crate::admin::{
    application::{
        admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        admin_brand_service::AdminBrandService,
    },
    domain::{
        admin_auth::{AdminPermission, AdminPrincipal},
        admin_brand::{AdminBrandError, BrandSubtitle},
    },
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

type HttpError = (StatusCode, String);

#[derive(Clone)]
pub struct AdminBrandWebState {
    pub service: Arc<AdminBrandService>,
    pub auth_service: Arc<AdminAuthService>,
}

#[derive(Deserialize)]
pub struct SetSubtitleBody {
    pub subtitle: String,
}

fn subtitle_json(subtitle: &BrandSubtitle) -> Value {
    json!({ "subtitle": subtitle.value() })
}

pub async fn get_subtitle(
    State(state): State<AdminBrandWebState>,
    headers: HeaderMap,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let subtitle = state.service.get_subtitle().await.map_err(map_error)?;
    Ok(Json(subtitle_json(&subtitle)))
}

pub async fn set_subtitle(
    State(state): State<AdminBrandWebState>,
    headers: HeaderMap,
    body: Result<Json<SetSubtitleBody>, JsonRejection>,
) -> Result<Json<Value>, HttpError> {
    let principal = authorize(&state, &headers).await?;
    let Json(body) =
        body.map_err(|_| (StatusCode::BAD_REQUEST, "invalid request body".to_string()))?;
    let subtitle = state
        .service
        .set_subtitle(&body.subtitle, principal.admin_id)
        .await
        .map_err(map_error)?;
    Ok(Json(subtitle_json(&subtitle)))
}

async fn authorize(
    state: &AdminBrandWebState,
    headers: &HeaderMap,
) -> Result<AdminPrincipal, HttpError> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "admin login required".into()))?;
    state
        .auth_service
        .authorize_checked(token, AdminPermission::ManageProducts)
        .await
        .map_err(|error| {
            let status = match error {
                AdminAuthorizationError::Unauthorized => StatusCode::UNAUTHORIZED,
                AdminAuthorizationError::Forbidden => StatusCode::FORBIDDEN,
                AdminAuthorizationError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, error.to_string())
        })
}

fn map_error(error: AdminBrandError) -> HttpError {
    let status = match error {
        AdminBrandError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AdminBrandError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}
