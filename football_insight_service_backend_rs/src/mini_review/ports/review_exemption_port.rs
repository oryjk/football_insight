use async_trait::async_trait;
use uuid::Uuid;

/// Accounts exempt from review-mode feature hiding (operator-managed test accounts).
#[async_trait]
pub trait ReviewExemptionPort: Send + Sync {
    /// Lookup failures must be treated as "not exempt" by callers so reviewers never see hidden features.
    async fn is_exempt(&self, user_id: Uuid) -> anyhow::Result<bool>;
}

/// Used where no exemption storage is wired (tests, minimal setups).
pub struct NoReviewExemption;

#[async_trait]
impl ReviewExemptionPort for NoReviewExemption {
    async fn is_exempt(&self, _user_id: Uuid) -> anyhow::Result<bool> {
        Ok(false)
    }
}
