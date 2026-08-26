use async_trait::async_trait;
use uuid::Uuid;

use crate::auth::{
    adapters::persistence::postgres_auth_repository::PostgresAuthRepository,
    domain::user::AuthUser,
    ports::h5_test_login_port::H5TestLoginUserPort,
};

#[async_trait]
impl H5TestLoginUserPort for PostgresAuthRepository {
    async fn find_active_user_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<AuthUser>> {
        PostgresAuthRepository::find_active_user_by_id(self, user_id).await
    }
}
