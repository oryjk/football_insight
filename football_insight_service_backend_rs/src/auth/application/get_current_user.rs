use std::sync::Arc;

use crate::auth::{
    domain::user::AuthUser,
    ports::{auth_repository::AuthRepository, token_port::TokenPort},
};

pub struct GetCurrentUserUseCase {
    repository: Arc<dyn AuthRepository>,
    token_port: Arc<dyn TokenPort>,
}

impl GetCurrentUserUseCase {
    pub fn new(repository: Arc<dyn AuthRepository>, token_port: Arc<dyn TokenPort>) -> Self {
        Self {
            repository,
            token_port,
        }
    }

    pub async fn execute(&self, access_token: &str) -> anyhow::Result<Option<AuthUser>> {
        let claims = self.token_port.verify_token(access_token)?;
        let user = self.repository.get_user_by_id(claims.sub).await?;
        if user.is_some() {
            self.repository.record_user_login(claims.sub).await?;
        }

        Ok(user)
    }
}
