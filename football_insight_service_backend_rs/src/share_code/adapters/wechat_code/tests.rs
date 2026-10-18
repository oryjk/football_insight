use super::*;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::post,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfixture";

#[derive(Default)]
struct Calls {
    tokens: AtomicUsize,
    codes: AtomicUsize,
    fail_token: bool,
    fail_code: bool,
}

async fn token(State(calls): State<Arc<Calls>>, Json(body): Json<Value>) -> Json<Value> {
    assert_eq!(
        body,
        json!({"grant_type":"client_credential","appid":"test-app","secret":"test-secret","force_refresh":false})
    );
    calls.tokens.fetch_add(1, Ordering::SeqCst);
    if calls.fail_token {
        Json(json!({"errcode":40013,"errmsg":"invalid appid"}))
    } else {
        Json(json!({"access_token":"test-token","expires_in":7200}))
    }
}

async fn code(
    State(calls): State<Arc<Calls>>,
    Query(query): Query<HashMap<String, String>>,
    Json(body): Json<Value>,
) -> Vec<u8> {
    assert_eq!(query["access_token"], "test-token");
    assert_eq!(
        body,
        json!({"scene":"s=share&c=ticket_poster","page":"pages/ticket-watch/index","check_path":false,"env_version":"release","width":280})
    );
    calls.codes.fetch_add(1, Ordering::SeqCst);
    if calls.fail_code {
        br#"{"errcode":45009,"errmsg":"quota exceeded"}"#.to_vec()
    } else {
        PNG.to_vec()
    }
}

async fn fixture(calls: Arc<Calls>) -> (WechatTicketWatchCode, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/cgi-bin/stable_token", post(token))
        .route("/wxa/getwxacodeunlimit", post(code))
        .with_state(calls);
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let mut adapter = WechatTicketWatchCode::new("test-app".into(), "test-secret".into());
    adapter.api_base = format!("http://{address}");
    adapter.client = reqwest::Client::builder().no_proxy().build().unwrap();
    (adapter, server)
}

#[tokio::test]
async fn code_request_uses_fixed_scene_and_reuses_success_for_twenty_four_hours() {
    let calls = Arc::new(Calls::default());
    let (adapter, server) = fixture(calls.clone()).await;
    let before = Instant::now();
    let first = adapter.image().await.unwrap();
    let second = adapter.image().await.unwrap();
    assert_eq!(first.image_base64, STANDARD.encode(PNG));
    assert_eq!(first.mime_type, "image/png");
    assert_eq!(first, second);
    let until = adapter.cache.lock().await.as_ref().unwrap().0;
    assert!(
        (before + Duration::from_secs(86400)..=Instant::now() + Duration::from_secs(86400))
            .contains(&until)
    );
    assert_eq!(calls.tokens.load(Ordering::SeqCst), 1);
    assert_eq!(calls.codes.load(Ordering::SeqCst), 1);
    // Expiring the entry must fetch a fresh image instead of retaining stale data.
    adapter.cache.lock().await.as_mut().unwrap().0 = Instant::now() - Duration::from_secs(1);
    assert_eq!(adapter.image().await.unwrap(), first);
    assert_eq!(calls.tokens.load(Ordering::SeqCst), 2);
    assert_eq!(calls.codes.load(Ordering::SeqCst), 2);
    server.abort();
}

#[tokio::test]
async fn token_and_code_failures_are_cached_for_thirty_seconds_and_retry_after_expiry() {
    for fail_token in [true, false] {
        let calls = Arc::new(Calls {
            fail_token,
            fail_code: !fail_token,
            ..Calls::default()
        });
        let (adapter, server) = fixture(calls.clone()).await;
        let before = Instant::now();
        for _ in 0..2 {
            assert_eq!(adapter.image().await, Err(ShareCodeError::Unavailable));
        }
        let until = adapter.cache.lock().await.as_ref().unwrap().0;
        assert!(
            (before + Duration::from_secs(30)..=Instant::now() + Duration::from_secs(30))
                .contains(&until)
        );
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 1);
        assert_eq!(calls.codes.load(Ordering::SeqCst), usize::from(!fail_token));
        adapter.cache.lock().await.as_mut().unwrap().0 = Instant::now() - Duration::from_secs(1);
        assert_eq!(adapter.image().await, Err(ShareCodeError::Unavailable));
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 2);
        server.abort();
    }
}

#[tokio::test]
async fn absent_or_blank_credentials_fail_without_contacting_wechat() {
    let calls = Arc::new(Calls::default());
    let (mut adapter, server) = fixture(calls.clone()).await;
    for (app_id, secret) in [
        ("", "test-secret"),
        ("test-app", ""),
        (" ", "test-secret"),
        ("test-app", " "),
    ] {
        adapter.app_id = app_id.into();
        adapter.secret = secret.into();
        *adapter.cache.lock().await = None;
        assert_eq!(adapter.image().await, Err(ShareCodeError::Unavailable));
    }
    assert_eq!(calls.tokens.load(Ordering::SeqCst), 0);
    assert_eq!(calls.codes.load(Ordering::SeqCst), 0);
    server.abort();
}

#[tokio::test]
async fn authenticated_http_endpoint_returns_only_image_contract() {
    use crate::{
        auth::{adapters::security::jwt_token_port::JwtTokenPort, ports::token_port::TokenPort},
        share_code::{
            adapters::web::{WebState, routes},
            application::ShareCodeService,
        },
    };
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use tower::ServiceExt;
    use uuid::Uuid;

    let calls = Arc::new(Calls::default());
    let (adapter, server) = fixture(calls.clone()).await;
    let token = Arc::new(JwtTokenPort::new("share-code-test-secret".into()));
    let access = token
        .issue_token(Uuid::nil(), "test", Utc::now() + chrono::Duration::hours(1))
        .unwrap();
    let app = routes(WebState {
        service: Arc::new(ShareCodeService::new(Arc::new(adapter))),
        token,
    });
    let response = app
        .oneshot(
            Request::post("/api/v1/share-codes/ticket-watch")
                .header("Authorization", format!("Bearer {access}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 8192).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        body,
        json!({"image_base64":"iVBORw0KGgpmaXh0dXJl", "mime_type":"image/png"})
    );
    assert_eq!(calls.codes.load(Ordering::SeqCst), 1);
    server.abort();
}

#[tokio::test]
async fn concurrent_callers_share_a_single_wechat_fetch() {
    let calls = Arc::new(Calls::default());
    let (adapter, server) = fixture(calls.clone()).await;
    let (first, second) = tokio::join!(adapter.image(), adapter.image());
    assert_eq!(first.unwrap(), second.unwrap());
    assert_eq!(calls.tokens.load(Ordering::SeqCst), 1);
    assert_eq!(calls.codes.load(Ordering::SeqCst), 1);
    server.abort();
}

#[test]
fn only_bounded_png_and_jpeg_images_are_returned() {
    let jpeg = [255, 216, 255, 0];
    assert_eq!(decode_image(&jpeg).unwrap().mime_type, "image/jpeg");
    assert_eq!(
        decode_image(br#"{"errcode":40001}"#),
        Err(ShareCodeError::Unavailable)
    );
    let mut oversized = PNG.to_vec();
    oversized.resize(1024 * 1024 + 1, 0);
    assert_eq!(decode_image(&oversized), Err(ShareCodeError::Unavailable));
}
