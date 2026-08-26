use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};

use crate::{
    auth::ports::token_port::TokenPort,
    match_id_unlock::{
        adapters::web::dto::{
            CreateMatchIdOrderRequest, CreateMatchIdOrderResponse, MatchIdEntitlementQuery,
            MatchIdEntitlementResponse,
        },
        application::{
            create_match_id_order::{CreateMatchIdOrderInput, CreateMatchIdOrderUseCase},
            get_match_id_entitlement::{GetMatchIdEntitlementUseCase, MatchIdEntitlementInput},
        },
        domain::match_id_unlock::MatchIdUnlockError,
    },
};

#[derive(Clone)]
pub struct MatchIdUnlockWebState {
    pub get_entitlement_use_case: Arc<GetMatchIdEntitlementUseCase>,
    pub create_order_use_case: Arc<CreateMatchIdOrderUseCase>,
    pub token_port: Arc<dyn TokenPort>,
}

pub async fn get_match_id_entitlement_handler(
    State(state): State<Arc<MatchIdUnlockWebState>>,
    headers: HeaderMap,
    Query(query): Query<MatchIdEntitlementQuery>,
) -> Result<Json<MatchIdEntitlementResponse>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let view = state
        .get_entitlement_use_case
        .execute(MatchIdEntitlementInput {
            user_id,
            match_id: query.match_id,
        })
        .await
        .map_err(map_match_id_unlock_error)?;

    Ok(Json(MatchIdEntitlementResponse::from(view)))
}

pub async fn create_match_id_order_handler(
    State(state): State<Arc<MatchIdUnlockWebState>>,
    headers: HeaderMap,
    Json(request): Json<CreateMatchIdOrderRequest>,
) -> Result<Json<CreateMatchIdOrderResponse>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let result = state
        .create_order_use_case
        .execute(CreateMatchIdOrderInput {
            user_id,
            match_id: request.match_id,
        })
        .await
        .map_err(map_match_id_unlock_error)?;

    Ok(Json(CreateMatchIdOrderResponse {
        order_no: result.order_no,
        wx_pay_params: result.wx_pay_params.into(),
    }))
}

fn authenticate_user(
    headers: &HeaderMap,
    token_port: &dyn TokenPort,
) -> Result<uuid::Uuid, (StatusCode, String)> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "请先登录".to_string()))?;

    token_port
        .verify_token(token)
        .map(|claims| claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "请先登录".to_string()))
}

fn map_match_id_unlock_error(error: anyhow::Error) -> (StatusCode, String) {
    if let Some(unlock_error) = error.downcast_ref::<MatchIdUnlockError>() {
        let status = match unlock_error {
            MatchIdUnlockError::MatchNotFound => StatusCode::NOT_FOUND,
            MatchIdUnlockError::MembershipTierSufficient => StatusCode::BAD_REQUEST,
            MatchIdUnlockError::AlreadyUnlocked => StatusCode::CONFLICT,
            MatchIdUnlockError::WechatBindingRequired => StatusCode::FORBIDDEN,
        };

        return (status, unlock_error.to_string());
    }

    let message = error.to_string();
    tracing::error!(error = %message, "match id unlock request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, message)
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use axum::http::StatusCode;

    use super::map_match_id_unlock_error;
    use crate::match_id_unlock::domain::match_id_unlock::MatchIdUnlockError;

    #[test]
    fn maps_typed_unlock_errors_to_specific_statuses() {
        let cases = [
            (MatchIdUnlockError::MatchNotFound, StatusCode::NOT_FOUND),
            (
                MatchIdUnlockError::MembershipTierSufficient,
                StatusCode::BAD_REQUEST,
            ),
            (MatchIdUnlockError::AlreadyUnlocked, StatusCode::CONFLICT),
            (
                MatchIdUnlockError::WechatBindingRequired,
                StatusCode::FORBIDDEN,
            ),
        ];

        for (error, expected_status) in cases {
            let (status, message) = map_match_id_unlock_error(error.into());

            assert_eq!(status, expected_status);
            assert!(!message.is_empty());
        }
    }

    #[test]
    fn keeps_unknown_unlock_errors_as_internal_server_error() {
        let (status, message) = map_match_id_unlock_error(anyhow!("postgres unavailable"));

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, "postgres unavailable");
    }
}
