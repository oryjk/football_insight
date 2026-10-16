use async_trait::async_trait;
use football_insight_service_backend_rs::payment::{
    application::{
        payment_events::PaymentEvents,
        report_payment_events::{PaymentEventsRequest, ReportPaymentEvents},
    },
    domain::{
        order::{NewPaymentOrder, OrderStatus, PaymentOrder},
        payment_event::NewPaymentEvent,
    },
    ports::{order_repository::OrderRepository, payment_event_recorder::PaymentEventRecorder},
};
use serde_json::json;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use uuid::Uuid;

#[derive(Default)]
struct Recorder(Mutex<Vec<NewPaymentEvent>>);
#[async_trait]
impl PaymentEventRecorder for Recorder {
    async fn record(&self, e: NewPaymentEvent) {
        self.0.lock().unwrap().push(e);
    }
}
struct Orders {
    owner: Option<Uuid>,
    fails: bool,
}
#[async_trait]
impl OrderRepository for Orders {
    async fn create_order(&self, _: NewPaymentOrder) -> anyhow::Result<PaymentOrder> {
        unreachable!()
    }
    async fn find_order_by_no(&self, no: &str) -> anyhow::Result<Option<PaymentOrder>> {
        if self.fails {
            anyhow::bail!("missing table")
        }
        Ok(self.owner.map(|user_id| PaymentOrder {
            id: Uuid::new_v4(),
            order_no: no.into(),
            user_id,
            amount_cents: 1,
            status: OrderStatus::Pending,
            prepay_id: None,
            transaction_id: None,
            product_type: "membership:V7".into(),
            paid_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }))
    }
}
fn payload(order: Option<&str>, events: serde_json::Value) -> PaymentEventsRequest {
    serde_json::from_value(json!({"attempt_id":Uuid::new_v4(),"order_no":order,"scenario":"membership","product_ref":"V7","events":events})).unwrap()
}
async fn flush() {
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
}

#[tokio::test]
async fn ingestion_discards_server_events_and_sanitizes_before_recording() {
    let r = Arc::new(Recorder::default());
    let user = Uuid::new_v4();
    let s = ReportPaymentEvents::new(
        Arc::new(Orders {
            owner: Some(user),
            fails: false,
        }),
        PaymentEvents::new(r.clone()),
    );
    s.execute(user,payload(Some("own"),json!([{"event":"delivered"},{"event":"pay_cancelled","error_code":"-2","error_msg":"中".repeat(400),"detail":{"token":"secret","elapsed_ms":12}}]))).await;
    flush().await;
    let v = r.0.lock().unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].event.as_str(), "pay_cancelled");
    assert_eq!(v[0].error_msg.as_ref().unwrap().chars().count(), 300);
    assert_eq!(v[0].detail, json!({"elapsed_ms":12}));
    assert_eq!(v[0].error_code.as_deref(), Some("-2"));
}
#[tokio::test]
async fn ingestion_silently_discards_foreign_missing_and_failed_ownership_checks() {
    for (owner, fails) in [(Some(Uuid::new_v4()), false), (None, false), (None, true)] {
        let r = Arc::new(Recorder::default());
        let s = ReportPaymentEvents::new(
            Arc::new(Orders { owner, fails }),
            PaymentEvents::new(r.clone()),
        );
        s.execute(
            Uuid::new_v4(),
            payload(Some("foreign"), json!([{"event":"pay_failed"}])),
        )
        .await;
        flush().await;
        assert!(r.0.lock().unwrap().is_empty());
    }
}
#[tokio::test]
async fn ingestion_limits_thirty_events_per_user_but_does_not_limit_other_users() {
    let r = Arc::new(Recorder::default());
    let user = Uuid::new_v4();
    let s = ReportPaymentEvents::new(
        Arc::new(Orders {
            owner: None,
            fails: false,
        }),
        PaymentEvents::new(r.clone()),
    );
    for _ in 0..35 {
        s.execute(user, payload(None, json!([{"event":"purchase_tapped"}])))
            .await;
    }
    s.execute(
        Uuid::new_v4(),
        payload(None, json!([{"event":"purchase_tapped"}])),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(5)).await;
    assert_eq!(r.0.lock().unwrap().len(), 31);
}
#[tokio::test]
async fn ingestion_drops_oversized_batches_and_invalid_scenarios() {
    let r = Arc::new(Recorder::default());
    let user = Uuid::new_v4();
    let s = ReportPaymentEvents::new(
        Arc::new(Orders {
            owner: None,
            fails: false,
        }),
        PaymentEvents::new(r.clone()),
    );
    s.execute(
        user,
        payload(None, json!(vec![json!({"event":"pay_failed"}); 11])),
    )
    .await;
    let mut p = payload(None, json!([{"event":"pay_failed"}]));
    p.scenario = "fake".into();
    s.execute(user, p).await;
    flush().await;
    assert!(r.0.lock().unwrap().is_empty());
}
