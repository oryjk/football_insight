use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::auth::{domain::wechat::WechatOauthProfile, ports::wechat_oauth_port::WechatOauthPort};

#[derive(Clone)]
pub struct OfficialWechatOauthPort {
    app_id: String,
    app_secret: String,
    client: Client,
}

impl OfficialWechatOauthPort {
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            app_id,
            app_secret,
            client: Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct WechatOauthAccessTokenResponse {
    openid: Option<String>,
    unionid: Option<String>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WechatMiniProgramSessionResponse {
    openid: Option<String>,
    unionid: Option<String>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[async_trait]
impl WechatOauthPort for OfficialWechatOauthPort {
    async fn fetch_user_profile(&self, code: &str) -> anyhow::Result<WechatOauthProfile> {
        let access_response = self
            .client
            .get("https://api.weixin.qq.com/sns/oauth2/access_token")
            .query(&[
                ("appid", self.app_id.as_str()),
                ("secret", self.app_secret.as_str()),
                ("code", code),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<WechatOauthAccessTokenResponse>()
            .await?;

        if let Some(errcode) = access_response.errcode {
            anyhow::bail!(
                "wechat oauth access token failed: {} {}",
                errcode,
                access_response.errmsg.unwrap_or_default()
            );
        }

        let open_id = access_response
            .openid
            .context("wechat oauth openid missing")?;

        Ok(WechatOauthProfile {
            open_id,
            union_id: access_response.unionid,
            display_name: None,
            avatar_url: None,
        })
    }

    async fn fetch_mini_program_profile(&self, code: &str) -> anyhow::Result<WechatOauthProfile> {
        let session_response = self
            .client
            .get("https://api.weixin.qq.com/sns/jscode2session")
            .query(&[
                ("appid", self.app_id.as_str()),
                ("secret", self.app_secret.as_str()),
                ("js_code", code),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<WechatMiniProgramSessionResponse>()
            .await?;

        if let Some(errcode) = session_response.errcode {
            anyhow::bail!(
                "wechat mini program session failed: {} {}",
                errcode,
                session_response.errmsg.unwrap_or_default()
            );
        }

        let open_id = session_response
            .openid
            .context("wechat mini program openid missing")?;

        Ok(WechatOauthProfile {
            open_id,
            union_id: session_response.unionid,
            display_name: None,
            avatar_url: None,
        })
    }
}
