use crate::admin::{
    application::{
        admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        admin_topic_readers_service::AdminTopicReadersService,
    },
    domain::{admin_auth::AdminPermission, admin_topic_readers::AdminTopicReadersError},
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde_json::{Value, json};
use std::sync::Arc;

type HttpError = (StatusCode, String);

#[derive(Clone)]
pub struct AdminTopicReadersWebState {
    pub service: Arc<AdminTopicReadersService>,
    pub auth_service: Arc<AdminAuthService>,
}

pub async fn topics(
    State(state): State<AdminTopicReadersWebState>,
    headers: HeaderMap,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let rows = state.service.recent_topics().await.map_err(map_error)?;
    Ok(Json(json!({
        "items": rows.into_iter().map(|t| json!({
            "team_id": t.team_id, "topic_id": t.topic_id, "title": t.title,
            "edition_date": t.edition_date, "reader_count": t.reader_count,
        })).collect::<Vec<_>>()
    })))
}

pub async fn readers(
    State(state): State<AdminTopicReadersWebState>,
    headers: HeaderMap,
    Path((team_id, topic_id)): Path<(i64, String)>,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let rows = state.service.readers(team_id, &topic_id).await.map_err(map_error)?;
    Ok(Json(json!({
        "items": rows.into_iter().map(|r| json!({
            "user_id": r.user_id, "display_name": r.display_name, "avatar_url": r.avatar_url,
            "first_read_at": r.first_read_at,
        })).collect::<Vec<_>>()
    })))
}

async fn authorize(state: &AdminTopicReadersWebState, headers: &HeaderMap) -> Result<(), HttpError> {
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

fn map_error(error: AdminTopicReadersError) -> HttpError {
    let status = match error {
        AdminTopicReadersError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AdminTopicReadersError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}
