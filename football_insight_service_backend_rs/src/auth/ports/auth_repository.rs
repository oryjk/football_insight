use async_trait::async_trait;
use uuid::Uuid;

use crate::auth::domain::{
    user::{AuthUser, StoredAuthUser},
    wechat::{IssuedInviteCode, WechatOauthProfile},
};

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn create_user_with_invite(
        &self,
        invite_code: &str,
        account_identifier: &str,
        password_hash: &str,
    ) -> anyhow::Result<AuthUser>;
    async fn create_user_with_invite_with_referral(
        &self,
        invite_code: &str,
        referral_code: Option<&str>,
        account_identifier: &str,
        password_hash: &str,
    ) -> anyhow::Result<AuthUser> {
        let _ = referral_code;
        self.create_user_with_invite(invite_code, account_identifier, password_hash)
            .await
    }
    async fn find_user_by_account_identifier(
        &self,
        account_identifier: &str,
    ) -> anyhow::Result<Option<StoredAuthUser>>;
    async fn find_user_by_wechat_open_id(&self, open_id: &str) -> anyhow::Result<Option<AuthUser>>;
    async fn bind_wechat_to_user(
        &self,
        user_id: Uuid,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser>;
    async fn create_user_with_invite_and_wechat(
        &self,
        invite_code: &str,
        phone_number: &str,
        password_hash: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser>;
    async fn create_user_with_invite_and_wechat_with_referral(
        &self,
        invite_code: &str,
        referral_code: Option<&str>,
        phone_number: &str,
        password_hash: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        let _ = referral_code;
        self.create_user_with_invite_and_wechat(invite_code, phone_number, password_hash, profile)
            .await
    }
    async fn create_user_with_invite_and_mini_program_wechat(
        &self,
        invite_code: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser>;
    async fn create_user_with_invite_and_mini_program_wechat_with_referral(
        &self,
        invite_code: &str,
        referral_code: Option<&str>,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        let _ = referral_code;
        self.create_user_with_invite_and_mini_program_wechat(invite_code, profile)
            .await
    }
    async fn get_user_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<AuthUser>>;
    async fn record_user_login(&self, user_id: Uuid) -> anyhow::Result<()> {
        let _ = user_id;
        Ok(())
    }
    async fn issue_invite_code_for_wechat_follower(
        &self,
        open_id: &str,
    ) -> anyhow::Result<IssuedInviteCode>;
    async fn mark_wechat_follower_unsubscribed(&self, open_id: &str) -> anyhow::Result<()>;
    async fn reset_password_with_invite(
        &self,
        invite_code: &str,
        account_identifier: &str,
        password_hash: &str,
    ) -> anyhow::Result<()>;
}
