use crate::admin::domain::admin_payment::*;
use async_trait::async_trait;

#[async_trait]
pub trait AdminPaymentRepository: Send + Sync {
    async fn funnel(&self, range: &PaymentRange) -> Result<Vec<FunnelCount>, AdminPaymentError>;
    async fn terminals(
        &self,
        range: &PaymentRange,
    ) -> Result<Vec<AttemptTerminalCount>, AdminPaymentError>;
    async fn errors(
        &self,
        range: &PaymentRange,
    ) -> Result<Vec<PaymentErrorCount>, AdminPaymentError>;
    async fn timeline(
        &self,
        order_no: &str,
    ) -> Result<Vec<PaymentTimelineEvent>, AdminPaymentError>;
    async fn stuck(
        &self,
        older_than_minutes: i64,
    ) -> Result<Vec<StuckPaymentOrder>, AdminPaymentError>;
}
