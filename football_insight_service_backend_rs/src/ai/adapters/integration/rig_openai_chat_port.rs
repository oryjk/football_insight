use std::sync::Arc;

use async_trait::async_trait;
use chrono::Local;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::ai::{
    domain::chat::{
        AiChatActor, AiChatMessage, AiChatReply, AiChatRole, AiChatStream, AiChatStreamEvent,
    },
    ports::ai_chat_port::AiChatPort,
};
use crate::insight::{
    domain::rankings::RankingsView, ports::insight_query_repository::InsightQueryRepository,
};
use crate::system_config::{
    domain::ai_chat_config::AiChatSystemConfig, ports::system_config_port::SystemConfigPort,
};

const AI_HTTP_REQUEST_TIMEOUT_SECS: u64 = 15;
const AI_HTTP_CONNECT_TIMEOUT_SECS: u64 = 15;
const AI_HTTP_STREAM_READ_TIMEOUT_SECS: u64 = 120;

pub struct RigOpenAiChatPort {
    http_client: Client,
    stream_http_client: Client,
    api_key: String,
    default_model: String,
    default_base_url: Option<String>,
    system_config_port: Arc<dyn SystemConfigPort>,
    insight_query_repository: Arc<dyn InsightQueryRepository>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedRigOpenAiConfig {
    model: String,
    base_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedStreamPayload {
    delta_text: Option<String>,
    is_done: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionResponse {
    model: Option<String>,
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionAssistantMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionAssistantMessage {
    #[serde(default)]
    content: Value,
}

impl RigOpenAiChatPort {
    pub fn new(
        api_key: String,
        default_model: String,
        default_base_url: Option<String>,
        system_config_port: Arc<dyn SystemConfigPort>,
        insight_query_repository: Arc<dyn InsightQueryRepository>,
    ) -> Self {
        Self {
            http_client: build_ai_http_client(
                Some(std::time::Duration::from_secs(AI_HTTP_REQUEST_TIMEOUT_SECS)),
                None,
            ),
            stream_http_client: build_ai_http_client(
                None,
                Some(std::time::Duration::from_secs(
                    AI_HTTP_STREAM_READ_TIMEOUT_SECS,
                )),
            ),
            api_key,
            default_model,
            default_base_url,
            system_config_port,
            insight_query_repository,
        }
    }

    fn build_preamble(actor: &AiChatActor) -> String {
        let current_date = current_date_string();

        format!(
            concat!(
                "今天日期：{current_date}。\n",
                "你是足球洞察的站内 AI 助手，默认使用简体中文回答。\n",
                "你的核心任务是为用户提供足球相关的问答和解读，既包括足球洞察产品内的数据、页面内容、球队走势、榜单和功能，也包括更广义的足球话题。\n",
                "你应该自然支持这些话题：球队信息、球员信息、中超联赛、亚冠、欧冠、五大联赛、国家队赛事、世界杯，以及其他常见足球知识与讨论。\n",
                "如果用户问的是足球洞察产品内的信息，请优先结合产品语境回答；如果用户问的是泛足球内容，也可以直接像专业足球助手一样回答。\n",
                "凡是涉及积分榜、排名、榜首、前几名、赛程、比分、伤停、转会等事实型问题，即使用户没有明确说今天或最新，也默认按最新事实处理；如果拿不到可靠最新信息，就直接说明无法确认，不要用过期知识硬答。\n",
                "如果你具备联网搜索结果，请优先基于最新搜索结果回答，并尽量交代来源和时间；如果你没有拿到足够可靠的搜索结果，不要编造，明确说明你暂时无法确认最新动态。\n",
                "当前用户昵称：{display_name}。\n",
                "当前用户会员等级：{membership_tier}。\n",
                "如果用户问题超出足球洞察产品和足球话题，也可以正常回答，但语气保持简洁、友好、专业。\n",
                "不要编造系统里并不存在的会员权益；如果用户追问权益，明确说明当前还在规划中。"
            ),
            current_date = current_date,
            display_name = actor.display_name,
            membership_tier = actor.membership_tier,
        )
    }

    fn map_history_message(message: &AiChatMessage) -> Value {
        json!({
            "role": match message.role {
                AiChatRole::User => "user",
                AiChatRole::Assistant => "assistant",
            },
            "content": message.content,
        })
    }

    async fn resolve_config(&self) -> anyhow::Result<ResolvedRigOpenAiConfig> {
        let db_config = self.system_config_port.get_ai_chat_config().await?;
        Ok(resolve_runtime_config(
            db_config,
            &self.default_model,
            self.default_base_url.as_deref(),
        ))
    }

    fn build_chat_completion_url(&self, base_url: Option<&str>) -> String {
        format!(
            "{}/chat/completions",
            base_url
                .unwrap_or("https://open.bigmodel.cn/api/coding/paas/v4")
                .trim_end_matches('/')
        )
    }

    async fn fetch_live_rankings_grounding_context(&self, message: &str) -> Option<String> {
        if !is_live_standings_question(message) {
            return None;
        }

        match self.insight_query_repository.get_live_rankings().await {
            Ok(rankings) => build_live_rankings_grounding_context(&rankings),
            Err(error) => {
                tracing::warn!(error = %error, "failed to fetch live rankings fallback");
                None
            }
        }
    }

    async fn fetch_live_rankings_direct_reply(&self, message: &str) -> Option<String> {
        if !is_live_standings_question(message) {
            return None;
        }

        match self.insight_query_repository.get_live_rankings().await {
            Ok(rankings) => build_live_rankings_direct_reply(&rankings),
            Err(error) => {
                tracing::warn!(error = %error, "failed to fetch live rankings for direct reply");
                None
            }
        }
    }

    async fn send_chat_completion(
        &self,
        runtime_config: &ResolvedRigOpenAiConfig,
        actor: &AiChatActor,
        message: &str,
        history: &[AiChatMessage],
        stream: bool,
        grounding_context: Option<&str>,
    ) -> anyhow::Result<reqwest::Response> {
        let body = build_chat_request_body(
            &runtime_config.model,
            actor,
            message,
            history,
            stream,
            grounding_context,
        );
        let url = self.build_chat_completion_url(runtime_config.base_url.as_deref());

        let http_client = if stream {
            &self.stream_http_client
        } else {
            &self.http_client
        };

        let response = http_client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("provider status code {}: {}", status.as_u16(), body);
    }

    async fn fetch_non_stream_reply(
        &self,
        runtime_config: &ResolvedRigOpenAiConfig,
        actor: &AiChatActor,
        message: &str,
        history: &[AiChatMessage],
    ) -> anyhow::Result<AiChatReply> {
        let grounding_context = self.fetch_live_rankings_grounding_context(message).await;
        let response = self
            .send_chat_completion(
                runtime_config,
                actor,
                message,
                history,
                false,
                grounding_context.as_deref(),
            )
            .await?;
        let payload = response.json::<ChatCompletionResponse>().await?;
        let reply = payload
            .choices
            .iter()
            .find_map(|choice| extract_text_content(&choice.message.content))
            .unwrap_or_default();
        let reply = guard_grounded_reply(message, &reply);
        if reply.is_empty() {
            anyhow::bail!("ai reply is empty");
        }

        Ok(AiChatReply {
            model: payload
                .model
                .unwrap_or_else(|| runtime_config.model.clone()),
            reply,
        })
    }
}

fn build_ai_http_client(
    request_timeout: Option<std::time::Duration>,
    read_timeout: Option<std::time::Duration>,
) -> Client {
    let mut builder = Client::builder()
        .connect_timeout(std::time::Duration::from_secs(AI_HTTP_CONNECT_TIMEOUT_SECS));

    if let Some(request_timeout) = request_timeout {
        builder = builder.timeout(request_timeout);
    }

    if let Some(read_timeout) = read_timeout {
        builder = builder.read_timeout(read_timeout);
    }

    builder.build().expect("failed to build ai http client")
}

fn resolve_runtime_config(
    db_config: AiChatSystemConfig,
    fallback_model: &str,
    fallback_base_url: Option<&str>,
) -> ResolvedRigOpenAiConfig {
    ResolvedRigOpenAiConfig {
        model: db_config
            .model
            .unwrap_or_else(|| fallback_model.trim().to_string()),
        base_url: db_config
            .base_url
            .or_else(|| fallback_base_url.map(|item| item.trim().to_string())),
    }
}

fn build_chat_request_body(
    model: &str,
    actor: &AiChatActor,
    message: &str,
    history: &[AiChatMessage],
    stream: bool,
    grounding_context: Option<&str>,
) -> Value {
    let mut messages = vec![json!({
        "role": "system",
        "content": RigOpenAiChatPort::build_preamble(actor),
    })];
    if let Some(grounding_context) = grounding_context.filter(|item| !item.trim().is_empty()) {
        messages.push(json!({
            "role": "system",
            "content": build_grounding_message(grounding_context),
        }));
    }
    messages.extend(history.iter().map(RigOpenAiChatPort::map_history_message));
    messages.push(json!({
        "role": "user",
        "content": message,
    }));

    let body = json!({
        "model": model,
        "messages": messages,
        "temperature": 0.2,
        "stream": stream,
    });

    body
}

fn current_date_string() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn build_grounding_message(grounding_context: &str) -> String {
    format!(
        concat!(
            "系统提供了一份与本次问题相关的站内实时数据摘要。\n",
            "回答事实型问题时，请优先依据这些数据；如果仍不足以确认，请明确说无法确认，不要编造。\n\n",
            "{grounding_context}"
        ),
        grounding_context = grounding_context.trim(),
    )
}

fn build_live_rankings_grounding_context(rankings: &RankingsView) -> Option<String> {
    let table = rankings
        .standings_tables
        .iter()
        .find(|item| item.slug == "standings_with_penalty")
        .or_else(|| rankings.standings_tables.first())?;

    if table.entries.is_empty() {
        return None;
    }

    let top_entries = table
        .entries
        .iter()
        .take(4)
        .map(|entry| {
            format!(
                "第 {} 名：{}，{} 分，{} 场，{}胜{}平{}负，净胜球 {}",
                entry.rank_no,
                entry.team_name,
                entry.points,
                entry.played,
                entry.wins,
                entry.draws,
                entry.losses,
                entry.goal_difference
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Some(format!(
        concat!(
            "以下是足球洞察站内实时榜单数据，可作为当前回答的优先依据。\n",
            "赛季：{season}\n",
            "视图：{view_kind}\n",
            "轮次：{round_number}\n",
            "榜单：{label}\n",
            "{entries}"
        ),
        season = rankings.current_season,
        view_kind = rankings.view_kind,
        round_number = rankings
            .round_number
            .map(|item| item.to_string())
            .unwrap_or_else(|| "实时".to_string()),
        label = table.label,
        entries = top_entries,
    ))
}

fn build_live_rankings_direct_reply(rankings: &RankingsView) -> Option<String> {
    let table = rankings
        .standings_tables
        .iter()
        .find(|item| item.slug == "standings_with_penalty")
        .or_else(|| rankings.standings_tables.first())?;

    if table.entries.is_empty() {
        return None;
    }

    let top_entries = table
        .entries
        .iter()
        .take(6)
        .map(|entry| {
            format!(
                "第 {}：{}（{} 分，{}胜{}平{}负，净胜球 {}）",
                entry.rank_no,
                entry.team_name,
                entry.points,
                entry.wins,
                entry.draws,
                entry.losses,
                entry.goal_difference
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Some(format!(
        concat!(
            "中超实时积分榜（站内数据）\n",
            "赛季：{season}，轮次：{round_number}\n",
            "{entries}"
        ),
        season = rankings.current_season,
        round_number = rankings
            .round_number
            .map(|item| item.to_string())
            .unwrap_or_else(|| "实时".to_string()),
        entries = top_entries
    ))
}

fn normalize_ai_reply(reply: &str) -> String {
    let mut normalized = reply.replace("\r\n", "\n");

    while let Some(start) = normalized.find("<think>") {
        let Some(end) = normalized[start..].find("</think>") else {
            normalized.replace_range(start..normalized.len(), "");
            break;
        };
        let end = start + end + "</think>".len();
        normalized.replace_range(start..end, "");
    }

    normalized
        .replace("**", "")
        .replace("__", "")
        .replace("###", "")
        .replace("##", "")
        .replace("`", "")
        .trim()
        .to_string()
}

fn is_live_standings_question(message: &str) -> bool {
    let normalized = message.trim();
    let asks_standings = normalized.contains("积分榜")
        || normalized.contains("榜首")
        || normalized.contains("排名")
        || normalized.contains("第几名")
        || normalized.contains("前二")
        || normalized.contains("前两")
        || normalized.contains("前四")
        || normalized.contains("第二名")
        || normalized.contains("第一名");
    let asks_football_scope = normalized.contains("中超")
        || normalized.contains("足球")
        || normalized.contains("联赛")
        || normalized.contains("欧冠")
        || normalized.contains("英超")
        || normalized.contains("西甲");

    asks_standings && asks_football_scope
}

fn looks_grounded_for_live_standings(reply: &str) -> bool {
    reply.contains("来源")
        || reply.contains("截至")
        || reply.contains("今日")
        || reply.contains("今天")
        || reply.contains("2026-")
        || reply.contains("2026年")
}

fn guard_grounded_reply(message: &str, reply: &str) -> String {
    let normalized = normalize_ai_reply(reply);

    if !is_live_standings_question(message) {
        return normalized;
    }

    if normalized.contains("无法确认")
        || normalized.contains("无法获取")
        || normalized.contains("很抱歉")
    {
        return normalized;
    }

    let contains_marketing = normalized.contains("足球洞察产品")
        || normalized.contains("V3会员")
        || normalized.contains("会员")
        || normalized.contains("查看完整榜单");

    if !looks_grounded_for_live_standings(&normalized) || contains_marketing {
        return "我暂时无法确认当前中超积分榜的准确排名，站内实时榜单数据暂不可用。建议直接查看榜单页。".to_string();
    }

    normalized
}

fn should_buffer_stream_response(message: &str) -> bool {
    // 联网搜索场景已移除，避免为“实时事实核对”走伪流式分支。
    // 目前统一走 provider 的 stream 响应；如需要强一致事实，后续再引入更可控的方案。
    let _ = message;
    false
}

fn extract_text_content(content: &Value) -> Option<String> {
    match content {
        Value::Null => None,
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        }
        Value::Array(items) => {
            let text = items
                .iter()
                .filter_map(|item| match item {
                    Value::String(text) => Some(text.trim().to_string()),
                    Value::Object(map) => map
                        .get("text")
                        .and_then(extract_text_content)
                        .or_else(|| map.get("content").and_then(extract_text_content)),
                    _ => None,
                })
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
                .join("");

            if text.is_empty() { None } else { Some(text) }
        }
        Value::Object(map) => map
            .get("text")
            .and_then(extract_text_content)
            .or_else(|| map.get("content").and_then(extract_text_content)),
        _ => None,
    }
}

fn parse_stream_payload(payload: &str) -> anyhow::Result<ParsedStreamPayload> {
    if payload.trim() == "[DONE]" {
        return Ok(ParsedStreamPayload {
            delta_text: None,
            is_done: true,
        });
    }

    let json = serde_json::from_str::<Value>(payload)?;
    let delta_text = json
        .get("choices")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find_map(|item| item.get("delta"))
        .and_then(|delta| delta.get("content"))
        .and_then(extract_text_content);

    Ok(ParsedStreamPayload {
        delta_text,
        is_done: false,
    })
}

fn drain_sse_payloads(buffer: &mut String) -> Vec<String> {
    let mut payloads = Vec::new();

    loop {
        let normalized = buffer.replace("\r\n", "\n");
        let Some(separator_index) = normalized.find("\n\n") else {
            *buffer = normalized;
            break;
        };

        let chunk = normalized[..separator_index].to_string();
        *buffer = normalized[separator_index + 2..].to_string();

        let payload = chunk
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n");

        if !payload.is_empty() {
            payloads.push(payload);
        }
    }

    payloads
}

#[async_trait]
impl AiChatPort for RigOpenAiChatPort {
    async fn chat(
        &self,
        actor: &AiChatActor,
        message: &str,
        history: &[AiChatMessage],
    ) -> anyhow::Result<AiChatReply> {
        if let Some(reply) = self.fetch_live_rankings_direct_reply(message).await {
            return Ok(AiChatReply {
                model: "internal_live_rankings".to_string(),
                reply,
            });
        }

        let runtime_config = self.resolve_config().await?;
        self.fetch_non_stream_reply(&runtime_config, actor, message, history)
            .await
    }

    async fn stream_chat(
        &self,
        actor: &AiChatActor,
        message: &str,
        history: &[AiChatMessage],
    ) -> anyhow::Result<AiChatStream> {
        if let Some(reply) = self.fetch_live_rankings_direct_reply(message).await {
            let stream = async_stream::stream! {
                yield Ok(AiChatStreamEvent::Started {
                    model: "internal_live_rankings".to_string(),
                });
                yield Ok(AiChatStreamEvent::Delta {
                    content: reply.clone(),
                });
                yield Ok(AiChatStreamEvent::Completed {
                    model: "internal_live_rankings".to_string(),
                    reply,
                });
            };

            return Ok(Box::pin(stream));
        }

        let runtime_config = self.resolve_config().await?;
        let model = runtime_config.model.clone();
        let message = message.to_string();
        let _ = should_buffer_stream_response(&message);

        let grounding_context = self.fetch_live_rankings_grounding_context(&message).await;
        let response = self
            .send_chat_completion(
                &runtime_config,
                actor,
                &message,
                history,
                true,
                grounding_context.as_deref(),
            )
            .await?;

        let stream = async_stream::stream! {
            yield Ok(AiChatStreamEvent::Started {
                model: model.clone(),
            });

            let mut full_reply = String::new();
            let mut is_completed = false;
            let mut buffer = String::new();
            let mut response = response;

            loop {
                match response.chunk().await {
                    Ok(Some(chunk)) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));

                        for payload in drain_sse_payloads(&mut buffer) {
                            match parse_stream_payload(&payload) {
                                Ok(parsed) if parsed.is_done => {
                                    let reply = full_reply.trim().to_string();
                                    if reply.is_empty() {
                                        yield Err(anyhow::anyhow!("ai reply is empty"));
                                    } else {
                                        is_completed = true;
                                        yield Ok(AiChatStreamEvent::Completed {
                                            model: model.clone(),
                                            reply: guard_grounded_reply(&message, &reply),
                                        });
                                    }
                                    break;
                                }
                                Ok(parsed) => {
                                    if let Some(text) = parsed.delta_text {
                                        if text.is_empty() {
                                            continue;
                                        }

                                        full_reply.push_str(&text);
                                        yield Ok(AiChatStreamEvent::Delta { content: text });
                                    }
                                }
                                Err(error) => {
                                    yield Err(error);
                                    break;
                                }
                            }
                        }

                        if is_completed {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(error) => {
                        yield Err(anyhow::anyhow!(error.to_string()));
                        break;
                    }
                }
            }

            if !is_completed {
                let reply = full_reply.trim().to_string();
                if !reply.is_empty() {
                    yield Ok(AiChatStreamEvent::Completed {
                        model: model.clone(),
                        reply: guard_grounded_reply(&message, &reply),
                    });
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

pub struct DisabledAiChatPort;

impl DisabledAiChatPort {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AiChatPort for DisabledAiChatPort {
    async fn chat(
        &self,
        _actor: &AiChatActor,
        _message: &str,
        _history: &[AiChatMessage],
    ) -> anyhow::Result<AiChatReply> {
        anyhow::bail!("ai chat is not configured")
    }

    async fn stream_chat(
        &self,
        _actor: &AiChatActor,
        _message: &str,
        _history: &[AiChatMessage],
    ) -> anyhow::Result<AiChatStream> {
        anyhow::bail!("ai chat is not configured")
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use async_trait::async_trait;
    use futures_util::StreamExt;
    use serde_json::json;
    use tokio::{io::AsyncWriteExt, net::TcpListener, time::sleep};

    use super::{
        RigOpenAiChatPort, build_chat_request_body, build_live_rankings_grounding_context,
        guard_grounded_reply, normalize_ai_reply, parse_stream_payload, resolve_runtime_config,
        should_buffer_stream_response,
    };
    use crate::{
        ai::{
            domain::chat::{AiChatActor, AiChatStreamEvent},
            ports::ai_chat_port::AiChatPort,
        },
        insight::{
            domain::{
                match_list::MatchListView,
                overview::InsightOverview,
                rankings::RankingsView,
                rankings::{StandingsTable, StandingsTableEntry},
                round_reference::RoundReference,
                team_insight::TeamInsightsView,
            },
            ports::insight_query_repository::InsightQueryRepository,
        },
        system_config::{
            domain::{
                ai_chat_config::AiChatSystemConfig, public_system_config::PublicSystemConfig,
            },
            ports::system_config_port::SystemConfigPort,
        },
    };

    struct StaticSystemConfigPort {
        ai_chat_config: AiChatSystemConfig,
    }

    #[async_trait]
    impl SystemConfigPort for StaticSystemConfigPort {
        async fn get_public_config(&self) -> anyhow::Result<PublicSystemConfig> {
            anyhow::bail!("public config is not needed in this test")
        }

        async fn get_ai_chat_config(&self) -> anyhow::Result<AiChatSystemConfig> {
            Ok(self.ai_chat_config.clone())
        }

        async fn get_config_value(&self, _config_key: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }
    }

    struct UnusedInsightQueryRepository;

    #[async_trait]
    impl InsightQueryRepository for UnusedInsightQueryRepository {
        async fn get_live_overview(&self) -> anyhow::Result<InsightOverview> {
            anyhow::bail!("not needed")
        }

        async fn get_round_overview(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<InsightOverview> {
            anyhow::bail!("not needed")
        }

        async fn list_available_rounds(&self, _season: i32) -> anyhow::Result<Vec<RoundReference>> {
            anyhow::bail!("not needed")
        }

        async fn get_live_rankings(&self) -> anyhow::Result<RankingsView> {
            anyhow::bail!("not needed")
        }

        async fn get_live_team_insights(&self) -> anyhow::Result<TeamInsightsView> {
            anyhow::bail!("not needed")
        }

        async fn get_round_rankings(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<RankingsView> {
            anyhow::bail!("not needed")
        }

        async fn get_live_matches(&self) -> anyhow::Result<MatchListView> {
            anyhow::bail!("not needed")
        }

        async fn get_round_matches(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<MatchListView> {
            anyhow::bail!("not needed")
        }
    }

    async fn spawn_delayed_sse_server(delay: Duration) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let address = listener.local_addr().expect("resolve test server addr");

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept test connection");

            socket
                .write_all(
                    concat!(
                        "HTTP/1.1 200 OK\r\n",
                        "Content-Type: text/event-stream\r\n",
                        "Cache-Control: no-cache\r\n",
                        "Connection: close\r\n\r\n",
                    )
                    .as_bytes(),
                )
                .await
                .expect("write response headers");

            sleep(delay).await;

            socket
                .write_all(
                    concat!(
                        "data: {\"id\":\"1\",\"choices\":[{\"delta\":{\"content\":\"你好\"},\"index\":0}]}\n\n",
                        "data: [DONE]\n\n",
                    )
                    .as_bytes(),
                )
                .await
                .expect("write sse payload");
        });

        format!("http://{}", address)
    }

    fn test_actor() -> AiChatActor {
        AiChatActor {
            display_name: "Carl".to_string(),
            membership_tier: "V3".to_string(),
        }
    }

    #[tokio::test]
    async fn stream_chat_waits_for_delayed_first_chunk_without_failing() {
        let base_url = spawn_delayed_sse_server(Duration::from_millis(400)).await;
        let port = RigOpenAiChatPort {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_millis(200))
                .build()
                .expect("build client"),
            stream_http_client: reqwest::Client::builder()
                .read_timeout(Duration::from_secs(1))
                .build()
                .expect("build stream client"),
            api_key: "test-key".to_string(),
            default_model: "glm-5.1".to_string(),
            default_base_url: Some(base_url),
            system_config_port: Arc::new(StaticSystemConfigPort {
                ai_chat_config: AiChatSystemConfig::new(Some("glm-5.1".to_string()), None),
            }),
            insight_query_repository: Arc::new(UnusedInsightQueryRepository),
        };

        let mut stream = port
            .stream_chat(&test_actor(), "帮我介绍一下这个产品", &[])
            .await
            .expect("stream should start");

        let first = stream
            .next()
            .await
            .expect("started event should exist")
            .expect("started event should succeed");
        assert_eq!(
            first,
            AiChatStreamEvent::Started {
                model: "glm-5.1".to_string(),
            }
        );

        let second = stream
            .next()
            .await
            .expect("second event should exist")
            .expect("delayed stream should not fail before first chunk arrives");
        assert_eq!(
            second,
            AiChatStreamEvent::Delta {
                content: "你好".to_string(),
            }
        );
    }

    #[test]
    fn resolve_runtime_config_prefers_database_values() {
        let resolved = resolve_runtime_config(
            AiChatSystemConfig::new(
                Some("glm-4.5-air".to_string()),
                Some("https://example.com/v1".to_string()),
            ),
            "glm-5.1",
            Some("https://fallback.example.com/v1"),
        );

        assert_eq!(resolved.model, "glm-4.5-air");
        assert_eq!(resolved.base_url.as_deref(), Some("https://example.com/v1"));
    }

    #[test]
    fn resolve_runtime_config_falls_back_to_env_defaults_when_database_values_are_missing() {
        let resolved = resolve_runtime_config(
            AiChatSystemConfig::new(None, None),
            "glm-5.1",
            Some("https://fallback.example.com/v1"),
        );

        assert_eq!(resolved.model, "glm-5.1");
        assert_eq!(
            resolved.base_url.as_deref(),
            Some("https://fallback.example.com/v1")
        );
    }

    #[test]
    fn build_chat_request_body_includes_grounding_context_when_present() {
        let actor = AiChatActor {
            display_name: "Carl".to_string(),
            membership_tier: "V3".to_string(),
        };

        let body = build_chat_request_body(
            "glm-4.5-air",
            &actor,
            "中超积分榜第二名是谁？",
            &[],
            false,
            Some("系统刚刚完成联网搜索：ref_1 2026-04-09 中超积分榜链接"),
        );

        assert_eq!(body["messages"][0]["role"], json!("system"));
        assert_eq!(body["messages"][1]["role"], json!("system"));
        assert!(
            body["messages"][1]["content"]
                .as_str()
                .unwrap_or_default()
                .contains("系统刚刚完成联网搜索")
        );
        assert_eq!(
            body["messages"][2]["content"],
            json!("中超积分榜第二名是谁？")
        );
    }

    #[test]
    fn build_chat_request_body_does_not_inject_failure_context_when_grounding_is_missing() {
        let actor = AiChatActor {
            display_name: "Carl".to_string(),
            membership_tier: "V3".to_string(),
        };

        let body = build_chat_request_body(
            "glm-4.7",
            &actor,
            "今天中超积分榜前二是谁？",
            &[],
            false,
            None,
        );

        assert_eq!(
            body["messages"].as_array().map(|items| items.len()),
            Some(2)
        );
        assert_eq!(body["messages"][0]["role"], json!("system"));
        assert_eq!(body["messages"][1]["role"], json!("user"));
    }

    #[test]
    fn build_live_rankings_grounding_context_uses_actual_standings_table() {
        let rankings = RankingsView {
            view_kind: "live".to_string(),
            round_number: Some(5),
            current_season: 2026,
            standings_tables: vec![
                StandingsTable {
                    slug: "standings_without_penalty".to_string(),
                    label: "无罚分版积分榜".to_string(),
                    note: "".to_string(),
                    entries: vec![],
                },
                StandingsTable {
                    slug: "standings_with_penalty".to_string(),
                    label: "含罚分版积分榜".to_string(),
                    note: "按当前实际积分排序".to_string(),
                    entries: vec![
                        StandingsTableEntry {
                            rank_no: 1,
                            team_id: 10,
                            team_name: "上海海港".to_string(),
                            played: 10,
                            wins: 8,
                            draws: 1,
                            losses: 1,
                            goals_for: 20,
                            goals_against: 8,
                            goal_difference: 12,
                            points: 25,
                            points_without_penalty: 25,
                            points_adjustment: 0,
                            avatar_storage_url: None,
                        },
                        StandingsTableEntry {
                            rank_no: 2,
                            team_id: 11,
                            team_name: "山东泰山".to_string(),
                            played: 10,
                            wins: 7,
                            draws: 2,
                            losses: 1,
                            goals_for: 18,
                            goals_against: 9,
                            goal_difference: 9,
                            points: 23,
                            points_without_penalty: 23,
                            points_adjustment: 0,
                            avatar_storage_url: None,
                        },
                    ],
                },
            ],
            team_categories: vec![],
            player_categories: vec![],
        };

        let grounding = build_live_rankings_grounding_context(&rankings)
            .expect("grounding should be built from live rankings");

        assert!(grounding.contains("站内实时榜单"));
        assert!(grounding.contains("2026"));
        assert!(grounding.contains("第 1 名：上海海港，25 分"));
        assert!(grounding.contains("第 2 名：山东泰山，23 分"));
    }

    #[test]
    fn parse_stream_payload_extracts_delta_text_and_done_marker() {
        let delta = parse_stream_payload(
            r#"{"id":"1","choices":[{"delta":{"content":"你好"},"index":0}]}"#,
        )
        .expect("delta payload should parse");
        assert_eq!(delta.delta_text.as_deref(), Some("你好"));
        assert!(!delta.is_done);

        let done = parse_stream_payload("[DONE]").expect("done payload should parse");
        assert_eq!(done.delta_text, None);
        assert!(done.is_done);
    }

    #[test]
    fn normalize_ai_reply_removes_markdown_emphasis() {
        let normalized = normalize_ai_reply("根据最新数据，**上海海港**暂列第一。");

        assert_eq!(normalized, "根据最新数据，上海海港暂列第一。");
    }

    #[test]
    fn normalize_ai_reply_removes_think_blocks() {
        let normalized = normalize_ai_reply(
            "我来为您搜索。<think>先查最新积分榜</think>根据结果，暂时无法确认。",
        );

        assert_eq!(normalized, "我来为您搜索。根据结果，暂时无法确认。");
    }

    #[test]
    fn guard_grounded_reply_replaces_unsourced_live_standings_claims() {
        let guarded = guard_grounded_reply(
            "今天中超积分榜前二名是谁",
            "根据最新数据，今天中超积分榜前两名是：1.**上海海港** 2.**山东泰山**。您是V3会员，可以在足球洞察产品内查看完整榜单。",
        );

        assert!(guarded.contains("暂时无法确认"));
        assert!(!guarded.contains("上海海港"));
        assert!(!guarded.contains("**"));
    }

    #[test]
    fn guard_grounded_reply_also_blocks_standings_questions_without_today_keyword() {
        let guarded = guard_grounded_reply(
            "中超积分榜第二名是谁",
            "根据当前中超积分榜，第二名是山东泰山，积22分，仅次于上海海港的23分。作为V3会员，您可以在足球洞察产品中查看更多数据分析。",
        );

        assert!(guarded.contains("暂时无法确认"));
        assert!(!guarded.contains("山东泰山"));
    }

    #[test]
    fn buffer_stream_response_for_live_standings_queries() {
        assert!(!should_buffer_stream_response("当前中超的积分榜"));
        assert!(!should_buffer_stream_response("中超积分榜第二名是谁"));
        assert!(!should_buffer_stream_response(
            "罗纳尔迪尼奥巅峰期最大的特点是什么"
        ));
    }
}
