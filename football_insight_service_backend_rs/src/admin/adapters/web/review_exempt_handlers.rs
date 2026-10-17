use crate::admin::{
    application::{
        admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        admin_review_exempt_service::AdminReviewExemptService,
    },
    domain::{
        admin_auth::{AdminPermission, AdminPrincipal},
        admin_review_exempt::{AdminReviewExemptError, ReviewExemptUser},
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

type HttpError = (StatusCode, String);

#[derive(Clone)]
pub struct AdminReviewExemptWebState {
    pub service: Arc<AdminReviewExemptService>,
    pub auth_service: Arc<AdminAuthService>,
}

fn user_json(user: ReviewExemptUser) -> Value {
    json!({
        "user_id": user.user_id, "display_name": user.display_name,
        "account_identifier": user.account_identifier, "avatar_url": user.avatar_url,
        "created_at": user.created_at,
    })
}

pub async fn list_exempt_users(
    State(state): State<AdminReviewExemptWebState>,
    headers: HeaderMap,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let rows = state.service.list().await.map_err(map_error)?;
    Ok(Json(json!({ "items": rows.into_iter().map(user_json).collect::<Vec<_>>() })))
}

pub async fn add_exempt_user(
    State(state): State<AdminReviewExemptWebState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, HttpError> {
    let principal = authorize(&state, &headers).await?;
    let user = state.service.add(user_id, principal.admin_id).await.map_err(map_error)?;
    Ok(Json(user_json(user)))
}

pub async fn remove_exempt_user(
    State(state): State<AdminReviewExemptWebState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, HttpError> {
    let principal = authorize(&state, &headers).await?;
    state.service.remove(user_id, principal.admin_id).await.map_err(map_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn authorize(
    state: &AdminReviewExemptWebState,
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

fn map_error(error: AdminReviewExemptError) -> HttpError {
    let status = match error {
        AdminReviewExemptError::NotFound => StatusCode::NOT_FOUND,
        AdminReviewExemptError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}
