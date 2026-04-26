use async_trait::async_trait;

use crate::ai::domain::chat::{AiChatActor, AiChatMessage, AiChatReply, AiChatStream};

#[async_trait]
pub trait AiChatPort: Send + Sync {
    async fn chat(
        &self,
        actor: &AiChatActor,
        message: &str,
        history: &[AiChatMessage],
    ) -> anyhow::Result<AiChatReply>;

    async fn stream_chat(
        &self,
        actor: &AiChatActor,
        message: &str,
        history: &[AiChatMessage],
    ) -> anyhow::Result<AiChatStream>;
}
