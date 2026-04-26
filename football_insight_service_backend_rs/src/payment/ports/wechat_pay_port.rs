use async_trait::async_trait;

use crate::payment::domain::order::WxPayParams;

#[async_trait]
pub trait WechatPayPort: Send + Sync {
    async fn unified_order(
        &self,
        order_no: &str,
        description: &str,
        amount_cents: i32,
        openid: &str,
    ) -> anyhow::Result<WxPayParams>;
}
