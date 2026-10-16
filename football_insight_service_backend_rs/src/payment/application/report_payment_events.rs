use super::payment_events::PaymentEvents;
use crate::payment::{
    domain::payment_event::{
        NewPaymentEvent, PaymentEventKind, sanitize_detail, sanitize_message, valid_scenario,
    },
    ports::order_repository::OrderRepository,
};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PaymentEventsRequest {
    pub attempt_id: Uuid,
    pub order_no: Option<String>,
    pub scenario: String,
    pub product_ref: Option<String>,
    pub events: Vec<ClientPaymentEvent>,
}
#[derive(Deserialize)]
pub struct ClientPaymentEvent {
    event: String,
    error_code: Option<String>,
    error_msg: Option<String>,
    #[serde(default)]
    detail: Value,
}

pub struct ReportPaymentEvents {
    orders: Arc<dyn OrderRepository>,
    events: PaymentEvents,
    limits: Mutex<HashMap<Uuid, (Instant, usize)>>,
}
impl ReportPaymentEvents {
    pub fn new(orders: Arc<dyn OrderRepository>, events: PaymentEvents) -> Self {
        Self {
            orders,
            events,
            limits: Mutex::new(HashMap::new()),
        }
    }
    pub async fn execute(&self, user_id: Uuid, input: PaymentEventsRequest) {
        if input.events.len() > 10 || !valid_scenario(&input.scenario) || input.attempt_id.is_nil()
        {
            return;
        }
        let order_no = input.order_no.filter(|v| !v.is_empty());
        if let Some(no) = &order_no {
            let owned = matches!(tokio::time::timeout(Duration::from_millis(100),self.orders.find_order_by_no(no)).await, Ok(Ok(Some(order))) if order.user_id==user_id);
            if !owned {
                tracing::warn!("payment telemetry ownership check discarded batch");
                return;
            }
        }
        for event in input.events {
            let Some(kind) = PaymentEventKind::from_client_str(&event.event) else {
                continue;
            };
            if !self.allow(user_id, Instant::now()) {
                continue;
            }
            self.events.record(NewPaymentEvent {
                attempt_id: input.attempt_id,
                order_no: order_no.clone(),
                user_id,
                scenario: input.scenario.clone(),
                product_ref: input
                    .product_ref
                    .as_ref()
                    .map(|v| v.chars().take(100).collect()),
                event: kind,
                error_code: event.error_code.map(|v| v.chars().take(100).collect()),
                error_msg: event.error_msg.map(|v| sanitize_message(&v)),
                detail: sanitize_detail(&event.detail),
                occurred_at: chrono::Utc::now(),
            });
        }
    }
    fn allow(&self, user_id: Uuid, now: Instant) -> bool {
        let Ok(mut limits) = self.limits.lock() else {
            tracing::warn!("payment telemetry limiter unavailable");
            return false;
        };
        limits.retain(|_, (start, _)| now.duration_since(*start) < Duration::from_secs(60));
        let (_, count) = limits.entry(user_id).or_insert((now, 0));
        if *count >= 30 {
            return false;
        }
        *count += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rate_limit_expires_and_is_per_user() {
        let now = Instant::now();
        let mut limits = HashMap::new();
        let id = Uuid::new_v4();
        limits.insert(id, (now, 30));
        // Exercise the actual limiter, without an order lookup or database.
        struct Orders;
        #[async_trait::async_trait]
        impl OrderRepository for Orders {
            async fn create_order(
                &self,
                _: crate::payment::domain::order::NewPaymentOrder,
            ) -> anyhow::Result<crate::payment::domain::order::PaymentOrder> {
                unreachable!()
            }
            async fn find_order_by_no(
                &self,
                _: &str,
            ) -> anyhow::Result<Option<crate::payment::domain::order::PaymentOrder>> {
                unreachable!()
            }
        }
        let s = ReportPaymentEvents {
            orders: Arc::new(Orders),
            events: PaymentEvents::default(),
            limits: Mutex::new(limits),
        };
        assert!(!s.allow(id, now));
        assert!(s.allow(Uuid::new_v4(), now));
        assert!(s.allow(id, now + Duration::from_secs(60)));
    }
}
