use crate::admin::{
    application::{
        admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        admin_support_bases_service::AdminSupportBasesService,
    },
    domain::{
        admin_auth::{AdminPermission, AdminPrincipal},
        admin_support_bases::{AdminSupportBasesError, SupportBaseMatch},
    },
};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

type HttpError = (StatusCode, String);

#[derive(Clone)]
pub struct AdminSupportBasesWebState {
    pub service: Arc<AdminSupportBasesService>,
    pub auth_service: Arc<AdminAuthService>,
}

#[derive(Deserialize)]
pub struct SetBasesBody {
    pub home_base: i64,
    pub away_base: i64,
}

fn match_json(m: SupportBaseMatch) -> Value {
    json!({
        "match_id": m.match_id, "round_number": m.round_number,
        "match_date": m.match_date, "match_time": m.match_time, "status": m.status,
        "home_team": {"team_id": m.home_team_id, "team_name": m.home_team_name,
                      "real_count": m.home_real_count, "base": m.home_base,
                      "displayed_count": m.home_real_count + m.home_base},
        "away_team": {"team_id": m.away_team_id, "team_name": m.away_team_name,
                      "real_count": m.away_real_count, "base": m.away_base,
                      "displayed_count": m.away_real_count + m.away_base},
    })
}

pub async fn list_matches(
    State(state): State<AdminSupportBasesWebState>,
    headers: HeaderMap,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let rows = state.service.list_matches().await.map_err(map_error)?;
    Ok(Json(json!({ "items": rows.into_iter().map(match_json).collect::<Vec<_>>() })))
}

pub async fn set_bases(
    State(state): State<AdminSupportBasesWebState>,
    headers: HeaderMap,
    Path(match_id): Path<i64>,
    body: Result<Json<SetBasesBody>, JsonRejection>,
) -> Result<Json<Value>, HttpError> {
    let principal = authorize(&state, &headers).await?;
    let Json(body) = body.map_err(|_| (StatusCode::BAD_REQUEST, "invalid request body".to_string()))?;
    let updated = state
        .service
        .set_bases(match_id, body.home_base, body.away_base, principal.admin_id)
        .await
        .map_err(map_error)?;
    Ok(Json(match_json(updated)))
}

async fn authorize(
    state: &AdminSupportBasesWebState,
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

fn map_error(error: AdminSupportBasesError) -> HttpError {
    let status = match error {
        AdminSupportBasesError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AdminSupportBasesError::NotFound => StatusCode::NOT_FOUND,
        AdminSupportBasesError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}
