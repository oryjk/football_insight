use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{TimeZone, Utc};
use football_insight_service_backend_rs::admin::{
    adapters::web::routes::admin_user_routes,
    application::admin_user_service::AdminUserService,
    domain::admin_user::{
        AdminCreateUserInput, AdminInviter, AdminUpdateUserInput, AdminUser, AdminUserList,
        AdminUserSearch,
    },
    ports::admin_user_repository::AdminUserRepository,
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

        if let Some(display_name) = search.display_name.as_deref() {
            filtered.retain(|user| {
                user.display_name
                    .as_deref()
                    .unwrap_or_default()
                    .contains(display_name)
            });
        }

        Ok(AdminUserList {
            total: filtered.len() as i64,
            page: search.page,
            page_size: search.page_size,
            items: filtered,
        })
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
        let original_len = users.len();
        users.retain(|user| user.id != user_id);
        Ok(users.len() != original_len)
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
                .header("X-Admin-Token", "test-admin-token")
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
async fn creates_user_with_hashed_password_and_membership_tier() {
    let repository = Arc::new(FakeAdminUserRepository::default());
    let app = app_with_repository(repository.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/users")
                .header("X-Admin-Token", "test-admin-token")
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
                .header("X-Admin-Token", "test-admin-token")
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
                .header("X-Admin-Token", "test-admin-token")
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
    admin_user_routes(service, Some("test-admin-token".to_string()))
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
