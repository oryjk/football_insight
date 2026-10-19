use super::dto::SquadResponse;
use crate::lineup::{application::LineupService, domain::LineupReadError};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use std::sync::Arc;

pub fn routes(service: Arc<LineupService>) -> Router {
    Router::new()
        .route("/api/v1/lineups/squads/{team_id}", get(get_squad))
        .with_state(service)
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
    async fn maps_errors_to_statuses() {
        assert_eq!(call(app(false), "/api/v1/lineups/squads/0").await.0, 400);
        assert_eq!(call(app(false), "/api/v1/lineups/squads/1").await.0, 404);
        assert_eq!(call(app(false), "/api/v1/lineups/squads/abc").await.0, 400);
        assert_eq!(call(app(true), "/api/v1/lineups/squads/77680").await.0, 503);
    }
}
