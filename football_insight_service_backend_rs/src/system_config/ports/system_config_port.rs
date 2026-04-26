use async_trait::async_trait;

use crate::system_config::domain::{
    ai_chat_config::AiChatSystemConfig, public_system_config::PublicSystemConfig,
};

#[async_trait]
pub trait SystemConfigPort: Send + Sync {
    async fn get_public_config(&self) -> anyhow::Result<PublicSystemConfig>;
    async fn get_ai_chat_config(&self) -> anyhow::Result<AiChatSystemConfig>;
    async fn get_config_value(&self, config_key: &str) -> anyhow::Result<Option<String>>;
}
