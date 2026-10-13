use crate::admin::domain::admin_credentials::{
    AdminCredentialsChange, AdminCredentialsError, AdminCredentialsSnapshot,
};
use async_trait::async_trait;
use uuid::Uuid;
#[async_trait]
pub trait AdminCredentialsRepository: Send + Sync {
    async fn load_for_session(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
    ) -> Result<AdminCredentialsSnapshot, AdminCredentialsError>;
    /// Compare snapshot/session under account row lock, mutate, revoke all sessions and audit atomically.
    async fn change_with_audit(
        &self,
        change: AdminCredentialsChange,
    ) -> Result<(), AdminCredentialsError>;
}
