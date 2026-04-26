use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use football_insight_service_backend_rs::auth::{
    application::bind_wechat_account::{BindWechatAccountInput, BindWechatAccountUseCase},
    application::bind_wechat_mini_program_account::{
        BindWechatMiniProgramAccountInput, BindWechatMiniProgramAccountUseCase,
    },
    application::login_with_mini_wechat::{
        CompleteMiniWechatLoginInput, CompleteMiniWechatLoginUseCase, MiniWechatLoginResult,
    },
    application::login_with_wechat::{
        CompleteWechatLoginInput, CompleteWechatLoginUseCase, WechatLoginResult,
    },
    domain::{
        user::{AuthUser, StoredAuthUser},
        wechat::{IssuedInviteCode, WechatOauthProfile},
    },
    ports::{
        auth_repository::AuthRepository,
        token_port::{AuthTokenClaims, TokenPort, WechatBindTokenClaims, WechatBindTokenPayload},
        wechat_oauth_port::WechatOauthPort,
    },
};
use uuid::Uuid;

#[derive(Clone)]
struct FakeWechatOauthPort {
    profile: WechatOauthProfile,
}

#[async_trait]
impl WechatOauthPort for FakeWechatOauthPort {
    async fn fetch_user_profile(&self, _code: &str) -> anyhow::Result<WechatOauthProfile> {
        Ok(self.profile.clone())
    }

    async fn fetch_mini_program_profile(&self, _code: &str) -> anyhow::Result<WechatOauthProfile> {
        Ok(self.profile.clone())
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
        Ok(format!("access::{account_identifier}"))
    }

    fn verify_token(&self, _token: &str) -> anyhow::Result<AuthTokenClaims> {
        unreachable!()
    }

    fn issue_wechat_bind_token(
        &self,
        payload: WechatBindTokenPayload,
        _expires_at: chrono::DateTime<Utc>,
    ) -> anyhow::Result<String> {
        Ok(format!(
            "bind::{}::{}::{}::{}",
            payload.open_id,
            payload.union_id.unwrap_or_default(),
            payload.display_name.unwrap_or_default(),
            payload.avatar_url.unwrap_or_default()
        ))
    }

    fn verify_wechat_bind_token(&self, token: &str) -> anyhow::Result<WechatBindTokenClaims> {
        let parts: Vec<&str> = token.splitn(5, "::").collect();
        if parts.len() != 5 || parts[0] != "bind" {
            anyhow::bail!("invalid bind token");
        }

        Ok(WechatBindTokenClaims {
            open_id: parts[1].to_string(),
            union_id: (!parts[2].is_empty()).then(|| parts[2].to_string()),
            display_name: (!parts[3].is_empty()).then(|| parts[3].to_string()),
            avatar_url: (!parts[4].is_empty()).then(|| parts[4].to_string()),
            exp: Utc::now() + Duration::minutes(10),
        })
    }
}

struct BoundRepo;

#[async_trait]
impl AuthRepository for BoundRepo {
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
        _account_identifier: &str,
    ) -> anyhow::Result<Option<StoredAuthUser>> {
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

    async fn find_user_by_wechat_open_id(&self, open_id: &str) -> anyhow::Result<Option<AuthUser>> {
        Ok(Some(AuthUser {
            id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            account_identifier: "13800138000".to_string(),
            display_name: Some(format!("bound::{open_id}")),
            invite_code: None,
            avatar_url: None,
            has_wechat_binding: true,
            membership_tier: "V3".to_string(),
            membership_expires_at: None,
            membership_benefits_enabled: true,
            ticket_watch_poll_interval_seconds: 300,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }))
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

    async fn create_user_with_invite_and_mini_program_wechat(
        &self,
        _invite_code: &str,
        _profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
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

struct UnboundRepo;

#[async_trait]
impl AuthRepository for UnboundRepo {
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
        _account_identifier: &str,
    ) -> anyhow::Result<Option<StoredAuthUser>> {
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

    async fn find_user_by_wechat_open_id(
        &self,
        _open_id: &str,
    ) -> anyhow::Result<Option<AuthUser>> {
        Ok(None)
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

    async fn create_user_with_invite_and_mini_program_wechat(
        &self,
        _invite_code: &str,
        _profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
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

#[tokio::test]
async fn complete_wechat_login_returns_authenticated_user_when_openid_is_bound() {
    let use_case = CompleteWechatLoginUseCase::new(
        Arc::new(BoundRepo),
        Arc::new(FakeWechatOauthPort {
            profile: WechatOauthProfile {
                open_id: "openid-123".to_string(),
                union_id: Some("union-123".to_string()),
                display_name: Some("何止编程".to_string()),
                avatar_url: Some("https://img.example.com/avatar.png".to_string()),
            },
        }),
        Arc::new(FakeTokenPort),
        Duration::days(30),
        Duration::minutes(10),
    );

    let result = use_case
        .execute(CompleteWechatLoginInput {
            code: "wechat-code".to_string(),
        })
        .await
        .unwrap();

    match result {
        WechatLoginResult::Authenticated(bundle) => {
            assert_eq!(bundle.user.account_identifier, "13800138000");
            assert_eq!(bundle.access_token, "access::13800138000");
        }
        WechatLoginResult::BindingRequired { .. } => {
            panic!("expected authenticated result")
        }
    }
}

#[tokio::test]
async fn complete_wechat_login_returns_bind_token_when_openid_is_unbound() {
    let use_case = CompleteWechatLoginUseCase::new(
        Arc::new(UnboundRepo),
        Arc::new(FakeWechatOauthPort {
            profile: WechatOauthProfile {
                open_id: "openid-123".to_string(),
                union_id: Some("union-123".to_string()),
                display_name: Some("何止编程".to_string()),
                avatar_url: Some("https://img.example.com/avatar.png".to_string()),
            },
        }),
        Arc::new(FakeTokenPort),
        Duration::days(30),
        Duration::minutes(10),
    );

    let result = use_case
        .execute(CompleteWechatLoginInput {
            code: "wechat-code".to_string(),
        })
        .await
        .unwrap();

    match result {
        WechatLoginResult::BindingRequired {
            bind_token,
            profile,
            ..
        } => {
            assert_eq!(
                bind_token,
                "bind::openid-123::union-123::何止编程::https://img.example.com/avatar.png"
            );
            assert_eq!(profile.open_id, "openid-123");
            assert_eq!(profile.display_name.as_deref(), Some("何止编程"));
        }
        WechatLoginResult::Authenticated(_) => {
            panic!("expected binding-required result")
        }
    }
}

#[tokio::test]
async fn complete_mini_wechat_login_returns_authenticated_user_when_openid_is_bound() {
    let use_case = CompleteMiniWechatLoginUseCase::new(
        Arc::new(BoundRepo),
        Arc::new(FakeWechatOauthPort {
            profile: WechatOauthProfile {
                open_id: "mini-openid-123".to_string(),
                union_id: Some("mini-union-123".to_string()),
                display_name: None,
                avatar_url: None,
            },
        }),
        Arc::new(FakeTokenPort),
        Duration::days(30),
        Duration::minutes(10),
    );

    let result = use_case
        .execute(CompleteMiniWechatLoginInput {
            code: "mini-wechat-code".to_string(),
        })
        .await
        .unwrap();

    match result {
        MiniWechatLoginResult::Authenticated(bundle) => {
            assert_eq!(bundle.user.account_identifier, "13800138000");
            assert_eq!(bundle.access_token, "access::13800138000");
        }
        MiniWechatLoginResult::BindingRequired { .. } => {
            panic!("expected authenticated result")
        }
    }
}

#[tokio::test]
async fn complete_mini_wechat_login_returns_bind_token_when_openid_is_unbound() {
    let use_case = CompleteMiniWechatLoginUseCase::new(
        Arc::new(UnboundRepo),
        Arc::new(FakeWechatOauthPort {
            profile: WechatOauthProfile {
                open_id: "mini-openid-123".to_string(),
                union_id: Some("mini-union-123".to_string()),
                display_name: None,
                avatar_url: None,
            },
        }),
        Arc::new(FakeTokenPort),
        Duration::days(30),
        Duration::minutes(10),
    );

    let result = use_case
        .execute(CompleteMiniWechatLoginInput {
            code: "mini-wechat-code".to_string(),
        })
        .await
        .unwrap();

    match result {
        MiniWechatLoginResult::BindingRequired {
            bind_token,
            profile,
            ..
        } => {
            assert_eq!(bind_token, "bind::mini-openid-123::mini-union-123::::");
            assert_eq!(profile.open_id, "mini-openid-123");
            assert_eq!(profile.display_name, None);
            assert_eq!(profile.avatar_url, None);
        }
        MiniWechatLoginResult::Authenticated(_) => {
            panic!("expected binding-required result")
        }
    }
}

#[derive(Default)]
struct BindRepo {
    find_existing_user: bool,
}

#[async_trait]
impl AuthRepository for BindRepo {
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
        if !self.find_existing_user {
            return Ok(None);
        }

        Ok(Some(StoredAuthUser {
            user: AuthUser {
                id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                account_identifier: account_identifier.to_string(),
                display_name: Some("已有账号".to_string()),
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

    async fn find_user_by_wechat_open_id(
        &self,
        _open_id: &str,
    ) -> anyhow::Result<Option<AuthUser>> {
        Ok(None)
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

    async fn bind_wechat_to_user(
        &self,
        _user_id: Uuid,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        Ok(AuthUser {
            id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
            account_identifier: "13800138000".to_string(),
            display_name: profile
                .display_name
                .clone()
                .or(Some("已有账号".to_string())),
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

    async fn create_user_with_invite_and_wechat(
        &self,
        _invite_code: &str,
        phone_number: &str,
        _password_hash: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        Ok(AuthUser {
            id: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap(),
            account_identifier: phone_number.to_string(),
            display_name: profile
                .display_name
                .clone()
                .or(Some(phone_number.to_string())),
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

    async fn create_user_with_invite_and_mini_program_wechat(
        &self,
        _invite_code: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        Ok(AuthUser {
            id: Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddddddd").unwrap(),
            account_identifier: "wx_mini_openid_123".to_string(),
            display_name: profile
                .display_name
                .clone()
                .or(Some("微信用户".to_string())),
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

impl football_insight_service_backend_rs::auth::ports::password_port::PasswordPort
    for FakePasswordPort
{
    fn hash_password(&self, password: &str) -> anyhow::Result<String> {
        Ok(format!("hashed::{password}"))
    }

    fn verify_password(&self, password: &str, password_hash: &str) -> anyhow::Result<bool> {
        Ok(format!("hashed::{password}") == password_hash)
    }
}

#[tokio::test]
async fn bind_wechat_account_binds_existing_phone_user() {
    let token_port = Arc::new(FakeTokenPort);
    let bind_token = token_port
        .issue_wechat_bind_token(
            WechatBindTokenPayload {
                open_id: "openid-123".to_string(),
                union_id: Some("union-123".to_string()),
                display_name: Some("何止编程".to_string()),
                avatar_url: Some("https://img.example.com/avatar.png".to_string()),
            },
            Utc::now() + Duration::minutes(10),
        )
        .unwrap();

    let use_case = BindWechatAccountUseCase::new(
        Arc::new(BindRepo {
            find_existing_user: true,
        }),
        Arc::new(FakePasswordPort),
        token_port.clone(),
        Duration::days(30),
    );

    let result = use_case
        .execute(BindWechatAccountInput {
            bind_token,
            invite_code: None,
            referral_code: None,
            phone_number: "13800138000".to_string(),
            password: "secret123".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(result.user.account_identifier, "13800138000");
    assert_eq!(result.access_token, "access::13800138000");
}

#[tokio::test]
async fn bind_wechat_account_creates_new_user_with_invite_when_phone_not_found() {
    let token_port = Arc::new(FakeTokenPort);
    let bind_token = token_port
        .issue_wechat_bind_token(
            WechatBindTokenPayload {
                open_id: "openid-123".to_string(),
                union_id: Some("union-123".to_string()),
                display_name: Some("何止编程".to_string()),
                avatar_url: Some("https://img.example.com/avatar.png".to_string()),
            },
            Utc::now() + Duration::minutes(10),
        )
        .unwrap();

    let use_case = BindWechatAccountUseCase::new(
        Arc::new(BindRepo {
            find_existing_user: false,
        }),
        Arc::new(FakePasswordPort),
        token_port.clone(),
        Duration::days(30),
    );

    let result = use_case
        .execute(BindWechatAccountInput {
            bind_token,
            invite_code: Some("FI-INVITE".to_string()),
            referral_code: None,
            phone_number: "13800138000".to_string(),
            password: "secret123".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(result.user.account_identifier, "13800138000");
    assert_eq!(result.access_token, "access::13800138000");
}

#[tokio::test]
async fn bind_wechat_mini_program_account_creates_new_user_with_invite() {
    let token_port = Arc::new(FakeTokenPort);
    let bind_token = token_port
        .issue_wechat_bind_token(
            WechatBindTokenPayload {
                open_id: "mini-openid-123".to_string(),
                union_id: Some("mini-union-123".to_string()),
                display_name: None,
                avatar_url: None,
            },
            Utc::now() + Duration::minutes(10),
        )
        .unwrap();

    let use_case = BindWechatMiniProgramAccountUseCase::new(
        Arc::new(BindRepo::default()),
        token_port.clone(),
        Duration::days(30),
    );

    let result = use_case
        .execute(BindWechatMiniProgramAccountInput {
            bind_token,
            invite_code: "FI-INVITE".to_string(),
            referral_code: None,
            display_name: "何止编程".to_string(),
            avatar_data_url: "data:image/png;base64,AAA".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(result.user.account_identifier, "wx_mini_openid_123");
    assert_eq!(result.user.display_name.as_deref(), Some("何止编程"));
    assert_eq!(
        result.user.avatar_url.as_deref(),
        Some("data:image/png;base64,AAA")
    );
    assert_eq!(result.access_token, "access::wx_mini_openid_123");
}
