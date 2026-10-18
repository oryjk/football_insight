use super::domain::{CodeImage, ShareCodeError};
use async_trait::async_trait;

#[async_trait]
pub trait TicketWatchCodePort: Send + Sync {
    async fn image(&self) -> Result<CodeImage, ShareCodeError>;
}
