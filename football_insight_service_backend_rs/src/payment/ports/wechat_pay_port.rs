use async_trait::async_trait;

use crate::payment::domain::order::WxPayParams;

#[async_trait]
pub trait WechatPayPort: Send + Sync {
    async fn unified_membership_order(
        &self,
        order_no: &str,
        description: &str,
        amount_cents: i32,
        openid: &str,
        _capabilities: &[String],
    ) -> anyhow::Result<WxPayParams> {
        self.unified_order(order_no, description, amount_cents, openid)
            .await
    }

    async fn unified_order(
        &self,
        order_no: &str,
        description: &str,
        amount_cents: i32,
        openid: &str,
    ) -> anyhow::Result<WxPayParams>;
}
