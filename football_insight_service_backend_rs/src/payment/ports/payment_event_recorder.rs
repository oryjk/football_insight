use crate::payment::domain::payment_event::NewPaymentEvent;
use async_trait::async_trait;

#[async_trait]
pub trait PaymentEventRecorder: Send + Sync {
    /// Best effort only. Implementations must warn and swallow storage failures.
    async fn record(&self, event: NewPaymentEvent);
    async fn attempt_for_order(&self, _order_no: &str) -> Option<uuid::Uuid> {
        None
    }
}
