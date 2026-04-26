use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthTokenClaims {
    pub sub: Uuid,
    pub account_identifier: String,
    pub exp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct WechatBindTokenPayload {
    pub open_id: String,
    pub union_id: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WechatBindTokenClaims {
    pub open_id: String,
    pub union_id: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub exp: DateTime<Utc>,
}

pub trait TokenPort: Send + Sync {
    fn issue_token(
        &self,
        user_id: Uuid,
        account_identifier: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<String>;
    fn verify_token(&self, token: &str) -> anyhow::Result<AuthTokenClaims>;
    fn issue_wechat_bind_token(
        &self,
        payload: WechatBindTokenPayload,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<String>;
    fn verify_wechat_bind_token(&self, token: &str) -> anyhow::Result<WechatBindTokenClaims>;
}
