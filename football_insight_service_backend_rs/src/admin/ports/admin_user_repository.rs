use async_trait::async_trait;
use uuid::Uuid;

use crate::admin::domain::admin_user::{
    AdminCreateUserInput, AdminUpdateUserInput, AdminUser, AdminUserList, AdminUserSearch,
};

#[async_trait]
pub trait AdminUserRepository: Send + Sync {
    async fn list_users(&self, search: AdminUserSearch) -> anyhow::Result<AdminUserList>;
    async fn create_user(
        &self,
        input: AdminCreateUserInput,
        password_hash: String,
    ) -> anyhow::Result<AdminUser>;
    async fn update_user(
        &self,
        user_id: Uuid,
        input: AdminUpdateUserInput,
    ) -> anyhow::Result<Option<AdminUser>>;
    async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<bool>;
}
