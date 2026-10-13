use crate::{
    admin::{
        domain::{
            admin_auth::AdminPrincipal,
            admin_credentials::{
                AdminCredentialsChange, AdminCredentialsError, AdminCredentialsInput,
            },
        },
        ports::admin_credentials_repository::AdminCredentialsRepository,
    },
    auth::ports::password_port::PasswordPort,
};
use chrono::Utc;
use std::sync::Arc;
pub struct AdminCredentialsService {
    repository: Arc<dyn AdminCredentialsRepository>,
    password_port: Arc<dyn PasswordPort>,
}
impl AdminCredentialsService {
    pub fn new(
        repository: Arc<dyn AdminCredentialsRepository>,
        password_port: Arc<dyn PasswordPort>,
    ) -> Self {
        Self {
            repository,
            password_port,
        }
    }
    pub async fn change(
        &self,
        principal: AdminPrincipal,
        input: AdminCredentialsInput,
    ) -> Result<(), AdminCredentialsError> {
        if principal.expires_at <= Utc::now() {
            return Err(AdminCredentialsError::Unauthorized);
        }
        let input = input.validate()?;
        let snapshot = self
            .repository
            .load_for_session(principal.admin_id, principal.session_id)
            .await?;
        let verified = self
            .password_port
            .verify_password(&input.current_password, &snapshot.password_hash)
            .map_err(|_| {
                tracing::error!("admin credentials password verification failed");
                AdminCredentialsError::Internal
            })?;
        if !verified {
            return Err(AdminCredentialsError::WrongPassword);
        }
        let new_password = input
            .new_password
            .filter(|value| value != &input.current_password);
        if input.username == snapshot.username && new_password.is_none() {
            return Err(AdminCredentialsError::InvalidInput(
                "no credential changes requested",
            ));
        }
        let password_hash = new_password
            .as_ref()
            .map(|password| self.password_port.hash_password(password))
            .transpose()
            .map_err(|_| {
                tracing::error!("admin credentials password hashing failed");
                AdminCredentialsError::Internal
            })?;
        self.repository
            .change_with_audit(AdminCredentialsChange {
                admin_id: principal.admin_id,
                session_id: principal.session_id,
                expected_username: snapshot.username,
                expected_password_hash: snapshot.password_hash,
                username: input.username,
                password_hash,
            })
            .await
    }
}
