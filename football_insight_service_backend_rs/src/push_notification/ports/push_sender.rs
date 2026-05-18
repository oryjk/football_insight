use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
    pub data: serde_json::Value,
}

#[async_trait]
pub trait PushSender: Send + Sync {
    async fn send(&self, device_token: &str, payload: &PushPayload) -> anyhow::Result<()>;
    async fn send_batch(
        &self,
        device_tokens: &[String],
        payload: &PushPayload,
    ) -> anyhow::Result<()> {
        for token in device_tokens {
            self.send(token, payload).await?;
        }
        Ok(())
    }
}
