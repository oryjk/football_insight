use crate::payment::domain::virtual_payment::{
    Delivery, Refund, VirtualOrder, VirtualPayParams, VirtualPaymentError, VirtualProduct,
};
use async_trait::async_trait;

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
