use crate::match_outlook::{application::OutlookService, domain::*};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use std::sync::Arc;
#[derive(Deserialize)]
struct MatchQuery {
    match_id: i64,
}
#[derive(Deserialize)]
struct RecordQuery {
    season: i32,
}
pub fn routes(service: Arc<OutlookService>) -> Router {
    Router::new()
        .route("/api/v1/live/match-outlook", get(get_outlook))
        .route("/api/v1/live/match-outlook/record", get(get_record))
        .with_state(service)
}
fn map_error(e: ReadError) -> (StatusCode, String) {
    (
        if e == ReadError::Invalid {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        e.to_string(),
    )
}
async fn get_outlook(
    State(s): State<Arc<OutlookService>>,
    Query(q): Query<MatchQuery>,
) -> Result<Json<OutlookResponse>, (StatusCode, String)> {
    s.get(q.match_id).await.map(Json).map_err(map_error)
}
async fn get_record(
    State(s): State<Arc<OutlookService>>,
    Query(q): Query<RecordQuery>,
) -> Result<Json<Record>, (StatusCode, String)> {
    s.record(q.season).await.map(Json).map_err(map_error)
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;
    #[tokio::test]
    async fn both_routes_cover_ready_absent_and_invalid_parameters() {
        let app = routes(Arc::new(OutlookService::new(Arc::new(
            crate::match_outlook::application::tests::Fake,
        ))));
        for (query, status) in [
            ("?match_id=1", 200),
            ("?match_id=2", 200),
            ("?match_id=-1", 400),
            ("?match_id=oops", 400),
            ("/record?season=2026", 200),
            ("/record?season=2013", 400),
            ("/record?season=oops", 400),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/api/v1/live/match-outlook{query}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status().as_u16(), status, "{query}");
            if query == "?match_id=2" {
                let body = to_bytes(response.into_body(), 4096).await.unwrap();
                assert!(
                    String::from_utf8(body.to_vec())
                        .unwrap()
                        .contains("unavailable")
                );
            }
        }
    }
}
