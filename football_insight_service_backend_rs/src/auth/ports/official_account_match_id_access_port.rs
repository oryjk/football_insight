use async_trait::async_trait;

use crate::auth::domain::official_account_match_id_access::OfficialAccountMatchIdAccess;

/// Read-only official-account identity and current-match entitlement lookup.
#[async_trait]
pub trait OfficialAccountMatchIdAccessPort: Send + Sync {
    async fn check(
        &self,
        official_open_id: &str,
        match_id: Option<i64>,
    ) -> anyhow::Result<OfficialAccountMatchIdAccess>;
}
