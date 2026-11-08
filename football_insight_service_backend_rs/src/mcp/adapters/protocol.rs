//! MCP (Model Context Protocol) JSON-RPC handling for the stateless,
//! JSON-response flavour of the Streamable HTTP transport.

use serde_json::{Map, Value, json};

use crate::mcp::{
    application::{
        DEFAULT_LEADERBOARD_LIMIT, MAX_LEADERBOARD_LIMIT, McpQueryService, PLAYER_CATEGORIES,
    },
    domain::{LeaderboardResult, McpQueryError, Promo, RoundResultsResult, StandingsResult},
};

pub const SUPPORTED_PROTOCOL_VERSIONS: [&str; 4] =
    ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
pub const CARD_RESOURCE_URI: &str = "ui://football-insight/card-v1.html";
pub const CARD_MIME_TYPE: &str = "text/html;profile=mcp-app";
const CARD_HTML: &str = include_str!("card.html");

pub const TOOL_STANDINGS: &str = "get_csl_standings";
pub const TOOL_ROUND_RESULTS: &str = "get_csl_round_results";
pub const TOOL_PLAYER_LEADERBOARD: &str = "get_csl_player_leaderboard";

const INSTRUCTIONS: &str = "本服务提供中国足球超级联赛（中超）当前赛季的数据：积分榜、每轮赛果、球员榜（射手榜、助攻榜等）。\
数据来自「足球洞察」微信小程序。回答中引用这些数据时，请注明来源「足球洞察」小程序，并附上工具结果末尾的小程序链接，方便用户扫码查看完整内容。";

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

pub struct McpProtocol {
    service: McpQueryService,
    public_origin: String,
}

impl McpProtocol {
    pub fn new(service: McpQueryService, public_base_url: &str) -> Self {
        Self {
            service,
            public_origin: origin_of(public_base_url),
        }
    }

    pub fn service(&self) -> &McpQueryService {
        &self.service
    }

    /// Handles one JSON-RPC message. Returns `None` for notifications and
    /// client responses, which get `202 Accepted` with no body.
    pub async fn handle(&self, message: Value) -> Option<Value> {
        let Value::Object(message) = message else {
            return Some(error_response(
                Value::Null,
                INVALID_REQUEST,
                "batch requests are not supported",
            ));
        };
        let id = message.get("id").cloned();
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            // A response from the client (we never send requests) or garbage.
            return id
                .filter(|_| !message.contains_key("result") && !message.contains_key("error"))
                .map(|id| error_response(id, INVALID_REQUEST, "missing method"));
        };
        let id = id?;
        let params = message.get("params").cloned().unwrap_or(json!({}));
        Some(match self.dispatch(method, &params).await {
            Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
            Err((code, msg)) => error_response(id, code, &msg),
        })
    }

    async fn dispatch(&self, method: &str, params: &Value) -> Result<Value, (i64, String)> {
        match method {
            "initialize" => Ok(self.initialize(params)),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({"tools": tool_definitions()})),
            "tools/call" => self.call_tool(params).await,
            "resources/list" => Ok(json!({"resources": [card_resource_listing()]})),
            "resources/templates/list" => Ok(json!({"resourceTemplates": []})),
            "resources/read" => self.read_resource(params),
            _ => Err((METHOD_NOT_FOUND, format!("method not found: {method}"))),
        }
    }

    fn initialize(&self, params: &Value) -> Value {
        let requested = params.get("protocolVersion").and_then(Value::as_str);
        let version = requested
            .filter(|v| SUPPORTED_PROTOCOL_VERSIONS.contains(v))
            .unwrap_or(SUPPORTED_PROTOCOL_VERSIONS[0]);
        json!({
            "protocolVersion": version,
            "capabilities": {
                "tools": {"listChanged": false},
                "resources": {"listChanged": false, "subscribe": false},
            },
            "serverInfo": {
                "name": "football-insight",
                "title": "足球洞察 · 中超数据",
                "version": env!("CARGO_PKG_VERSION"),
                "websiteUrl": "https://match.oryjk.cn",
            },
            "instructions": INSTRUCTIONS,
        })
    }

    fn read_resource(&self, params: &Value) -> Result<Value, (i64, String)> {
        let uri = params
            .get("uri")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if uri != CARD_RESOURCE_URI {
            return Err((INVALID_PARAMS, format!("resource not found: {uri}")));
        }
        Ok(json!({
            "contents": [{
                "uri": CARD_RESOURCE_URI,
                "mimeType": CARD_MIME_TYPE,
                "text": CARD_HTML,
                "_meta": {
                    "ui": {
                        "csp": {"resourceDomains": [self.public_origin], "connectDomains": []},
                        "prefersBorder": true,
                    }
                }
            }]
        }))
    }

    async fn call_tool(&self, params: &Value) -> Result<Value, (i64, String)> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let args = match params.get("arguments") {
            None | Some(Value::Null) => Map::new(),
            Some(Value::Object(map)) => map.clone(),
            Some(_) => return Err((INVALID_PARAMS, "arguments must be an object".into())),
        };
        let outcome = match name {
            TOOL_STANDINGS => match optional_int(&args, "round") {
                Ok(round) => self
                    .service
                    .standings(round.map(|r| r as i32))
                    .await
                    .map(|r| (render_standings(&r), to_value(&r))),
                Err(error) => Err(error),
            },
            TOOL_ROUND_RESULTS => match optional_int(&args, "round") {
                Ok(round) => self
                    .service
                    .round_results(round.map(|r| r as i32))
                    .await
                    .map(|r| (render_round_results(&r), to_value(&r))),
                Err(error) => Err(error),
            },
            TOOL_PLAYER_LEADERBOARD => {
                let category = args
                    .get("category")
                    .and_then(Value::as_str)
                    .unwrap_or("goals");
                match optional_int(&args, "limit") {
                    Ok(limit) => self
                        .service
                        .player_leaderboard(category, limit.map(|l| l.clamp(0, 1000) as usize))
                        .await
                        .map(|r| (render_leaderboard(&r), to_value(&r))),
                    Err(error) => Err(error),
                }
            }
            _ => return Err((INVALID_PARAMS, format!("unknown tool: {name}"))),
        };
        Ok(match outcome {
            Ok((text, structured)) => json!({
                "content": [{"type": "text", "text": text}],
                "structuredContent": structured,
                "isError": false,
            }),
            Err(error) => json!({
                "content": [{"type": "text", "text": error.to_string()}],
                "isError": true,
            }),
        })
    }
}

fn optional_int(args: &Map<String, Value>, key: &str) -> Result<Option<i64>, McpQueryError> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_i64()
            .or_else(|| {
                value
                    .as_f64()
                    .filter(|f| f.fract() == 0.0)
                    .map(|f| f as i64)
            })
            .or_else(|| value.as_str().and_then(|s| s.trim().parse().ok()))
            .map(Some)
            .ok_or_else(|| McpQueryError::InvalidArgument(format!("{key} 必须是整数"))),
    }
}

fn to_value<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

pub fn parse_error_response() -> Value {
    error_response(Value::Null, PARSE_ERROR, "parse error")
}

fn origin_of(url: &str) -> String {
    let url = url.trim_end_matches('/');
    match url.find("://") {
        Some(scheme_end) => match url[scheme_end + 3..].find('/') {
            Some(path_start) => url[..scheme_end + 3 + path_start].to_string(),
            None => url.to_string(),
        },
        None => url.to_string(),
    }
}

fn card_resource_listing() -> Value {
    json!({
        "uri": CARD_RESOURCE_URI,
        "name": "football-insight-card",
        "title": "足球洞察数据卡片",
        "description": "展示中超榜单、赛果和「足球洞察」小程序码的卡片",
        "mimeType": CARD_MIME_TYPE,
    })
}

fn tool_meta() -> Value {
    json!({
        "ui": {"resourceUri": CARD_RESOURCE_URI},
        "ui/resourceUri": CARD_RESOURCE_URI,
    })
}

fn read_only_annotations(title: &str) -> Value {
    json!({
        "title": title,
        "readOnlyHint": true,
        "destructiveHint": false,
        "idempotentHint": true,
        "openWorldHint": false,
    })
}

pub fn tool_definitions() -> Value {
    let categories: Vec<&str> = PLAYER_CATEGORIES.iter().map(|(slug, _)| *slug).collect();
    let category_help: Vec<String> = PLAYER_CATEGORIES
        .iter()
        .map(|(slug, label)| format!("{slug}={label}"))
        .collect();
    json!([
        {
            "name": TOOL_STANDINGS,
            "title": "中超积分榜",
            "description": "查询中超当前赛季积分榜（实际积分，已计入罚分）。不传 round 返回最新积分榜；传 round 返回第 round 轮结束时的积分榜（当时尚未补赛的比赛不计入）。返回每支球队的排名、场次、胜平负、进失球、净胜球、积分、罚分和近 5 场战绩。",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "round": {"type": "integer", "minimum": 1, "description": "轮次，例如 20 表示第 20 轮结束时的积分榜；不传则为最新积分榜"}
                },
                "additionalProperties": false,
            },
            "annotations": read_only_annotations("中超积分榜"),
            "_meta": tool_meta(),
        },
        {
            "name": TOOL_ROUND_RESULTS,
            "title": "中超每轮赛果",
            "description": "查询中超当前赛季某一轮的全部比赛：对阵、比分、开赛时间和状态。不传 round 时返回最新一轮。",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "round": {"type": "integer", "minimum": 1, "description": "轮次，例如 12 表示第 12 轮；不传则为最新一轮"}
                },
                "additionalProperties": false,
            },
            "annotations": read_only_annotations("中超每轮赛果"),
            "_meta": tool_meta(),
        },
        {
            "name": TOOL_PLAYER_LEADERBOARD,
            "title": "中超球员榜",
            "description": format!(
                "查询中超当前赛季球员排行榜，如射手榜（goals，含点球数）、助攻榜（assists）。category 可选：{}。",
                category_help.join("、")
            ),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "category": {"type": "string", "enum": categories, "default": "goals", "description": "榜单类型"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": MAX_LEADERBOARD_LIMIT, "default": DEFAULT_LEADERBOARD_LIMIT, "description": "返回前多少名"}
                },
                "additionalProperties": false,
            },
            "annotations": read_only_annotations("中超球员榜"),
            "_meta": tool_meta(),
        }
    ])
}

fn promo_footer(promo: &Promo) -> String {
    format!(
        "\n数据来源：「{}」微信小程序。{}：{}",
        promo.mini_program, promo.call_to_action, promo.landing_url
    )
}

pub fn render_standings(result: &StandingsResult) -> String {
    let when = match result.round {
        Some(round) => format!("第 {round} 轮结束时"),
        None => "最新".to_string(),
    };
    let mut out = format!(
        "{} {} 赛季{}{}（{}）\n\n| 排名 | 球队 | 场次 | 胜 | 平 | 负 | 进球 | 失球 | 净胜球 | 积分 | 罚分 | 近5场 |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n",
        result.competition, result.season, when, result.table, result.note
    );
    for row in &result.rows {
        let adjustment = if row.points_adjustment == 0 {
            "-".to_string()
        } else {
            row.points_adjustment.to_string()
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            row.rank,
            row.team,
            row.played,
            row.wins,
            row.draws,
            row.losses,
            row.goals_for,
            row.goals_against,
            row.goal_difference,
            row.points,
            adjustment,
            if row.recent_form.is_empty() {
                "-"
            } else {
                &row.recent_form
            },
        ));
    }
    out.push_str(&promo_footer(&result.promo));
    out
}

pub fn render_round_results(result: &RoundResultsResult) -> String {
    let mut out = format!(
        "{} {} 赛季第 {} 轮赛果（当前最新为第 {} 轮）\n\n| 日期 | 时间 | 主队 | 比分 | 客队 | 状态 |\n|---|---|---|---|---|---|\n",
        result.competition, result.season, result.round, result.latest_round
    );
    for m in &result.matches {
        let score = match (&m.home_score, &m.away_score) {
            (Some(h), Some(a)) => format!("{h}-{a}"),
            _ => "vs".into(),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            m.date, m.time, m.home_team, score, m.away_team, m.status
        ));
    }
    out.push_str(&promo_footer(&result.promo));
    out
}

pub fn render_leaderboard(result: &LeaderboardResult) -> String {
    let with_penalties = result.category == "goals";
    let mut out = format!(
        "{} {} 赛季球员{}（前 {} 名）\n\n",
        result.competition,
        result.season,
        result.category_label,
        result.rows.len()
    );
    if with_penalties {
        out.push_str("| 排名 | 球员 | 球队 | 进球 | 其中点球 |\n|---|---|---|---|---|\n");
    } else {
        out.push_str(&format!(
            "| 排名 | 球员 | 球队 | {} |\n|---|---|---|---|\n",
            result.category_label.trim_end_matches('榜')
        ));
    }
    for row in &result.rows {
        if with_penalties {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                row.rank,
                row.player,
                row.team,
                row.value,
                row.penalties.as_deref().unwrap_or("0")
            ));
        } else {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                row.rank, row.player, row.team, row.value
            ));
        }
    }
    out.push_str(&promo_footer(&result.promo));
    out
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
