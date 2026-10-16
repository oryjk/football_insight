use crate::payment::domain::virtual_payment::{
    Delivery, PrivateMembershipProduct, Refund, VirtualOrder, VirtualPayParams,
    VirtualPaymentError, VirtualProduct,
};
use async_trait::async_trait;

#[async_trait]
pub trait VirtualProductRepository: Send + Sync {
    async fn membership_catalog_products(
        &self,
        _user_id: uuid::Uuid,
    ) -> Result<Vec<crate::payment::domain::product_kind::ProductSpec>, VirtualPaymentError> {
        Ok(vec![])
    }
    async fn find_catalog_product(
        &self,
        key: &str,
        user_id: uuid::Uuid,
    ) -> Result<Option<crate::payment::domain::product_kind::ProductSpec>, VirtualPaymentError>
    {
        Ok(self
            .membership_catalog_products(user_id)
            .await?
            .into_iter()
            .find(|p| p.product_key == key))
    }

    async fn find_enabled(
        &self,
        key: &str,
        user_id: uuid::Uuid,
    ) -> Result<Option<VirtualProduct>, VirtualPaymentError>;
    async fn has_enabled(&self) -> Result<bool, VirtualPaymentError>;
    /// Enabled membership offers explicitly assigned to this user, across all supported tiers.
    async fn private_membership_products(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<PrivateMembershipProduct>, VirtualPaymentError>;
}

#[async_trait]
pub trait VirtualPaymentGateway: Send + Sync {
    fn available(&self) -> bool;
    async fn prepare_session(
        &self,
        code: &str,
        expected_open_id: &str,
    ) -> Result<(), VirtualPaymentError>;
    async fn payment_params(
        &self,
        order_no: &str,
        open_id: &str,
        product: &VirtualProduct,
    ) -> Result<VirtualPayParams, VirtualPaymentError>;
    async fn query_order(
        &self,
        order: &VirtualOrder,
    ) -> Result<Option<Delivery>, VirtualPaymentError>;
    async fn confirm_delivery(&self, order: &VirtualOrder) -> Result<(), VirtualPaymentError>;
}

#[async_trait]
pub trait VirtualOrderRepository: Send + Sync {
    async fn record_with_snapshot(
        &self,
        order: VirtualOrder,
        _snapshot: Option<crate::payment::domain::product_kind::GrantSnapshot>,
    ) -> anyhow::Result<()> {
        self.record(order).await
    }

    async fn record(&self, order: VirtualOrder) -> anyhow::Result<()>;
    async fn find(&self, order_no: &str) -> anyhow::Result<Option<VirtualOrder>>;
    async fn record_delivery(&self, delivery: &Delivery) -> anyhow::Result<()>;
    async fn acknowledge_delivery(&self, order_no: &str) -> anyhow::Result<()>;
    async fn refund(&self, refund: &Refund) -> anyhow::Result<()>;
    async fn pending_orders(&self) -> anyhow::Result<Vec<VirtualOrder>>;
}

pub trait VirtualWebhookPort: Send + Sync {
    fn challenge(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        echo: &str,
    ) -> anyhow::Result<String>;
    fn decode(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        body: &str,
    ) -> anyhow::Result<crate::payment::domain::virtual_payment::VirtualEvent>;
}
