use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::{Body, Bytes, to_bytes},
    extract::State,
    http::{
        HeaderMap, Method, Request, Response, StatusCode,
        header::{AUTHORIZATION, CONTENT_LENGTH},
    },
    middleware::Next,
};
use tokio::sync::RwLock;

const MAX_CACHEABLE_BODY_SIZE: usize = 1024 * 1024 * 2;

#[derive(Clone)]
pub struct HttpResponseCache {
    ttl: Duration,
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
            ttl,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get(&self, key: &str) -> Option<Response<Body>> {
        let mut entries = self.entries.write().await;
        let cached = entries.get(key)?.clone();
        if cached.expires_at <= Instant::now() {
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

    let cache_key = build_cache_key(&request);
    if let Some(response) = cache.get(&cache_key).await {
        tracing::info!(path = %request.uri().path(), "served response from in-memory cache");
        return response;
    }

    let response = next.run(request).await;
    if response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > MAX_CACHEABLE_BODY_SIZE)
    {
        tracing::info!("skipped caching oversized response");
        return response;
    }

    let (parts, body) = response.into_parts();
    let body_bytes = match to_bytes(body, MAX_CACHEABLE_BODY_SIZE).await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::warn!(error = %error, "failed to read response body for cache");
            return Response::from_parts(parts, Body::empty());
        }
    };

    let response = Response::from_parts(parts, Body::from(body_bytes.clone()));

    if response.status().is_success() {
        let mut headers = response.headers().clone();
        headers.remove(CONTENT_LENGTH);
        cache
            .put(
                cache_key,
                CachedResponse {
                    status: response.status(),
                    headers,
                    body: body_bytes,
                    expires_at: Instant::now() + cache.ttl,
                },
            )
            .await;
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
    path.starts_with("/api/v1/auth/")
        || path.starts_with("/api/v1/ticket-watch/")
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
}
