use std::sync::Arc;

use crate::ai::{
    domain::chat::{AiChatActor, AiChatMessage, AiChatReply, AiChatStream},
    ports::ai_chat_port::AiChatPort,
};

#[derive(Debug, Clone)]
pub struct ChatWithModelInput {
    pub actor: AiChatActor,
    pub message: String,
    pub history: Vec<AiChatMessage>,
}

pub struct ChatWithModelUseCase {
    ai_chat_port: Arc<dyn AiChatPort>,
}

#[derive(Debug, Clone)]
struct PreparedChatWithModelInput {
    actor: AiChatActor,
    message: String,
    history: Vec<AiChatMessage>,
}

impl ChatWithModelUseCase {
    pub fn new(ai_chat_port: Arc<dyn AiChatPort>) -> Self {
        Self { ai_chat_port }
    }

    fn prepare_input(
        &self,
        input: ChatWithModelInput,
    ) -> anyhow::Result<PreparedChatWithModelInput> {
        let message = input.message.trim();
        if message.is_empty() {
            anyhow::bail!("message is required");
        }

        if message.chars().count() > 4_000 {
            anyhow::bail!("message is too long");
        }

        let history = input
            .history
            .into_iter()
            .filter_map(|item| {
                let content = item.content.trim().to_string();
                if content.is_empty() {
                    return None;
                }

                Some(AiChatMessage {
                    role: item.role,
                    content,
                })
            })
            .rev()
            .take(20)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();

        Ok(PreparedChatWithModelInput {
            actor: input.actor,
            message: message.to_string(),
            history,
        })
    }

    pub async fn execute(&self, input: ChatWithModelInput) -> anyhow::Result<AiChatReply> {
        let prepared = self.prepare_input(input)?;

        self.ai_chat_port
            .chat(&prepared.actor, &prepared.message, &prepared.history)
            .await
    }

    pub async fn stream(&self, input: ChatWithModelInput) -> anyhow::Result<AiChatStream> {
        let prepared = self.prepare_input(input)?;

        self.ai_chat_port
            .stream_chat(&prepared.actor, &prepared.message, &prepared.history)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use crate::ai::{
        application::chat_with_model::{ChatWithModelInput, ChatWithModelUseCase},
        domain::chat::{AiChatActor, AiChatMessage, AiChatReply, AiChatRole},
        ports::ai_chat_port::AiChatPort,
    };

    #[derive(Default)]
    struct FakeAiChatPort {
        requests: Mutex<Vec<(String, usize)>>,
    }

    #[async_trait]
    impl AiChatPort for FakeAiChatPort {
        async fn chat(
            &self,
            _actor: &AiChatActor,
            message: &str,
            history: &[AiChatMessage],
        ) -> anyhow::Result<AiChatReply> {
            self.requests
                .lock()
                .expect("requests lock")
                .push((message.to_string(), history.len()));

            Ok(AiChatReply {
                model: "gpt-test".to_string(),
                reply: "ok".to_string(),
            })
        }

        async fn stream_chat(
            &self,
            _actor: &AiChatActor,
            _message: &str,
            _history: &[AiChatMessage],
        ) -> anyhow::Result<crate::ai::domain::chat::AiChatStream> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn execute_rejects_empty_message() {
        let use_case = ChatWithModelUseCase::new(Arc::new(FakeAiChatPort::default()));

        let error = use_case
            .execute(ChatWithModelInput {
                actor: AiChatActor {
                    display_name: "Carl".to_string(),
                    membership_tier: "V3".to_string(),
                },
                message: "   ".to_string(),
                history: vec![],
            })
            .await
            .expect_err("empty message should fail");

        assert!(error.to_string().contains("message is required"));
    }

    #[tokio::test]
    async fn execute_trims_message_and_limits_history() {
        let port = Arc::new(FakeAiChatPort::default());
        let use_case = ChatWithModelUseCase::new(port.clone());

        let history = (0..25)
            .map(|index| AiChatMessage {
                role: if index % 2 == 0 {
                    AiChatRole::User
                } else {
                    AiChatRole::Assistant
                },
                content: format!(" message {index} "),
            })
            .collect::<Vec<_>>();

        let reply = use_case
            .execute(ChatWithModelInput {
                actor: AiChatActor {
                    display_name: "Carl".to_string(),
                    membership_tier: "V3".to_string(),
                },
                message: "  hello model  ".to_string(),
                history,
            })
            .await
            .expect("chat should succeed");

        let requests = port.requests.lock().expect("requests lock");
        assert_eq!(reply.reply, "ok");
        assert_eq!(requests[0].0, "hello model");
        assert_eq!(requests[0].1, 20);
    }
}
