use async_trait::async_trait;

use super::domain::{ContentSecurityError, ModerationScene};

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
