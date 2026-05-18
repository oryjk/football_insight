use async_trait::async_trait;

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_html(&self, to: &str, subject: &str, body_html: &str) -> anyhow::Result<()>;
}
