use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    auth::ports::token_port::TokenPort,
    seat_swap::{
        adapters::web::dto::{
            CancelMatchedSeatSwapRequestDto, SeatSwapCurrentResponse, UpsertSeatSwapRequestDto,
        },
        application::{
            cancel_matched_seat_swap::CancelMatchedSeatSwapUseCase,
            cancel_my_seat_swap_request::CancelMySeatSwapRequestUseCase,
            cancel_seat_swap_confirmation::CancelSeatSwapConfirmationUseCase,
            confirm_seat_swap_candidate::ConfirmSeatSwapCandidateUseCase,
            get_current_seat_swap::GetCurrentSeatSwapUseCase,
            upsert_my_seat_swap_request::UpsertMySeatSwapRequestUseCase,
        },
        domain::SeatSwapError,
    },
};

#[derive(Clone)]
pub struct SeatSwapWebState {
    pub get_current_use_case: Arc<GetCurrentSeatSwapUseCase>,
    pub upsert_my_request_use_case: Arc<UpsertMySeatSwapRequestUseCase>,
    pub cancel_my_request_use_case: Arc<CancelMySeatSwapRequestUseCase>,
    pub confirm_candidate_use_case: Arc<ConfirmSeatSwapCandidateUseCase>,
    pub cancel_confirmation_use_case: Arc<CancelSeatSwapConfirmationUseCase>,
    pub cancel_matched_use_case: Arc<CancelMatchedSeatSwapUseCase>,
    pub token_port: Arc<dyn TokenPort>,
}

pub async fn get_current_seat_swap_handler(
    State(state): State<Arc<SeatSwapWebState>>,
    headers: HeaderMap,
) -> Result<Json<SeatSwapCurrentResponse>, (StatusCode, String)> {
    let viewer_user_id = authenticate_user_optional(&headers, state.token_port.as_ref());
    let view = state
        .get_current_use_case
        .execute(viewer_user_id)
        .await
        .map_err(map_anyhow_error)?;

    Ok(Json(view.into()))
}

pub async fn upsert_my_seat_swap_request_handler(
    State(state): State<Arc<SeatSwapWebState>>,
    headers: HeaderMap,
    Json(request): Json<UpsertSeatSwapRequestDto>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    state
        .upsert_my_request_use_case
        .execute(user_id, request.into())
        .await
        .map_err(map_seat_swap_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_my_seat_swap_request_handler(
    State(state): State<Arc<SeatSwapWebState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    state
        .cancel_my_request_use_case
        .execute(user_id)
        .await
        .map_err(map_seat_swap_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn confirm_seat_swap_candidate_handler(
    State(state): State<Arc<SeatSwapWebState>>,
    headers: HeaderMap,
    Path(target_request_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    state
        .confirm_candidate_use_case
        .execute(user_id, target_request_id)
        .await
        .map_err(map_seat_swap_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn cancel_seat_swap_confirmation_handler(
    State(state): State<Arc<SeatSwapWebState>>,
    headers: HeaderMap,
    Path(target_request_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    state
        .cancel_confirmation_use_case
        .execute(user_id, target_request_id)
        .await
        .map_err(map_seat_swap_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn cancel_matched_seat_swap_handler(
    State(state): State<Arc<SeatSwapWebState>>,
    headers: HeaderMap,
    Path(target_request_id): Path<Uuid>,
    Json(request): Json<CancelMatchedSeatSwapRequestDto>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    state
        .cancel_matched_use_case
        .execute(
            user_id,
            request
                .into_input(target_request_id)
                .map_err(map_seat_swap_error)?,
        )
        .await
        .map_err(map_seat_swap_error)?;
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

fn authenticate_user_optional(headers: &HeaderMap, token_port: &dyn TokenPort) -> Option<Uuid> {
    let token = extract_bearer_token(headers)?;
    token_port.verify_token(token).ok().map(|claims| claims.sub)
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    header_value.strip_prefix("Bearer ")
}

fn format_error_chain(error: &anyhow::Error) -> String {
    let mut chain = error.chain();
    let mut formatted = chain
        .next()
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| "unknown error".to_string());

    for cause in chain {
        formatted.push_str(" | caused by: ");
        formatted.push_str(&cause.to_string());
    }

    formatted
}

fn map_anyhow_error(error: anyhow::Error) -> (StatusCode, String) {
    tracing::error!(
        error = %error,
        error_chain = %format_error_chain(&error),
        "seat swap request failed"
    );
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn map_seat_swap_error(error: SeatSwapError) -> (StatusCode, String) {
    if error.is_bad_request() {
        return (StatusCode::BAD_REQUEST, error.to_string());
    }

    let message = error.to_string();
    match error {
        SeatSwapError::Internal(source) => map_anyhow_error(source),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, message),
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, anyhow};

    use super::{format_error_chain, map_seat_swap_error};
    use crate::seat_swap::domain::SeatSwapError;
    use axum::http::StatusCode;

    #[test]
    fn format_error_chain_includes_all_contexts() {
        let error = Err::<(), _>(anyhow!("db write failed"))
            .context("persist seat swap request failed")
            .context("upsert my seat swap request failed")
            .unwrap_err();

        assert_eq!(
            format_error_chain(&error),
            "upsert my seat swap request failed | caused by: persist seat swap request failed | caused by: db write failed"
        );
    }

    #[test]
    fn maps_typed_business_errors_without_string_matching() {
        let (status, message) = map_seat_swap_error(SeatSwapError::NoCurrentMatch.into());

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(message, "当前暂无可换座比赛");
    }
}
