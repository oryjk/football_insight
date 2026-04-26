use std::pin::Pin;

use futures_util::Stream;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiChatActor {
    pub display_name: String,
    pub membership_tier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiChatRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiChatMessage {
    pub role: AiChatRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiChatReply {
    pub model: String,
    pub reply: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiChatStreamEvent {
    Started { model: String },
    Delta { content: String },
    Completed { model: String, reply: String },
}

pub type AiChatStream = Pin<Box<dyn Stream<Item = anyhow::Result<AiChatStreamEvent>> + Send>>;
