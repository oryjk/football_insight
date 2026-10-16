use crate::admin::{
    application::{
        admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        admin_payment_service::AdminPaymentService,
    },
    domain::{
        admin_auth::AdminPermission,
        admin_payment::{AdminPaymentError, PaymentRange},
    },
};
use axum::{
    Json,
    extract::{Path, Query, State, rejection::QueryRejection},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

type HttpError = (StatusCode, String);
#[derive(Clone)]
pub struct AdminPaymentWebState {
    pub service: Arc<AdminPaymentService>,
    pub auth_service: Arc<AdminAuthService>,
}
#[derive(Deserialize)]
pub struct PaymentRangeQuery {
    pub from: String,
    pub to: String,
    pub scenario: Option<String>,
}
#[derive(Deserialize)]
pub struct StuckQuery {
    pub older_than_minutes: Option<i64>,
}

pub async fn funnel(
    State(state): State<AdminPaymentWebState>,
    headers: HeaderMap,
    query: Result<Query<PaymentRangeQuery>, QueryRejection>,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let range = parse_range(query)?;
    let data = state.service.funnel(range).await.map_err(map_error)?;
    Ok(Json(json!({
        "timezone":"UTC", "stages":data.stages.into_iter().map(|s| json!({"day":s.counts.day,"scenario":s.counts.scenario,"product_ref":s.counts.product_ref,"event":s.counts.event,"attempt_count":s.counts.attempt_count,"user_count":s.counts.user_count,"previous_event":s.counts.previous_event,"previous_attempt_count":s.counts.previous_attempt_count,"converted_attempt_count":s.counts.converted_attempt_count,"conversion_rate":s.conversion_rate})).collect::<Vec<_>>(),
        "terminal_stages":data.terminal_stages.into_iter().map(|s|json!({"day":s.day,"scenario":s.scenario,"product_ref":s.product_ref,"stage":s.stage,"attempt_count":s.attempt_count,"user_count":s.user_count})).collect::<Vec<_>>()
    })))
}
pub async fn errors(
    State(state): State<AdminPaymentWebState>,
    headers: HeaderMap,
    query: Result<Query<PaymentRangeQuery>, QueryRejection>,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let data = state
        .service
        .errors(parse_range(query)?)
        .await
        .map_err(map_error)?;
    Ok(Json(
        json!({"items":data.into_iter().map(|s|json!({"event":s.event,"error_code":s.error_code,"error_msg":s.error_msg,"count":s.count,"total_error_count":s.total_error_count,"share":if s.total_error_count>0 {Some(s.count as f64/s.total_error_count as f64)} else {None},"last_occurred_at":s.last_occurred_at})).collect::<Vec<_>>()}),
    ))
}
pub async fn timeline(
    State(state): State<AdminPaymentWebState>,
    headers: HeaderMap,
    Path(order_no): Path<String>,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let data = state.service.timeline(&order_no).await.map_err(map_error)?;
    Ok(Json(
        json!({"order_no":order_no,"events":data.into_iter().map(|s|json!({"id":s.id,"attempt_id":s.attempt_id,"order_no":s.order_no,"user_id":s.user_id,"scenario":s.scenario,"product_ref":s.product_ref,"event":s.event,"source":s.source,"error_code":s.error_code,"error_msg":s.error_msg,"detail":s.detail,"occurred_at":s.occurred_at})).collect::<Vec<_>>()}),
    ))
}
pub async fn stuck(
    State(state): State<AdminPaymentWebState>,
    headers: HeaderMap,
    query: Result<Query<StuckQuery>, QueryRejection>,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let Query(query) =
        query.map_err(|_| (StatusCode::BAD_REQUEST, "invalid payment query".into()))?;
    let data = state
        .service
        .stuck(query.older_than_minutes.unwrap_or(30))
        .await
        .map_err(map_error)?;
    Ok(Json(
        json!({"items":data.into_iter().map(|s|json!({"order_no":s.order.order_no,"user_id":s.order.user_id,"product_type":s.order.product_type,"created_at":s.order.created_at,"last_event":s.order.last_event,"last_event_at":s.order.last_event_at,"stage":s.stage})).collect::<Vec<_>>()}),
    ))
}
fn parse_range(
    query: Result<Query<PaymentRangeQuery>, QueryRejection>,
) -> Result<PaymentRange, HttpError> {
    let Query(query) =
        query.map_err(|_| (StatusCode::BAD_REQUEST, "invalid payment query".into()))?;
    PaymentRange::parse(&query.from, &query.to, query.scenario).map_err(map_error)
}
async fn authorize(state: &AdminPaymentWebState, headers: &HeaderMap) -> Result<(), HttpError> {
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
fn map_error(error: AdminPaymentError) -> HttpError {
    let status = match error {
        AdminPaymentError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AdminPaymentError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}
