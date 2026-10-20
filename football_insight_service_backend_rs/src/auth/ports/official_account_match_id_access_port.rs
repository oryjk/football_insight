use async_trait::async_trait;

use crate::auth::domain::official_account_match_id_access::OfficialAccountMatchIdAccess;

/// Official-account identity, current-match entitlement and lifetime acquisition history.
#[async_trait]
pub trait OfficialAccountMatchIdAccessPort: Send + Sync {
    async fn check(
        &self,
        official_open_id: &str,
        match_id: Option<i64>,
    ) -> anyhow::Result<OfficialAccountMatchIdAccess>;

    /// Atomically record acquisition for this official identity and its linked accounts.
    /// When `require_first` is true, return false if either has ever received an ID.
    /// Entitled acquisitions are also recorded so they cannot later claim a free first ID.
    async fn record_acquisition(
        &self,
        official_open_id: &str,
        require_first: bool,
    ) -> anyhow::Result<bool>;
}
