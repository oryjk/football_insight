use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::{Body, Bytes, to_bytes},
    extract::State,
    http::{
        HeaderMap, HeaderValue, Method, Request, Response, StatusCode,
        header::{AUTHORIZATION, CONTENT_LENGTH},
    },
    middleware::Next,
};
use tokio::sync::RwLock;

const MAX_CACHEABLE_BODY_SIZE: usize = 1024 * 1024 * 2;
const X_CACHE_HEADER: &str = "x-cache";
const CACHE_HIT: &str = "HIT";
const CACHE_MISS: &str = "MISS";

#[derive(Clone)]
pub struct HttpResponseCache {
    default_ttl: Duration,
    path_ttls: Vec<(String, Duration)>,
    entries: Arc<RwLock<HashMap<String, CachedResponse>>>,
}

#[derive(Clone)]
struct CachedResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
    expires_at: Instant,
}

impl HttpResponseCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            default_ttl: ttl,
            path_ttls: Vec::new(),
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn new_with_path_ttls(default_ttl: Duration, path_ttls: Vec<(String, Duration)>) -> Self {
        Self {
            default_ttl,
            path_ttls,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn resolve_ttl(&self, path: &str) -> Duration {
        for (prefix, ttl) in &self.path_ttls {
            if path.starts_with(prefix) {
                return *ttl;
            }
        }
        self.default_ttl
    }

    async fn get(&self, key: &str) -> Option<Response<Body>> {
        let cached = {
            let entries = self.entries.read().await;
            entries.get(key)?.clone()
        };
        if cached.expires_at <= Instant::now() {
            let mut entries = self.entries.write().await;
            entries.remove(key);
            return None;
        }

        Some(build_response(cached.status, &cached.headers, cached.body))
    }

    async fn put(&self, key: String, response: CachedResponse) {
        self.entries.write().await.insert(key, response);
    }
}

pub async fn cache_get_responses(
    State(cache): State<HttpResponseCache>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    if !should_cache_request(&request) {
        return next.run(request).await;
    }

    let path = request.uri().path().to_string();
    let cache_key = build_cache_key(&request);
    if let Some(mut response) = cache.get(&cache_key).await {
        tracing::info!(path = %path, "缓存命中");
        response
            .headers_mut()
            .insert(X_CACHE_HEADER, HeaderValue::from_static(CACHE_HIT));
        return response;
    }

    let response = next.run(request).await;
    let content_length = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok());
    if content_length.is_some_and(|length| length > MAX_CACHEABLE_BODY_SIZE) {
        tracing::info!(path = %path, size = content_length, "响应体超过缓存上限，跳过缓存");
        return response;
    }

    let (parts, body) = response.into_parts();
    let body_bytes = match to_bytes(body, MAX_CACHEABLE_BODY_SIZE).await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::error!(path = %path, error = %error, "缓存层读取响应体失败");
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("缓存层处理失败"))
                .unwrap();
        }
    };

    let mut response = Response::from_parts(parts, Body::from(body_bytes.clone()));

    if response.status().is_success() {
        let mut headers = response.headers().clone();
        headers.remove(CONTENT_LENGTH);
        headers.remove(X_CACHE_HEADER);
        let ttl = cache.resolve_ttl(&path);
        cache
            .put(
                cache_key,
                CachedResponse {
                    status: response.status(),
                    headers,
                    body: body_bytes,
                    expires_at: Instant::now() + ttl,
                },
            )
            .await;
        response
            .headers_mut()
            .insert(X_CACHE_HEADER, HeaderValue::from_static(CACHE_MISS));
    }

    response
}

fn should_cache_request(request: &Request<Body>) -> bool {
    if request.method() != Method::GET {
        return false;
    }

    if request.headers().contains_key(AUTHORIZATION) {
        return false;
    }

    let path = request.uri().path();
    !matches_excluded_prefix(path)
}

fn matches_excluded_prefix(path: &str) -> bool {
    if path == "/api/v1/system/public-config" {
        return false;
    }

    path.starts_with("/api/v1/auth/")
        || path.starts_with("/api/v1/ticket-watch/")
        || path.starts_with("/api/v1/match-id/")
        || path.starts_with("/api/v1/system")
        || path.starts_with("/api/v1/mini-review/")
        || path == "/football/wechat/webhook"
        || path == "/api/health"
}

fn build_cache_key(request: &Request<Body>) -> String {
    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|item| item.as_str())
        .unwrap_or(request.uri().path());
    let auth = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    format!("GET:{path_and_query}:{auth}")
}

fn build_response(status: StatusCode, headers: &HeaderMap, body: Bytes) -> Response<Body> {
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    response.headers_mut().extend(
        headers
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<Vec<_>>(),
    );
    response
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Duration;

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        middleware::from_fn_with_state,
        routing::get,
    };
    use tower::ServiceExt;

    use super::{HttpResponseCache, MAX_CACHEABLE_BODY_SIZE, cache_get_responses};

    #[tokio::test]
    async fn caches_safe_get_requests_for_ttl_window() {
        let hits = Arc::new(AtomicUsize::new(0));
        let cache = HttpResponseCache::new(Duration::from_secs(600));
        let app = {
            let hits = hits.clone();
            Router::new()
                .route(
                    "/api/v1/live/overview",
                    get(move || {
                        let hits = hits.clone();
                        async move {
                            let value = hits.fetch_add(1, Ordering::SeqCst) + 1;
                            format!("hit-{value}")
                        }
                    }),
                )
                .layer(from_fn_with_state(cache, cache_get_responses))
        };

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/live/overview")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let second = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/live/overview")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let first_status = first.status();
        let second_status = second.status();
        let first_body = to_bytes(first.into_body(), MAX_CACHEABLE_BODY_SIZE)
            .await
            .unwrap();
        let second_body = to_bytes(second.into_body(), MAX_CACHEABLE_BODY_SIZE)
            .await
            .unwrap();

        assert_eq!(first_status, StatusCode::OK);
        assert_eq!(second_status, StatusCode::OK);
        assert_eq!(first_body, second_body);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn marks_cache_miss_then_hit_for_cacheable_requests() {
        let hits = Arc::new(AtomicUsize::new(0));
        let cache = HttpResponseCache::new(Duration::from_secs(600));
        let app = {
            let hits = hits.clone();
            Router::new()
                .route(
                    "/api/v1/live/rankings",
                    get(move || {
                        let hits = hits.clone();
                        async move {
                            let value = hits.fetch_add(1, Ordering::SeqCst) + 1;
                            format!("rankings-hit-{value}")
                        }
                    }),
                )
                .layer(from_fn_with_state(cache, cache_get_responses))
        };

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/live/rankings")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let first_cache = first.headers().get("x-cache").cloned();
        let first_body = to_bytes(first.into_body(), MAX_CACHEABLE_BODY_SIZE)
            .await
            .unwrap();

        let second = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/live/rankings")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let second_cache = second.headers().get("x-cache").cloned();
        let second_body = to_bytes(second.into_body(), MAX_CACHEABLE_BODY_SIZE)
            .await
            .unwrap();

        assert_eq!(first_cache.unwrap(), "MISS");
        assert_eq!(second_cache.unwrap(), "HIT");
        assert_eq!(first_body, second_body);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn does_not_cache_excluded_auth_routes() {
        let hits = Arc::new(AtomicUsize::new(0));
        let cache = HttpResponseCache::new(Duration::from_secs(600));
        let app = {
            let hits = hits.clone();
            Router::new()
                .route(
                    "/api/v1/auth/me",
                    get(move || {
                        let hits = hits.clone();
                        async move {
                            let value = hits.fetch_add(1, Ordering::SeqCst) + 1;
                            format!("auth-hit-{value}")
                        }
                    }),
                )
                .layer(from_fn_with_state(cache, cache_get_responses))
        };

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/auth/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let _ = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/auth/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn does_not_cache_mini_review_review_status() {
        // 审核状态可被 PUT 随时切换，读取必须实时，否则切换后小程序仍拿到缓存旧值。
        let hits = Arc::new(AtomicUsize::new(0));
        let cache = HttpResponseCache::new(Duration::from_secs(600));
        let app = {
            let hits = hits.clone();
            Router::new()
                .route(
                    "/api/v1/mini-review/review-status",
                    get(move || {
                        let hits = hits.clone();
                        async move {
                            let value = hits.fetch_add(1, Ordering::SeqCst) + 1;
                            format!("review-hit-{value}")
                        }
                    }),
                )
                .layer(from_fn_with_state(cache, cache_get_responses))
        };

        for _ in 0..2 {
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/mini-review/review-status?project_code=football_insight_mini&version=1.0.56")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
        }

        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn does_not_cache_match_id_entitlement() {
        // 解锁状态在支付回调后会变化，且按用户区分，绝不能进缓存。
        let hits = Arc::new(AtomicUsize::new(0));
        let cache = HttpResponseCache::new(Duration::from_secs(600));
        let app = {
            let hits = hits.clone();
            Router::new()
                .route(
                    "/api/v1/match-id/entitlement",
                    get(move || {
                        let hits = hits.clone();
                        async move {
                            let value = hits.fetch_add(1, Ordering::SeqCst) + 1;
                            format!("entitlement-hit-{value}")
                        }
                    }),
                )
                .layer(from_fn_with_state(cache, cache_get_responses))
        };

        for _ in 0..2 {
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/match-id/entitlement?match_id=571")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
        }

        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn caches_public_system_config_but_not_other_system_routes() {
        let public_hits = Arc::new(AtomicUsize::new(0));
        let review_hits = Arc::new(AtomicUsize::new(0));
        let cache = HttpResponseCache::new(Duration::from_secs(600));
        let app = {
            let public_hits = public_hits.clone();
            let review_hits = review_hits.clone();
            Router::new()
                .route(
                    "/api/v1/system/public-config",
                    get(move || {
                        let public_hits = public_hits.clone();
                        async move {
                            let value = public_hits.fetch_add(1, Ordering::SeqCst) + 1;
                            format!("public-config-hit-{value}")
                        }
                    }),
                )
                .route(
                    "/api/v1/system/mini-program-review",
                    get(move || {
                        let review_hits = review_hits.clone();
                        async move {
                            let value = review_hits.fetch_add(1, Ordering::SeqCst) + 1;
                            format!("review-hit-{value}")
                        }
                    }),
                )
                .layer(from_fn_with_state(cache, cache_get_responses))
        };

        let public_request = || {
            Request::builder()
                .uri("/api/v1/system/public-config")
                .body(Body::empty())
                .unwrap()
        };
        let review_request = || {
            Request::builder()
                .uri("/api/v1/system/mini-program-review?version=1.0.0")
                .body(Body::empty())
                .unwrap()
        };

        let public_first = app.clone().oneshot(public_request()).await.unwrap();
        let public_second = app.clone().oneshot(public_request()).await.unwrap();
        let review_first = app.clone().oneshot(review_request()).await.unwrap();
        let review_second = app.oneshot(review_request()).await.unwrap();

        assert_eq!(public_first.headers().get("x-cache").unwrap(), "MISS");
        assert_eq!(public_second.headers().get("x-cache").unwrap(), "HIT");
        assert_eq!(public_hits.load(Ordering::SeqCst), 1);

        let review_first_body = to_bytes(review_first.into_body(), MAX_CACHEABLE_BODY_SIZE)
            .await
            .unwrap();
        let review_second_body = to_bytes(review_second.into_body(), MAX_CACHEABLE_BODY_SIZE)
            .await
            .unwrap();
        assert_ne!(review_first_body, review_second_body);
        assert_eq!(review_hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn does_not_cache_authorized_get_requests() {
        let hits = Arc::new(AtomicUsize::new(0));
        let cache = HttpResponseCache::new(Duration::from_secs(600));
        let app = {
            let hits = hits.clone();
            Router::new()
                .route(
                    "/api/v1/team-boards/77680",
                    get(move || {
                        let hits = hits.clone();
                        async move {
                            let value = hits.fetch_add(1, Ordering::SeqCst) + 1;
                            format!("board-hit-{value}")
                        }
                    }),
                )
                .layer(from_fn_with_state(cache, cache_get_responses))
        };

        let request = || {
            Request::builder()
                .uri("/api/v1/team-boards/77680")
                .header("Authorization", "Bearer token")
                .body(Body::empty())
                .unwrap()
        };

        let _ = app.clone().oneshot(request()).await.unwrap();
        let _ = app.oneshot(request()).await.unwrap();

        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn does_not_cache_ticket_watch_routes() {
        let hits = Arc::new(AtomicUsize::new(0));
        let cache = HttpResponseCache::new(Duration::from_secs(600));
        let app = {
            let hits = hits.clone();
            Router::new()
                .route(
                    "/api/v1/ticket-watch/matches/574/inventory",
                    get(move || {
                        let hits = hits.clone();
                        async move {
                            let value = hits.fetch_add(1, Ordering::SeqCst) + 1;
                            format!("inventory-hit-{value}")
                        }
                    }),
                )
                .layer(from_fn_with_state(cache, cache_get_responses))
        };

        let request = || {
            Request::builder()
                .uri("/api/v1/ticket-watch/matches/574/inventory?since=2026-04-23T14%3A10%3A00%2B08%3A00")
                .body(Body::empty())
                .unwrap()
        };

        let first = app.clone().oneshot(request()).await.unwrap();
        let second = app.oneshot(request()).await.unwrap();

        let first_body = to_bytes(first.into_body(), MAX_CACHEABLE_BODY_SIZE)
            .await
            .unwrap();
        let second_body = to_bytes(second.into_body(), MAX_CACHEABLE_BODY_SIZE)
            .await
            .unwrap();

        assert_ne!(first_body, second_body);
        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn applies_shorter_ttl_for_live_and_rounds_paths() {
        let hits = Arc::new(AtomicUsize::new(0));
        let cache = HttpResponseCache::new_with_path_ttls(
            Duration::from_secs(600),
            vec![
                ("/api/v1/live/".to_string(), Duration::from_millis(50)),
                ("/api/v1/rounds/".to_string(), Duration::from_millis(50)),
            ],
        );
        let app = {
            let hits = hits.clone();
            Router::new()
                .route(
                    "/api/v1/live/overview",
                    get(move || {
                        let hits = hits.clone();
                        async move {
                            let value = hits.fetch_add(1, Ordering::SeqCst) + 1;
                            format!("live-hit-{value}")
                        }
                    }),
                )
                .layer(from_fn_with_state(cache, cache_get_responses))
        };

        let request = || {
            Request::builder()
                .uri("/api/v1/live/overview")
                .body(Body::empty())
                .unwrap()
        };

        let first = app.clone().oneshot(request()).await.unwrap();
        let first_body = to_bytes(first.into_body(), MAX_CACHEABLE_BODY_SIZE)
            .await
            .unwrap();
        assert_eq!(first_body, "live-hit-1");

        // 等待短 TTL 过期
        tokio::time::sleep(Duration::from_millis(100)).await;

        let second = app.oneshot(request()).await.unwrap();
        let second_body = to_bytes(second.into_body(), MAX_CACHEABLE_BODY_SIZE)
            .await
            .unwrap();
        assert_eq!(second_body, "live-hit-2");
        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }
}
