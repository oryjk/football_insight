use super::super::{
    domain::{CodeImage, RecapError},
    ports::RecapCodePort,
};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
// Stable token reuses the current credential; it does not force refresh tokens used by other features.
const API_BASE: &str = "https://api.weixin.qq.com";
type Cache = HashMap<i32, (Instant, Result<CodeImage, RecapError>)>;
pub struct WechatRecapCode {
    app_id: String,
    secret: String,
    client: reqwest::Client,
    cache: Mutex<Cache>,
    api_base: String,
}
impl WechatRecapCode {
    pub fn new(app_id: String, secret: String) -> Self {
        Self {
            app_id,
            secret,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("valid recap HTTP client"),
            cache: Mutex::new(HashMap::new()),
            api_base: API_BASE.into(),
        }
    }
    async fn fetch(&self, season: i32) -> Result<CodeImage, RecapError> {
        if self.app_id.is_empty() || self.secret.is_empty() {
            return Err(RecapError::Unavailable);
        }
        let token:Value=self.client.post(format!("{}/cgi-bin/stable_token", self.api_base)).json(&json!({"grant_type":"client_credential","appid":self.app_id,"secret":self.secret,"force_refresh":false})).send().await.map_err(|_|RecapError::Unavailable)?.error_for_status().map_err(|_|RecapError::Unavailable)?.json().await.map_err(|_|RecapError::Unavailable)?;
        let token = token
            .get("access_token")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
            .ok_or(RecapError::Unavailable)?;
        let response = self
            .client
            .post(format!("{}/wxa/getwxacodeunlimit", self.api_base))
            .query(&[("access_token", token)])
            .json(&code_payload(season))
            .send()
            .await
            .map_err(|_| RecapError::Unavailable)?
            .error_for_status()
            .map_err(|_| RecapError::Unavailable)?;
        let bytes = response
            .bytes()
            .await
            .map_err(|_| RecapError::Unavailable)?;
        decode_image(&bytes)
    }
}
fn code_payload(season: i32) -> Value {
    json!({"scene":format!("s=share&c=recap{season}"),"page":"pages/season-recap/index","check_path":false,"env_version":"release","width":280})
}
fn decode_image(bytes: &[u8]) -> Result<CodeImage, RecapError> {
    let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(&[255, 216, 255]) {
        "image/jpeg"
    } else {
        return Err(RecapError::Unavailable);
    };
    if bytes.len() > 1024 * 1024 {
        return Err(RecapError::Unavailable);
    }
    Ok(CodeImage {
        image_base64: STANDARD.encode(bytes),
        mime_type: mime.into(),
    })
}
#[async_trait]
impl RecapCodePort for WechatRecapCode {
    async fn png_base64(&self, season: i32) -> Result<CodeImage, RecapError> {
        let mut cache = self.cache.lock().await;
        if let Some((until, result)) = cache.get(&season)
            && *until > Instant::now()
        {
            return result.clone();
        }
        let result = self.fetch(season).await;
        if result.is_err() {
            tracing::warn!("season recap mini code unavailable");
        }
        let ttl = if result.is_ok() { 86400 } else { 30 };
        cache.insert(
            season,
            (Instant::now() + Duration::from_secs(ttl), result.clone()),
        );
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn code_has_real_mini_page_and_attributable_short_scene_without_identity() {
        let body = code_payload(2026);
        assert_eq!(body["page"], "pages/season-recap/index");
        assert_eq!(body["scene"], "s=share&c=recap2026");
        assert!(body["scene"].as_str().unwrap().len() <= 32);
        assert_eq!(body["env_version"], "release");
    }
    #[test]
    fn rejects_wechat_json_errors_but_accepts_image_formats() {
        assert!(decode_image(br#"{"errcode":40001,"errmsg":"invalid credential"}"#).is_err());
        assert_eq!(
            decode_image(b"\x89PNG\r\n\x1a\nimage").unwrap().mime_type,
            "image/png"
        );
        assert_eq!(
            decode_image(&[255, 216, 255, 0]).unwrap().mime_type,
            "image/jpeg"
        );
    }
}

#[cfg(test)]
mod http_tests {
    use super::*;
    use axum::{
        Json, Router,
        extract::{Query, State},
        routing::post,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    #[derive(Default)]
    struct Calls {
        tokens: AtomicUsize,
        codes: AtomicUsize,
    }
    async fn token(State(calls): State<Arc<Calls>>, Json(body): Json<Value>) -> Json<Value> {
        assert_eq!(body["appid"], "test-app");
        assert_eq!(body["secret"], "test-secret");
        assert_eq!(body["force_refresh"], false);
        calls.tokens.fetch_add(1, Ordering::SeqCst);
        Json(json!({"access_token":"mock-token"}))
    }
    async fn code(
        State(calls): State<Arc<Calls>>,
        Query(query): Query<HashMap<String, String>>,
        Json(body): Json<Value>,
    ) -> (axum::http::HeaderMap, Vec<u8>) {
        assert_eq!(query["access_token"], "mock-token");
        assert_eq!(body, code_payload(2026));
        calls.codes.fetch_add(1, Ordering::SeqCst);
        (
            axum::http::HeaderMap::new(),
            b"\x89PNG\r\n\x1a\nfixture".to_vec(),
        )
    }
    #[tokio::test]
    async fn real_http_serializes_code_contract_and_reuses_cached_image() {
        let calls = Arc::new(Calls::default());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new()
            .route("/cgi-bin/stable_token", post(token))
            .route("/wxa/getwxacodeunlimit", post(code))
            .with_state(calls.clone());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let mut adapter = WechatRecapCode::new("test-app".into(), "test-secret".into());
        adapter.api_base = format!("http://{address}");
        adapter.client = reqwest::Client::builder().no_proxy().build().unwrap();
        let first = adapter.png_base64(2026).await.unwrap();
        let second = adapter.png_base64(2026).await.unwrap();
        assert_eq!(first.image_base64, second.image_base64);
        assert_eq!(first.mime_type, "image/png");
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 1);
        assert_eq!(calls.codes.load(Ordering::SeqCst), 1);
        server.abort();
    }
    #[tokio::test]
    async fn token_errors_return_unavailable_and_are_cooldown_cached() {
        let calls = Arc::new(Calls::default());
        let copy = calls.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new().route(
            "/cgi-bin/stable_token",
            post(move || {
                let c = copy.clone();
                async move {
                    c.tokens.fetch_add(1, Ordering::SeqCst);
                    Json(json!({"errcode":40013}))
                }
            }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let mut adapter = WechatRecapCode::new("test-app".into(), "test-secret".into());
        adapter.api_base = format!("http://{address}");
        adapter.client = reqwest::Client::builder().no_proxy().build().unwrap();
        for _ in 0..2 {
            assert!(matches!(
                adapter.png_base64(2026).await,
                Err(RecapError::Unavailable)
            ));
        }
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 1);
        server.abort();
    }
}
