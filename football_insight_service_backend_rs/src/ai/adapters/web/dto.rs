use serde::{Deserialize, Serialize};

use crate::ai::domain::chat::{AiChatMessage, AiChatReply, AiChatRole};

#[derive(Debug, Deserialize)]
pub struct AiChatRequest {
    pub message: String,
    #[serde(default)]
    pub history: Vec<AiChatHistoryMessageRequest>,
}

#[derive(Debug, Deserialize)]
pub struct AiChatHistoryMessageRequest {
    pub role: String,
    pub content: String,
}

impl TryFrom<AiChatHistoryMessageRequest> for AiChatMessage {
    type Error = anyhow::Error;

    fn try_from(value: AiChatHistoryMessageRequest) -> Result<Self, Self::Error> {
        let role = match value.role.trim().to_ascii_lowercase().as_str() {
            "user" => AiChatRole::User,
            "assistant" => AiChatRole::Assistant,
            _ => anyhow::bail!("history role is invalid"),
        };

        Ok(Self {
            role,
            content: value.content,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AiChatResponseDto {
    pub model: String,
    pub reply: String,
}

impl From<AiChatReply> for AiChatResponseDto {
    fn from(value: AiChatReply) -> Self {
        Self {
            model: value.model,
            reply: value.reply,
        }
    }
}
