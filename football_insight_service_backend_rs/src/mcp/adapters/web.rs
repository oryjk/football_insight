use std::sync::Arc;

use axum::{
    Router,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;

use crate::mcp::{
    adapters::protocol::{McpProtocol, parse_error_response},
    domain::PromoTarget,
    ports::PromoCodePort,
};

pub const MCP_PATH: &str = "/api/v1/mcp";

#[derive(Clone)]
pub struct McpWebState {
    pub protocol: Arc<McpProtocol>,
    pub codes: Arc<dyn PromoCodePort>,
}

pub fn routes(state: McpWebState) -> Router {
    Router::new()
        .route(
            MCP_PATH,
            get(method_not_allowed)
                .post(handle_rpc)
                .delete(method_not_allowed),
        )
        .route("/api/v1/mcp/promo/{target}", get(landing_page))
        .route("/api/v1/mcp/promo/{target}/code", get(code_image))
        .with_state(state)
}

/// Stateless server: no server-initiated SSE stream and no sessions to delete.
async fn method_not_allowed() -> Response {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        [(header::ALLOW, HeaderValue::from_static("POST"))],
    )
        .into_response()
}

async fn handle_rpc(State(state): State<McpWebState>, body: Bytes) -> Response {
    let message: Value = match serde_json::from_slice(&body) {
        Ok(message) => message,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, axum::Json(parse_error_response())).into_response();
        }
    };
    if let Some(method) = message.get("method").and_then(Value::as_str) {
        let tool = message
            .pointer("/params/name")
            .and_then(Value::as_str)
            .unwrap_or("");
        tracing::info!(method, tool, "mcp request");
    }
    match state.protocol.handle(message).await {
        Some(response) => axum::Json(response).into_response(),
        None => StatusCode::ACCEPTED.into_response(),
    }
}

async fn landing_page(Path(target): Path<String>) -> Response {
    let Some(target) = PromoTarget::from_slug(&target) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Html(landing_html(target)).into_response()
}

async fn code_image(State(state): State<McpWebState>, Path(target): Path<String>) -> Response {
    let Some(target) = PromoTarget::from_slug(&target) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let image = match state.codes.image(target).await {
        Ok(image) => image,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let Ok(bytes) = STANDARD.decode(image.image_base64) else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    (
        [
            (header::CONTENT_TYPE, image.mime_type),
            (header::CACHE_CONTROL, "public, max-age=86400".to_string()),
        ],
        bytes,
    )
        .into_response()
}

fn landing_html(target: PromoTarget) -> String {
    let title = match target {
        PromoTarget::Rankings => "中超积分榜 · 射手榜 · 助攻榜",
        PromoTarget::Home => "中超每轮赛果 · 比赛详情",
    };
    LANDING_TEMPLATE
        .replace("{{TITLE}}", title)
        .replace("{{CTA}}", target.call_to_action())
        .replace(
            "{{CODE}}",
            &format!("/api/v1/mcp/promo/{}/code", target.slug()),
        )
}

const LANDING_TEMPLATE: &str = r#"<!doctype html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>足球洞察小程序</title>
<style>
  :root { --bg: #f6f6f4; --card: #ffffff; --fg: #1d2129; --muted: #6b7280; --accent: #b4232c; }
  @media (prefers-color-scheme: dark) {
    :root { --bg: #111214; --card: #1c1d21; --fg: #f3f4f6; --muted: #9ca3af; --accent: #f87171; }
  }
  * { box-sizing: border-box; }
  body { margin: 0; min-height: 100vh; display: flex; align-items: center; justify-content: center;
    background: var(--bg); color: var(--fg); padding: 16px;
    font: 15px/1.6 -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", sans-serif; }
  .card { width: 100%; max-width: 360px; background: var(--card); border-radius: 16px; padding: 28px 24px;
    text-align: center; box-shadow: 0 6px 24px rgba(0,0,0,.08); }
  h1 { font-size: 20px; margin: 0 0 4px; }
  .sub { color: var(--muted); margin: 0 0 20px; font-size: 13px; }
  img { width: 220px; height: 220px; border-radius: 12px; background: #fff; }
  .cta { color: var(--accent); font-weight: 600; margin: 16px 0 4px; }
  .hint { color: var(--muted); font-size: 12px; margin: 0; }
</style>
</head>
<body>
<main class="card">
  <h1>「足球洞察」小程序</h1>
  <p class="sub">{{TITLE}}</p>
  <img src="{{CODE}}" alt="足球洞察小程序码" width="220" height="220">
  <p class="cta">{{CTA}}</p>
  <p class="hint">手机上可长按识别，或用微信「扫一扫」</p>
</main>
</body>
</html>
"#;
