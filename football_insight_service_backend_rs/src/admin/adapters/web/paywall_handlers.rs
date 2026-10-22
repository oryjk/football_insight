use crate::admin::{
    application::{
        admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        admin_paywall_service::AdminPaywallService,
    },
    domain::{
        admin_auth::AdminPermission,
        admin_paywall::{AdminPaywallError, PaywallRange, tap_rate},
    },
};
use axum::{
    Json,
    extract::{Query, State, rejection::QueryRejection},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

type HttpError = (StatusCode, String);

#[derive(Clone)]
pub struct AdminPaywallWebState {
    pub service: Arc<AdminPaywallService>,
    pub auth_service: Arc<AdminAuthService>,
}

#[derive(Deserialize)]
pub struct PaywallQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub surface: Option<String>,
}

pub async fn funnel(
    State(state): State<AdminPaywallWebState>,
    headers: HeaderMap,
    query: Result<Query<PaywallQuery>, QueryRejection>,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let range = parse_range(query)?;
    let rows = state.service.funnel(&range).await.map_err(map_error)?;
    Ok(Json(json!({
        "timezone": "Asia/Shanghai",
        "items": rows.into_iter().map(|r| json!({
            "day": r.day, "surface": r.surface,
            "impressions": r.impressions, "impression_users": r.impression_users,
            "taps": r.taps, "tap_users": r.tap_users,
            "user_tap_rate": tap_rate(r.tap_users, r.impression_users),
        })).collect::<Vec<_>>()
    })))
}

pub async fn by_tier(
    State(state): State<AdminPaywallWebState>,
    headers: HeaderMap,
    query: Result<Query<PaywallQuery>, QueryRejection>,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let range = parse_range(query)?;
    let rows = state.service.by_tier(&range).await.map_err(map_error)?;
    Ok(Json(json!({
        "items": rows.into_iter().map(|r| json!({
            "membership_tier": r.membership_tier, "surface": r.surface,
            "impression_users": r.impression_users, "tap_users": r.tap_users,
            "user_tap_rate": tap_rate(r.tap_users, r.impression_users),
        })).collect::<Vec<_>>()
    })))
}

fn parse_range(
    query: Result<Query<PaywallQuery>, QueryRejection>,
) -> Result<PaywallRange, HttpError> {
    let Query(query) =
        query.map_err(|_| (StatusCode::BAD_REQUEST, "invalid paywall query".into()))?;
    PaywallRange::parse(
        query.from.as_deref(),
        query.to.as_deref(),
        query.surface,
        Utc::now(),
    )
    .map_err(map_error)
}

async fn authorize(state: &AdminPaywallWebState, headers: &HeaderMap) -> Result<(), HttpError> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "admin login required".into()))?;
    state
        .auth_service
        .authorize_checked(token, AdminPermission::ViewAuditLogs)
        .await
        .map_err(|error| {
            let status = match error {
                AdminAuthorizationError::Unauthorized => StatusCode::UNAUTHORIZED,
                AdminAuthorizationError::Forbidden => StatusCode::FORBIDDEN,
                AdminAuthorizationError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, error.to_string())
        })?;
    Ok(())
}

fn map_error(error: AdminPaywallError) -> HttpError {
    let status = match error {
        AdminPaywallError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AdminPaywallError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}
