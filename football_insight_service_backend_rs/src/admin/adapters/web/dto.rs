use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::admin::domain::admin_user::{
    AdminCreateUserInput, AdminInviter, AdminUpdateUserInput, AdminUser, AdminUserList,
    AdminUserSearch, normalize_optional_text, normalize_page, normalize_page_size,
};

#[derive(Debug, Deserialize)]
pub struct AdminUserListQuery {
    pub display_name: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl From<AdminUserListQuery> for AdminUserSearch {
    fn from(value: AdminUserListQuery) -> Self {
        Self {
            display_name: normalize_optional_text(value.display_name),
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
