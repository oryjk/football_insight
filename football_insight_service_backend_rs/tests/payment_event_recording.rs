use async_trait::async_trait;
use football_insight_service_backend_rs::payment::{
    application::payment_events::{OrderAttempt, PaymentEvents},
    domain::{
        payment_event::{NewPaymentEvent, PaymentEventKind},
        virtual_payment::VirtualPaymentError,
    },
    ports::payment_event_recorder::PaymentEventRecorder,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Default)]
struct Recorder(Mutex<Vec<NewPaymentEvent>>);
#[async_trait]
impl PaymentEventRecorder for Recorder {
    async fn record(&self, event: NewPaymentEvent) {
        self.0.lock().unwrap().push(event);
    }
}
struct Blocked;
#[async_trait]
impl PaymentEventRecorder for Blocked {
    async fn record(&self, _: NewPaymentEvent) {
        std::future::pending::<()>().await;
    }
}
struct Panics;
#[async_trait]
impl PaymentEventRecorder for Panics {
    async fn record(&self, _: NewPaymentEvent) {
        panic!("telemetry failed");
    }
}
fn attempt(recorder: Arc<dyn PaymentEventRecorder>) -> OrderAttempt {
    OrderAttempt::new(
        PaymentEvents::new(recorder),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        "membership",
        "V7".into(),
        "test-order".into(),
    )
}

#[tokio::test]
async fn parameter_failure_records_typed_code_without_replacing_original_error() {
    let recorder = Arc::new(Recorder::default());
    let a = attempt(recorder.clone());
    a.created();
    let error = a
        .params::<()>(Err(VirtualPaymentError::Provider.into()))
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref(),
        Some(VirtualPaymentError::Provider)
    ));
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    let events = recorder.0.lock().unwrap();
    assert_eq!(
        events.iter().map(|v| v.event.as_str()).collect::<Vec<_>>(),
        ["order_created", "pay_params_failed"]
    );
    assert_eq!(events[1].error_code.as_deref(), Some("Provider"));
    assert_eq!(events[1].order_no.as_deref(), Some("test-order"));
    assert_eq!(events[1].attempt_id, events[0].attempt_id);
}

#[tokio::test]
async fn blocked_or_panicking_recorder_cannot_block_or_fail_payment_result() {
    for recorder in [
        Arc::new(Blocked) as Arc<dyn PaymentEventRecorder>,
        Arc::new(Panics),
    ] {
        let result = tokio::time::timeout(Duration::from_millis(50), async {
            let a = attempt(recorder);
            a.created();
            a.params(Ok(42))
        })
        .await
        .unwrap()
        .unwrap();
        assert_eq!(result, 42);
        tokio::task::yield_now().await;
    }
}

#[tokio::test]
async fn successful_parameters_emit_issued_and_other_error_uses_other_code() {
    let recorder = Arc::new(Recorder::default());
    let a = attempt(recorder.clone());
    assert_eq!(a.params(Ok(7)).unwrap(), 7);
    assert_eq!(
        a.params::<()>(Err(anyhow::anyhow!("original")))
            .unwrap_err()
            .to_string(),
        "original"
    );
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    let events = recorder.0.lock().unwrap();
    assert_eq!(events[0].event, PaymentEventKind::PayParamsIssued);
    assert_eq!(events[1].error_code.as_deref(), Some("Other"));
}
