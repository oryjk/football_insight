use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::admin::{
    adapters::web::dto::{
        AdminCreateUserRequestDto, AdminUpdateUserRequestDto, AdminUserDto, AdminUserListQuery,
        AdminUserListResponseDto,
    },
    application::admin_user_service::AdminUserService,
};

#[derive(Clone)]
pub struct AdminUserWebState {
    pub service: Arc<AdminUserService>,
    pub admin_api_token: Option<String>,
}

pub async fn list_admin_users_handler(
    State(state): State<AdminUserWebState>,
    headers: HeaderMap,
    Query(query): Query<AdminUserListQuery>,
) -> Result<Json<AdminUserListResponseDto>, (StatusCode, String)> {
    require_admin_token(&headers, &state)?;

    let result = state
        .service
        .list_users(query.into())
        .await
        .map_err(map_admin_error)?;

    Ok(Json(result.into()))
}

pub async fn create_admin_user_handler(
    State(state): State<AdminUserWebState>,
    headers: HeaderMap,
    Json(request): Json<AdminCreateUserRequestDto>,
) -> Result<(StatusCode, Json<AdminUserDto>), (StatusCode, String)> {
    require_admin_token(&headers, &state)?;

    let user = state
        .service
        .create_user(request.into())
        .await
        .map_err(map_admin_error)?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

pub async fn update_admin_user_handler(
    State(state): State<AdminUserWebState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(request): Json<AdminUpdateUserRequestDto>,
) -> Result<Json<AdminUserDto>, (StatusCode, String)> {
    require_admin_token(&headers, &state)?;

    let user = state
        .service
        .update_user(user_id, request.into())
        .await
        .map_err(map_admin_error)?
        .ok_or((StatusCode::NOT_FOUND, "user not found".to_string()))?;

    Ok(Json(user.into()))
}

pub async fn delete_admin_user_handler(
    State(state): State<AdminUserWebState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_admin_token(&headers, &state)?;

    let deleted = state
        .service
        .delete_user(user_id)
        .await
        .map_err(map_admin_error)?;

    if !deleted {
        return Err((StatusCode::NOT_FOUND, "user not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

fn require_admin_token(
    headers: &HeaderMap,
    state: &AdminUserWebState,
) -> Result<(), (StatusCode, String)> {
    let Some(expected_token) = state
        .admin_api_token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "admin api token is not configured".to_string(),
        ));
    };

    let actual_token = headers
        .get("X-Admin-Token")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if actual_token != expected_token {
        return Err((StatusCode::UNAUTHORIZED, "invalid admin token".to_string()));
    }

    Ok(())
}

fn map_admin_error(error: anyhow::Error) -> (StatusCode, String) {
    let message = error.to_string();
    if message.contains("required")
        || message.contains("must be")
        || message.contains("too long")
        || message.contains("already exists")
    {
        return (StatusCode::BAD_REQUEST, message);
    }

    (StatusCode::INTERNAL_SERVER_ERROR, message)
}
