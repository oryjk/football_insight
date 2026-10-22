use async_trait::async_trait;

use crate::admin::domain::admin_paywall::{
    AdminPaywallError, PaywallFunnelRow, PaywallRange, PaywallTierRow,
};

#[async_trait]
pub trait AdminPaywallRepository: Send + Sync {
    async fn funnel(
        &self,
        range: &PaywallRange,
    ) -> Result<Vec<PaywallFunnelRow>, AdminPaywallError>;
    async fn by_tier(&self, range: &PaywallRange)
    -> Result<Vec<PaywallTierRow>, AdminPaywallError>;
}
