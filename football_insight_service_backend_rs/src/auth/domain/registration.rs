use super::{user::AuthUser, wechat::WechatOauthProfile};

/// Carries the actual INSERT/UPDATE outcome; reactivation is not new acquisition.
pub struct RegisteredUser {
    pub user: AuthUser,
    pub newly_created: bool,
}

pub struct UserRegistration<'a> {
    pub invite_code: Option<&'a str>,
    pub referral_code: Option<&'a str>,
    pub identity: RegistrationIdentity<'a>,
}

pub enum RegistrationIdentity<'a> {
    Password {
        account_identifier: &'a str,
        password_hash: &'a str,
    },
    Wechat {
        phone_number: &'a str,
        password_hash: &'a str,
        profile: &'a WechatOauthProfile,
    },
    MiniWechat {
        profile: &'a WechatOauthProfile,
    },
}
