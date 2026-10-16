use crate::{
    auth::ports::token_port::TokenPort,
    payment::application::report_payment_events::{PaymentEventsRequest, ReportPaymentEvents},
};
use axum::{
    Router,
    body::to_bytes,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use std::sync::Arc;

#[derive(Clone)]
struct EventWebState {
    service: Arc<ReportPaymentEvents>,
    tokens: Arc<dyn TokenPort>,
}

pub fn payment_event_routes(
    service: Arc<ReportPaymentEvents>,
    tokens: Arc<dyn TokenPort>,
) -> Router {
    Router::new()
        .route("/api/v1/payment/events", post(report))
        .with_state(EventWebState { service, tokens })
}
async fn report(
    State(state): State<EventWebState>,
    headers: HeaderMap,
    request: Request,
) -> StatusCode {
    let Ok(user_id) = super::handlers::authenticate_user(&headers, state.tokens.as_ref()) else {
        return StatusCode::UNAUTHORIZED;
    };
    // Parse manually so invalid JSON, oversized bodies and unsupported content types remain 204.
    let Ok(bytes) = to_bytes(request.into_body(), 8192).await else {
        tracing::warn!("payment telemetry body discarded");
        return StatusCode::NO_CONTENT;
    };
    if let Ok(input) = serde_json::from_slice::<PaymentEventsRequest>(&bytes) {
        state.service.execute(user_id, input).await;
    }
    StatusCode::NO_CONTENT
}

pub fn attempt_from_headers(headers: &HeaderMap) -> uuid::Uuid {
    crate::payment::domain::payment_event::attempt_id(
        headers
            .get("X-Payment-Attempt-Id")
            .and_then(|v| v.to_str().ok()),
    )
}
