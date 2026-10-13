use crate::admin::domain::admin_setup::AdminSetupError;
use async_trait::async_trait;
#[async_trait]
pub trait AdminSetupRepository: Send + Sync {
    async fn has_any_admin(&self) -> Result<bool, AdminSetupError>;
    /// Serialize with all bootstrap paths; initialize once, then refuse even disabled accounts.
    async fn initialize_superadmin(&self, password_hash: String) -> Result<(), AdminSetupError>;
}
