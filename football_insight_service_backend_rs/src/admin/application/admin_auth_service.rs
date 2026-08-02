use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::{
    admin::{
        domain::admin_auth::{
            AdminAccount, AdminBootstrapOwner, AdminPermission, AdminPrincipal, AdminSession,
            validate_admin_login_password, validate_admin_username,
        },
        ports::{
            admin_auth_repository::AdminAuthRepository,
            admin_authorization_port::AdminAuthorizationPort, admin_token_port::AdminTokenPort,
        },
    },
    auth::ports::password_port::PasswordPort,
};

pub type AdminClock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminLoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminIdentity {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminLoginResult {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub admin: AdminIdentity,
}

pub struct AdminAuthService {
    repository: Arc<dyn AdminAuthRepository>,
    password_port: Arc<dyn PasswordPort>,
    token_port: Arc<dyn AdminTokenPort>,
    authorization_port: Arc<dyn AdminAuthorizationPort>,
    session_duration: Duration,
    clock: AdminClock,
}

impl AdminAuthService {
    pub fn new(
        repository: Arc<dyn AdminAuthRepository>,
        password_port: Arc<dyn PasswordPort>,
        token_port: Arc<dyn AdminTokenPort>,
        authorization_port: Arc<dyn AdminAuthorizationPort>,
        session_duration: Duration,
        clock: AdminClock,
    ) -> Self {
        Self {
            repository,
            password_port,
            token_port,
            authorization_port,
            session_duration,
            clock,
        }
    }

    pub async fn login(&self, input: AdminLoginInput) -> anyhow::Result<AdminLoginResult> {
        let username = validate_admin_username(&input.username)?;
        let password = validate_admin_login_password(&input.password)?;
        let account = self
            .repository
            .find_by_username(&username)
            .await?
            .filter(|account| account.status == "active")
            .ok_or_else(|| anyhow::anyhow!("invalid admin credentials"))?;

        if !self
            .password_port
            .verify_password(&password, &account.password_hash)?
        {
            anyhow::bail!("invalid admin credentials");
        }

        let now = (self.clock)();
        let expires_at = now + self.session_duration;
        let session_id = Uuid::new_v4();
        self.repository
            .create_session(AdminSession {
                id: session_id,
                admin_user_id: account.id,
                expires_at,
                revoked_at: None,
                created_at: now,
            })
            .await?;
        let access_token = self.token_port.issue_token(
            account.id,
            session_id,
            &account.username,
            &account.role,
            expires_at,
        )?;

        Ok(AdminLoginResult {
            access_token,
            expires_at,
            admin: account.into(),
        })
    }

    pub async fn authenticate(&self, token: &str) -> anyhow::Result<AdminPrincipal> {
        let claims = self.token_port.verify_token(token)?;
        let now = (self.clock)();
        let account = self
            .repository
            .find_active_account_for_session(claims.admin_id, claims.session_id, now)
            .await?
            .filter(|account| account.status == "active")
            .ok_or_else(|| anyhow::anyhow!("admin session is not active"))?;

        Ok(AdminPrincipal {
            admin_id: account.id,
            session_id: claims.session_id,
            username: account.username,
            display_name: account.display_name,
            role: account.role,
            expires_at: claims.expires_at,
        })
    }

    pub async fn authorize(
        &self,
        token: &str,
        permission: AdminPermission,
    ) -> anyhow::Result<AdminPrincipal> {
        let principal = self.authenticate(token).await?;
        if !self
            .authorization_port
            .is_allowed(&principal.role, permission)
        {
            anyhow::bail!("admin permission denied");
        }
        Ok(principal)
    }

    pub async fn logout(&self, token: &str) -> anyhow::Result<()> {
        let principal = self.authenticate(token).await?;
        self.repository
            .revoke_session(principal.admin_id, principal.session_id, (self.clock)())
            .await?;
        Ok(())
    }

    pub async fn ensure_owner(
        &self,
        username: String,
        password: String,
        display_name: String,
    ) -> anyhow::Result<AdminAccount> {
        let username = validate_admin_username(&username)?;
        let password = validate_admin_login_password(&password)?;
        let display_name = display_name.trim().to_string();
        if display_name.is_empty() {
            anyhow::bail!("admin display name is required");
        }
        let password_hash = self.password_port.hash_password(&password)?;
        self.repository
            .ensure_owner(AdminBootstrapOwner {
                username,
                password_hash,
                display_name,
            })
            .await
    }
}

impl From<AdminAccount> for AdminIdentity {
    fn from(value: AdminAccount) -> Self {
        Self {
            id: value.id,
            username: value.username,
            display_name: value.display_name,
            role: value.role,
        }
    }
}
