use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use football_insight_service_backend_rs::{
    admin::{
        application::admin_auth_service::{AdminAuthService, AdminLoginInput},
        domain::admin_auth::{AdminAccount, AdminSession, AdminTokenClaims},
        ports::{
            admin_auth_repository::AdminAuthRepository,
            admin_authorization_port::AdminAuthorizationPort, admin_token_port::AdminTokenPort,
        },
    },
    auth::ports::password_port::PasswordPort,
};
use uuid::Uuid;

struct AllowAllAuthorization;

impl AdminAuthorizationPort for AllowAllAuthorization {
    fn is_allowed(
        &self,
        _role: &str,
        _permission: football_insight_service_backend_rs::admin::domain::admin_auth::AdminPermission,
    ) -> bool {
        true
    }
}

struct FakeAdminAuthRepository {
    account: AdminAccount,
    sessions: Mutex<Vec<AdminSession>>,
}

#[async_trait]
impl AdminAuthRepository for FakeAdminAuthRepository {
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
            session.id == session_id
                && session.admin_user_id == admin_id
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
            .find(|item| item.id == session_id && item.admin_user_id == admin_id)
        else {
            return Ok(false);
        };
        session.revoked_at = Some(revoked_at);
        Ok(true)
    }
}

struct FakePasswordPort;

impl PasswordPort for FakePasswordPort {
    fn hash_password(&self, password: &str) -> anyhow::Result<String> {
        Ok(format!("hashed::{password}"))
    }

    fn verify_password(&self, password: &str, password_hash: &str) -> anyhow::Result<bool> {
        Ok(password_hash == format!("hashed::{password}"))
    }
}

#[derive(Default)]
struct FakeAdminTokenPort {
    claims: Mutex<Option<AdminTokenClaims>>,
}

impl AdminTokenPort for FakeAdminTokenPort {
    fn issue_token(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
        username: &str,
        role: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<String> {
        *self.claims.lock().unwrap() = Some(AdminTokenClaims {
            admin_id,
            session_id,
            username: username.to_string(),
            role: role.to_string(),
            expires_at,
        });
        Ok("admin-access-token".to_string())
    }

    fn verify_token(&self, token: &str) -> anyhow::Result<AdminTokenClaims> {
        if token != "admin-access-token" {
            anyhow::bail!("invalid token");
        }
        self.claims
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| anyhow::anyhow!("token was not issued"))
    }
}

#[tokio::test]
async fn login_creates_revocable_session_and_returns_admin_identity() {
    let now = Utc.with_ymd_and_hms(2026, 8, 2, 12, 0, 0).unwrap();
    let (service, repository) = service_fixture(now);

    let result = service
        .login(AdminLoginInput {
            username: " Owner ".to_string(),
            password: "strong-password".to_string(),
        })
        .await
        .expect("admin login succeeds");

    assert_eq!(result.access_token, "admin-access-token");
    assert_eq!(result.admin.username, "owner");
    assert_eq!(result.admin.role, "owner");
    assert_eq!(result.expires_at, now + Duration::hours(12));
    assert_eq!(repository.sessions.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn login_rejects_wrong_password_without_creating_session() {
    let now = Utc.with_ymd_and_hms(2026, 8, 2, 12, 0, 0).unwrap();
    let (service, repository) = service_fixture(now);

    let error = service
        .login(AdminLoginInput {
            username: "owner".to_string(),
            password: "wrong-password".to_string(),
        })
        .await
        .expect_err("wrong password must fail");

    assert!(error.to_string().contains("invalid admin credentials"));
    assert!(repository.sessions.lock().unwrap().is_empty());
}

#[tokio::test]
async fn authentication_requires_a_live_server_side_session() {
    let now = Utc.with_ymd_and_hms(2026, 8, 2, 12, 0, 0).unwrap();
    let (service, _) = service_fixture(now);
    let login = service
        .login(AdminLoginInput {
            username: "owner".to_string(),
            password: "strong-password".to_string(),
        })
        .await
        .unwrap();

    let principal = service.authenticate(&login.access_token).await.unwrap();
    assert_eq!(principal.admin_id, login.admin.id);

    service.logout(&login.access_token).await.unwrap();
    assert!(service.authenticate(&login.access_token).await.is_err());
}

fn service_fixture(now: DateTime<Utc>) -> (Arc<AdminAuthService>, Arc<FakeAdminAuthRepository>) {
    let repository = Arc::new(FakeAdminAuthRepository {
        account: AdminAccount {
            id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            username: "owner".to_string(),
            password_hash: "hashed::strong-password".to_string(),
            display_name: "Owner".to_string(),
            role: "owner".to_string(),
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        },
        sessions: Mutex::new(Vec::new()),
    });
    let service = Arc::new(AdminAuthService::new(
        repository.clone(),
        Arc::new(FakePasswordPort),
        Arc::new(FakeAdminTokenPort::default()),
        Arc::new(AllowAllAuthorization),
        Duration::hours(12),
        Arc::new(move || now),
    ));
    (service, repository)
}
