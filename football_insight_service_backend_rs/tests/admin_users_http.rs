use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use football_insight_service_backend_rs::admin::{
    adapters::{
        security::{
            jwt_admin_token_port::JwtAdminTokenPort,
            role_based_admin_authorization::RoleBasedAdminAuthorization,
        },
        web::routes::admin_user_routes,
    },
    application::{admin_auth_service::AdminAuthService, admin_user_service::AdminUserService},
    domain::{
        admin_auth::{AdminAccount, AdminSession},
        admin_user::{
            AdminCreateUserInput, AdminInviter, AdminMembershipAdjustment, AdminUpdateUserInput,
            AdminUser, AdminUserDetail, AdminUserList, AdminUserSearch,
        },
    },
    ports::{
        admin_auth_repository::AdminAuthRepository, admin_token_port::AdminTokenPort,
        admin_user_repository::AdminUserRepository,
    },
};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Default)]
struct FakeAdminUserRepository {
    users: Mutex<Vec<AdminUser>>,
    created_password_hashes: Mutex<Vec<String>>,
    deleted_user_ids: Mutex<Vec<Uuid>>,
}

#[async_trait]
impl AdminUserRepository for FakeAdminUserRepository {
    async fn list_users(&self, search: AdminUserSearch) -> anyhow::Result<AdminUserList> {
        let users = self.users.lock().unwrap();
        let mut filtered = users.clone();

        if let Some(query) = search.query.as_deref() {
            filtered.retain(|user| {
                user.display_name
                    .as_deref()
                    .unwrap_or_default()
                    .contains(query)
                    || user.account_identifier.contains(query)
            });
        }
        if let Some(status) = search.status.as_deref() {
            filtered.retain(|user| user.status == status);
        }
        if let Some(tier) = search.membership_tier.as_deref() {
            filtered.retain(|user| user.membership_tier == tier);
        }

        Ok(AdminUserList {
            total: filtered.len() as i64,
            page: search.page,
            page_size: search.page_size,
            items: filtered,
        })
    }

    async fn get_user(&self, user_id: Uuid) -> anyhow::Result<Option<AdminUser>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|user| user.id == user_id)
            .cloned())
    }

    async fn get_user_detail(&self, user_id: Uuid) -> anyhow::Result<Option<AdminUserDetail>> {
        Ok(self.get_user(user_id).await?.map(|user| AdminUserDetail {
            user,
            referrals: Vec::new(),
            activity: None,
            orders: Vec::new(),
            subscriptions: Vec::new(),
            devices: Vec::new(),
        }))
    }

    async fn create_user(
        &self,
        input: AdminCreateUserInput,
        password_hash: String,
    ) -> anyhow::Result<AdminUser> {
        self.created_password_hashes
            .lock()
            .unwrap()
            .push(password_hash);

        let user = AdminUser {
            id: Uuid::new_v4(),
            account_identifier: input.account_identifier,
            display_name: Some(input.display_name),
            avatar_url: input.avatar_url,
            has_wechat_binding: false,
            status: "active".to_string(),
            invite_code: None,
            invited_by: None,
            membership_tier: input.membership_tier,
            membership_expires_at: input.membership_expires_at,
            created_at: Utc.with_ymd_and_hms(2026, 5, 25, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 5, 25, 10, 0, 0).unwrap(),
        };

        self.users.lock().unwrap().push(user.clone());
        Ok(user)
    }

    async fn update_user(
        &self,
        user_id: Uuid,
        input: AdminUpdateUserInput,
    ) -> anyhow::Result<Option<AdminUser>> {
        let mut users = self.users.lock().unwrap();
        let Some(user) = users.iter_mut().find(|item| item.id == user_id) else {
            return Ok(None);
        };

        if let Some(account_identifier) = input.account_identifier {
            user.account_identifier = account_identifier;
        }
        if let Some(display_name) = input.display_name {
            user.display_name = Some(display_name);
        }
        if let Some(avatar_url) = input.avatar_url {
            user.avatar_url = avatar_url;
        }
        if let Some(membership_tier) = input.membership_tier {
            user.membership_tier = membership_tier;
        }
        if input.membership_expires_at_set {
            user.membership_expires_at = input.membership_expires_at;
        }
        user.updated_at = Utc.with_ymd_and_hms(2026, 5, 25, 11, 0, 0).unwrap();

        Ok(Some(user.clone()))
    }

    async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<bool> {
        self.deleted_user_ids.lock().unwrap().push(user_id);
        let mut users = self.users.lock().unwrap();
        let Some(user) = users.iter_mut().find(|user| user.id == user_id) else {
            return Ok(false);
        };
        user.status = "disabled".to_string();
        Ok(true)
    }

    async fn set_user_status(
        &self,
        user_id: Uuid,
        status: &str,
        _admin_id: Uuid,
        _reason: &str,
    ) -> anyhow::Result<Option<AdminUser>> {
        let mut users = self.users.lock().unwrap();
        let Some(user) = users.iter_mut().find(|item| item.id == user_id) else {
            return Ok(None);
        };
        user.status = status.to_string();
        Ok(Some(user.clone()))
    }

    async fn adjust_membership(
        &self,
        user_id: Uuid,
        adjustment: AdminMembershipAdjustment,
        _admin_id: Uuid,
    ) -> anyhow::Result<Option<AdminUser>> {
        let mut users = self.users.lock().unwrap();
        let Some(user) = users.iter_mut().find(|item| item.id == user_id) else {
            return Ok(None);
        };
        user.membership_tier = adjustment.membership_tier;
        if adjustment.membership_expires_at_set {
            user.membership_expires_at = adjustment.membership_expires_at;
        }
        Ok(Some(user.clone()))
    }
}

#[derive(Default)]
struct FakePasswordPort;

impl football_insight_service_backend_rs::auth::ports::password_port::PasswordPort
    for FakePasswordPort
{
    fn hash_password(&self, password: &str) -> anyhow::Result<String> {
        Ok(format!("hashed::{password}"))
    }

    fn verify_password(&self, _password: &str, _password_hash: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

struct AlwaysActiveAdminAuthRepository;

#[async_trait]
impl AdminAuthRepository for AlwaysActiveAdminAuthRepository {
    async fn find_by_username(&self, _username: &str) -> anyhow::Result<Option<AdminAccount>> {
        Ok(None)
    }

    async fn create_session(&self, _session: AdminSession) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_active_account_for_session(
        &self,
        admin_id: Uuid,
        _session_id: Uuid,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<AdminAccount>> {
        Ok(Some(AdminAccount {
            id: admin_id,
            username: "owner".to_string(),
            password_hash: String::new(),
            display_name: "Owner".to_string(),
            role: "owner".to_string(),
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        }))
    }

    async fn revoke_session(
        &self,
        _admin_id: Uuid,
        _session_id: Uuid,
        _revoked_at: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        Ok(true)
    }
}

#[tokio::test]
async fn rejects_admin_users_request_without_admin_token() {
    let app = app_with_users(vec![]);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/users")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn searches_users_by_display_name_substring() {
    let matching_user_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    let referrer_user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let app = app_with_users(vec![
        test_user(matching_user_id, "13800138000", Some("王小明"), "V3")
            .with_invite_code("FI-OWN-001")
            .with_invited_by(AdminInviter {
                id: referrer_user_id,
                display_name: Some("邀请人".to_string()),
                account_identifier: "referrer-account".to_string(),
                referral_invite_code: "FI-REF-001".to_string(),
            }),
        test_user(
            Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
            "footballfan",
            Some("张三"),
            "V1",
        ),
    ]);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/users?display_name=%E7%8E%8B&page=1&page_size=10")
                .header("Authorization", admin_authorization_header())
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["total"], json!(1));
    assert_eq!(body["items"][0]["id"], json!(matching_user_id.to_string()));
    assert_eq!(body["items"][0]["display_name"], json!("王小明"));
    assert_eq!(body["items"][0]["invite_code"], json!("FI-OWN-001"));
    assert_eq!(
        body["items"][0]["invited_by"],
        json!({
            "id": referrer_user_id.to_string(),
            "display_name": "邀请人",
            "account_identifier": "referrer-account",
            "referral_invite_code": "FI-REF-001"
        })
    );
}

#[tokio::test]
async fn gets_admin_user_detail_by_id() {
    let user_id = Uuid::parse_str("88888888-8888-8888-8888-888888888888").unwrap();
    let response = app_with_users(vec![test_user(
        user_id,
        "detail-user",
        Some("详情用户"),
        "V5",
    )])
    .oneshot(
        Request::builder()
            .uri(format!("/api/v1/admin/users/{user_id}"))
            .header("Authorization", admin_authorization_header())
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["id"], json!(user_id.to_string()));
    assert_eq!(body["account_identifier"], json!("detail-user"));
    assert_eq!(body["membership_tier"], json!("V5"));
}

#[tokio::test]
async fn creates_user_with_hashed_password_and_membership_tier() {
    let repository = Arc::new(FakeAdminUserRepository::default());
    let app = app_with_repository(repository.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/users")
                .header("Authorization", admin_authorization_header())
                .header("Content-Type", "application/json")
                .method("POST")
                .body(Body::from(
                    json!({
                        "account_identifier": "newfootballfan",
                        "display_name": "新球迷",
                        "password": "secret123",
                        "membership_tier": "V9"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = json_body(response).await;
    assert_eq!(body["account_identifier"], json!("newfootballfan"));
    assert_eq!(body["display_name"], json!("新球迷"));
    assert_eq!(body["membership_tier"], json!("V9"));
    assert_eq!(
        repository
            .created_password_hashes
            .lock()
            .unwrap()
            .as_slice(),
        ["hashed::secret123"]
    );
}

#[tokio::test]
async fn updates_user_membership_tier() {
    let user_id = Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap();
    let app = app_with_users(vec![test_user(
        user_id,
        "member-user",
        Some("会员用户"),
        "V1",
    )]);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/admin/users/{user_id}"))
                .header("Authorization", admin_authorization_header())
                .header("Content-Type", "application/json")
                .method("PATCH")
                .body(Body::from(json!({ "membership_tier": "V6" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["membership_tier"], json!("V6"));
}

#[tokio::test]
async fn deletes_user_by_id() {
    let user_id = Uuid::parse_str("44444444-4444-4444-4444-444444444444").unwrap();
    let repository = Arc::new(FakeAdminUserRepository {
        users: Mutex::new(vec![test_user(
            user_id,
            "delete-user",
            Some("待删除用户"),
            "V1",
        )]),
        ..Default::default()
    });
    let app = app_with_repository(repository.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/admin/users/{user_id}"))
                .header("Authorization", admin_authorization_header())
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        repository.deleted_user_ids.lock().unwrap().as_slice(),
        [user_id]
    );
}

#[tokio::test]
async fn disables_and_restores_user_without_deleting_it() {
    let user_id = Uuid::parse_str("55555555-5555-5555-5555-555555555555").unwrap();
    let repository = Arc::new(FakeAdminUserRepository {
        users: Mutex::new(vec![test_user(
            user_id,
            "status-user",
            Some("状态用户"),
            "V1",
        )]),
        ..Default::default()
    });

    let disabled = app_with_repository(repository.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/admin/users/{user_id}/disable"))
                .method("POST")
                .header("Authorization", admin_authorization_header())
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"reason": "账号风险"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(disabled.status(), StatusCode::OK);
    assert_eq!(json_body(disabled).await["status"], json!("disabled"));

    let restored = app_with_repository(repository)
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/admin/users/{user_id}/restore"))
                .method("POST")
                .header("Authorization", admin_authorization_header())
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"reason": "人工复核通过"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restored.status(), StatusCode::OK);
    assert_eq!(json_body(restored).await["status"], json!("active"));
}

#[tokio::test]
async fn adjusts_membership_with_expiration_mode_and_reason() {
    let user_id = Uuid::parse_str("66666666-6666-6666-6666-666666666666").unwrap();
    let app = app_with_users(vec![test_user(
        user_id,
        "membership-user",
        Some("会员调整用户"),
        "V1",
    )]);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/admin/users/{user_id}/membership"))
                .method("POST")
                .header("Authorization", admin_authorization_header())
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "membership_tier": "V8",
                        "expiration_mode": "never",
                        "reason": "线下年度会员"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["membership_tier"], json!("V8"));
    assert_eq!(body["membership_expires_at"], Value::Null);
}

#[tokio::test]
async fn rejects_membership_adjustment_without_reason() {
    let user_id = Uuid::parse_str("77777777-7777-7777-7777-777777777777").unwrap();
    let response = app_with_users(vec![test_user(user_id, "member", Some("会员"), "V1")])
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/admin/users/{user_id}/membership"))
                .method("POST")
                .header("Authorization", admin_authorization_header())
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"membership_tier": "V9", "expiration_mode": "never"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

fn app_with_users(users: Vec<AdminUser>) -> axum::Router {
    app_with_repository(Arc::new(FakeAdminUserRepository {
        users: Mutex::new(users),
        ..Default::default()
    }))
}

fn app_with_repository(repository: Arc<FakeAdminUserRepository>) -> axum::Router {
    let service = Arc::new(AdminUserService::new(
        repository,
        Arc::new(FakePasswordPort),
    ));
    let auth_service = Arc::new(AdminAuthService::new(
        Arc::new(AlwaysActiveAdminAuthRepository),
        Arc::new(FakePasswordPort),
        Arc::new(JwtAdminTokenPort::new(admin_jwt_secret().to_string())),
        Arc::new(RoleBasedAdminAuthorization),
        Duration::hours(12),
        Arc::new(Utc::now),
    ));
    admin_user_routes(service, auth_service)
}

fn admin_authorization_header() -> String {
    let token = JwtAdminTokenPort::new(admin_jwt_secret().to_string())
        .issue_token(
            Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
            "owner",
            "owner",
            Utc::now() + Duration::hours(1),
        )
        .unwrap();
    format!("Bearer {token}")
}

fn admin_jwt_secret() -> &'static str {
    "admin-users-http-test-secret"
}

fn test_user(
    id: Uuid,
    account_identifier: &str,
    display_name: Option<&str>,
    membership_tier: &str,
) -> AdminUser {
    AdminUser {
        id,
        account_identifier: account_identifier.to_string(),
        display_name: display_name.map(str::to_string),
        avatar_url: None,
        has_wechat_binding: false,
        status: "active".to_string(),
        invite_code: None,
        invited_by: None,
        membership_tier: membership_tier.to_string(),
        membership_expires_at: None,
        created_at: Utc.with_ymd_and_hms(2026, 5, 25, 9, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 5, 25, 9, 0, 0).unwrap(),
    }
}

trait AdminUserFixtureExt {
    fn with_invite_code(self, invite_code: &str) -> Self;
    fn with_invited_by(self, invited_by: AdminInviter) -> Self;
}

impl AdminUserFixtureExt for AdminUser {
    fn with_invite_code(mut self, invite_code: &str) -> Self {
        self.invite_code = Some(invite_code.to_string());
        self
    }

    fn with_invited_by(mut self, invited_by: AdminInviter) -> Self {
        self.invited_by = Some(invited_by);
        self
    }
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("read response body");
    serde_json::from_slice(&bytes).expect("parse json response")
}
