use serde::Serialize;

use crate::auth::application::login_with_mini_wechat::MiniWechatLoginResult;
use crate::auth::domain::user::{AuthTokenBundle, AuthUser};

#[derive(Debug, Serialize)]
pub struct AuthResponseDto {
    pub access_token: String,
    pub expires_at: String,
    pub user: CurrentUserDto,
}

#[derive(Debug, Serialize)]
pub struct CurrentUserDto {
    pub id: String,
    pub display_name: String,
    pub account_identifier: String,
    pub invite_code: Option<String>,
    pub avatar_url: Option<String>,
    pub has_wechat_binding: bool,
    pub membership_tier: String,
    pub membership_expires_at: Option<String>,
    pub membership_benefits_enabled: bool,
    pub ticket_watch_poll_interval_seconds: i32,
    pub created_at: String,
}

impl From<AuthUser> for CurrentUserDto {
    fn from(value: AuthUser) -> Self {
        Self {
            id: value.id.to_string(),
            display_name: value
                .display_name
                .unwrap_or_else(|| value.account_identifier.clone()),
            account_identifier: value.account_identifier,
            invite_code: value.invite_code,
            avatar_url: value.avatar_url,
            has_wechat_binding: value.has_wechat_binding,
            membership_tier: value.membership_tier,
            membership_expires_at: value
                .membership_expires_at
                .map(|expires_at| expires_at.to_rfc3339()),
            membership_benefits_enabled: value.membership_benefits_enabled,
            ticket_watch_poll_interval_seconds: value.ticket_watch_poll_interval_seconds,
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

impl From<AuthTokenBundle> for AuthResponseDto {
    fn from(value: AuthTokenBundle) -> Self {
        Self {
            access_token: value.access_token,
            expires_at: value.expires_at.to_rfc3339(),
            user: value.user.into(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MiniWechatLoginResponseDto {
    Authenticated {
        access_token: String,
        expires_at: String,
        user: CurrentUserDto,
    },
    BindingRequired {
        bind_token: String,
        expires_at: String,
        display_name: Option<String>,
        avatar_url: Option<String>,
    },
}

impl From<MiniWechatLoginResult> for MiniWechatLoginResponseDto {
    fn from(value: MiniWechatLoginResult) -> Self {
        match value {
            MiniWechatLoginResult::Authenticated(bundle) => Self::Authenticated {
                access_token: bundle.access_token,
                expires_at: bundle.expires_at.to_rfc3339(),
                user: bundle.user.into(),
            },
            MiniWechatLoginResult::BindingRequired {
                bind_token,
                expires_at,
                profile,
            } => Self::BindingRequired {
                bind_token,
                expires_at: expires_at.to_rfc3339(),
                display_name: profile.display_name,
                avatar_url: profile.avatar_url,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::Value;
    use uuid::Uuid;

    use super::CurrentUserDto;
    use crate::auth::domain::user::AuthUser;

    #[test]
    fn current_user_dto_serializes_account_identifier_field() {
        let user = AuthUser {
            id: Uuid::new_v4(),
            account_identifier: "footballfan".to_string(),
            display_name: Some("Football Fan".to_string()),
            invite_code: Some("FI-ABC123-XYZ789".to_string()),
            avatar_url: None,
            has_wechat_binding: false,
            membership_tier: "V1".to_string(),
            membership_expires_at: None,
            membership_benefits_enabled: false,
            ticket_watch_poll_interval_seconds: 600,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json =
            serde_json::to_value(CurrentUserDto::from(user)).expect("serialize current user");

        assert_eq!(
            json.get("account_identifier"),
            Some(&Value::String("footballfan".to_string()))
        );
        assert_eq!(
            json.get("membership_tier"),
            Some(&Value::String("V1".to_string()))
        );
        assert_eq!(json.get("membership_expires_at"), Some(&Value::Null));
        assert_eq!(
            json.get("invite_code"),
            Some(&Value::String("FI-ABC123-XYZ789".to_string()))
        );
        assert_eq!(
            json.get("membership_benefits_enabled"),
            Some(&Value::Bool(false))
        );
        assert_eq!(
            json.get("ticket_watch_poll_interval_seconds"),
            Some(&Value::Number(600.into()))
        );
        assert!(json.get("phone_number").is_none());
    }
}
