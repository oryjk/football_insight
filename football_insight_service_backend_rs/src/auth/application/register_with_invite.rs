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
    acquisition_recorder: Option<crate::auth::application::record_acquisition::RecordAcquisition>,
}

impl RegisterWithInviteUseCase {
    pub fn with_acquisition_recorder(
        mut self,
        recorder: crate::auth::application::record_acquisition::RecordAcquisition,
    ) -> Self {
        self.acquisition_recorder = Some(recorder);
        self
    }

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
            acquisition_recorder: None,
        }
    }

    pub async fn execute(&self, input: RegisterInput) -> anyhow::Result<AuthTokenBundle> {
        self.execute_with_acquisition(input, None).await
    }

    pub async fn execute_with_acquisition(
        &self,
        input: RegisterInput,
        acquisition: Option<crate::auth::domain::acquisition::AcquisitionSource>,
    ) -> anyhow::Result<AuthTokenBundle> {
        let invite_code = input.invite_code.trim().to_string();
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
        let registration = self
            .repository
            .register_user(crate::auth::domain::registration::UserRegistration {
                invite_code: (!invite_code.is_empty()).then_some(invite_code.as_str()),
                referral_code: referral_code.as_deref(),
                identity: crate::auth::domain::registration::RegistrationIdentity::Password {
                    account_identifier: &account_identifier,
                    password_hash: &password_hash,
                },
            })
            .await?;
        let user = registration.user;
        if registration.newly_created
            && let Some(recorder) = &self.acquisition_recorder
        {
            recorder.record(user.id, acquisition);
        }

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
    #[tokio::test]
    async fn acquisition_does_not_attribute_reactivated_accounts_or_failed_registration() {
        use crate::auth::{
            application::{
                acquisition_test_support::Sources, record_acquisition::RecordAcquisition,
            },
            domain::acquisition::AcquisitionSource,
        };
        for invite in ["invite", ""] {
            let sources = Arc::new(Sources::default());
            let repository = Arc::new(FakeRepository {
                reactivated: true,
                ..Default::default()
            });
            let uc = RegisterWithInviteUseCase::new(
                repository,
                Arc::new(FakePasswordPort),
                Arc::new(FakeTokenPort),
                Duration::days(30),
            )
            .with_acquisition_recorder(RecordAcquisition::new(sources.clone()));
            let mut input = RegisterInput {
                invite_code: invite.into(),
                referral_code: None,
                account_identifier: "tester".into(),
                password: "secret123".into(),
            };
            let response = uc
                .execute_with_acquisition(
                    input.clone(),
                    AcquisitionSource::new(Some("douyin"), None, None),
                )
                .await
                .unwrap();
            assert!(!response.access_token.is_empty());
            tokio::task::yield_now().await;
            assert!(sources.0.lock().unwrap().is_empty());
            input.password = "x".into();
            assert!(uc
                .execute_with_acquisition(input, AcquisitionSource::new(Some("xhs"), None, None))
                .await
                .is_err());
            tokio::task::yield_now().await;
            assert!(sources.0.lock().unwrap().is_empty());
        }
    }

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

    #[tokio::test]
    async fn acquisition_registration_keeps_auth_result_when_source_storage_fails() {
        use crate::auth::{
            application::record_acquisition::RecordAcquisition,
            domain::acquisition::AcquisitionSource,
            ports::acquisition_repository::AcquisitionRepository,
        };
        struct FailingSource(std::sync::atomic::AtomicUsize);
        #[async_trait]
        impl AcquisitionRepository for FailingSource {
            async fn insert_first(
                &self,
                user_id: Uuid,
                source: AcquisitionSource,
            ) -> anyhow::Result<()> {
                assert_eq!(
                    user_id,
                    Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
                );
                assert_eq!(source.src, "douyin");
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                anyhow::bail!("table missing")
            }
        }
        let source = Arc::new(FailingSource(std::sync::atomic::AtomicUsize::new(0)));
        let use_case = RegisterWithInviteUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        )
        .with_acquisition_recorder(RecordAcquisition::new(source.clone()));
        let response = use_case
            .execute_with_acquisition(
                RegisterInput {
                    invite_code: "invite".into(),
                    referral_code: None,
                    account_identifier: "tester".into(),
                    password: "secret123".into(),
                },
                AcquisitionSource::new(Some("douyin"), Some("test1"), Some(1047)),
            )
            .await
            .unwrap();
        assert_eq!(response.user.account_identifier, "tester");
        assert!(!response.access_token.is_empty());
        tokio::task::yield_now().await;
        assert_eq!(source.0.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[derive(Default)]
    struct FakeRepository {
        events: Mutex<Vec<String>>,
        reactivated: bool,
    }

    #[async_trait]
    impl AuthRepository for FakeRepository {
        async fn register_user(
            &self,
            request: crate::auth::domain::registration::UserRegistration<'_>,
        ) -> anyhow::Result<crate::auth::domain::registration::RegisteredUser> {
            let crate::auth::domain::registration::RegistrationIdentity::Password {
                account_identifier,
                password_hash,
            } = request.identity
            else {
                unreachable!()
            };
            let user = if let Some(code) = request.invite_code {
                self.create_user_with_invite_with_referral(
                    code,
                    request.referral_code,
                    account_identifier,
                    password_hash,
                )
                .await?
            } else {
                self.create_user_without_invite_with_referral(
                    request.referral_code,
                    account_identifier,
                    password_hash,
                )
                .await?
            };
            Ok(crate::auth::domain::registration::RegisteredUser {
                user,
                newly_created: !self.reactivated,
            })
        }
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
            self.events.lock().unwrap().push(format!(
                "register-without-invite:{}:{account_identifier}:{password_hash}",
                referral_code.unwrap_or("")
            ));

            Ok(AuthUser {
                id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
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

    #[tokio::test]
    async fn execute_registers_v1_user_when_invite_code_is_blank() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = RegisterWithInviteUseCase::new(
            repository.clone(),
            Arc::new(FakePasswordPort),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(RegisterInput {
                invite_code: "   ".to_string(),
                referral_code: Some("  REF-CODE-001  ".to_string()),
                account_identifier: "footballfan".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.user.account_identifier, "footballfan");
        assert_eq!(result.user.membership_tier, "V1");
        assert_eq!(result.access_token, "jwt::footballfan");

        let events = repository.events.lock().unwrap().clone();
        assert_eq!(
            events,
            vec!["register-without-invite:REF-CODE-001:footballfan:hashed::secret123".to_string()]
        );
    }
}
