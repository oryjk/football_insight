use async_trait::async_trait;

use crate::{
    mcp::domain::PromoTarget,
    share_code::domain::{CodeImage, ShareCodeError},
};

/// Mini program QR code for a landing target.
#[async_trait]
pub trait PromoCodePort: Send + Sync {
    async fn image(&self, target: PromoTarget) -> Result<CodeImage, ShareCodeError>;
}
