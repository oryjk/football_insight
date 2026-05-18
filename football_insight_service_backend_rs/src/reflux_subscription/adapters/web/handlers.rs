use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};

use crate::{
    auth::ports::token_port::TokenPort,
    reflux_subscription::{
        adapters::web::dto::{
            CreateRefluxSubscriptionOrderRequest, CreateRefluxSubscriptionOrderResponse,
            RefluxNotificationEmailResponse, RefluxSubscriptionPlansResponse,
            RefluxSubscriptionQuery, RefluxSubscriptionStatusResponse,
            UpdateRefluxNotificationEmailRequest,
        },
        application::{
            create_reflux_subscription_order::{
                CreateRefluxSubscriptionOrderInput, CreateRefluxSubscriptionOrderUseCase,
            },
            get_reflux_subscription_plans::GetRefluxSubscriptionPlansUseCase,
        },
        domain::subscription::{is_valid_notification_email, subscription_matches_current_match},
        ports::reflux_subscription_repository::RefluxSubscriptionRepository,
    },
};

#[derive(Clone)]
pub struct RefluxSubscriptionWebState {
    pub get_plans_use_case: Arc<GetRefluxSubscriptionPlansUseCase>,
    pub create_order_use_case: Arc<CreateRefluxSubscriptionOrderUseCase>,
    pub repository: Arc<dyn RefluxSubscriptionRepository>,
    pub token_port: Arc<dyn TokenPort>,
}

pub async fn get_reflux_subscription_plans_handler(
    State(state): State<Arc<RefluxSubscriptionWebState>>,
    headers: HeaderMap,
    Query(query): Query<RefluxSubscriptionQuery>,
) -> Result<Json<RefluxSubscriptionPlansResponse>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let view = state
        .get_plans_use_case
        .execute(user_id, &query.team_code)
        .await
        .map_err(map_reflux_subscription_error)?;

    Ok(Json(RefluxSubscriptionPlansResponse {
        plans: view.plans.into_iter().map(Into::into).collect(),
        active_subscriptions: view
            .active_subscriptions
            .into_iter()
            .map(Into::into)
            .collect(),
        email_target: view.email_target.map(Into::into),
    }))
}

pub async fn get_reflux_subscription_status_handler(
    State(state): State<Arc<RefluxSubscriptionWebState>>,
    headers: HeaderMap,
    Query(query): Query<RefluxSubscriptionQuery>,
) -> Result<Json<RefluxSubscriptionStatusResponse>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let season = query.season.unwrap_or(2026);
    let match_id = query.match_id.unwrap_or_default();
    let view = state
        .get_plans_use_case
        .execute(user_id, &query.team_code)
        .await
        .map_err(map_reflux_subscription_error)?;
    let subscribed = view.active_subscriptions.iter().any(|subscription| {
        subscription_matches_current_match(
            subscription,
            &query.team_code,
            season,
            match_id,
            chrono::Utc::now(),
        )
    });

    Ok(Json(RefluxSubscriptionStatusResponse {
        subscribed,
        active_subscriptions: view
            .active_subscriptions
            .into_iter()
            .map(Into::into)
            .collect(),
        email_target: view.email_target.map(Into::into),
    }))
}

pub async fn create_reflux_subscription_order_handler(
    State(state): State<Arc<RefluxSubscriptionWebState>>,
    headers: HeaderMap,
    Json(request): Json<CreateRefluxSubscriptionOrderRequest>,
) -> Result<Json<CreateRefluxSubscriptionOrderResponse>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let result = state
        .create_order_use_case
        .execute(CreateRefluxSubscriptionOrderInput {
            user_id,
            plan_code: request.plan_code,
            team_code: request.team_code,
            match_id: request.match_id,
            email: request.email,
        })
        .await
        .map_err(map_reflux_subscription_error)?;

    Ok(Json(CreateRefluxSubscriptionOrderResponse {
        order_no: result.order_no,
        params: result.wx_pay_params.into(),
    }))
}

pub async fn get_reflux_notification_email_handler(
    State(state): State<Arc<RefluxSubscriptionWebState>>,
    headers: HeaderMap,
) -> Result<Json<RefluxNotificationEmailResponse>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let target = state
        .repository
        .get_user_email_target(user_id)
        .await
        .map_err(map_reflux_subscription_error)?;

    Ok(Json(RefluxNotificationEmailResponse {
        email: target.map(|target| target.target),
    }))
}

pub async fn update_reflux_notification_email_handler(
    State(state): State<Arc<RefluxSubscriptionWebState>>,
    headers: HeaderMap,
    Json(request): Json<UpdateRefluxNotificationEmailRequest>,
) -> Result<Json<RefluxNotificationEmailResponse>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let email = request.email.trim();
    if !is_valid_notification_email(email) {
        return Err((StatusCode::BAD_REQUEST, "请输入有效的邮箱地址".to_string()));
    }

    let target = state
        .repository
        .upsert_user_email_target(user_id, email)
        .await
        .map_err(map_reflux_subscription_error)?;

    Ok(Json(RefluxNotificationEmailResponse {
        email: Some(target.target),
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

fn map_reflux_subscription_error(error: anyhow::Error) -> (StatusCode, String) {
    let message = error.to_string();
    if message.contains("邮箱")
        || message.contains("套餐")
        || message.contains("比赛")
        || message.contains("绑定微信")
    {
        return (StatusCode::BAD_REQUEST, message);
    }

    tracing::error!(error = %message, "reflux subscription request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, message)
}
