use crate::share_code::{
    domain::{CodeImage, ShareCodeError},
    ports::TicketWatchCodePort,
};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

type Cache = Option<(Instant, Result<CodeImage, ShareCodeError>)>;

pub struct WechatTicketWatchCode {
    app_id: String,
    secret: String,
    client: reqwest::Client,
    cache: Mutex<Cache>,
    api_base: String,
}

impl WechatTicketWatchCode {
    pub fn new(app_id: String, secret: String) -> Self {
        Self {
            app_id,
            secret,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("valid share code HTTP client"),
            cache: Mutex::new(None),
            api_base: "https://api.weixin.qq.com".into(),
        }
    }

    async fn fetch(&self) -> Result<CodeImage, ShareCodeError> {
        if self.app_id.trim().is_empty() || self.secret.trim().is_empty() {
            return Err(ShareCodeError::Unavailable);
        }
        // Reuse WeChat's stable token; never invalidate credentials used by other features.
        let token: Value = self.client
            .post(format!("{}/cgi-bin/stable_token", self.api_base))
            .json(&json!({"grant_type":"client_credential","appid":self.app_id,"secret":self.secret,"force_refresh":false}))
            .send().await.map_err(|_| ShareCodeError::Unavailable)?
            .error_for_status().map_err(|_| ShareCodeError::Unavailable)?
            .json().await.map_err(|_| ShareCodeError::Unavailable)?;
        let token = token
            .get("access_token")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or(ShareCodeError::Unavailable)?;
        let response = self.client
            .post(format!("{}/wxa/getwxacodeunlimit", self.api_base))
            .query(&[("access_token", token)])
            .json(&json!({"scene":"s=share&c=ticket_poster","page":"pages/ticket-watch/index","check_path":false,"env_version":"release","width":280}))
            .send().await.map_err(|_| ShareCodeError::Unavailable)?
            .error_for_status().map_err(|_| ShareCodeError::Unavailable)?;
        let bytes = response
            .bytes()
            .await
            .map_err(|_| ShareCodeError::Unavailable)?;
        decode_image(&bytes)
    }
}

fn decode_image(bytes: &[u8]) -> Result<CodeImage, ShareCodeError> {
    let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(&[255, 216, 255]) {
        "image/jpeg"
    } else {
        return Err(ShareCodeError::Unavailable);
    };
    if bytes.len() > 1024 * 1024 {
        return Err(ShareCodeError::Unavailable);
    }
    Ok(CodeImage {
        image_base64: STANDARD.encode(bytes),
        mime_type: mime.into(),
    })
}

#[async_trait]
impl TicketWatchCodePort for WechatTicketWatchCode {
    async fn image(&self) -> Result<CodeImage, ShareCodeError> {
        // Single fixed scene needs one cache slot; holding the lock also coalesces concurrent fetches.
        let mut cache = self.cache.lock().await;
        if let Some((until, result)) = cache.as_ref()
            && *until > Instant::now()
        {
            return result.clone();
        }
        let result = self.fetch().await;
        if result.is_err() {
            tracing::warn!("ticket watch share code unavailable");
        }
        let ttl = if result.is_ok() { 86400 } else { 30 };
        *cache = Some((Instant::now() + Duration::from_secs(ttl), result.clone()));
        result
    }
}

#[cfg(test)]
mod tests;
