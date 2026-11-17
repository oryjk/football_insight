use std::{
    fmt::{Display, Formatter},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ModerationScene {
    Profile = 1,
    Comment = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentSecurityError {
    Rejected,
    Unavailable,
}

impl Display for ContentSecurityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected => f.write_str("内容未通过安全检测"),
            Self::Unavailable => f.write_str("内容安全校验服务暂不可用，请稍后重试"),
        }
    }
}

impl std::error::Error for ContentSecurityError {}

#[async_trait]
pub trait ContentSecurityPort: Send + Sync {
    async fn check_text(
        &self,
        scene: ModerationScene,
        open_id: Option<&str>,
        text: &str,
    ) -> Result<(), ContentSecurityError>;

    async fn check_image(&self, bytes: &[u8], mime_type: &str) -> Result<(), ContentSecurityError>;
}

pub struct NoopContentSecurityPort;

#[async_trait]
impl ContentSecurityPort for NoopContentSecurityPort {
    async fn check_text(
        &self,
        _scene: ModerationScene,
        _open_id: Option<&str>,
        _text: &str,
    ) -> Result<(), ContentSecurityError> {
        Ok(())
    }

    async fn check_image(&self, _bytes: &[u8], _mime_type: &str) -> Result<(), ContentSecurityError> {
        Ok(())
    }
}

struct CachedAccessToken {
    token: String,
    expires_at: Instant,
}

pub struct WechatContentSecurityAdapter {
    app_id: String,
    secret: String,
    api_base: String,
    client: reqwest::Client,
    token_cache: Mutex<Option<CachedAccessToken>>,
}

impl WechatContentSecurityAdapter {
    pub fn new(app_id: String, secret: String) -> Self {
        Self::with_api_base(app_id, secret, "https://api.weixin.qq.com".to_string())
    }

    pub fn with_api_base(app_id: String, secret: String, api_base: String) -> Self {
        Self {
            app_id: app_id.trim().to_string(),
            secret: secret.trim().to_string(),
            api_base: api_base.trim_end_matches('/').to_string(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(6))
                .build()
                .expect("valid content security HTTP client"),
            token_cache: Mutex::new(None),
        }
    }

    fn credentials_configured(&self) -> bool {
        self.app_id.starts_with("wx")
            && self.app_id.len() == 18
            && self.app_id.chars().all(|c| c.is_ascii_alphanumeric())
            && !self.secret.is_empty()
            && self.secret != "test"
    }

    async fn access_token(&self, force_refresh: bool) -> Result<String, ContentSecurityError> {
        let mut cache = self.token_cache.lock().await;
        if !force_refresh
            && let Some(cached) = cache.as_ref()
            && cached.expires_at > Instant::now()
        {
            return Ok(cached.token.clone());
        }

        let response = self
            .client
            .post(format!("{}/cgi-bin/stable_token", self.api_base))
            .json(&json!({
                "grant_type": "client_credential",
                "appid": self.app_id,
                "secret": self.secret,
                "force_refresh": false,
            }))
            .send()
            .await
            .map_err(|error| {
                tracing::warn!(%error, "wechat stable_token request failed");
                ContentSecurityError::Unavailable
            })?
            .error_for_status()
            .map_err(|error| {
                tracing::warn!(%error, "wechat stable_token returned error status");
                ContentSecurityError::Unavailable
            })?
            .json::<Value>()
            .await
            .map_err(|error| {
                tracing::warn!(%error, "wechat stable_token response parse failed");
                ContentSecurityError::Unavailable
            })?;

        let token = response
            .get("access_token")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                tracing::warn!(body = %response, "wechat stable_token missing access_token");
                ContentSecurityError::Unavailable
            })?
            .to_string();

        let expires_in = response
            .get("expires_in")
            .and_then(Value::as_u64)
            .unwrap_or(7200);
        let ttl_secs = expires_in.saturating_sub(120).max(60);
        *cache = Some(CachedAccessToken {
            token: token.clone(),
            expires_at: Instant::now() + Duration::from_secs(ttl_secs),
        });
        Ok(token)
    }

    async fn call_msg_sec_check(
        &self,
        access_token: &str,
        payload: &Value,
    ) -> Result<Value, ContentSecurityError> {
        self.client
            .post(format!("{}/wxa/msg_sec_check", self.api_base))
            .query(&[("access_token", access_token)])
            .json(payload)
            .send()
            .await
            .map_err(|error| {
                tracing::warn!(%error, "wechat msg_sec_check request failed");
                ContentSecurityError::Unavailable
            })?
            .error_for_status()
            .map_err(|error| {
                tracing::warn!(%error, "wechat msg_sec_check returned error status");
                ContentSecurityError::Unavailable
            })?
            .json::<Value>()
            .await
            .map_err(|error| {
                tracing::warn!(%error, "wechat msg_sec_check parse failed");
                ContentSecurityError::Unavailable
            })
    }

    async fn check_text_v1(&self, text: &str) -> Result<(), ContentSecurityError> {
        let payload = json!({ "content": text });
        for attempt in 0..2 {
            let token = self.access_token(attempt > 0).await?;
            let body = self.call_msg_sec_check(&token, &payload).await?;
            let errcode = body.get("errcode").and_then(Value::as_i64).unwrap_or(0);
            match errcode {
                0 => return Ok(()),
                87014 => return Err(ContentSecurityError::Rejected),
                40001 | 40014 | 42001 if attempt == 0 => continue,
                other => {
                    tracing::warn!(errcode = other, errmsg = ?body.get("errmsg"), "wechat v1 msg_sec_check failed");
                    return Err(ContentSecurityError::Unavailable);
                }
            }
        }
        Err(ContentSecurityError::Unavailable)
    }

    async fn check_text_v2(
        &self,
        scene: ModerationScene,
        open_id: &str,
        text: &str,
    ) -> Result<(), ContentSecurityError> {
        let payload = json!({
            "content": text,
            "version": 2,
            "scene": scene as u8,
            "openid": open_id,
        });
        for attempt in 0..2 {
            let token = self.access_token(attempt > 0).await?;
            let body = self.call_msg_sec_check(&token, &payload).await?;
            let errcode = body.get("errcode").and_then(Value::as_i64).unwrap_or(0);
            match errcode {
                0 => {
                    let suggest = body
                        .get("result")
                        .and_then(|result| result.get("suggest"))
                        .and_then(Value::as_str);
                    return if suggest == Some("pass") {
                        Ok(())
                    } else {
                        Err(ContentSecurityError::Rejected)
                    };
                }
                87014 => return Err(ContentSecurityError::Rejected),
                40003 => return self.check_text_v1(text).await,
                40001 | 40014 | 42001 if attempt == 0 => continue,
                other => {
                    tracing::warn!(errcode = other, errmsg = ?body.get("errmsg"), "wechat v2 msg_sec_check failed");
                    return Err(ContentSecurityError::Unavailable);
                }
            }
        }
        Err(ContentSecurityError::Unavailable)
    }
}

fn build_image_multipart_body(boundary: &str, bytes: &[u8], mime_type: &str) -> Vec<u8> {
    let filename = match mime_type {
        "image/png" => "avatar.png",
        "image/webp" => "avatar.webp",
        _ => "avatar.jpg",
    };
    let header = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"media\"; filename=\"{filename}\"\r\nContent-Type: {mime_type}\r\n\r\n"
    );
    let footer = format!("\r\n--{boundary}--\r\n");
    let mut body = Vec::with_capacity(header.len() + bytes.len() + footer.len());
    body.extend_from_slice(header.as_bytes());
    body.extend_from_slice(bytes);
    body.extend_from_slice(footer.as_bytes());
    body
}

#[async_trait]
impl ContentSecurityPort for WechatContentSecurityAdapter {
    async fn check_text(
        &self,
        scene: ModerationScene,
        open_id: Option<&str>,
        text: &str,
    ) -> Result<(), ContentSecurityError> {
        if !self.credentials_configured() {
            return Ok(());
        }
        let open_id = open_id.map(str::trim).filter(|value| !value.is_empty());
        match open_id {
            Some(oid) => self.check_text_v2(scene, oid, text).await,
            None => self.check_text_v1(text).await,
        }
    }

    async fn check_image(&self, bytes: &[u8], mime_type: &str) -> Result<(), ContentSecurityError> {
        if !self.credentials_configured() {
            return Ok(());
        }
        for attempt in 0..2 {
            let token = self.access_token(attempt > 0).await?;
            let boundary = format!("----fi{}", Uuid::new_v4().simple());
            let body = build_image_multipart_body(&boundary, bytes, mime_type);
            let response = self
                .client
                .post(format!("{}/wxa/img_sec_check", self.api_base))
                .query(&[("access_token", token.as_str())])
                .header(
                    reqwest::header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(body)
                .send()
                .await
                .map_err(|error| {
                    tracing::warn!(%error, "wechat img_sec_check request failed");
                    ContentSecurityError::Unavailable
                })?
                .error_for_status()
                .map_err(|error| {
                    tracing::warn!(%error, "wechat img_sec_check returned error status");
                    ContentSecurityError::Unavailable
                })?
                .json::<Value>()
                .await
                .map_err(|error| {
                    tracing::warn!(%error, "wechat img_sec_check parse failed");
                    ContentSecurityError::Unavailable
                })?;

            let errcode = response
                .get("errcode")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            match errcode {
                0 => return Ok(()),
                87014 => return Err(ContentSecurityError::Rejected),
                40001 | 40014 | 42001 if attempt == 0 => continue,
                other => {
                    tracing::warn!(errcode = other, errmsg = ?response.get("errmsg"), "wechat img_sec_check failed");
                    return Err(ContentSecurityError::Unavailable);
                }
            }
        }
        Err(ContentSecurityError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::{
        Json, Router,
        body::Bytes,
        extract::State,
        http::HeaderMap,
        routing::post,
    };
    use serde_json::{Value, json};
    use tokio::net::TcpListener;

    use super::{
        ContentSecurityError, ContentSecurityPort, ModerationScene, WechatContentSecurityAdapter,
    };

    #[derive(Default)]
    struct MockWechatState {
        text_calls: Mutex<Vec<Value>>,
        image_calls: Mutex<Vec<(String, Vec<u8>)>>,
    }

    async fn spawn_mock_wechat(state: Arc<MockWechatState>) -> String {
        let app = Router::new()
            .route(
                "/cgi-bin/stable_token",
                post(|| async { Json(json!({ "access_token": "mock-token", "expires_in": 7200 })) }),
            )
            .route(
                "/wxa/msg_sec_check",
                post(
                    |State(state): State<Arc<MockWechatState>>, Json(payload): Json<Value>| async move {
                        state.text_calls.lock().unwrap().push(payload.clone());
                        let content = payload
                            .get("content")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        let openid = payload.get("openid").and_then(Value::as_str);
                        if openid == Some("bad-openid") {
                            return Json(json!({ "errcode": 40003, "errmsg": "invalid openid" }));
                        }
                        if payload.get("version").and_then(Value::as_u64) == Some(2) {
                            let suggest = if content.contains("违规") {
                                "risky"
                            } else {
                                "pass"
                            };
                            return Json(json!({
                                "errcode": 0,
                                "result": { "suggest": suggest, "label": 100 }
                            }));
                        }
                        if content.contains("敏感") {
                            return Json(json!({ "errcode": 87014, "errmsg": "risky content" }));
                        }
                        Json(json!({ "errcode": 0, "errmsg": "ok" }))
                    },
                ),
            )
            .route(
                "/wxa/img_sec_check",
                post(
                    |State(state): State<Arc<MockWechatState>>,
                     headers: HeaderMap,
                     body: Bytes| async move {
                        let content_type = headers
                            .get(axum::http::header::CONTENT_TYPE)
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or_default()
                            .to_string();
                        let raw = body.to_vec();
                        let is_bad = raw.windows(3).any(|w| w == b"BAD");
                        state.image_calls.lock().unwrap().push((content_type, raw));
                        if is_bad {
                            return Json(json!({ "errcode": 87014, "errmsg": "risky image" }));
                        }
                        Json(json!({ "errcode": 0, "errmsg": "ok" }))
                    },
                ),
            )
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn wechat_content_security_handles_v2_pass_reject_and_v1_fallback() {
        let state = Arc::new(MockWechatState::default());
        let base = spawn_mock_wechat(state.clone()).await;
        let adapter = WechatContentSecurityAdapter::with_api_base(
            "wx1234567890abcdef".to_string(),
            "real-secret".to_string(),
            base,
        );

        adapter
            .check_text(ModerationScene::Comment, Some("valid-openid"), "好球")
            .await
            .expect("v2 pass");

        let err = adapter
            .check_text(ModerationScene::Comment, Some("valid-openid"), "违规词")
            .await
            .unwrap_err();
        assert_eq!(err, ContentSecurityError::Rejected);

        adapter
            .check_text(ModerationScene::Profile, Some("bad-openid"), "正常昵称")
            .await
            .expect("40003 falls back to v1");

        let err = adapter
            .check_text(ModerationScene::Profile, Some("bad-openid"), "敏感昵称")
            .await
            .unwrap_err();
        assert_eq!(err, ContentSecurityError::Rejected);

        let calls = state.text_calls.lock().unwrap();
        assert_eq!(calls.len(), 6);
    }

    #[tokio::test]
    async fn wechat_content_security_checks_multipart_image_and_rejects_87014() {
        let state = Arc::new(MockWechatState::default());
        let base = spawn_mock_wechat(state.clone()).await;
        let adapter = WechatContentSecurityAdapter::with_api_base(
            "wx1234567890abcdef".to_string(),
            "real-secret".to_string(),
            base,
        );

        adapter
            .check_image(&[0xff, 0xd8, 0xff, 0xe0], "image/jpeg")
            .await
            .expect("clean image passes");

        let err = adapter
            .check_image(b"\x89PNG\r\n\x1a\nBAD", "image/png")
            .await
            .unwrap_err();
        assert_eq!(err, ContentSecurityError::Rejected);

        let calls = state.image_calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert!(calls[0].0.starts_with("multipart/form-data; boundary=----fi"));
        let body_text = String::from_utf8_lossy(&calls[0].1);
        assert!(body_text.contains("name=\"media\"; filename=\"avatar.jpg\""));
        assert!(body_text.contains("Content-Type: image/jpeg"));
    }

    #[tokio::test]
    async fn skips_remote_checks_when_test_or_non_wechat_credentials() {
        let adapter = WechatContentSecurityAdapter::new("test-app".into(), "test".into());
        adapter
            .check_text(ModerationScene::Comment, Some("oid"), "hello")
            .await
            .unwrap();
        adapter
            .check_image(&[0xff, 0xd8, 0xff], "image/jpeg")
            .await
            .unwrap();
    }
}
