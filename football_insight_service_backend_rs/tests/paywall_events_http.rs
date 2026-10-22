use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use football_insight_service_backend_rs::{
    activity::{
        adapters::web::{handlers::ActivityWebState, routes::activity_routes},
        application::{
            record_page_activity::RecordPageActivityUseCase,
            record_paywall_events::RecordPaywallEvents,
        },
        domain::paywall::NewPaywallEvent,
        ports::{
            paywall_event_repository::PaywallEventRepository,
            user_activity_repository::UserActivityRepository,
        },
    },
    auth::{adapters::security::jwt_token_port::JwtTokenPort, ports::token_port::TokenPort},
};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use uuid::Uuid;

struct Pages;
#[async_trait]
impl UserActivityRepository for Pages {
    async fn record_page_activity(&self, _: Uuid, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

struct Store(Arc<Mutex<Vec<NewPaywallEvent>>>);
#[async_trait]
impl PaywallEventRepository for Store {
    async fn record(&self, events: Vec<NewPaywallEvent>) {
        self.0.lock().unwrap().extend(events);
    }
}

fn app() -> (axum::Router, String, Uuid, Arc<Mutex<Vec<NewPaywallEvent>>>) {
    let tokens = Arc::new(JwtTokenPort::new("isolated-test-key".into()));
    let user = Uuid::new_v4();
    let token = tokens
        .issue_token(user, "test", Utc::now() + Duration::hours(1))
        .unwrap();
    let stored = Arc::new(Mutex::new(Vec::new()));
    let state = Arc::new(ActivityWebState {
        record_page_activity_use_case: Arc::new(RecordPageActivityUseCase::new(Arc::new(Pages))),
        record_paywall_events_use_case: Arc::new(RecordPaywallEvents::new(Arc::new(Store(
            stored.clone(),
        )))),
        token_port: tokens,
    });
    (activity_routes(state), token, user, stored)
}

async fn post(app: &axum::Router, token: Option<&str>, body: String) -> StatusCode {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/v1/activity/paywall-events")
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    app.clone()
        .oneshot(request.body(Body::from(body)).unwrap())
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn only_a_missing_or_invalid_token_is_an_error() {
    let (app, token, _, stored) = app();
    assert_eq!(
        post(&app, None, "{}".into()).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        post(&app, Some("bad"), "{}".into()).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        post(&app, Some(&token), "not json".into()).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        post(&app, Some(&token), "a".repeat(4097)).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        post(&app, Some(&token), r#"{"events":"x"}"#.into()).await,
        StatusCode::NO_CONTENT
    );
    tokio::time::sleep(std::time::Duration::from_millis(60)).await;
    assert!(stored.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_valid_batch_is_stored_for_the_token_owner_only() {
    let (app, token, user, stored) = app();
    let body = serde_json::json!({"events":[
        {"surface":"user_membership_entry","action":"tap","membership_tier":"V3","detail":{"platform":"android","openid":"leak"}},
        {"surface":"not_a_surface","action":"tap"}
    ]})
    .to_string();
    assert_eq!(post(&app, Some(&token), body).await, StatusCode::NO_CONTENT);
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    let events = stored.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].user_id, user);
    assert_eq!(events[0].detail, serde_json::json!({"platform":"android"}));
}
