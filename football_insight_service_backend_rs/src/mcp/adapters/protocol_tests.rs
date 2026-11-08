use std::{sync::Arc, time::Duration};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::*;
use crate::{
    mcp::{
        adapters::web::{McpWebState, routes},
        application::{McpQueryService, tests::FakeRepository},
        domain::PromoTarget,
        ports::PromoCodePort,
    },
    share_code::domain::{CodeImage, ShareCodeError},
};

struct FakeCodes;

#[async_trait::async_trait]
impl PromoCodePort for FakeCodes {
    async fn image(&self, target: PromoTarget) -> Result<CodeImage, ShareCodeError> {
        match target {
            PromoTarget::Rankings => Ok(CodeImage {
                image_base64: "iVBORw0KGgo=".into(),
                mime_type: "image/png".into(),
            }),
            PromoTarget::Home => Err(ShareCodeError::Unavailable),
        }
    }
}

fn protocol() -> McpProtocol {
    McpProtocol::new(
        McpQueryService::new(
            Arc::new(FakeRepository::default()),
            "https://match.example".into(),
            Duration::from_secs(60),
        ),
        "https://match.example/",
    )
}

async fn call(message: Value) -> Value {
    protocol().handle(message).await.expect("response")
}

async fn tool(name: &str, arguments: Value) -> Value {
    call(json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": name, "arguments": arguments}}))
        .await["result"]
        .clone()
}

#[tokio::test]
async fn initialize_negotiates_known_versions_and_falls_back_to_latest() {
    let result = call(json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}})).await;
    assert_eq!(result["id"], 1);
    assert_eq!(result["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(result["result"]["serverInfo"]["name"], "football-insight");
    assert!(result["result"]["capabilities"]["tools"].is_object());
    assert!(
        result["result"]["instructions"]
            .as_str()
            .unwrap()
            .contains("足球洞察")
    );

    let result = call(json!({"jsonrpc": "2.0", "id": "a", "method": "initialize", "params": {"protocolVersion": "1999-01-01"}})).await;
    assert_eq!(
        result["result"]["protocolVersion"],
        SUPPORTED_PROTOCOL_VERSIONS[0]
    );
}

#[tokio::test]
async fn notifications_and_client_responses_get_no_body() {
    let p = protocol();
    assert!(
        p.handle(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))
            .await
            .is_none()
    );
    assert!(
        p.handle(json!({"jsonrpc": "2.0", "id": 3, "result": {}}))
            .await
            .is_none()
    );
}

#[tokio::test]
async fn malformed_and_unknown_requests_return_jsonrpc_errors() {
    assert_eq!(
        call(json!([{"jsonrpc": "2.0", "id": 1, "method": "ping"}])).await["error"]["code"],
        -32600
    );
    assert_eq!(
        call(json!({"jsonrpc": "2.0", "id": 1})).await["error"]["code"],
        -32600
    );
    assert_eq!(
        call(json!({"jsonrpc": "2.0", "id": 1, "method": "prompts/list"})).await["error"]["code"],
        -32601
    );
    let unknown = call(
        json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "nope"}}),
    )
    .await;
    assert_eq!(unknown["error"]["code"], -32602);
    assert_eq!(
        call(json!({"jsonrpc": "2.0", "id": 9, "method": "ping"})).await["result"],
        json!({})
    );
}

#[tokio::test]
async fn tools_list_declares_read_only_tools_linked_to_the_card() {
    let result = call(json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})).await;
    let tools = result["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(
        names,
        [TOOL_STANDINGS, TOOL_ROUND_RESULTS, TOOL_PLAYER_LEADERBOARD]
    );
    for tool in tools {
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
        assert_eq!(tool["_meta"]["ui"]["resourceUri"], CARD_RESOURCE_URI);
        assert_eq!(tool["inputSchema"]["type"], "object");
    }
    let categories = tools[2]["inputSchema"]["properties"]["category"]["enum"]
        .as_array()
        .unwrap();
    assert!(categories.contains(&json!("goals")) && categories.contains(&json!("assists")));
}

#[tokio::test]
async fn card_resource_is_listed_and_allows_images_from_the_public_origin() {
    let list = call(json!({"jsonrpc": "2.0", "id": 1, "method": "resources/list"})).await;
    assert_eq!(list["result"]["resources"][0]["uri"], CARD_RESOURCE_URI);
    let read = call(json!({"jsonrpc": "2.0", "id": 2, "method": "resources/read", "params": {"uri": CARD_RESOURCE_URI}})).await;
    let content = &read["result"]["contents"][0];
    assert_eq!(content["mimeType"], CARD_MIME_TYPE);
    assert!(content["text"].as_str().unwrap().contains("ui/initialize"));
    assert_eq!(
        content["_meta"]["ui"]["csp"]["resourceDomains"],
        json!(["https://match.example"])
    );
    let missing = call(json!({"jsonrpc": "2.0", "id": 3, "method": "resources/read", "params": {"uri": "ui://other"}})).await;
    assert_eq!(missing["error"]["code"], -32602);
}

#[tokio::test]
async fn standings_tool_returns_markdown_structured_content_and_promo_link() {
    let result = tool(TOOL_STANDINGS, json!({})).await;
    assert_eq!(result["isError"], false);
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("| 2 | 北京国安 | 30 | 15 | 7 | 8 | 50 | 30 | 20 | 50 | -5 | 胜胜平负胜 |"),
        "{text}"
    );
    assert!(text.contains("https://match.example/api/v1/mcp/promo/rankings"));
    assert_eq!(result["structuredContent"]["kind"], "standings");
    assert_eq!(result["structuredContent"]["rows"][0]["team"], "大连英博");
}

#[tokio::test]
async fn round_results_tool_accepts_numeric_strings_and_reports_bad_input_as_tool_errors() {
    let result = tool(TOOL_ROUND_RESULTS, json!({"round": "12"})).await;
    assert_eq!(result["structuredContent"]["round"], 12);
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("| 2026-11-08 | 15:30 | 上海海港 | 2-2 | 辽宁铁人 | 已结束 |"),
        "{text}"
    );
    assert!(text.contains("| 上海海港 | vs | 辽宁铁人 | 未开赛 |"));

    let latest = tool(TOOL_ROUND_RESULTS, json!(null)).await;
    assert_eq!(latest["structuredContent"]["round"], 30);

    for bad in [json!({"round": "abc"}), json!({"round": 99})] {
        let error = tool(TOOL_ROUND_RESULTS, bad).await;
        assert_eq!(error["isError"], true);
        assert!(error.get("structuredContent").is_none());
    }
}

#[tokio::test]
async fn leaderboard_tool_defaults_to_goals_and_shows_penalties() {
    let result = tool(TOOL_PLAYER_LEADERBOARD, json!({"limit": 2})).await;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("| 排名 | 球员 | 球队 | 进球 | 其中点球 |"));
    assert!(text.contains("| 1 | 奥斯卡 | 成都蓉城 | 23 | 2 |"));
    assert!(text.contains("| 2 | 费利佩 | 成都蓉城 | 18 | 0 |"));
    assert!(!text.contains("韦世豪"));
    let bad = tool(TOOL_PLAYER_LEADERBOARD, json!({"limit": -1})).await;
    assert_eq!(bad["isError"], true);
}

async fn http(request: Request<Body>) -> (StatusCode, Vec<u8>, Option<String>) {
    let app = routes(McpWebState {
        protocol: Arc::new(protocol()),
        codes: Arc::new(FakeCodes),
    });
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_string());
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    (status, body, content_type)
}

fn post(body: &str) -> Request<Body> {
    Request::post("/api/v1/mcp")
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn http_transport_status_codes() {
    let (status, body, content_type) =
        http(post(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type.as_deref(), Some("application/json"));
    assert_eq!(
        serde_json::from_slice::<Value>(&body).unwrap()["result"],
        json!({})
    );

    let (status, body, _) = http(post(
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    ))
    .await;
    assert_eq!((status, body.is_empty()), (StatusCode::ACCEPTED, true));

    let (status, body, _) = http(post("{not json")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        serde_json::from_slice::<Value>(&body).unwrap()["error"]["code"],
        -32700
    );

    let (status, _, _) = http(Request::get("/api/v1/mcp").body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn promo_landing_page_and_code_image() {
    let (status, body, content_type) = http(
        Request::get("/api/v1/mcp/promo/rankings")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.unwrap().starts_with("text/html"));
    let html = String::from_utf8(body).unwrap();
    assert!(html.contains(r#"src="/api/v1/mcp/promo/rankings/code""#));
    assert!(html.contains("看完整榜单"));

    let (status, body, content_type) = http(
        Request::get("/api/v1/mcp/promo/rankings/code")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(
        (status, content_type.as_deref()),
        (StatusCode::OK, Some("image/png"))
    );
    assert!(body.starts_with(b"\x89PNG"));

    let (status, _, _) = http(
        Request::get("/api/v1/mcp/promo/home/code")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    let (status, _, _) = http(
        Request::get("/api/v1/mcp/promo/nope")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
