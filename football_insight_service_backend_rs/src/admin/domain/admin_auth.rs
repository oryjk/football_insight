use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAccount {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminPrincipal {
    pub admin_id: Uuid,
    pub session_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminSession {
    pub id: Uuid,
    pub admin_user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminBootstrapOwner {
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminTokenClaims {
    pub admin_id: Uuid,
    pub session_id: Uuid,
    pub username: String,
    pub role: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminPermission {
    ManageUsers,
    ViewAuditLogs,
}

pub fn validate_admin_username(value: &str) -> anyhow::Result<String> {
    let normalized = value.trim().to_lowercase();
    let valid = (3..=64).contains(&normalized.len())
        && normalized
            .chars()
            .all(|item| item.is_ascii_alphanumeric() || matches!(item, '_' | '-' | '.'));
    if !valid {
        anyhow::bail!(
            "admin username must be 3 to 64 letters, numbers, dots, dashes, or underscores"
        );
    }
    Ok(normalized)
}

pub fn validate_admin_login_password(value: &str) -> anyhow::Result<String> {
    if value.chars().count() < 8 {
        anyhow::bail!("admin password must be at least 8 characters");
    }
    Ok(value.to_string())
}
