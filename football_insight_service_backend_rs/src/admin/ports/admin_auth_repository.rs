use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::admin::domain::admin_auth::{AdminAccount, AdminBootstrapOwner, AdminSession};

#[async_trait]
pub trait AdminAuthRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<AdminAccount>>;

    async fn create_session(&self, session: AdminSession) -> anyhow::Result<()>;

    async fn find_active_account_for_session(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<AdminAccount>>;

    async fn revoke_session(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
        revoked_at: DateTime<Utc>,
    ) -> anyhow::Result<bool>;

    async fn ensure_owner(&self, owner: AdminBootstrapOwner) -> anyhow::Result<AdminAccount> {
        let _ = owner;
        anyhow::bail!("admin owner bootstrap is not implemented")
    }
}
