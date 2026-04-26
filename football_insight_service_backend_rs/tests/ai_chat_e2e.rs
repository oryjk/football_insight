//! AI Chat 对话功能测试
//!
//! 验证 ChatWithModelUseCase → AiChatPort 整条链路是否打通。
//!
//! ```bash
//! cargo test --test ai_chat_e2e -- --nocapture
//! ```

use std::sync::Arc;

use async_trait::async_trait;
use football_insight_service_backend_rs::ai::{
    application::chat_with_model::{ChatWithModelInput, ChatWithModelUseCase},
    domain::chat::{AiChatActor, AiChatMessage, AiChatReply, AiChatRole, AiChatStream},
    ports::ai_chat_port::AiChatPort,
};

// ── Fake AiChatPort：模拟 AI 回复 ────────────────────────────────────

struct FakeAiChatPort;

#[async_trait]
impl AiChatPort for FakeAiChatPort {
    async fn chat(
        &self,
        actor: &AiChatActor,
        message: &str,
        history: &[AiChatMessage],
    ) -> anyhow::Result<AiChatReply> {
        Ok(AiChatReply {
            model: "fake-glm".to_string(),
            reply: format!(
                "你好 {}，收到消息「{}」，上下文 {} 条",
                actor.display_name,
                message,
                history.len(),
            ),
        })
    }

    async fn stream_chat(
        &self,
        _actor: &AiChatActor,
        _message: &str,
        _history: &[AiChatMessage],
    ) -> anyhow::Result<AiChatStream> {
        anyhow::bail!("stream not needed for this test")
    }
}

// ── Tests ────────────────────────────────────────────────────────────

fn actor() -> AiChatActor {
    AiChatActor {
        display_name: "球迷小王".to_string(),
        membership_tier: "V3".to_string(),
    }
}

#[tokio::test]
async fn basic_chat_returns_reply() {
    let use_case = ChatWithModelUseCase::new(Arc::new(FakeAiChatPort));

    let reply = use_case
        .execute(ChatWithModelInput {
            actor: actor(),
            message: "中超榜首是谁？".to_string(),
            history: vec![],
        })
        .await
        .expect("chat should succeed");

    assert_eq!(reply.model, "fake-glm");
    assert!(reply.reply.contains("球迷小王"));
    assert!(reply.reply.contains("中超榜首是谁？"));
    assert!(reply.reply.contains("上下文 0 条"));
}

#[tokio::test]
async fn chat_with_history_passes_context() {
    let use_case = ChatWithModelUseCase::new(Arc::new(FakeAiChatPort));

    let reply = use_case
        .execute(ChatWithModelInput {
            actor: actor(),
            message: "那第二名呢？".to_string(),
            history: vec![
                AiChatMessage {
                    role: AiChatRole::User,
                    content: "中超榜首是谁".to_string(),
                },
                AiChatMessage {
                    role: AiChatRole::Assistant,
                    content: "目前榜首是上海海港".to_string(),
                },
            ],
        })
        .await
        .expect("chat should succeed");

    assert!(reply.reply.contains("上下文 2 条"));
}

#[tokio::test]
async fn chat_rejects_empty_message() {
    let use_case = ChatWithModelUseCase::new(Arc::new(FakeAiChatPort));

    let err = use_case
        .execute(ChatWithModelInput {
            actor: actor(),
            message: "   ".to_string(),
            history: vec![],
        })
        .await
        .expect_err("should reject empty");

    assert!(err.to_string().contains("message is required"));
}

#[tokio::test]
async fn chat_rejects_too_long_message() {
    let use_case = ChatWithModelUseCase::new(Arc::new(FakeAiChatPort));

    let err = use_case
        .execute(ChatWithModelInput {
            actor: actor(),
            message: "啊".repeat(4001),
            history: vec![],
        })
        .await
        .expect_err("should reject too long");

    assert!(err.to_string().contains("message is too long"));
}

#[tokio::test]
async fn chat_truncates_history_to_20() {
    let use_case = ChatWithModelUseCase::new(Arc::new(FakeAiChatPort));

    let history: Vec<AiChatMessage> = (0..25)
        .map(|i| AiChatMessage {
            role: if i % 2 == 0 {
                AiChatRole::User
            } else {
                AiChatRole::Assistant
            },
            content: format!("msg {i}"),
        })
        .collect();

    let reply = use_case
        .execute(ChatWithModelInput {
            actor: actor(),
            message: "最新的问题".to_string(),
            history,
        })
        .await
        .expect("chat should succeed");

    // FakeAiChatPort reports history count, should be capped at 20
    assert!(reply.reply.contains("上下文 20 条"));
}
