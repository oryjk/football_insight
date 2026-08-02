use std::sync::Arc;

use uuid::Uuid;

use crate::{
    admin::{
        domain::admin_user::{
            AdminCreateUserInput, AdminMembershipAdjustment, AdminUpdateUserInput, AdminUser,
            AdminUserDetail, AdminUserList, AdminUserSearch, validate_admin_password,
            validate_admin_reason, validate_display_name, validate_membership_tier,
        },
        ports::admin_user_repository::AdminUserRepository,
    },
    auth::ports::password_port::PasswordPort,
};

pub struct AdminUserService {
    repository: Arc<dyn AdminUserRepository>,
    password_port: Arc<dyn PasswordPort>,
}

impl AdminUserService {
    pub fn new(
        repository: Arc<dyn AdminUserRepository>,
        password_port: Arc<dyn PasswordPort>,
    ) -> Self {
        Self {
            repository,
            password_port,
        }
    }

    pub async fn list_users(&self, search: AdminUserSearch) -> anyhow::Result<AdminUserList> {
        self.repository.list_users(search).await
    }

    pub async fn get_user(&self, user_id: Uuid) -> anyhow::Result<Option<AdminUser>> {
        self.repository.get_user(user_id).await
    }

    pub async fn get_user_detail(&self, user_id: Uuid) -> anyhow::Result<Option<AdminUserDetail>> {
        self.repository.get_user_detail(user_id).await
    }

    pub async fn create_user(&self, mut input: AdminCreateUserInput) -> anyhow::Result<AdminUser> {
        input.display_name = validate_display_name(&input.display_name)?;
        input.password = validate_admin_password(&input.password)?;
        input.membership_tier = validate_membership_tier(&input.membership_tier)?;

        let password_hash = self.password_port.hash_password(&input.password)?;
        self.repository.create_user(input, password_hash).await
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        mut input: AdminUpdateUserInput,
    ) -> anyhow::Result<Option<AdminUser>> {
        if let Some(display_name) = input.display_name.take() {
            input.display_name = Some(validate_display_name(&display_name)?);
        }
        if let Some(membership_tier) = input.membership_tier.take() {
            input.membership_tier = Some(validate_membership_tier(&membership_tier)?);
        }

        self.repository.update_user(user_id, input).await
    }

    pub async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<bool> {
        self.repository.delete_user(user_id).await
    }

    pub async fn set_user_status(
        &self,
        user_id: Uuid,
        status: &str,
        admin_id: Uuid,
        reason: String,
    ) -> anyhow::Result<Option<AdminUser>> {
        if !matches!(status, "active" | "disabled") {
            anyhow::bail!("invalid user status");
        }
        let reason = validate_admin_reason(&reason)?;
        self.repository
            .set_user_status(user_id, status, admin_id, &reason)
            .await
    }

    pub async fn adjust_membership(
        &self,
        user_id: Uuid,
        mut adjustment: AdminMembershipAdjustment,
        admin_id: Uuid,
    ) -> anyhow::Result<Option<AdminUser>> {
        adjustment.membership_tier = validate_membership_tier(&adjustment.membership_tier)?;
        adjustment.reason = validate_admin_reason(&adjustment.reason)?;
        self.repository
            .adjust_membership(user_id, adjustment, admin_id)
            .await
    }
}
