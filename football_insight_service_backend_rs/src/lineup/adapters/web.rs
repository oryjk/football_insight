use super::dto::{PlayersResponse, SquadResponse};
use crate::lineup::{application::LineupService, domain::LineupReadError};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use std::sync::Arc;

pub fn routes(service: Arc<LineupService>) -> Router {
    Router::new()
        .route("/api/v1/lineups/squads/{team_id}", get(get_squad))
        .route("/api/v1/lineups/players/search", get(search_players))
        .route("/api/v1/lineups/players", get(find_players))
        .with_state(service)
}

#[derive(Deserialize)]
struct SearchParams {
    #[serde(default)]
    q: String,
}

#[derive(Deserialize)]
struct LookupParams {
    #[serde(default)]
    ids: String,
}

/// 选人弹层的搜索：全部中超球队名单里名字包含 `q` 的球员。
async fn search_players(
    State(s): State<Arc<LineupService>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<PlayersResponse>, (StatusCode, String)> {
    s.search_players(&params.q)
        .await
        .map(|players| {
            Json(PlayersResponse {
                players: players.into_iter().map(Into::into).collect(),
            })
        })
        .map_err(map_error)
}

/// `ids=1,2,3`：补查阵容里外队球员的资料。
async fn find_players(
    State(s): State<Arc<LineupService>>,
    Query(params): Query<LookupParams>,
) -> Result<Json<PlayersResponse>, (StatusCode, String)> {
    let ids = params
        .ids
        .split(',')
        .map(|id| id.trim().parse::<i64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| map_error(LineupReadError::Invalid))?;
    s.find_players(&ids)
        .await
        .map(|players| {
            Json(PlayersResponse {
                players: players.into_iter().map(Into::into).collect(),
            })
        })
        .map_err(map_error)
}

fn map_error(e: LineupReadError) -> (StatusCode, String) {
    let status = match e {
        LineupReadError::Invalid => StatusCode::BAD_REQUEST,
        LineupReadError::NotFound => StatusCode::NOT_FOUND,
        LineupReadError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, e.to_string())
}

async fn get_squad(
    State(s): State<Arc<LineupService>>,
    Path(team_id): Path<i64>,
) -> Result<Json<SquadResponse>, (StatusCode, String)> {
    s.squad(team_id)
        .await
        .map(|squad| Json(squad.into()))
        .map_err(map_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lineup::{
        application::tests::{FakeSquads, player},
        domain::Position,
    };
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;

    fn app(missing_table: bool) -> Router {
        routes(Arc::new(LineupService::new(Arc::new(FakeSquads {
            players: vec![
                player(1, Some(1), Some(Position::GK), 2026, true),
                player(2, Some(9), Some(Position::FW), 2025, false),
            ],
            missing_table,
        }))))
    }

    async fn call(app: Router, uri: &str) -> (u16, String) {
        let res = app
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = res.status().as_u16();
        let body = String::from_utf8(
            to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn returns_grouped_squad_without_internal_flags() {
        let (status, body) = call(app(false), "/api/v1/lineups/squads/77680").await;
        assert_eq!(status, 200);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["team"]["team_name"], "成都蓉城");
        assert_eq!(json["active"][0]["position"], "GK");
        assert_eq!(json["former"][0]["last_season"], 2025);
        assert!(json["active"][0].get("active").is_none());
    }

    #[tokio::test]
    async fn searches_and_looks_up_players_with_their_club() {
        let (status, body) = call(app(false), "/api/v1/lineups/players/search?q=P2").await;
        assert_eq!(status, 200);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["players"][0]["player_id"], 2);
        assert_eq!(json["players"][0]["team"]["team_name"], "成都蓉城");

        let (status, body) = call(app(false), "/api/v1/lineups/players?ids=1,2").await;
        assert_eq!(status, 200);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["players"].as_array().unwrap().len(), 2);

        for bad in [
            "/api/v1/lineups/players/search?q=",
            "/api/v1/lineups/players/search?q=%20%20",
            "/api/v1/lineups/players/search?q=123456789012345678901",
            "/api/v1/lineups/players?ids=",
            "/api/v1/lineups/players?ids=1,x",
            "/api/v1/lineups/players?ids=0",
        ] {
            assert_eq!(call(app(false), bad).await.0, 400, "{bad}");
        }
        let many = (1..=31)
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            call(app(false), &format!("/api/v1/lineups/players?ids={many}"))
                .await
                .0,
            400
        );
    }

    #[tokio::test]
    async fn maps_errors_to_statuses() {
        assert_eq!(call(app(false), "/api/v1/lineups/squads/0").await.0, 400);
        assert_eq!(call(app(false), "/api/v1/lineups/squads/1").await.0, 404);
        assert_eq!(call(app(false), "/api/v1/lineups/squads/abc").await.0, 400);
        assert_eq!(call(app(true), "/api/v1/lineups/squads/77680").await.0, 503);
    }
}
