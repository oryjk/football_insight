use std::sync::Arc;

use crate::admin::{
    domain::admin_paywall::{AdminPaywallError, PaywallFunnelRow, PaywallRange, PaywallTierRow},
    ports::admin_paywall_repository::AdminPaywallRepository,
};

pub struct AdminPaywallService {
    repository: Arc<dyn AdminPaywallRepository>,
}

impl AdminPaywallService {
    pub fn new(repository: Arc<dyn AdminPaywallRepository>) -> Self {
        Self { repository }
    }

    pub async fn funnel(
        &self,
        range: &PaywallRange,
    ) -> Result<Vec<PaywallFunnelRow>, AdminPaywallError> {
        self.repository.funnel(range).await
    }

    pub async fn by_tier(
        &self,
        range: &PaywallRange,
    ) -> Result<Vec<PaywallTierRow>, AdminPaywallError> {
        self.repository.by_tier(range).await
    }
}
