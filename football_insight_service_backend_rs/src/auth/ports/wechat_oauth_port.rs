use async_trait::async_trait;

use crate::auth::domain::wechat::WechatOauthProfile;

#[async_trait]
pub trait WechatOauthPort: Send + Sync {
    async fn fetch_user_profile(&self, code: &str) -> anyhow::Result<WechatOauthProfile>;
    async fn fetch_mini_program_profile(&self, code: &str) -> anyhow::Result<WechatOauthProfile>;
}
