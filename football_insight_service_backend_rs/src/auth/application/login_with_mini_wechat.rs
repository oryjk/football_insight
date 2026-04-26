use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::auth::{
    domain::{user::AuthTokenBundle, wechat::WechatOauthProfile},
    ports::{
        auth_repository::AuthRepository,
        token_port::{TokenPort, WechatBindTokenPayload},
        wechat_oauth_port::WechatOauthPort,
    },
};

#[derive(Debug, Clone)]
pub struct CompleteMiniWechatLoginInput {
    pub code: String,
}

#[derive(Debug, Clone)]
pub enum MiniWechatLoginResult {
    Authenticated(AuthTokenBundle),
    BindingRequired {
        bind_token: String,
        expires_at: chrono::DateTime<Utc>,
        profile: WechatOauthProfile,
    },
}

pub struct CompleteMiniWechatLoginUseCase {
    repository: Arc<dyn AuthRepository>,
    wechat_oauth_port: Arc<dyn WechatOauthPort>,
    token_port: Arc<dyn TokenPort>,
    session_ttl: Duration,
    bind_ttl: Duration,
}

impl CompleteMiniWechatLoginUseCase {
    pub fn new(
        repository: Arc<dyn AuthRepository>,
        wechat_oauth_port: Arc<dyn WechatOauthPort>,
        token_port: Arc<dyn TokenPort>,
        session_ttl: Duration,
        bind_ttl: Duration,
    ) -> Self {
        Self {
            repository,
            wechat_oauth_port,
            token_port,
            session_ttl,
            bind_ttl,
        }
    }

    pub async fn execute(
        &self,
        input: CompleteMiniWechatLoginInput,
    ) -> anyhow::Result<MiniWechatLoginResult> {
        if input.code.trim().is_empty() {
            anyhow::bail!("wechat code is required");
        }

        let profile = self
            .wechat_oauth_port
            .fetch_mini_program_profile(&input.code)
            .await?;

        if let Some(user) = self
            .repository
            .find_user_by_wechat_open_id(&profile.open_id)
            .await?
        {
            self.repository.record_user_login(user.id).await?;

            let expires_at = Utc::now() + self.session_ttl;
            let access_token =
                self.token_port
                    .issue_token(user.id, &user.account_identifier, expires_at)?;

            return Ok(MiniWechatLoginResult::Authenticated(AuthTokenBundle {
                user,
                access_token,
                expires_at,
            }));
        }

        let expires_at = Utc::now() + self.bind_ttl;
        let bind_token = self.token_port.issue_wechat_bind_token(
            WechatBindTokenPayload {
                open_id: profile.open_id.clone(),
                union_id: profile.union_id.clone(),
                display_name: profile.display_name.clone(),
                avatar_url: profile.avatar_url.clone(),
            },
            expires_at,
        )?;

        Ok(MiniWechatLoginResult::BindingRequired {
            bind_token,
            expires_at,
            profile,
        })
    }
}
