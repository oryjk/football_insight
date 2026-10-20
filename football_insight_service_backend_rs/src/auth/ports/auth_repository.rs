use async_trait::async_trait;
use uuid::Uuid;

use crate::auth::domain::{
    registration::{RegisteredUser, RegistrationIdentity, UserRegistration},
    user::{AuthUser, StoredAuthUser},
    wechat::{IssuedInviteCode, WechatOauthProfile},
};

#[async_trait]
pub trait AuthRepository: Send + Sync {
    /// Compatibility for existing repository implementations; persistence overrides this
    /// to distinguish actual new rows from reactivated accounts.
    async fn register_user(&self, request: UserRegistration<'_>) -> anyhow::Result<RegisteredUser> {
        let user = match (request.identity, request.invite_code) {
            (
                RegistrationIdentity::Password {
                    account_identifier,
                    password_hash,
                },
                Some(code),
            ) => {
                self.create_user_with_invite_with_referral(
                    code,
                    request.referral_code,
                    account_identifier,
                    password_hash,
                )
                .await?
            }
            (
                RegistrationIdentity::Password {
                    account_identifier,
                    password_hash,
                },
                None,
            ) => {
                self.create_user_without_invite_with_referral(
                    request.referral_code,
                    account_identifier,
                    password_hash,
                )
                .await?
            }
            (
                RegistrationIdentity::Wechat {
                    phone_number,
                    password_hash,
                    profile,
                },
                Some(code),
            ) => {
                self.create_user_with_invite_and_wechat_with_referral(
                    code,
                    request.referral_code,
                    phone_number,
                    password_hash,
                    profile,
                )
                .await?
            }
            (
                RegistrationIdentity::Wechat {
                    phone_number,
                    password_hash,
                    profile,
                },
                None,
            ) => {
                self.create_user_with_wechat_without_invite_with_referral(
                    request.referral_code,
                    phone_number,
                    password_hash,
                    profile,
                )
                .await?
            }
            (RegistrationIdentity::MiniWechat { profile }, Some(code)) => {
                self.create_user_with_invite_and_mini_program_wechat_with_referral(
                    code,
                    request.referral_code,
                    profile,
                )
                .await?
            }
            (RegistrationIdentity::MiniWechat { profile }, None) => {
                self.create_user_with_mini_program_wechat_without_invite_with_referral(
                    request.referral_code,
                    profile,
                )
                .await?
            }
        };
        Ok(RegisteredUser {
            user,
            newly_created: true,
        })
    }
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
    async fn create_user_without_invite(
        &self,
        account_identifier: &str,
        password_hash: &str,
    ) -> anyhow::Result<AuthUser> {
        self.create_user_without_invite_with_referral(None, account_identifier, password_hash)
            .await
    }
    async fn create_user_without_invite_with_referral(
        &self,
        referral_code: Option<&str>,
        account_identifier: &str,
        password_hash: &str,
    ) -> anyhow::Result<AuthUser> {
        let _ = referral_code;
        let _ = account_identifier;
        let _ = password_hash;
        anyhow::bail!("create user without invite is not supported")
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
    async fn create_user_with_wechat_without_invite(
        &self,
        phone_number: &str,
        password_hash: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        self.create_user_with_wechat_without_invite_with_referral(
            None,
            phone_number,
            password_hash,
            profile,
        )
        .await
    }
    async fn create_user_with_wechat_without_invite_with_referral(
        &self,
        referral_code: Option<&str>,
        phone_number: &str,
        password_hash: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        let _ = referral_code;
        let _ = phone_number;
        let _ = password_hash;
        let _ = profile;
        anyhow::bail!("create wechat user without invite is not supported")
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
    async fn create_user_with_mini_program_wechat_without_invite(
        &self,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        self.create_user_with_mini_program_wechat_without_invite_with_referral(None, profile)
            .await
    }
    async fn create_user_with_mini_program_wechat_without_invite_with_referral(
        &self,
        referral_code: Option<&str>,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        let _ = referral_code;
        let _ = profile;
        anyhow::bail!("create mini program wechat user without invite is not supported")
    }
    async fn get_user_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<AuthUser>>;
    async fn official_wechat_bound(&self, _user_id: Uuid) -> anyhow::Result<Option<bool>> {
        Ok(None)
    }
    async fn official_wechat_binding_required(
        &self,
        _user_id: Uuid,
    ) -> anyhow::Result<Option<bool>> {
        Ok(None)
    }
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
