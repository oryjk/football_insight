use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::admin::domain::admin_auth::AdminTokenClaims;

pub trait AdminTokenPort: Send + Sync {
    fn issue_token(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
        username: &str,
        role: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<String>;

    fn verify_token(&self, token: &str) -> anyhow::Result<AdminTokenClaims>;
}
