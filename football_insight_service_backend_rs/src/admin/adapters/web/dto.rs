use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::admin::{
    application::admin_auth_service::{AdminIdentity, AdminLoginInput, AdminLoginResult},
    domain::{
        admin_audit::{AdminAuditLog, AdminAuditLogPage},
        admin_user::{
            AdminCreateUserInput, AdminInviter, AdminMembershipAdjustment, AdminPaymentOrder,
            AdminReferredUser, AdminSubscription, AdminUpdateUserInput, AdminUser,
            AdminUserActivity, AdminUserDetail, AdminUserDevice, AdminUserList, AdminUserSearch,
            normalize_optional_text, normalize_page, normalize_page_size,
        },
    },
};

#[derive(Debug, Deserialize)]
pub struct AdminAuditListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AdminAuditLogDto {
    pub id: String,
    pub admin_user_id: String,
    pub admin_username: String,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub reason: Option<String>,
    pub created_at: String,
}

impl From<AdminAuditLog> for AdminAuditLogDto {
    fn from(value: AdminAuditLog) -> Self {
        Self {
            id: value.id.to_string(),
            admin_user_id: value.admin_user_id.to_string(),
            admin_username: value.admin_username,
            action: value.action,
            target_type: value.target_type,
            target_id: value.target_id,
            reason: value.reason,
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminAuditPageDto {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub items: Vec<AdminAuditLogDto>,
}

impl From<AdminAuditLogPage> for AdminAuditPageDto {
    fn from(value: AdminAuditLogPage) -> Self {
        Self {
            total: value.total,
            page: value.page,
            page_size: value.page_size,
            items: value.items.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AdminLoginRequestDto {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminReasonRequestDto {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminMembershipAdjustmentRequestDto {
    pub membership_tier: String,
    pub expiration_mode: String,
    pub membership_expires_at: Option<DateTime<Utc>>,
    pub reason: String,
}

impl TryFrom<AdminMembershipAdjustmentRequestDto> for AdminMembershipAdjustment {
    type Error = anyhow::Error;

    fn try_from(value: AdminMembershipAdjustmentRequestDto) -> Result<Self, Self::Error> {
        let (membership_expires_at, membership_expires_at_set) =
            match value.expiration_mode.as_str() {
                "preserve" => (None, false),
                "never" => (None, true),
                "specific" => (
                    Some(value.membership_expires_at.ok_or_else(|| {
                        anyhow::anyhow!("membership_expires_at is required for specific expiration")
                    })?),
                    true,
                ),
                _ => anyhow::bail!("expiration_mode must be preserve, never, or specific"),
            };
        Ok(Self {
            membership_tier: value.membership_tier,
            membership_expires_at,
            membership_expires_at_set,
            reason: value.reason,
        })
    }
}

impl From<AdminLoginRequestDto> for AdminLoginInput {
    fn from(value: AdminLoginRequestDto) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminIdentityDto {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
}

impl From<AdminIdentity> for AdminIdentityDto {
    fn from(value: AdminIdentity) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            display_name: value.display_name,
            role: value.role,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminAuthResponseDto {
    pub access_token: String,
    pub expires_at: String,
    pub admin: AdminIdentityDto,
}

impl From<AdminLoginResult> for AdminAuthResponseDto {
    fn from(value: AdminLoginResult) -> Self {
        Self {
            access_token: value.access_token,
            expires_at: value.expires_at.to_rfc3339(),
            admin: value.admin.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AdminUserListQuery {
    pub query: Option<String>,
    pub display_name: Option<String>,
    pub status: Option<String>,
    pub membership_tier: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl From<AdminUserListQuery> for AdminUserSearch {
    fn from(value: AdminUserListQuery) -> Self {
        Self {
            query: normalize_optional_text(value.query)
                .or_else(|| normalize_optional_text(value.display_name)),
            status: normalize_optional_text(value.status),
            membership_tier: normalize_optional_text(value.membership_tier)
                .map(|value| value.to_uppercase()),
            page: normalize_page(value.page),
            page_size: normalize_page_size(value.page_size),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AdminCreateUserRequestDto {
    pub account_identifier: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub password: String,
    pub membership_tier: String,
    pub membership_expires_at: Option<DateTime<Utc>>,
}

impl From<AdminCreateUserRequestDto> for AdminCreateUserInput {
    fn from(value: AdminCreateUserRequestDto) -> Self {
        Self {
            account_identifier: value.account_identifier.trim().to_string(),
            display_name: value.display_name,
            avatar_url: value.avatar_url,
            password: value.password,
            membership_tier: value.membership_tier,
            membership_expires_at: value.membership_expires_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequestDto {
    pub account_identifier: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<Option<String>>,
    pub membership_tier: Option<String>,
    pub membership_expires_at: Option<DateTime<Utc>>,
}

impl From<AdminUpdateUserRequestDto> for AdminUpdateUserInput {
    fn from(value: AdminUpdateUserRequestDto) -> Self {
        Self {
            account_identifier: value
                .account_identifier
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty()),
            display_name: value.display_name,
            avatar_url: value.avatar_url,
            membership_tier: value.membership_tier,
            membership_expires_at: value.membership_expires_at,
            membership_expires_at_set: false,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminUserListResponseDto {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub items: Vec<AdminUserDto>,
}

impl From<AdminUserList> for AdminUserListResponseDto {
    fn from(value: AdminUserList) -> Self {
        Self {
            total: value.total,
            page: value.page,
            page_size: value.page_size,
            items: value.items.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminUserDto {
    pub id: String,
    pub account_identifier: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub has_wechat_binding: bool,
    pub status: String,
    pub invite_code: Option<String>,
    pub invited_by: Option<AdminInviterDto>,
    pub membership_tier: String,
    pub membership_expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct AdminUserDetailDto {
    #[serde(flatten)]
    pub user: AdminUserDto,
    pub referrals: Vec<AdminReferredUserDto>,
    pub activity: Option<AdminUserActivityDto>,
    pub orders: Vec<AdminPaymentOrderDto>,
    pub subscriptions: Vec<AdminSubscriptionDto>,
    pub devices: Vec<AdminUserDeviceDto>,
}

#[derive(Debug, Serialize)]
pub struct AdminReferredUserDto {
    pub id: String,
    pub account_identifier: String,
    pub display_name: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct AdminUserActivityDto {
    pub last_login_at: Option<String>,
    pub last_active_at: Option<String>,
    pub last_active_page_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminPaymentOrderDto {
    pub order_no: String,
    pub amount_cents: i32,
    pub status: String,
    pub product_type: String,
    pub paid_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct AdminSubscriptionDto {
    pub id: String,
    pub plan_code: String,
    pub scope: String,
    pub team_code: String,
    pub season: Option<i32>,
    pub match_id: Option<i64>,
    pub status: String,
    pub starts_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminUserDeviceDto {
    pub id: i64,
    pub platform: String,
    pub masked_device_token: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<AdminUserDetail> for AdminUserDetailDto {
    fn from(value: AdminUserDetail) -> Self {
        Self {
            user: value.user.into(),
            referrals: value.referrals.into_iter().map(Into::into).collect(),
            activity: value.activity.map(Into::into),
            orders: value.orders.into_iter().map(Into::into).collect(),
            subscriptions: value.subscriptions.into_iter().map(Into::into).collect(),
            devices: value.devices.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<AdminReferredUser> for AdminReferredUserDto {
    fn from(value: AdminReferredUser) -> Self {
        Self {
            id: value.id.to_string(),
            account_identifier: value.account_identifier.clone(),
            display_name: value.display_name.unwrap_or(value.account_identifier),
            status: value.status,
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

impl From<AdminUserActivity> for AdminUserActivityDto {
    fn from(value: AdminUserActivity) -> Self {
        Self {
            last_login_at: value.last_login_at.map(|item| item.to_rfc3339()),
            last_active_at: value.last_active_at.map(|item| item.to_rfc3339()),
            last_active_page_key: value.last_active_page_key,
        }
    }
}

impl From<AdminPaymentOrder> for AdminPaymentOrderDto {
    fn from(value: AdminPaymentOrder) -> Self {
        Self {
            order_no: value.order_no,
            amount_cents: value.amount_cents,
            status: value.status,
            product_type: value.product_type,
            paid_at: value.paid_at.map(|item| item.to_rfc3339()),
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

impl From<AdminSubscription> for AdminSubscriptionDto {
    fn from(value: AdminSubscription) -> Self {
        Self {
            id: value.id.to_string(),
            plan_code: value.plan_code,
            scope: value.scope,
            team_code: value.team_code,
            season: value.season,
            match_id: value.match_id,
            status: value.status,
            starts_at: value.starts_at.to_rfc3339(),
            expires_at: value.expires_at.map(|item| item.to_rfc3339()),
        }
    }
}

impl From<AdminUserDevice> for AdminUserDeviceDto {
    fn from(value: AdminUserDevice) -> Self {
        Self {
            id: value.id,
            platform: value.platform,
            masked_device_token: value.masked_device_token,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminInviterDto {
    pub id: String,
    pub display_name: String,
    pub account_identifier: String,
    pub referral_invite_code: String,
}

impl From<AdminInviter> for AdminInviterDto {
    fn from(value: AdminInviter) -> Self {
        let display_name = value
            .display_name
            .unwrap_or_else(|| value.account_identifier.clone());

        Self {
            id: value.id.to_string(),
            display_name,
            account_identifier: value.account_identifier,
            referral_invite_code: value.referral_invite_code,
        }
    }
}

impl From<AdminUser> for AdminUserDto {
    fn from(value: AdminUser) -> Self {
        let display_name = value
            .display_name
            .unwrap_or_else(|| value.account_identifier.clone());

        Self {
            id: value.id.to_string(),
            account_identifier: value.account_identifier,
            display_name,
            avatar_url: value.avatar_url,
            has_wechat_binding: value.has_wechat_binding,
            status: value.status,
            invite_code: value.invite_code,
            invited_by: value.invited_by.map(Into::into),
            membership_tier: value.membership_tier,
            membership_expires_at: value.membership_expires_at.map(|item| item.to_rfc3339()),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}
