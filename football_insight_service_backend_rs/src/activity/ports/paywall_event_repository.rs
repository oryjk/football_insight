use async_trait::async_trait;

use crate::activity::domain::paywall::NewPaywallEvent;

#[async_trait]
pub trait PaywallEventRepository: Send + Sync {
    /// Measurement only: implementations log and swallow failures, never surface them to callers.
    async fn record(&self, events: Vec<NewPaywallEvent>);
}
