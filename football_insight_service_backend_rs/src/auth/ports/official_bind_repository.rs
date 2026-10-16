use crate::auth::domain::official_bind::{ConsumeBindCodeOutcome, IssueBindCodeOutcome};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait OfficialBindRepository: Send + Sync {
    async fn issue(
        &self,
        user_id: Uuid,
        code_hash: &str,
        now: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<IssueBindCodeOutcome>;
    /// Both code consumption and the active-account binding commit atomically.
    async fn consume(
        &self,
        code_hash: &str,
        verified_official_open_id: &str,
    ) -> anyhow::Result<ConsumeBindCodeOutcome>;
}
