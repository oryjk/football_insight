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
pub struct AdminReferredUser {
    pub id: Uuid,
    pub account_identifier: String,
    pub display_name: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminUserActivity {
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub last_active_page_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminPaymentOrder {
    pub order_no: String,
    pub amount_cents: i32,
    pub status: String,
    pub product_type: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminSubscription {
    pub id: Uuid,
    pub plan_code: String,
    pub scope: String,
    pub team_code: String,
    pub season: Option<i32>,
    pub match_id: Option<i64>,
    pub status: String,
    pub starts_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminUserDevice {
    pub id: i64,
    pub platform: String,
    pub masked_device_token: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminUserDetail {
    pub user: AdminUser,
    pub referrals: Vec<AdminReferredUser>,
    pub activity: Option<AdminUserActivity>,
    pub orders: Vec<AdminPaymentOrder>,
    pub subscriptions: Vec<AdminSubscription>,
    pub devices: Vec<AdminUserDevice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminUserSearch {
    pub query: Option<String>,
    pub status: Option<String>,
    pub membership_tier: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminMembershipAdjustment {
    pub membership_tier: String,
    pub membership_expires_at: Option<DateTime<Utc>>,
    pub membership_expires_at_set: bool,
    pub reason: String,
}

pub fn validate_admin_reason(value: &str) -> anyhow::Result<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        anyhow::bail!("reason is required");
    }
    if value.chars().count() > 500 {
        anyhow::bail!("reason is too long");
    }
    Ok(value)
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
