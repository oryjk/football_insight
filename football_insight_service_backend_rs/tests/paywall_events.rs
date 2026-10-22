use async_trait::async_trait;
use football_insight_service_backend_rs::activity::{
    application::record_paywall_events::{PaywallEventInput, RecordPaywallEvents},
    domain::paywall::NewPaywallEvent,
    ports::paywall_event_repository::PaywallEventRepository,
};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;
use uuid::Uuid;

struct Recording(mpsc::UnboundedSender<NewPaywallEvent>);
#[async_trait]
impl PaywallEventRepository for Recording {
    async fn record(&self, events: Vec<NewPaywallEvent>) {
        for event in events {
            let _ = self.0.send(event);
        }
    }
}

struct Blocked;
#[async_trait]
impl PaywallEventRepository for Blocked {
    async fn record(&self, _: Vec<NewPaywallEvent>) {
        std::future::pending::<()>().await;
    }
}

struct Panicking;
#[async_trait]
impl PaywallEventRepository for Panicking {
    async fn record(&self, _: Vec<NewPaywallEvent>) {
        panic!("storage exploded");
    }
}

fn input(surface: &str, action: &str) -> PaywallEventInput {
    PaywallEventInput {
        surface: surface.into(),
        action: action.into(),
        membership_tier: Some("V3".into()),
        detail: json!({"mini_version": "1.0.77", "token": "leak"}),
    }
}

fn service() -> (
    RecordPaywallEvents,
    mpsc::UnboundedReceiver<NewPaywallEvent>,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    (RecordPaywallEvents::new(Arc::new(Recording(tx))), rx)
}

async fn drain(rx: &mut mpsc::UnboundedReceiver<NewPaywallEvent>) -> Vec<NewPaywallEvent> {
    tokio::time::sleep(Duration::from_millis(80)).await;
    let mut out = Vec::new();
    while let Ok(event) = rx.try_recv() {
        out.push(event);
    }
    out
}

#[tokio::test]
async fn records_valid_events_with_sanitized_detail_for_the_caller() {
    let (service, mut rx) = service();
    let user = Uuid::new_v4();
    service.execute(
        user,
        vec![input("ticket_watch_recent_reflux_lock", "impression")],
    );
    let events = drain(&mut rx).await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].user_id, user);
    assert_eq!(events[0].surface, "ticket_watch_recent_reflux_lock");
    assert_eq!(events[0].action.as_str(), "impression");
    assert_eq!(events[0].membership_tier.as_deref(), Some("V3"));
    assert_eq!(events[0].detail, json!({"mini_version": "1.0.77"}));
}

#[tokio::test]
async fn drops_unknown_surfaces_and_actions_but_keeps_the_valid_ones() {
    let (service, mut rx) = service();
    service.execute(
        Uuid::new_v4(),
        vec![
            input("home", "impression"),
            input("user_membership_entry", "click"),
            input("user_membership_entry", "tap"),
        ],
    );
    let events = drain(&mut rx).await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].surface, "user_membership_entry");
    assert_eq!(events[0].action.as_str(), "tap");
}

#[tokio::test]
async fn an_oversized_batch_is_ignored_entirely() {
    let (service, mut rx) = service();
    let batch = (0..11)
        .map(|_| input("user_membership_entry", "tap"))
        .collect();
    service.execute(Uuid::new_v4(), batch);
    assert!(drain(&mut rx).await.is_empty());
}

#[tokio::test]
async fn each_user_is_limited_to_sixty_events_per_minute() {
    let (service, mut rx) = service();
    let noisy = Uuid::new_v4();
    let quiet = Uuid::new_v4();
    for _ in 0..7 {
        service.execute(
            noisy,
            (0..10)
                .map(|_| input("user_membership_entry", "tap"))
                .collect(),
        );
    }
    service.execute(quiet, vec![input("user_membership_entry", "tap")]);
    let events = drain(&mut rx).await;
    assert_eq!(events.iter().filter(|e| e.user_id == noisy).count(), 60);
    assert_eq!(events.iter().filter(|e| e.user_id == quiet).count(), 1);
}

#[tokio::test]
async fn a_blocked_or_panicking_store_never_delays_or_breaks_the_caller() {
    for repo in [
        Arc::new(Blocked) as Arc<dyn PaywallEventRepository>,
        Arc::new(Panicking),
    ] {
        let service = RecordPaywallEvents::new(repo);
        let started = std::time::Instant::now();
        service.execute(Uuid::new_v4(), vec![input("user_membership_entry", "tap")]);
        assert!(started.elapsed() < Duration::from_millis(50));
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
}
