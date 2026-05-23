use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    activity::{
        adapters::web::dto::RecordPageActivityRequest,
        application::record_page_activity::{RecordPageActivityInput, RecordPageActivityUseCase},
        domain::page_key::ActivityError,
    },
    auth::ports::token_port::TokenPort,
};

#[derive(Clone)]
pub struct ActivityWebState {
    pub record_page_activity_use_case: Arc<RecordPageActivityUseCase>,
    pub token_port: Arc<dyn TokenPort>,
}

pub async fn record_page_activity_handler(
    State(state): State<Arc<ActivityWebState>>,
    headers: HeaderMap,
    Json(request): Json<RecordPageActivityRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;

    state
        .record_page_activity_use_case
        .execute(RecordPageActivityInput {
            user_id,
            page_key: request.page_key,
        })
        .await
        .map_err(map_activity_error)?;

    Ok(StatusCode::NO_CONTENT)
}

fn authenticate_user(
    headers: &HeaderMap,
    token_port: &dyn TokenPort,
) -> Result<Uuid, (StatusCode, String)> {
    let token =
        extract_bearer_token(headers).ok_or((StatusCode::UNAUTHORIZED, "请先登录".to_string()))?;

    token_port
        .verify_token(token)
        .map(|claims| claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "请先登录".to_string()))
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    header_value.strip_prefix("Bearer ")
}

fn map_activity_error(error: anyhow::Error) -> (StatusCode, String) {
    if let Some(activity_error) = error.downcast_ref::<ActivityError>() {
        tracing::warn!(error = %activity_error, "activity page-view request rejected");
        return (StatusCode::BAD_REQUEST, activity_error.to_string());
    }

    let message = error.to_string();
    tracing::error!(error = %message, "activity page-view request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, message)
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use axum::http::StatusCode;

    use super::map_activity_error;
    use crate::activity::domain::page_key::ActivityError;

    #[test]
    fn maps_typed_activity_validation_error_to_bad_request() {
        let (status, message) = map_activity_error(ActivityError::UnsupportedPageKey.into());

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(message, "页面标识不支持");
    }

    #[test]
    fn keeps_unknown_activity_errors_as_internal_server_error() {
        let (status, message) = map_activity_error(anyhow!("db write failed"));

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, "db write failed");
    }
}
