use std::sync::Arc;

use crate::push_notification::ports::{
    device_token_repository::DeviceTokenRepository,
    push_sender::{PushPayload, PushSender},
};

pub struct ProcessPushNotificationsUseCase {
    token_repository: Arc<dyn DeviceTokenRepository>,
    push_sender: Arc<dyn PushSender>,
}

impl ProcessPushNotificationsUseCase {
    pub fn new(
        token_repository: Arc<dyn DeviceTokenRepository>,
        push_sender: Arc<dyn PushSender>,
    ) -> Self {
        Self {
            token_repository,
            push_sender,
        }
    }

    pub async fn send_to_user(
        &self,
        user_id: uuid::Uuid,
        payload: &PushPayload,
    ) -> anyhow::Result<usize> {
        let tokens = self.token_repository.list_by_user(user_id).await?;
        if tokens.is_empty() {
            return Ok(0);
        }
        let token_strings: Vec<String> = tokens.iter().map(|t| t.device_token.clone()).collect();
        self.push_sender.send_batch(&token_strings, payload).await?;
        Ok(tokens.len())
    }

    pub async fn send_to_users(
        &self,
        user_ids: &[uuid::Uuid],
        payload: &PushPayload,
    ) -> anyhow::Result<usize> {
        let tokens = self.token_repository.list_by_users(user_ids).await?;
        if tokens.is_empty() {
            return Ok(0);
        }
        let token_strings: Vec<String> = tokens.iter().map(|t| t.device_token.clone()).collect();
        self.push_sender.send_batch(&token_strings, payload).await?;
        Ok(tokens.len())
    }
}
