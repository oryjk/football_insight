use async_trait::async_trait;
use uuid::Uuid;

use crate::admin::domain::admin_user::{
    AdminCreateUserInput, AdminMembershipAdjustment, AdminUpdateUserInput, AdminUser,
    AdminUserDetail, AdminUserList, AdminUserSearch,
};

#[async_trait]
pub trait AdminUserRepository: Send + Sync {
    async fn list_users(&self, search: AdminUserSearch) -> anyhow::Result<AdminUserList>;
    async fn get_user(&self, user_id: Uuid) -> anyhow::Result<Option<AdminUser>>;
    async fn get_user_detail(&self, user_id: Uuid) -> anyhow::Result<Option<AdminUserDetail>>;
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
    async fn set_user_status(
        &self,
        user_id: Uuid,
        status: &str,
        admin_id: Uuid,
        reason: &str,
    ) -> anyhow::Result<Option<AdminUser>>;
    async fn adjust_membership(
        &self,
        user_id: Uuid,
        adjustment: AdminMembershipAdjustment,
        admin_id: Uuid,
    ) -> anyhow::Result<Option<AdminUser>>;
}
