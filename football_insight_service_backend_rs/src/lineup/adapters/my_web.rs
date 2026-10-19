use super::dto::{DraftResponse, EditorResponse, LineupResponse, PublishResponse, SaveRequest};
use crate::{
    auth::ports::token_port::TokenPort,
    lineup::{application::my_lineup::MyLineupService, domain::LineupSaveError},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::{get, put},
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct MyLineupWebState {
    pub service: Arc<MyLineupService>,
    pub token_port: Arc<dyn TokenPort>,
}

pub fn my_routes(state: Arc<MyLineupWebState>) -> Router {
    Router::new()
        .route("/api/v1/lineups/me/{team_id}", get(get_mine).put(save_mine))
        .route("/api/v1/lineups/me/{team_id}/editor", get(get_editor))
        .route("/api/v1/lineups/me/{team_id}/draft", put(save_draft))
        .route("/api/v1/lineups/me/{team_id}/publish", put(publish))
        .with_state(state)
}

fn authenticate(
    headers: &HeaderMap,
    token_port: &dyn TokenPort,
) -> Result<Uuid, (StatusCode, String)> {
    let unauthorized = || (StatusCode::UNAUTHORIZED, "请先登录".to_string());
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(unauthorized)?;
    token_port
        .verify_token(token)
        .map(|c| c.sub)
        .map_err(|_| unauthorized())
}

fn map_error(e: LineupSaveError) -> (StatusCode, String) {
    let status = match e {
        LineupSaveError::Unauthorized => StatusCode::UNAUTHORIZED,
        LineupSaveError::InvalidFormation
        | LineupSaveError::IncompleteLineup
        | LineupSaveError::InvalidSlots
        | LineupSaveError::PlayerNotInSquad
        | LineupSaveError::DuplicatePlayer => StatusCode::BAD_REQUEST,
        LineupSaveError::TeamNotFound => StatusCode::NOT_FOUND,
        LineupSaveError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, e.to_string())
}

async fn get_mine(
    State(s): State<Arc<MyLineupWebState>>,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
) -> Result<Json<LineupResponse>, (StatusCode, String)> {
    let user = authenticate(&headers, s.token_port.as_ref())?;
    s.service
        .get(user, team_id)
        .await
        .map(|lineup| {
            Json(LineupResponse {
                lineup: lineup.map(Into::into),
            })
        })
        .map_err(map_error)
}

async fn save_mine(
    State(s): State<Arc<MyLineupWebState>>,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<LineupResponse>, (StatusCode, String)> {
    let user = authenticate(&headers, s.token_port.as_ref())?;
    s.service
        .save(
            user,
            team_id,
            &req.formation,
            req.slots.into_iter().map(Into::into).collect(),
        )
        .await
        .map(|lineup| {
            Json(LineupResponse {
                lineup: Some(lineup.into()),
            })
        })
        .map_err(map_error)
}

async fn get_editor(
    State(s): State<Arc<MyLineupWebState>>,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
) -> Result<Json<EditorResponse>, (StatusCode, String)> {
    let user = authenticate(&headers, s.token_port.as_ref())?;
    s.service
        .editor(user, team_id)
        .await
        .map(|editor| Json(editor.into()))
        .map_err(map_error)
}

async fn save_draft(
    State(s): State<Arc<MyLineupWebState>>,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<DraftResponse>, (StatusCode, String)> {
    let user = authenticate(&headers, s.token_port.as_ref())?;
    s.service
        .save_draft(
            user,
            team_id,
            &req.formation,
            req.slots.into_iter().map(Into::into).collect(),
        )
        .await
        .map(|draft| {
            Json(DraftResponse {
                draft: draft.into(),
            })
        })
        .map_err(map_error)
}

async fn publish(
    State(s): State<Arc<MyLineupWebState>>,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<PublishResponse>, (StatusCode, String)> {
    let user = authenticate(&headers, s.token_port.as_ref())?;
    s.service
        .publish(
            user,
            team_id,
            &req.formation,
            req.slots.into_iter().map(Into::into).collect(),
        )
        .await
        .map(|published| Json(published.into()))
        .map_err(map_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::ports::token_port::{AuthTokenClaims, WechatBindTokenClaims, WechatBindTokenPayload},
        lineup::application::{
            my_lineup::tests::{FakeLineups, FakeUserAccess},
            tests::FakeSquads,
        },
    };
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use chrono::{DateTime, Utc};
    use tower::ServiceExt;
    use uuid::Uuid;

    const USER: Uuid = Uuid::from_u128(0x1111);

    struct FakeTokenPort;

    impl TokenPort for FakeTokenPort {
        fn issue_token(&self, _: Uuid, _: &str, _: DateTime<Utc>) -> anyhow::Result<String> {
            unimplemented!()
        }
        fn verify_token(&self, token: &str) -> anyhow::Result<AuthTokenClaims> {
            if token == "good" || token == "other" {
                Ok(AuthTokenClaims {
                    sub: if token == "other" {
                        Uuid::from_u128(0x2222)
                    } else {
                        USER
                    },
                    account_identifier: "tester".into(),
                    exp: Utc::now() + chrono::Duration::hours(1),
                })
            } else {
                anyhow::bail!("bad token")
            }
        }
        fn issue_wechat_bind_token(
            &self,
            _: WechatBindTokenPayload,
            _: DateTime<Utc>,
        ) -> anyhow::Result<String> {
            unimplemented!()
        }
        fn verify_wechat_bind_token(&self, _: &str) -> anyhow::Result<WechatBindTokenClaims> {
            unimplemented!()
        }
    }

    fn app(members: Vec<i64>) -> Router {
        app_with_access(members, FakeUserAccess::default())
    }

    fn app_with_access(members: Vec<i64>, access: FakeUserAccess) -> Router {
        let lineups = Arc::new(FakeLineups {
            members,
            ..Default::default()
        });
        let service = Arc::new(MyLineupService::new(
            Arc::new(FakeSquads {
                players: vec![],
                missing_table: false,
            }),
            lineups,
            Arc::new(access),
        ));
        my_routes(Arc::new(MyLineupWebState {
            service,
            token_port: Arc::new(FakeTokenPort),
        }))
    }

    async fn send(
        app: &Router,
        method: &str,
        uri: &str,
        token: Option<&str>,
        body: Option<String>,
    ) -> (u16, String) {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        let res = app
            .clone()
            .oneshot(
                builder
                    .body(body.map(Body::from).unwrap_or_else(Body::empty))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status().as_u16();
        let text = String::from_utf8(
            to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        (status, text)
    }

    fn body_with(formation: &str, first_player: Option<i64>) -> String {
        let slots: Vec<String> = (0..11)
            .map(|i| {
                let player = if i == 0 {
                    first_player.map_or("null".to_string(), |p| p.to_string())
                } else {
                    "null".to_string()
                };
                format!(r#"{{"slot":{i},"player_id":{player},"x":0.5,"y":0.5}}"#)
            })
            .collect();
        format!(
            r#"{{"formation":"{formation}","slots":[{}]}}"#,
            slots.join(",")
        )
    }

    const URI: &str = "/api/v1/lineups/me/77680";

    #[tokio::test]
    async fn valid_jwt_does_not_authorize_a_disabled_user() {
        let app = app_with_access(
            vec![],
            FakeUserAccess {
                active: false,
                unavailable: false,
            },
        );
        assert_eq!(send(&app, "GET", URI, Some("good"), None).await.0, 401);
        assert_eq!(
            send(
                &app,
                "PUT",
                URI,
                Some("good"),
                Some(body_with("4-3-3", None))
            )
            .await
            .0,
            401
        );
    }

    #[tokio::test]
    async fn requires_login_for_get_and_put() {
        let app = app(vec![]);
        assert_eq!(send(&app, "GET", URI, None, None).await.0, 401);
        assert_eq!(
            send(&app, "PUT", URI, None, Some(body_with("4-3-3", None)))
                .await
                .0,
            401
        );
        assert_eq!(send(&app, "GET", URI, Some("bad"), None).await.0, 401);
    }

    #[tokio::test]
    async fn get_returns_null_before_saving_then_the_saved_lineup() {
        let app = app(vec![7]);
        let (status, body) = send(&app, "GET", URI, Some("good"), None).await;
        assert_eq!(status, 200);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(json["lineup"].is_null());

        let (status, body) = send(
            &app,
            "PUT",
            URI,
            Some("good"),
            Some(body_with("4-4-2", Some(7))),
        )
        .await;
        assert_eq!(status, 200, "{body}");
        let saved: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(saved["lineup"]["formation"], "4-4-2");
        assert_eq!(saved["lineup"]["share_code"].as_str().unwrap().len(), 8);

        let (_, body) = send(&app, "GET", URI, Some("good"), None).await;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["lineup"]["slots"][0]["player_id"], 7);
        assert_eq!(json["lineup"]["slots"].as_array().unwrap().len(), 11);
    }

    #[tokio::test]
    async fn put_maps_validation_errors_to_400_and_unknown_team_to_404() {
        let app = app(vec![7]);
        let put = |uri: &'static str, body: String| {
            let app = app.clone();
            async move { send(&app, "PUT", uri, Some("good"), Some(body)).await.0 }
        };
        assert_eq!(put(URI, body_with("9-9-9", None)).await, 400);
        assert_eq!(put(URI, body_with("4-3-3", Some(99))).await, 400);
        assert_eq!(
            put("/api/v1/lineups/me/5", body_with("4-3-3", None)).await,
            404
        );
        let bad_slots = r#"{"formation":"4-3-3","slots":[]}"#.to_string();
        assert_eq!(put(URI, bad_slots).await, 400);
    }

    #[tokio::test]
    async fn put_rejects_malformed_json_without_server_error() {
        let app = app(vec![]);
        for body in ["not json", r#"{"formation":"4-3-3"}"#, r#"{"slots":[]}"#] {
            let status = send(&app, "PUT", URI, Some("good"), Some(body.to_string()))
                .await
                .0;
            assert!((400..500).contains(&status), "{body} -> {status}");
        }
    }
    #[tokio::test]
    async fn editor_draft_and_publish_contracts_keep_private_edits_and_users_separate() {
        let app = app((1..=11).collect());
        let editor_uri = format!("{URI}/editor");
        let draft_uri = format!("{URI}/draft");
        let publish_uri = format!("{URI}/publish");
        let (_, initial) = send(&app, "GET", &editor_uri, Some("good"), None).await;
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&initial).unwrap(),
            serde_json::json!({"lineup":null,"draft":null})
        );
        let (status, saved) = send(
            &app,
            "PUT",
            &draft_uri,
            Some("good"),
            Some(body_with("4-4-2", Some(1))),
        )
        .await;
        assert_eq!(status, 200, "{saved}");
        let saved: serde_json::Value = serde_json::from_str(&saved).unwrap();
        assert_eq!(saved["draft"]["formation"], "4-4-2");
        assert!(saved["draft"].get("share_code").is_none());
        assert_eq!(
            send(
                &app,
                "PUT",
                &publish_uri,
                Some("good"),
                Some(body_with("4-4-2", Some(1)))
            )
            .await
            .0,
            400
        );
        let (_, other) = send(&app, "GET", &editor_uri, Some("other"), None).await;
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&other).unwrap(),
            serde_json::json!({"lineup":null,"draft":null})
        );
        let slots: Vec<_> = (0..11)
            .map(|i| serde_json::json!({"slot":i,"player_id":i+1,"x":0.5,"y":0.5}))
            .collect();
        let full = serde_json::json!({"formation":"4-3-3","slots":slots}).to_string();
        let (status, published) = send(&app, "PUT", &publish_uri, Some("good"), Some(full)).await;
        assert_eq!(status, 200, "{published}");
        let published: serde_json::Value = serde_json::from_str(&published).unwrap();
        assert_eq!(published["lineup"]["formation"], "4-3-3");
        assert_eq!(published["draft"]["formation"], "4-3-3");
        send(
            &app,
            "PUT",
            &draft_uri,
            Some("good"),
            Some(body_with("3-5-2", None)),
        )
        .await;
        let (_, editor) = send(&app, "GET", &editor_uri, Some("good"), None).await;
        let editor: serde_json::Value = serde_json::from_str(&editor).unwrap();
        assert_eq!(editor["lineup"], published["lineup"]);
        assert_eq!(editor["draft"]["formation"], "3-5-2");
    }

    #[tokio::test]
    async fn new_endpoints_require_authenticated_active_users() {
        let authenticated_app = app(vec![]);
        for (method, suffix) in [("GET", "editor"), ("PUT", "draft"), ("PUT", "publish")] {
            let uri = format!("{URI}/{suffix}");
            let body = (method == "PUT").then(|| body_with("4-3-3", None));
            assert_eq!(
                send(&authenticated_app, method, &uri, None, body.clone())
                    .await
                    .0,
                401
            );
            assert_eq!(
                send(&authenticated_app, method, &uri, Some("bad"), body)
                    .await
                    .0,
                401
            );
        }
        let app = app_with_access(
            vec![],
            FakeUserAccess {
                active: false,
                unavailable: false,
            },
        );
        for (method, suffix) in [("GET", "editor"), ("PUT", "draft"), ("PUT", "publish")] {
            assert_eq!(
                send(
                    &app,
                    method,
                    &format!("{URI}/{suffix}"),
                    Some("good"),
                    (method == "PUT").then(|| body_with("4-3-3", None))
                )
                .await
                .0,
                401
            );
        }
    }
}
