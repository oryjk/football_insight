use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::admin::domain::admin_auth::{AdminAccount, AdminBootstrapOwner, AdminSession};

#[async_trait]
pub trait AdminAuthRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<AdminAccount>>;

    async fn create_session(&self, session: AdminSession) -> anyhow::Result<()>;
    /// Adapters must serialize this check/insertion with credentials changes. Default supports in-memory test adapters.
    async fn create_session_if_current(
        &self,
        session: AdminSession,
        expected_username: &str,
        expected_password_hash: &str,
    ) -> anyhow::Result<bool> {
        let Some(account) = self.find_by_username(expected_username).await? else {
            return Ok(false);
        };
        if account.id != session.admin_user_id
            || account.password_hash != expected_password_hash
            || account.status != "active"
        {
            return Ok(false);
        }
        self.create_session(session).await?;
        Ok(true)
    }

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
