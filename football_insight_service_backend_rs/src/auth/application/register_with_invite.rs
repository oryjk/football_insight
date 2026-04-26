use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::auth::{
    domain::user::{AuthTokenBundle, normalize_password, validate_account_identifier},
    ports::{auth_repository::AuthRepository, password_port::PasswordPort, token_port::TokenPort},
};

#[derive(Debug, Clone)]
pub struct RegisterInput {
    pub invite_code: String,
    pub referral_code: Option<String>,
    pub account_identifier: String,
    pub password: String,
}

pub struct RegisterWithInviteUseCase {
    repository: Arc<dyn AuthRepository>,
    password_port: Arc<dyn PasswordPort>,
    token_port: Arc<dyn TokenPort>,
    session_ttl: Duration,
}

impl RegisterWithInviteUseCase {
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

    pub async fn execute(&self, input: RegisterInput) -> anyhow::Result<AuthTokenBundle> {
        let invite_code = input.invite_code.trim().to_string();
        if invite_code.is_empty() {
            anyhow::bail!("invite code is required");
        }
        let referral_code = input
            .referral_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        let account_identifier = validate_account_identifier(&input.account_identifier)?;
        let password = normalize_password(&input.password);
        if password.len() < 6 {
            anyhow::bail!("password must be at least 6 characters");
        }

        let password_hash = self.password_port.hash_password(&password)?;
        let user = self
            .repository
            .create_user_with_invite_with_referral(
                &invite_code,
                referral_code.as_deref(),
                &account_identifier,
                &password_hash,
            )
            .await?;

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
    use std::sync::{Arc, Mutex};

    use crate::auth::{
        application::register_with_invite::{RegisterInput, RegisterWithInviteUseCase},
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
            invite_code: &str,
            account_identifier: &str,
            password_hash: &str,
        ) -> anyhow::Result<AuthUser> {
            self.events.lock().unwrap().push(format!(
                "register:{invite_code}:{account_identifier}:{password_hash}"
            ));

            Ok(AuthUser {
                id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                account_identifier: account_identifier.to_string(),
                display_name: Some(account_identifier.to_string()),
                invite_code: None,
                avatar_url: None,
                has_wechat_binding: false,
                membership_tier: "V3".to_string(),
                membership_expires_at: None,
                membership_benefits_enabled: true,
                ticket_watch_poll_interval_seconds: 300,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        async fn create_user_with_invite_with_referral(
            &self,
            invite_code: &str,
            referral_code: Option<&str>,
            account_identifier: &str,
            password_hash: &str,
        ) -> anyhow::Result<AuthUser> {
            self.events.lock().unwrap().push(format!(
                "register:{invite_code}:{}:{account_identifier}:{password_hash}",
                referral_code.unwrap_or("")
            ));

            Ok(AuthUser {
                id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                account_identifier: account_identifier.to_string(),
                display_name: Some(account_identifier.to_string()),
                invite_code: None,
                avatar_url: None,
                has_wechat_binding: false,
                membership_tier: "V3".to_string(),
                membership_expires_at: None,
                membership_benefits_enabled: true,
                ticket_watch_poll_interval_seconds: 300,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        async fn find_user_by_account_identifier(
            &self,
            _account_identifier: &str,
        ) -> anyhow::Result<Option<StoredAuthUser>> {
            Ok(None)
        }

        async fn get_user_by_id(&self, _user_id: uuid::Uuid) -> anyhow::Result<Option<AuthUser>> {
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
        fn hash_password(&self, password: &str) -> anyhow::Result<String> {
            Ok(format!("hashed::{password}"))
        }

        fn verify_password(&self, _password: &str, _password_hash: &str) -> anyhow::Result<bool> {
            unreachable!()
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
    async fn execute_registers_user_and_creates_session() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = RegisterWithInviteUseCase::new(
            repository.clone(),
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(RegisterInput {
                invite_code: "INVITE-001".to_string(),
                referral_code: None,
                account_identifier: "13800138000".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.account_identifier, "13800138000");
        assert_eq!(result.access_token, "jwt::13800138000");

        let events = repository.events.lock().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            "register:INVITE-001::13800138000:hashed::secret123"
        );
    }

    #[tokio::test]
    async fn execute_accepts_username_longer_than_five_characters() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = RegisterWithInviteUseCase::new(
            repository.clone(),
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(RegisterInput {
                invite_code: "INVITE-001".to_string(),
                referral_code: None,
                account_identifier: "footballfan".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.account_identifier, "footballfan");
        assert_eq!(result.access_token, "jwt::footballfan");

        let events = repository.events.lock().unwrap().clone();
        assert_eq!(
            events[0],
            "register:INVITE-001::footballfan:hashed::secret123"
        );
    }

    #[tokio::test]
    async fn execute_trims_password_before_hashing() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = RegisterWithInviteUseCase::new(
            repository.clone(),
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(RegisterInput {
                invite_code: "INVITE-001".to_string(),
                referral_code: None,
                account_identifier: "13800138000".to_string(),
                password: "  secret123  ".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.account_identifier, "13800138000");

        let events = repository.events.lock().unwrap().clone();
        assert_eq!(
            events[0],
            "register:INVITE-001::13800138000:hashed::secret123"
        );
    }

    #[tokio::test]
    async fn execute_trims_and_passes_referral_code() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = RegisterWithInviteUseCase::new(
            repository.clone(),
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(RegisterInput {
                invite_code: "INVITE-001".to_string(),
                referral_code: Some("  REF-CODE-001  ".to_string()),
                account_identifier: "13800138000".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.account_identifier, "13800138000");

        let events = repository.events.lock().unwrap().clone();
        assert_eq!(
            events[0],
            "register:INVITE-001:REF-CODE-001:13800138000:hashed::secret123"
        );
    }
}
