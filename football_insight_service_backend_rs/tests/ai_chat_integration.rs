//! AI Chat 集成测试
//!
//! 验证 3 轮对话能正常调用远端大模型并拿到回复。
//!
//! ```bash
//! cargo test --test ai_chat_integration -- --nocapture
//! ```

use std::sync::OnceLock;

use serial_test::serial;

fn init_env() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let env_path = std::path::Path::new(&manifest_dir).join(".env");
        if env_path.exists() {
            dotenvy::from_path(&env_path).ok();
        }
    });
}

fn load_config() -> Option<(String, String, String)> {
    init_env();
    let api_key = std::env::var("OPENAI_API_KEY").ok()?;
    let model = std::env::var("AI_CHAT_MODEL").unwrap_or_else(|_| "glm-5.1".to_string());
    let base_url = std::env::var("OPENAI_BASE_URL")
        .ok()
        .unwrap_or_else(|| "https://open.bigmodel.cn/api/coding/paas/v4".to_string());
    Some((api_key, model, base_url))
}

/// 发一条消息给大模型，返回回复文本
async fn ask(
    client: &reqwest::Client,
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: &[(&str, &str)],
) -> anyhow::Result<String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let messages_json: Vec<serde_json::Value> = messages
        .iter()
        .map(|(role, content)| serde_json::json!({"role": role, "content": content}))
        .collect();

    let body = {
        let mut attempt = 0u32;
        loop {
            let resp = client
                .post(&url)
                .bearer_auth(api_key)
                .json(&serde_json::json!({
                    "model": model,
                    "messages": &messages_json,
                    "temperature": 0.2,
                    "stream": false
                }))
                .send()
                .await?;

            let status = resp.status();
            if status.as_u16() == 429 && attempt < 3 {
                attempt += 1;
                println!("  429 限流，30s 后重试 ({}/3)...", attempt);
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                continue;
            }
            let body = resp.text().await?;
            anyhow::ensure!(
                status.is_success(),
                "API 错误 {}: {}",
                status.as_u16(),
                body
            );
            break body;
        }
    };

    let json: serde_json::Value = serde_json::from_str(&body)?;
    let reply = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    Ok(reply)
}

#[tokio::test]
#[serial]
async fn test_three_rounds() {
    let Some((api_key, model, base_url)) = load_config() else {
        println!("跳过：未设置 OPENAI_API_KEY");
        return;
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .expect("build client");

    let sep = "─".repeat(50);

    // 第 1 轮
    let r1 = ask(
        &client,
        &api_key,
        &base_url,
        &model,
        &[
            ("system", "你是足球洞察的 AI 助手，用简体中文简洁回答。"),
            ("user", "当前中超的排名是什么样的？"),
        ],
    )
    .await
    .expect("第 1 轮失败");

    println!(
        "\n{}\n第 1 轮提问: 当前中超的排名是什么样的？\n{}\n{}\n{}",
        sep, sep, r1, sep
    );
    assert!(!r1.trim().is_empty(), "第 1 轮回复为空");

    // 第 2 轮（带第 1 轮上下文）
    let r2 = ask(
        &client,
        &api_key,
        &base_url,
        &model,
        &[
            ("system", "你是足球洞察的 AI 助手，用简体中文简洁回答。"),
            ("user", "当前中超的排名是什么样的？"),
            ("assistant", &r1),
            ("user", "那榜首球队最近表现怎么样？"),
        ],
    )
    .await
    .expect("第 2 轮失败");

    println!(
        "\n{}\n第 2 轮提问: 那榜首球队最近表现怎么样？\n{}\n{}\n{}",
        sep, sep, r2, sep
    );
    assert!(!r2.trim().is_empty(), "第 2 轮回复为空");

    // 第 3 轮（带前两轮上下文）
    let r3 = ask(
        &client,
        &api_key,
        &base_url,
        &model,
        &[
            ("system", "你是足球洞察的 AI 助手，用简体中文简洁回答。"),
            ("user", "当前中超的排名是什么样的？"),
            ("assistant", &r1),
            ("user", "那榜首球队最近表现怎么样？"),
            ("assistant", &r2),
            ("user", "他们下一场打谁？"),
        ],
    )
    .await
    .expect("第 3 轮失败");

    println!(
        "\n{}\n第 3 轮提问: 他们下一场打谁？\n{}\n{}\n{}",
        sep, sep, r3, sep
    );
    assert!(!r3.trim().is_empty(), "第 3 轮回复为空");

    println!("\n3 轮对话全部通过");
}
