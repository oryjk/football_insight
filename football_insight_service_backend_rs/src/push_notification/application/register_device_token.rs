use std::sync::Arc;
use uuid::Uuid;

use crate::push_notification::ports::device_token_repository::DeviceTokenRepository;

pub struct RegisterDeviceTokenUseCase {
    repository: Arc<dyn DeviceTokenRepository>,
}

impl RegisterDeviceTokenUseCase {
    pub fn new(repository: Arc<dyn DeviceTokenRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        device_token: &str,
        platform: &str,
    ) -> anyhow::Result<()> {
        if device_token.is_empty() {
            return Err(anyhow::anyhow!("device token cannot be empty"));
        }
        if platform != "jpush" && platform != "fcm" && platform != "apns" {
            return Err(anyhow::anyhow!("platform must be jpush, fcm or apns"));
        }
        self.repository
            .upsert(user_id, device_token, platform)
            .await
    }
}
