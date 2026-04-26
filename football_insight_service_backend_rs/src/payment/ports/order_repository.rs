use async_trait::async_trait;

use crate::payment::domain::order::{NewPaymentOrder, PaymentOrder};

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn create_order(&self, order: NewPaymentOrder) -> anyhow::Result<PaymentOrder>;
    async fn find_order_by_no(&self, order_no: &str) -> anyhow::Result<Option<PaymentOrder>>;
}
