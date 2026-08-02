use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::admin::{
    adapters::web::dto::{
        AdminAuditListQuery, AdminAuditPageDto, AdminAuthResponseDto, AdminCreateUserRequestDto,
        AdminIdentityDto, AdminLoginRequestDto, AdminMembershipAdjustmentRequestDto,
        AdminReasonRequestDto, AdminUpdateUserRequestDto, AdminUserDetailDto, AdminUserDto,
        AdminUserListQuery, AdminUserListResponseDto,
    },
    application::{
        admin_audit_service::AdminAuditService, admin_auth_service::AdminAuthService,
        admin_user_service::AdminUserService,
    },
    domain::admin_auth::{AdminPermission, AdminPrincipal},
};

#[derive(Clone)]
pub struct AdminUserWebState {
    pub service: Arc<AdminUserService>,
    pub auth_service: Arc<AdminAuthService>,
}

#[derive(Clone)]
pub struct AdminAuditWebState {
    pub service: Arc<AdminAuditService>,
    pub auth_service: Arc<AdminAuthService>,
}

pub async fn list_admin_audit_logs_handler(
    State(state): State<AdminAuditWebState>,
    headers: HeaderMap,
    Query(query): Query<AdminAuditListQuery>,
) -> Result<Json<AdminAuditPageDto>, (StatusCode, String)> {
    let token = extract_bearer_token(&headers)?;
    state
        .auth_service
        .authorize(token, AdminPermission::ViewAuditLogs)
        .await
        .map_err(map_admin_auth_error)?;
    let page = state
        .service
        .list(query.page, query.page_size)
        .await
        .map_err(map_admin_error)?;
    Ok(Json(page.into()))
}

pub async fn admin_login_handler(
    State(service): State<Arc<AdminAuthService>>,
    Json(request): Json<AdminLoginRequestDto>,
) -> Result<Json<AdminAuthResponseDto>, (StatusCode, String)> {
    let result = service
        .login(request.into())
        .await
        .map_err(map_admin_auth_error)?;
    Ok(Json(result.into()))
}

pub async fn admin_me_handler(
    State(service): State<Arc<AdminAuthService>>,
    headers: HeaderMap,
) -> Result<Json<AdminIdentityDto>, (StatusCode, String)> {
    let principal = authenticate_admin(&headers, &service).await?;
    Ok(Json(principal_to_identity(principal).into()))
}

pub async fn admin_logout_handler(
    State(service): State<Arc<AdminAuthService>>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = extract_bearer_token(&headers)?;
    service.logout(token).await.map_err(map_admin_auth_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_admin_users_handler(
    State(state): State<AdminUserWebState>,
    headers: HeaderMap,
    Query(query): Query<AdminUserListQuery>,
) -> Result<Json<AdminUserListResponseDto>, (StatusCode, String)> {
    authorize_admin(&headers, &state, AdminPermission::ManageUsers).await?;

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
    authorize_admin(&headers, &state, AdminPermission::ManageUsers).await?;

    let user = state
        .service
        .create_user(request.into())
        .await
        .map_err(map_admin_error)?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

pub async fn get_admin_user_handler(
    State(state): State<AdminUserWebState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<Json<AdminUserDetailDto>, (StatusCode, String)> {
    authorize_admin(&headers, &state, AdminPermission::ManageUsers).await?;
    let user = state
        .service
        .get_user_detail(user_id)
        .await
        .map_err(map_admin_error)?
        .ok_or((StatusCode::NOT_FOUND, "user not found".to_string()))?;
    Ok(Json(user.into()))
}

pub async fn update_admin_user_handler(
    State(state): State<AdminUserWebState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(request): Json<AdminUpdateUserRequestDto>,
) -> Result<Json<AdminUserDto>, (StatusCode, String)> {
    authorize_admin(&headers, &state, AdminPermission::ManageUsers).await?;

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
    authorize_admin(&headers, &state, AdminPermission::ManageUsers).await?;

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

pub async fn disable_admin_user_handler(
    State(state): State<AdminUserWebState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(request): Json<AdminReasonRequestDto>,
) -> Result<Json<AdminUserDto>, (StatusCode, String)> {
    let principal = authorize_admin(&headers, &state, AdminPermission::ManageUsers).await?;
    let user = state
        .service
        .set_user_status(user_id, "disabled", principal.admin_id, request.reason)
        .await
        .map_err(map_admin_error)?
        .ok_or((StatusCode::NOT_FOUND, "user not found".to_string()))?;
    Ok(Json(user.into()))
}

pub async fn restore_admin_user_handler(
    State(state): State<AdminUserWebState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(request): Json<AdminReasonRequestDto>,
) -> Result<Json<AdminUserDto>, (StatusCode, String)> {
    let principal = authorize_admin(&headers, &state, AdminPermission::ManageUsers).await?;
    let user = state
        .service
        .set_user_status(user_id, "active", principal.admin_id, request.reason)
        .await
        .map_err(map_admin_error)?
        .ok_or((StatusCode::NOT_FOUND, "user not found".to_string()))?;
    Ok(Json(user.into()))
}

pub async fn adjust_admin_user_membership_handler(
    State(state): State<AdminUserWebState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(request): Json<AdminMembershipAdjustmentRequestDto>,
) -> Result<Json<AdminUserDto>, (StatusCode, String)> {
    let principal = authorize_admin(&headers, &state, AdminPermission::ManageUsers).await?;
    let adjustment = request.try_into().map_err(map_admin_error)?;
    let user = state
        .service
        .adjust_membership(user_id, adjustment, principal.admin_id)
        .await
        .map_err(map_admin_error)?
        .ok_or((StatusCode::NOT_FOUND, "user not found".to_string()))?;
    Ok(Json(user.into()))
}

async fn authorize_admin(
    headers: &HeaderMap,
    state: &AdminUserWebState,
    permission: AdminPermission,
) -> Result<AdminPrincipal, (StatusCode, String)> {
    let token = extract_bearer_token(headers)?;
    state
        .auth_service
        .authorize(token, permission)
        .await
        .map_err(map_admin_auth_error)
}

async fn authenticate_admin(
    headers: &HeaderMap,
    service: &AdminAuthService,
) -> Result<AdminPrincipal, (StatusCode, String)> {
    let token = extract_bearer_token(headers)?;
    service
        .authenticate(token)
        .await
        .map_err(map_admin_auth_error)
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, (StatusCode, String)> {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "admin login required".to_string()))
}

fn principal_to_identity(
    principal: AdminPrincipal,
) -> crate::admin::application::admin_auth_service::AdminIdentity {
    crate::admin::application::admin_auth_service::AdminIdentity {
        id: principal.admin_id,
        username: principal.username,
        display_name: principal.display_name,
        role: principal.role,
    }
}

fn map_admin_auth_error(error: anyhow::Error) -> (StatusCode, String) {
    let message = error.to_string();
    if message.contains("permission denied") {
        return (StatusCode::FORBIDDEN, message);
    }
    if message.contains("username must") || message.contains("password must") {
        return (StatusCode::BAD_REQUEST, message);
    }
    (
        StatusCode::UNAUTHORIZED,
        "invalid admin credentials".to_string(),
    )
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
