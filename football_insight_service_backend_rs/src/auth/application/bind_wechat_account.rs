use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::auth::{
    domain::{
        user::{AuthTokenBundle, normalize_password, validate_phone_number},
        wechat::WechatOauthProfile,
    },
    ports::{auth_repository::AuthRepository, password_port::PasswordPort, token_port::TokenPort},
};

#[derive(Debug, Clone)]
pub struct BindWechatAccountInput {
    pub bind_token: String,
    pub invite_code: Option<String>,
    pub referral_code: Option<String>,
    pub phone_number: String,
    pub password: String,
}

pub struct BindWechatAccountUseCase {
    repository: Arc<dyn AuthRepository>,
    password_port: Arc<dyn PasswordPort>,
    token_port: Arc<dyn TokenPort>,
    session_ttl: Duration,
}

impl BindWechatAccountUseCase {
    pub fn new(
        repository: Arc<dyn AuthRepository>,
        password_port: Arc<dyn PasswordPort>,
        token_port: Arc<dyn TokenPort>,
        session_ttl: Duration,
    ) -> Self {
        Self {
            repository,
            password_port,
            token_port,
            session_ttl,
        }
    }

    pub async fn execute(&self, input: BindWechatAccountInput) -> anyhow::Result<AuthTokenBundle> {
        if input.bind_token.trim().is_empty() {
            anyhow::bail!("bind token is required");
        }

        let phone_number = validate_phone_number(&input.phone_number)?;
        let password = normalize_password(&input.password);
        if password.len() < 6 {
            anyhow::bail!("password must be at least 6 characters");
        }

        let claims = self
            .token_port
            .verify_wechat_bind_token(&input.bind_token)?;
        let profile = WechatOauthProfile {
            open_id: claims.open_id,
            union_id: claims.union_id,
            display_name: claims.display_name,
            avatar_url: claims.avatar_url,
        };

        let user = if let Some(stored_user) = self
            .repository
            .find_user_by_account_identifier(&phone_number)
            .await?
        {
            let is_valid = self
                .password_port
                .verify_password(&password, &stored_user.password_hash)?;

            if !is_valid {
                anyhow::bail!("invalid phone number or password");
            }

            self.repository
                .bind_wechat_to_user(stored_user.user.id, &profile)
                .await?
        } else {
            let invite_code = input
                .invite_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow::anyhow!("invite code is required"))?;
            let referral_code = input
                .referral_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let password_hash = self.password_port.hash_password(&password)?;

            self.repository
                .create_user_with_invite_and_wechat_with_referral(
                    invite_code,
                    referral_code,
                    &phone_number,
                    &password_hash,
                    &profile,
                )
                .await?
        };

        self.repository.record_user_login(user.id).await?;

        let expires_at = Utc::now() + self.session_ttl;
        let access_token =
            self.token_port
                .issue_token(user.id, &user.account_identifier, expires_at)?;

        Ok(AuthTokenBundle {
            user,
            access_token,
            expires_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::auth::{
        application::bind_wechat_account::{BindWechatAccountInput, BindWechatAccountUseCase},
        domain::{
            user::{AuthUser, StoredAuthUser},
            wechat::{IssuedInviteCode, WechatOauthProfile},
        },
        ports::{
            auth_repository::AuthRepository,
            password_port::PasswordPort,
            token_port::{
                AuthTokenClaims, TokenPort, WechatBindTokenClaims, WechatBindTokenPayload,
            },
        },
    };

    struct FakeRepository;

    #[async_trait]
    impl AuthRepository for FakeRepository {
        async fn create_user_with_invite(
            &self,
            _invite_code: &str,
            _account_identifier: &str,
            _password_hash: &str,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn find_user_by_account_identifier(
            &self,
            account_identifier: &str,
        ) -> anyhow::Result<Option<StoredAuthUser>> {
            Ok(Some(StoredAuthUser {
                user: AuthUser {
                    id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                    account_identifier: account_identifier.to_string(),
                    display_name: Some(account_identifier.to_string()),
                    invite_code: None,
                    avatar_url: None,
                    has_wechat_binding: false,
                    membership_tier: "V1".to_string(),
                    membership_expires_at: None,
                    membership_benefits_enabled: true,
                    ticket_watch_poll_interval_seconds: 600,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                password_hash: "hashed::secret123".to_string(),
            }))
        }

        async fn get_user_by_id(&self, _user_id: Uuid) -> anyhow::Result<Option<AuthUser>> {
            unreachable!()
        }

        async fn find_user_by_wechat_open_id(
            &self,
            _open_id: &str,
        ) -> anyhow::Result<Option<AuthUser>> {
            unreachable!()
        }

        async fn bind_wechat_to_user(
            &self,
            user_id: Uuid,
            _profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            Ok(AuthUser {
                id: user_id,
                account_identifier: "18602812970".to_string(),
                display_name: Some("18602812970".to_string()),
                invite_code: None,
                avatar_url: None,
                has_wechat_binding: true,
                membership_tier: "V1".to_string(),
                membership_expires_at: None,
                membership_benefits_enabled: true,
                ticket_watch_poll_interval_seconds: 600,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        async fn create_user_with_invite_and_wechat(
            &self,
            _invite_code: &str,
            _phone_number: &str,
            _password_hash: &str,
            _profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn create_user_with_invite_and_mini_program_wechat(
            &self,
            _invite_code: &str,
            _profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn issue_invite_code_for_wechat_follower(
            &self,
            _open_id: &str,
        ) -> anyhow::Result<IssuedInviteCode> {
            unreachable!()
        }

        async fn mark_wechat_follower_unsubscribed(&self, _open_id: &str) -> anyhow::Result<()> {
            unreachable!()
        }

        async fn reset_password_with_invite(
            &self,
            _invite_code: &str,
            _account_identifier: &str,
            _password_hash: &str,
        ) -> anyhow::Result<()> {
            unreachable!()
        }
    }

    struct FakePasswordPort;

    impl PasswordPort for FakePasswordPort {
        fn hash_password(&self, _password: &str) -> anyhow::Result<String> {
            unreachable!()
        }

        fn verify_password(&self, password: &str, password_hash: &str) -> anyhow::Result<bool> {
            Ok(format!("hashed::{password}") == password_hash)
        }
    }

    struct FakeTokenPort;

    impl TokenPort for FakeTokenPort {
        fn issue_token(
            &self,
            _user_id: Uuid,
            account_identifier: &str,
            _expires_at: chrono::DateTime<Utc>,
        ) -> anyhow::Result<String> {
            Ok(format!("jwt::{account_identifier}"))
        }

        fn verify_token(&self, _token: &str) -> anyhow::Result<AuthTokenClaims> {
            unreachable!()
        }

        fn issue_wechat_bind_token(
            &self,
            _payload: WechatBindTokenPayload,
            _expires_at: chrono::DateTime<Utc>,
        ) -> anyhow::Result<String> {
            unreachable!()
        }

        fn verify_wechat_bind_token(&self, _token: &str) -> anyhow::Result<WechatBindTokenClaims> {
            Ok(WechatBindTokenClaims {
                open_id: "openid".to_string(),
                union_id: None,
                display_name: Some("tester".to_string()),
                avatar_url: None,
                exp: Utc::now() + Duration::minutes(10),
            })
        }
    }

    #[tokio::test]
    async fn execute_trims_password_before_verifying_existing_user() {
        let use_case = BindWechatAccountUseCase::new(
            Arc::new(FakeRepository),
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(BindWechatAccountInput {
                bind_token: "bind-token".to_string(),
                invite_code: None,
                referral_code: None,
                phone_number: "18602812970".to_string(),
                password: "  secret123  ".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.account_identifier, "18602812970");
        assert!(result.user.has_wechat_binding);
    }
}
