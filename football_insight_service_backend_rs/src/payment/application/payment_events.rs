use crate::payment::{
    domain::{
        order::PaymentOrder,
        payment_event::{NewPaymentEvent, PaymentEventKind, sanitize_message},
        virtual_payment::VirtualPaymentError,
    },
    ports::{order_repository::OrderRepository, payment_event_recorder::PaymentEventRecorder},
};
use chrono::Utc;
use futures_util::FutureExt;
use std::{panic::AssertUnwindSafe, sync::Arc, time::Duration};
use uuid::Uuid;

#[derive(Clone)]
pub struct PaymentEvents {
    enabled: bool,
    recorder: Arc<dyn PaymentEventRecorder>,
    slots: Arc<tokio::sync::Semaphore>,
}

struct NoopRecorder;
#[async_trait::async_trait]
impl PaymentEventRecorder for NoopRecorder {
    async fn record(&self, _: NewPaymentEvent) {}
}
impl Default for PaymentEvents {
    fn default() -> Self {
        let mut value = Self::new(Arc::new(NoopRecorder));
        value.enabled = false;
        value
    }
}

impl PaymentEvents {
    pub fn new(recorder: Arc<dyn PaymentEventRecorder>) -> Self {
        Self {
            enabled: true,
            recorder,
            slots: Arc::new(tokio::sync::Semaphore::new(128)),
        }
    }
    fn dispatch(&self, work: impl std::future::Future<Output = ()> + Send + 'static) {
        if !self.enabled {
            return;
        }
        let Ok(permit) = self.slots.clone().try_acquire_owned() else {
            tracing::warn!("payment telemetry capacity exceeded");
            return;
        };
        tokio::spawn(async move {
            let _permit = permit;
            match tokio::time::timeout(
                Duration::from_millis(250),
                AssertUnwindSafe(work).catch_unwind(),
            )
            .await
            {
                Ok(Ok(())) => {}
                Ok(Err(_)) => tracing::warn!("payment telemetry panicked"),
                Err(_) => tracing::warn!("payment telemetry timed out"),
            }
        });
    }
    pub fn record(&self, event: NewPaymentEvent) {
        let recorder = self.recorder.clone();
        self.dispatch(async move {
            recorder.record(event).await;
        });
    }
    pub fn record_order(&self, order: PaymentOrder, kind: PaymentEventKind) {
        let recorder = self.recorder.clone();
        let occurred_at = Utc::now();
        self.dispatch(async move {
            record_order(recorder, order, kind, occurred_at).await;
        });
    }
    /// Refund callbacks do not carry user/product metadata. Resolve it off the critical path.
    pub fn record_order_no(
        &self,
        orders: Arc<dyn OrderRepository>,
        order_no: String,
        kind: PaymentEventKind,
    ) {
        let recorder = self.recorder.clone();
        let occurred_at = Utc::now();
        self.dispatch(async move {
            match orders.find_order_by_no(&order_no).await {
                Ok(Some(order)) => record_order(recorder, order, kind, occurred_at).await,
                _ => tracing::warn!("payment telemetry order lookup failed"),
            }
        });
    }
}

async fn record_order(
    recorder: Arc<dyn PaymentEventRecorder>,
    order: PaymentOrder,
    kind: PaymentEventKind,
    occurred_at: chrono::DateTime<Utc>,
) {
    let mut parts = order.product_type.split(':');
    let scenario = parts.next().unwrap_or("membership").to_owned();
    if !crate::payment::domain::payment_event::valid_scenario(&scenario) {
        return;
    }
    let product_ref = parts.next().map(str::to_owned);
    let attempt_id = recorder
        .attempt_for_order(&order.order_no)
        .await
        .unwrap_or(order.id);
    recorder
        .record(NewPaymentEvent {
            attempt_id,
            order_no: Some(order.order_no),
            user_id: order.user_id,
            scenario,
            product_ref,
            event: kind,
            error_code: None,
            error_msg: None,
            detail: serde_json::json!({}),
            occurred_at,
        })
        .await;
}

pub struct OrderAttempt {
    events: PaymentEvents,
    base: NewPaymentEvent,
}
impl OrderAttempt {
    pub fn new(
        events: PaymentEvents,
        attempt_id: Uuid,
        user_id: Uuid,
        scenario: &str,
        product_ref: String,
        order_no: String,
    ) -> Self {
        Self {
            events,
            base: NewPaymentEvent {
                attempt_id,
                order_no: Some(order_no),
                user_id,
                scenario: scenario.into(),
                product_ref: Some(product_ref),
                event: PaymentEventKind::OrderCreated,
                error_code: None,
                error_msg: None,
                detail: serde_json::json!({}),
                occurred_at: Utc::now(),
            },
        }
    }
    pub fn created(&self) {
        self.events.record(self.base.clone());
    }
    pub fn params<T>(&self, result: anyhow::Result<T>) -> anyhow::Result<T> {
        let mut event = self.base.clone();
        event.occurred_at = Utc::now();
        match &result {
            Ok(_) => event.event = PaymentEventKind::PayParamsIssued,
            Err(error) => {
                event.event = PaymentEventKind::PayParamsFailed;
                event.error_code = Some(
                    match error.downcast_ref::<VirtualPaymentError>() {
                        Some(VirtualPaymentError::Unavailable) => "Unavailable",
                        Some(VirtualPaymentError::ProductUnavailable) => "ProductUnavailable",
                        Some(VirtualPaymentError::CatalogUnavailable) => "CatalogUnavailable",
                        Some(VirtualPaymentError::SessionExpired) => "SessionExpired",
                        Some(VirtualPaymentError::IdentityMismatch) => "IdentityMismatch",
                        Some(VirtualPaymentError::InvalidOrder) => "InvalidOrder",
                        Some(VirtualPaymentError::Provider) => "Provider",
                        None => "Other",
                    }
                    .into(),
                );
                // Only typed payment errors have safe, fixed text. Arbitrary provider errors may contain credentials.
                event.error_msg = error
                    .downcast_ref::<VirtualPaymentError>()
                    .map(|e| sanitize_message(&e.to_string()));
            }
        }
        self.events.record(event);
        result
    }
}
