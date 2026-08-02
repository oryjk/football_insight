use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{DateTime, Duration, Utc};
use football_insight_service_backend_rs::{
    admin::{
        adapters::{
            security::{
                jwt_admin_token_port::JwtAdminTokenPort,
                role_based_admin_authorization::RoleBasedAdminAuthorization,
            },
            web::routes::admin_auth_routes,
        },
        application::admin_auth_service::AdminAuthService,
        domain::admin_auth::{AdminAccount, AdminBootstrapOwner, AdminSession},
        ports::{admin_auth_repository::AdminAuthRepository, admin_token_port::AdminTokenPort},
    },
    auth::{adapters::security::jwt_token_port::JwtTokenPort, ports::token_port::TokenPort},
};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

use football_insight_service_backend_rs::auth::ports::password_port::PasswordPort;

#[test]
fn admin_token_roundtrip_preserves_admin_and_session_identity() {
    let token_port = JwtAdminTokenPort::new("admin-secret".to_string());
    let admin_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(12);

    let token = token_port
        .issue_token(admin_id, session_id, "owner", "owner", expires_at)
        .expect("issue admin token");
    let claims = token_port.verify_token(&token).expect("verify admin token");

    assert_eq!(claims.admin_id, admin_id);
    assert_eq!(claims.session_id, session_id);
    assert_eq!(claims.username, "owner");
    assert_eq!(claims.role, "owner");
    assert_eq!(claims.expires_at.timestamp(), expires_at.timestamp());
}

#[test]
fn admin_token_port_rejects_c_end_access_tokens() {
    let secret = "shared-test-secret".to_string();
    let user_token_port = JwtTokenPort::new(secret.clone());
    let admin_token_port = JwtAdminTokenPort::new(secret);
    let token = user_token_port
        .issue_token(
            Uuid::new_v4(),
            "footballfan",
            Utc::now() + Duration::hours(1),
        )
        .expect("issue C-end token");

    assert!(admin_token_port.verify_token(&token).is_err());
}

#[test]
fn admin_token_port_rejects_expired_tokens() {
    let token_port = JwtAdminTokenPort::new("admin-secret".to_string());
    let token = token_port
        .issue_token(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "owner",
            "owner",
            Utc::now() - Duration::seconds(1),
        )
        .expect("issue expired admin token");

    assert!(token_port.verify_token(&token).is_err());
}

struct HttpFakeAdminRepository {
    account: AdminAccount,
    sessions: Mutex<Vec<AdminSession>>,
}

#[async_trait]
impl AdminAuthRepository for HttpFakeAdminRepository {
    async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<AdminAccount>> {
        Ok((self.account.username == username).then(|| self.account.clone()))
    }

    async fn create_session(&self, session: AdminSession) -> anyhow::Result<()> {
        self.sessions.lock().unwrap().push(session);
        Ok(())
    }

    async fn find_active_account_for_session(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<AdminAccount>> {
        let active = self.sessions.lock().unwrap().iter().any(|session| {
            session.admin_user_id == admin_id
                && session.id == session_id
                && session.revoked_at.is_none()
                && session.expires_at > now
        });
        Ok(active.then(|| self.account.clone()))
    }

    async fn revoke_session(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
        revoked_at: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        let mut sessions = self.sessions.lock().unwrap();
        let Some(session) = sessions
            .iter_mut()
            .find(|item| item.admin_user_id == admin_id && item.id == session_id)
        else {
            return Ok(false);
        };
        session.revoked_at = Some(revoked_at);
        Ok(true)
    }

    async fn ensure_owner(&self, _owner: AdminBootstrapOwner) -> anyhow::Result<AdminAccount> {
        Ok(self.account.clone())
    }
}

struct HttpFakePasswordPort;

impl PasswordPort for HttpFakePasswordPort {
    fn hash_password(&self, value: &str) -> anyhow::Result<String> {
        Ok(format!("hashed::{value}"))
    }

    fn verify_password(&self, value: &str, hash: &str) -> anyhow::Result<bool> {
        Ok(hash == format!("hashed::{value}"))
    }
}

#[tokio::test]
async fn login_me_and_logout_form_a_revocable_admin_http_session() {
    let app = auth_app();
    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/auth/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"username": "owner", "password": "strong-password"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);
    let login_body = json_body(login_response).await;
    let access_token = login_body["access_token"].as_str().unwrap();
    assert_eq!(login_body["admin"]["username"], json!("owner"));
    assert_eq!(login_body["admin"]["role"], json!("owner"));

    let me_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/auth/me")
                .header("Authorization", format!("Bearer {access_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me_response.status(), StatusCode::OK);
    assert_eq!(json_body(me_response).await["username"], json!("owner"));

    let logout_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/auth/logout")
                .method("POST")
                .header("Authorization", format!("Bearer {access_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout_response.status(), StatusCode::NO_CONTENT);

    let rejected_me = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/auth/me")
                .header("Authorization", format!("Bearer {access_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected_me.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_me_rejects_missing_bearer_token() {
    let response = auth_app()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

fn auth_app() -> axum::Router {
    let now = Utc::now();
    let repository = Arc::new(HttpFakeAdminRepository {
        account: AdminAccount {
            id: Uuid::new_v4(),
            username: "owner".to_string(),
            password_hash: "hashed::strong-password".to_string(),
            display_name: "Football Insight Owner".to_string(),
            role: "owner".to_string(),
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        },
        sessions: Mutex::new(Vec::new()),
    });
    let service = Arc::new(AdminAuthService::new(
        repository,
        Arc::new(HttpFakePasswordPort),
        Arc::new(JwtAdminTokenPort::new("http-admin-secret".to_string())),
        Arc::new(RoleBasedAdminAuthorization),
        Duration::hours(12),
        Arc::new(Utc::now),
    ));
    admin_auth_routes(service)
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
