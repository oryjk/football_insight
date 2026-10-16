use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use football_insight_service_backend_rs::{
    auth::{adapters::security::jwt_token_port::JwtTokenPort, ports::token_port::TokenPort},
    payment::{
        adapters::web::event_handlers::payment_event_routes,
        application::{payment_events::PaymentEvents, report_payment_events::ReportPaymentEvents},
        domain::order::{NewPaymentOrder, PaymentOrder},
        ports::order_repository::OrderRepository,
    },
};
use std::sync::Arc;
use tower::ServiceExt;
struct Orders;
#[async_trait]
impl OrderRepository for Orders {
    async fn create_order(&self, _: NewPaymentOrder) -> anyhow::Result<PaymentOrder> {
        unreachable!()
    }
    async fn find_order_by_no(&self, _: &str) -> anyhow::Result<Option<PaymentOrder>> {
        Ok(None)
    }
}
fn app() -> (axum::Router, String) {
    let tokens = Arc::new(JwtTokenPort::new("isolated-test-key".into()));
    let token = tokens
        .issue_token(
            uuid::Uuid::new_v4(),
            "test",
            Utc::now() + Duration::hours(1),
        )
        .unwrap();
    (
        payment_event_routes(
            Arc::new(ReportPaymentEvents::new(
                Arc::new(Orders),
                PaymentEvents::default(),
            )),
            tokens,
        ),
        token,
    )
}
#[tokio::test]
async fn authentication_precedes_body_parsing_and_is_only_401_error() {
    let (app, token) = app();
    for (authorization, body, expected) in [
        (None, "bad".into(), StatusCode::UNAUTHORIZED),
        (
            Some("bad".to_owned()),
            "bad".into(),
            StatusCode::UNAUTHORIZED,
        ),
        (Some(token.clone()), "bad".into(), StatusCode::NO_CONTENT),
        (Some(token), "a".repeat(8193), StatusCode::NO_CONTENT),
    ] {
        let mut request = Request::builder()
            .method("POST")
            .uri("/api/v1/payment/events")
            .header("content-type", "application/json");
        if let Some(token) = authorization {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
}
#[tokio::test]
async fn foreign_order_and_server_event_payloads_return_empty_204() {
    let (app, token) = app();
    let payload=serde_json::json!({"attempt_id":uuid::Uuid::new_v4(),"order_no":"not-owned","scenario":"membership","events":[{"event":"delivered"}]}).to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/payment/events")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(
        axum::body::to_bytes(response.into_body(), 1)
            .await
            .unwrap()
            .is_empty()
    );
}
