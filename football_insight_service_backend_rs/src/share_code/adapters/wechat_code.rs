use crate::share_code::{
    domain::{CodeImage, ShareCodeError},
    ports::TicketWatchCodePort,
};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

type Cache = Option<(Instant, Result<CodeImage, ShareCodeError>)>;

pub struct WechatMiniCode {
    app_id: String,
    secret: String,
    client: reqwest::Client,
    cache: Mutex<Cache>,
    api_base: String,
    scene: String,
    page: String,
}

/// Preserve the ticket poster's existing constructor and port contract.
pub type WechatTicketWatchCode = WechatMiniCode;

impl WechatMiniCode {
    pub fn new(app_id: String, secret: String) -> Self {
        Self::for_target(
            app_id,
            secret,
            "s=share&c=ticket_poster".into(),
            "pages/ticket-watch/index".into(),
        )
    }

    fn for_target(app_id: String, secret: String, scene: String, page: String) -> Self {
        Self {
            app_id,
            secret,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("valid share code HTTP client"),
            cache: Mutex::new(None),
            api_base: "https://api.weixin.qq.com".into(),
            scene,
            page,
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
            .json(&json!({"scene":self.scene,"page":self.page,"check_path":false,"env_version":"release","width":280}))
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

impl WechatMiniCode {
    async fn cached_image(&self) -> Result<CodeImage, ShareCodeError> {
        // Each fixed scene/page target owns one cache slot and one in-flight lock.
        let mut cache = self.cache.lock().await;
        if let Some((until, result)) = cache.as_ref()
            && *until > Instant::now()
        {
            return result.clone();
        }
        let result = self.fetch().await;
        if result.is_err() {
            tracing::warn!("mini program share code unavailable");
        }
        let ttl = if result.is_ok() { 86400 } else { 30 };
        *cache = Some((Instant::now() + Duration::from_secs(ttl), result.clone()));
        result
    }
}

#[async_trait]
impl TicketWatchCodePort for WechatMiniCode {
    async fn image(&self) -> Result<CodeImage, ShareCodeError> {
        self.cached_image().await
    }
}

pub(crate) const MAX_PER_CODE_TARGETS: usize = 128;

#[derive(Debug, PartialEq, Eq)]
enum PerCodeError {
    Invalid,
    Unavailable,
}

/// 按分享码缓存小程序码：每个码一个缓存槽与在途锁，最多 MAX_PER_CODE_TARGETS 个；
/// 满了淘汰一个没有在途请求的槽。战术板与转会提名共用。
async fn per_code_image(
    codes: &Mutex<HashMap<String, Arc<WechatMiniCode>>>,
    template: (&str, &str, &reqwest::Client, &str),
    code: &str,
    scene: String,
    page: &str,
) -> Result<CodeImage, PerCodeError> {
    if code.len() != 8 || !code.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err(PerCodeError::Invalid);
    }
    let (app_id, secret, client, api_base) = template;
    let adapter = {
        let mut codes = codes.lock().await;
        if let Some(adapter) = codes.get(code) {
            adapter.clone()
        } else {
            if codes.len() >= MAX_PER_CODE_TARGETS {
                // An in-flight entry has an extra Arc owner; keep its coalescing lock.
                let key = codes
                    .iter()
                    .find_map(|(key, entry)| (Arc::strong_count(entry) == 1).then(|| key.clone()))
                    .ok_or(PerCodeError::Unavailable)?;
                codes.remove(&key);
            }
            let mut adapter =
                WechatMiniCode::for_target(app_id.into(), secret.into(), scene, page.into());
            adapter.client = client.clone();
            adapter.api_base = api_base.into();
            let adapter = Arc::new(adapter);
            codes.insert(code.into(), adapter.clone());
            adapter
        }
    };
    adapter
        .cached_image()
        .await
        .map_err(|_| PerCodeError::Unavailable)
}

/// Each target owns its cached result and in-flight lock; unrelated lineups can fetch concurrently.
pub struct WechatLineupCode {
    app_id: String,
    secret: String,
    client: reqwest::Client,
    api_base: String,
    codes: Mutex<HashMap<String, Arc<WechatMiniCode>>>,
}

impl WechatLineupCode {
    pub fn new(app_id: String, secret: String) -> Self {
        let template = WechatMiniCode::new(app_id, secret);
        Self {
            app_id: template.app_id,
            secret: template.secret,
            client: template.client,
            api_base: template.api_base,
            codes: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl crate::lineup::ports::LineupCodePort for WechatLineupCode {
    async fn image(
        &self,
        code: &str,
    ) -> Result<crate::lineup::domain::LineupCodeImage, crate::lineup::domain::LineupPublicError>
    {
        use crate::lineup::domain::{LineupCodeImage, LineupPublicError};
        let image = per_code_image(
            &self.codes,
            (&self.app_id, &self.secret, &self.client, &self.api_base),
            code,
            format!("s=share&c=lineup&l={code}"),
            "pages/lineup/view",
        )
        .await
        .map_err(|error| match error {
            PerCodeError::Invalid => LineupPublicError::Invalid,
            PerCodeError::Unavailable => LineupPublicError::Unavailable,
        })?;
        Ok(LineupCodeImage {
            image_base64: image.image_base64,
            mime_type: image.mime_type,
        })
    }
}

/// 转会提名卡的小程序码：扫码直达 pages/transfer/view，场景值携带提名分享码。
pub struct WechatTransferCode {
    app_id: String,
    secret: String,
    client: reqwest::Client,
    api_base: String,
    codes: Mutex<HashMap<String, Arc<WechatMiniCode>>>,
}

impl WechatTransferCode {
    pub fn new(app_id: String, secret: String) -> Self {
        let template = WechatMiniCode::new(app_id, secret);
        Self {
            app_id: template.app_id,
            secret: template.secret,
            client: template.client,
            api_base: template.api_base,
            codes: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl crate::transfer::ports::TransferCodePort for WechatTransferCode {
    async fn image(
        &self,
        code: &str,
    ) -> Result<crate::transfer::domain::TransferCodeImage, crate::transfer::domain::TransferError>
    {
        use crate::transfer::domain::{TransferCodeImage, TransferError};
        let image = per_code_image(
            &self.codes,
            (&self.app_id, &self.secret, &self.client, &self.api_base),
            code,
            format!("s=share&c=transfer&t={code}"),
            "pages/transfer/view",
        )
        .await
        .map_err(|error| match error {
            PerCodeError::Invalid => TransferError::Invalid,
            PerCodeError::Unavailable => TransferError::Unavailable,
        })?;
        Ok(TransferCodeImage {
            image_base64: image.image_base64,
            mime_type: image.mime_type,
        })
    }
}

#[cfg(test)]
mod tests;
