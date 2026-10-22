use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::auth::{
    domain::{user::AuthTokenBundle, wechat::WechatOauthProfile},
    ports::{auth_repository::AuthRepository, token_port::TokenPort},
};

#[derive(Debug, Clone)]
pub struct BindWechatMiniProgramAccountInput {
    pub bind_token: String,
    pub invite_code: String,
    pub referral_code: Option<String>,
    pub display_name: String,
    pub avatar_data_url: String,
}

pub struct BindWechatMiniProgramAccountUseCase {
    repository: Arc<dyn AuthRepository>,
    token_port: Arc<dyn TokenPort>,
    session_ttl: Duration,
    acquisition_recorder: Option<crate::auth::application::record_acquisition::RecordAcquisition>,
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
            let uc = BindWechatMiniProgramAccountUseCase::new(
                repository,
                Arc::new(FakeTokenPort),
                Duration::days(30),
            )
            .with_acquisition_recorder(RecordAcquisition::new(sources.clone()));
            let mut input = BindWechatMiniProgramAccountInput {
                bind_token: "bind-token".into(),
                invite_code: invite.into(),
                referral_code: None,
                display_name: "name".into(),
                avatar_data_url: "data:image/png;base64,abcd".into(),
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
            input.avatar_data_url = "invalid".into();
            assert!(
                uc.execute_with_acquisition(input, AcquisitionSource::new(Some("xhs"), None, None))
                    .await
                    .is_err()
            );
            tokio::task::yield_now().await;
            assert!(sources.0.lock().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn acquisition_is_recorded_for_new_wechat_account_only() {
        use crate::auth::{
            application::{
                acquisition_test_support::Sources, record_acquisition::RecordAcquisition,
            },
            domain::acquisition::AcquisitionSource,
        };
        let sources = Arc::new(Sources::default());
        let use_case = BindWechatMiniProgramAccountUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        )
        .with_acquisition_recorder(RecordAcquisition::new(sources.clone()));
        let result = use_case
            .execute_with_acquisition(
                BindWechatMiniProgramAccountInput {
                    bind_token: "bind-token".into(),
                    invite_code: "invite".into(),
                    referral_code: None,
                    display_name: "name".into(),
                    avatar_data_url: "data:image/png;base64,abcd".into(),
                },
                AcquisitionSource::new(Some("douyin"), Some("test1"), None),
            )
            .await
            .unwrap();
        tokio::task::yield_now().await;
        assert_eq!(
            *sources.0.lock().unwrap(),
            vec![(
                result.user.id,
                AcquisitionSource::new(Some("douyin"), Some("test1"), None).unwrap()
            )]
        );
    }

    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::auth::{
        application::bind_wechat_mini_program_account::{
            BindWechatMiniProgramAccountInput, BindWechatMiniProgramAccountUseCase,
        },
        domain::{
            user::{AuthUser, StoredAuthUser},
            wechat::{IssuedInviteCode, WechatOauthProfile},
        },
        ports::{
            auth_repository::AuthRepository,
            token_port::{
                AuthTokenClaims, TokenPort, WechatBindTokenClaims, WechatBindTokenPayload,
            },
        },
    };

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
            let crate::auth::domain::registration::RegistrationIdentity::MiniWechat { profile } =
                request.identity
            else {
                unreachable!()
            };
            let user = if let Some(code) = request.invite_code {
                self.create_user_with_invite_and_mini_program_wechat_with_referral(
                    code,
                    request.referral_code,
                    profile,
                )
                .await?
            } else {
                self.create_user_with_mini_program_wechat_without_invite_with_referral(
                    request.referral_code,
                    profile,
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
            _invite_code: &str,
            _account_identifier: &str,
            _password_hash: &str,
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

        async fn create_user_with_invite_and_mini_program_wechat_with_referral(
            &self,
            invite_code: &str,
            referral_code: Option<&str>,
            profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            self.events.lock().unwrap().push(format!(
                "mini-bind:{invite_code}:{}:{}",
                referral_code.unwrap_or(""),
                profile.display_name.as_deref().unwrap_or("")
            ));

            Ok(AuthUser {
                id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                account_identifier: "wx_test_open_id".to_string(),
                display_name: profile.display_name.clone(),
                invite_code: None,
                avatar_url: profile.avatar_url.clone(),
                has_wechat_binding: true,
                membership_tier: "V3".to_string(),
                membership_expires_at: None,
                membership_benefits_enabled: true,
                ticket_watch_poll_interval_seconds: 300,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
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
            self.events.lock().unwrap().push(format!(
                "mini-bind-without-invite:{}:{}",
                referral_code.unwrap_or(""),
                profile.display_name.as_deref().unwrap_or("")
            ));

            Ok(AuthUser {
                id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                account_identifier: "wx_test_open_id".to_string(),
                display_name: profile.display_name.clone(),
                invite_code: None,
                avatar_url: profile.avatar_url.clone(),
                has_wechat_binding: true,
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
            _user_id: Uuid,
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

        async fn get_user_by_id(&self, _user_id: Uuid) -> anyhow::Result<Option<AuthUser>> {
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
                open_id: "test-open-id".to_string(),
                union_id: Some("test-union-id".to_string()),
                display_name: None,
                avatar_url: None,
                exp: Utc::now() + Duration::minutes(10),
            })
        }
    }

    #[tokio::test]
    async fn execute_trims_and_passes_referral_code() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = BindWechatMiniProgramAccountUseCase::new(
            repository.clone(),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(BindWechatMiniProgramAccountInput {
                bind_token: "bind-token".to_string(),
                invite_code: "INVITE-001".to_string(),
                referral_code: Some("  REF-CODE-001 ".to_string()),
                display_name: "小罗".to_string(),
                avatar_data_url: "data:image/png;base64,abcd".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.access_token, "jwt::wx_test_open_id");
        assert_eq!(
            repository.events.lock().unwrap().clone(),
            vec!["mini-bind:INVITE-001:REF-CODE-001:小罗".to_string()]
        );
    }

    #[tokio::test]
    async fn execute_creates_v1_user_when_invite_code_is_blank() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = BindWechatMiniProgramAccountUseCase::new(
            repository.clone(),
            Arc::new(FakeTokenPort),
            Duration::days(30),
        );

        let result = use_case
            .execute(BindWechatMiniProgramAccountInput {
                bind_token: "bind-token".to_string(),
                invite_code: "   ".to_string(),
                referral_code: Some("  REF-CODE-001 ".to_string()),
                display_name: "小罗".to_string(),
                avatar_data_url: "data:image/png;base64,abcd".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.access_token, "jwt::wx_test_open_id");
        assert_eq!(result.user.membership_tier, "V1");
        assert_eq!(
            repository.events.lock().unwrap().clone(),
            vec!["mini-bind-without-invite:REF-CODE-001:小罗".to_string()]
        );
    }
}

impl BindWechatMiniProgramAccountUseCase {
    pub fn with_acquisition_recorder(
        mut self,
        recorder: crate::auth::application::record_acquisition::RecordAcquisition,
    ) -> Self {
        self.acquisition_recorder = Some(recorder);
        self
    }

    pub fn new(
        repository: Arc<dyn AuthRepository>,
        token_port: Arc<dyn TokenPort>,
        session_ttl: Duration,
    ) -> Self {
        Self {
            repository,
            token_port,
            session_ttl,
            acquisition_recorder: None,
        }
    }

    pub async fn execute(
        &self,
        input: BindWechatMiniProgramAccountInput,
    ) -> anyhow::Result<AuthTokenBundle> {
        self.execute_with_acquisition(input, None).await
    }

    pub async fn execute_with_acquisition(
        &self,
        input: BindWechatMiniProgramAccountInput,
        acquisition: Option<crate::auth::domain::acquisition::AcquisitionSource>,
    ) -> anyhow::Result<AuthTokenBundle> {
        if input.bind_token.trim().is_empty() {
            anyhow::bail!("bind token is required");
        }

        let invite_code = input.invite_code.trim();
        let referral_code = input
            .referral_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        let display_name = input.display_name.trim();
        if display_name.is_empty() {
            anyhow::bail!("display name is required");
        }

        let avatar_data_url = input.avatar_data_url.trim();
        if avatar_data_url.is_empty() {
            anyhow::bail!("avatar data url is required");
        }
        if !avatar_data_url.starts_with("data:image/") || !avatar_data_url.contains(";base64,") {
            anyhow::bail!("avatar data url is invalid");
        }

        let claims = self
            .token_port
            .verify_wechat_bind_token(&input.bind_token)?;
        let profile = WechatOauthProfile {
            open_id: claims.open_id,
            union_id: claims.union_id,
            display_name: Some(display_name.to_string()),
            avatar_url: Some(avatar_data_url.to_string()),
        };

        let registration = self
            .repository
            .register_user(crate::auth::domain::registration::UserRegistration {
                invite_code: (!invite_code.is_empty()).then_some(invite_code),
                referral_code: referral_code.as_deref(),
                identity: crate::auth::domain::registration::RegistrationIdentity::MiniWechat {
                    profile: &profile,
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
