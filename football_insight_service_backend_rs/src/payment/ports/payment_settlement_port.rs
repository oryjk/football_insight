use async_trait::async_trait;
use uuid::Uuid;

use crate::payment::domain::order::RefluxSubscriptionProductType;

#[async_trait]
pub trait PaymentSettlementPort: Send + Sync {
    async fn settle_student_service_order(
        &self,
        _order_no: &str,
        _transaction_id: &str,
        _user_id: Uuid,
        _code: crate::student::domain::ServiceCode,
    ) -> anyhow::Result<()> {
        anyhow::bail!("student settlement is not configured")
    }
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

    async fn settle_match_id_unlock_order(
        &self,
        order_no: &str,
        transaction_id: &str,
        user_id: Uuid,
        match_id: i64,
    ) -> anyhow::Result<()>;
}
