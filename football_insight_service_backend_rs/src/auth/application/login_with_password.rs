use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::auth::{
    domain::user::{AuthTokenBundle, normalize_password, validate_account_identifier},
    ports::{auth_repository::AuthRepository, password_port::PasswordPort, token_port::TokenPort},
};

#[derive(Debug, Clone)]
pub struct LoginInput {
    pub account_identifier: String,
    pub password: String,
}

pub struct LoginWithPasswordUseCase {
    repository: Arc<dyn AuthRepository>,
    password_port: Arc<dyn PasswordPort>,
    token_port: Arc<dyn TokenPort>,
    session_ttl: Duration,
}

impl LoginWithPasswordUseCase {
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

    pub async fn execute(&self, input: LoginInput) -> anyhow::Result<AuthTokenBundle> {
        let account_identifier = validate_account_identifier(&input.account_identifier)?;
        let password = normalize_password(&input.password);
        if password.len() < 6 {
            anyhow::bail!("password must be at least 6 characters");
        }
        let stored_user = self
            .repository
            .find_user_by_account_identifier(&account_identifier)
            .await?
            .ok_or_else(|| anyhow::anyhow!("invalid account identifier or password"))?;

        let is_valid = self
            .password_port
            .verify_password(&password, &stored_user.password_hash)?;

        if !is_valid {
            anyhow::bail!("invalid account identifier or password");
        }

        self.repository
            .record_user_login(stored_user.user.id)
            .await?;

        let expires_at = Utc::now() + self.session_ttl;
        let access_token = self.token_port.issue_token(
            stored_user.user.id,
            &stored_user.user.account_identifier,
            expires_at,
        )?;

        Ok(AuthTokenBundle {
            user: stored_user.user,
            access_token,
            expires_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::auth::{
        application::login_with_password::{LoginInput, LoginWithPasswordUseCase},
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
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    #[derive(Default)]
    struct FakeRepository {
        events: Mutex<Vec<String>>,
    }

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

        async fn get_user_by_id(&self, _user_id: uuid::Uuid) -> anyhow::Result<Option<AuthUser>> {
            unreachable!()
        }

        async fn record_user_login(&self, user_id: uuid::Uuid) -> anyhow::Result<()> {
            self.events
                .lock()
                .unwrap()
                .push(format!("record_login:{user_id}"));
            Ok(())
        }

        async fn find_user_by_wechat_open_id(
            &self,
            _open_id: &str,
        ) -> anyhow::Result<Option<AuthUser>> {
            unreachable!()
        }

        async fn bind_wechat_to_user(
            &self,
            _user_id: uuid::Uuid,
            _profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
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
            _user_id: uuid::Uuid,
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
            unreachable!()
        }
    }

    #[tokio::test]
    async fn execute_logs_in_existing_user_and_creates_session() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = LoginWithPasswordUseCase::new(
            repository.clone(),
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(LoginInput {
                account_identifier: "13800138000".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.account_identifier, "13800138000");
        assert_eq!(result.access_token, "jwt::13800138000");
        assert_eq!(
            repository.events.lock().unwrap().as_slice(),
            ["record_login:bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"]
        );
    }

    #[tokio::test]
    async fn execute_accepts_username_longer_than_five_characters() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = LoginWithPasswordUseCase::new(
            repository.clone(),
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(LoginInput {
                account_identifier: "footballfan".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.account_identifier, "footballfan");
        assert_eq!(result.access_token, "jwt::footballfan");
        assert_eq!(
            repository.events.lock().unwrap().as_slice(),
            ["record_login:bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"]
        );
    }

    #[tokio::test]
    async fn execute_rejects_invalid_password() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = LoginWithPasswordUseCase::new(
            repository,
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let error = use_case
            .execute(LoginInput {
                account_identifier: "13800138000".to_string(),
                password: "wrong123".to_string(),
            })
            .await
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("invalid account identifier or password")
        );
    }

    #[tokio::test]
    async fn execute_trims_password_before_verification() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = LoginWithPasswordUseCase::new(
            repository,
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(LoginInput {
                account_identifier: "13800138000".to_string(),
                password: "  secret123  ".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.account_identifier, "13800138000");
    }
}
