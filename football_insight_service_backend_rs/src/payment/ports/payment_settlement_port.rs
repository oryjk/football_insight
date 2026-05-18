use async_trait::async_trait;
use uuid::Uuid;

use crate::payment::domain::order::RefluxSubscriptionProductType;

#[async_trait]
pub trait PaymentSettlementPort: Send + Sync {
    async fn settle_membership_order(
        &self,
        order_no: &str,
        transaction_id: &str,
        user_id: Uuid,
        tier: &str,
    ) -> anyhow::Result<()>;

    async fn settle_reflux_subscription_order(
        &self,
        order_no: &str,
        transaction_id: &str,
        user_id: Uuid,
        product: RefluxSubscriptionProductType,
    ) -> anyhow::Result<()>;
}
