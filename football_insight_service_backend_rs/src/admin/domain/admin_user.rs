use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminInviter {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub account_identifier: String,
    pub referral_invite_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminUser {
    pub id: Uuid,
    pub account_identifier: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub has_wechat_binding: bool,
    pub status: String,
    pub invite_code: Option<String>,
    pub invited_by: Option<AdminInviter>,
    pub membership_tier: String,
    pub membership_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminUserList {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub items: Vec<AdminUser>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminUserSearch {
    pub display_name: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminCreateUserInput {
    pub account_identifier: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub password: String,
    pub membership_tier: String,
    pub membership_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AdminUpdateUserInput {
    pub account_identifier: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<Option<String>>,
    pub membership_tier: Option<String>,
    pub membership_expires_at: Option<DateTime<Utc>>,
    pub membership_expires_at_set: bool,
}

pub fn normalize_page(value: Option<i64>) -> i64 {
    value.unwrap_or(1).clamp(1, 10_000)
}

pub fn normalize_page_size(value: Option<i64>) -> i64 {
    value.unwrap_or(20).clamp(1, 100)
}

pub fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

pub fn validate_membership_tier(tier: &str) -> anyhow::Result<String> {
    let normalized = tier.trim().to_uppercase();
    let suffix = normalized.strip_prefix('V').unwrap_or_default();
    let Ok(rank) = suffix.parse::<i32>() else {
        anyhow::bail!("membership tier must be V1 to V9");
    };
    if !(1..=9).contains(&rank) {
        anyhow::bail!("membership tier must be V1 to V9");
    }
    Ok(format!("V{rank}"))
}

pub fn validate_display_name(value: &str) -> anyhow::Result<String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        anyhow::bail!("display name is required");
    }
    if normalized.chars().count() > 128 {
        anyhow::bail!("display name is too long");
    }
    Ok(normalized)
}

pub fn validate_admin_password(value: &str) -> anyhow::Result<String> {
    let normalized = value.trim().to_string();
    if normalized.chars().count() < 6 {
        anyhow::bail!("password must be at least 6 characters");
    }
    Ok(normalized)
}
