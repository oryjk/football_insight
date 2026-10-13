use crate::{
    admin::{
        domain::admin_setup::{AdminSetupError, AdminSetupInput},
        ports::admin_setup_repository::AdminSetupRepository,
    },
    auth::ports::password_port::PasswordPort,
};
use std::sync::Arc;
pub struct AdminSetupService {
    repository: Arc<dyn AdminSetupRepository>,
    password_port: Arc<dyn PasswordPort>,
}
impl AdminSetupService {
    pub fn new(
        repository: Arc<dyn AdminSetupRepository>,
        password_port: Arc<dyn PasswordPort>,
    ) -> Self {
        Self {
            repository,
            password_port,
        }
    }
    pub async fn needs_setup(&self) -> Result<bool, AdminSetupError> {
        Ok(!self.repository.has_any_admin().await?)
    }
    pub async fn initialize(&self, input: AdminSetupInput) -> Result<(), AdminSetupError> {
        let input = input.validate()?;
        if self.repository.has_any_admin().await? {
            return Err(AdminSetupError::AlreadyInitialized);
        }
        let hash = self
            .password_port
            .hash_password(&input.password)
            .map_err(|_| {
                tracing::error!("admin initialization password hashing failed");
                AdminSetupError::Internal
            })?;
        self.repository.initialize_superadmin(hash).await
    }
}
