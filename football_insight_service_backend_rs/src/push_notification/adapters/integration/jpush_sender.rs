use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::Client;

use crate::push_notification::ports::push_sender::{PushPayload, PushSender};

pub struct JPushSender {
    client: Client,
    app_key: String,
    master_secret: String,
}

impl JPushSender {
    pub fn new(app_key: String, master_secret: String) -> Self {
        Self {
            client: Client::new(),
            app_key,
            master_secret,
        }
    }

    fn auth_header(&self) -> String {
        format!(
            "Basic {}",
            STANDARD.encode(format!("{}:{}", self.app_key, self.master_secret))
        )
    }
}

#[async_trait]
impl PushSender for JPushSender {
    async fn send(&self, device_token: &str, payload: &PushPayload) -> anyhow::Result<()> {
        let body = serde_json::json!({
            "platform": "all",
            "audience": { "registration_id": [device_token] },
            "notification": {
                "alert": payload.body,
                "android": { "title": payload.title, "alert": payload.body, "extras": payload.data },
                "ios": { "title": payload.title, "alert": payload.body, "extras": payload.data, "sound": "default" }
            }
        });

        let resp = self
            .client
            .post("https://api.jpush.cn/v3/push")
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!(status = %status, body = %text, "JPush send failed");
        }

        Ok(())
    }

    async fn send_batch(
        &self,
        device_tokens: &[String],
        payload: &PushPayload,
    ) -> anyhow::Result<()> {
        if device_tokens.is_empty() {
            return Ok(());
        }

        let body = serde_json::json!({
            "platform": "all",
            "audience": { "registration_id": device_tokens },
            "notification": {
                "alert": payload.body,
                "android": { "title": payload.title, "alert": payload.body, "extras": payload.data },
                "ios": { "title": payload.title, "alert": payload.body, "extras": payload.data, "sound": "default" }
            }
        });

        let resp = self
            .client
            .post("https://api.jpush.cn/v3/push")
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!(status = %status, body = %text, "JPush batch send failed");
        }

        Ok(())
    }
}
