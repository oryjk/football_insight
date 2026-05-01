use std::net::SocketAddr;

use anyhow::Context;
use axum::{extract::Request, middleware::from_fn_with_state};
use football_insight_service_backend_rs::{
    app::build_router,
    config::AppConfig,
    http_cache::{HttpResponseCache, cache_get_responses},
    logging::init_tracing,
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultOnFailure, TraceLayer},
};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    tracing::info!("=== football insight service starting ===");

    let config = AppConfig::from_env()?;
    tracing::info!(port = config.port, "application config loaded");

    tracing::info!("connecting to postgres");
    tracing::info!(
        max_connections = config.database_pool.max_connections,
        min_connections = config.database_pool.min_connections,
        acquire_timeout_secs = config.database_pool.acquire_timeout_secs,
        idle_timeout_secs = config.database_pool.idle_timeout_secs,
        max_lifetime_secs = config.database_pool.max_lifetime_secs,
        "configuring postgres connection pool"
    );
    let pool = PgPoolOptions::new()
        .max_connections(config.database_pool.max_connections)
        .min_connections(config.database_pool.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            config.database_pool.acquire_timeout_secs,
        ))
        .idle_timeout(std::time::Duration::from_secs(
            config.database_pool.idle_timeout_secs,
        ))
        .max_lifetime(std::time::Duration::from_secs(
            config.database_pool.max_lifetime_secs,
        ))
        .connect(&config.database_url)
        .await
        .context("failed to connect to postgres")?;
    tracing::info!("postgres connection established");

    let app = build_router(pool, &config)
        .layer(CorsLayer::permissive())
        .layer(from_fn_with_state(
            HttpResponseCache::new_with_path_ttls(
                std::time::Duration::from_secs(600),
                vec![
                    (
                        "/api/v1/live/".to_string(),
                        std::time::Duration::from_secs(30),
                    ),
                    (
                        "/api/v1/rounds/".to_string(),
                        std::time::Duration::from_secs(30),
                    ),
                ],
            ),
            cache_get_responses,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        path = %request.uri().path()
                    )
                })
                .on_request(|request: &Request, _span: &tracing::Span| {
                    tracing::info!(
                        method = %request.method(),
                        path = %request.uri().path(),
                        "started processing request"
                    );
                })
                .on_response(
                    |response: &axum::response::Response,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = response.status().as_u16(),
                            latency_ms = latency.as_millis(),
                            "finished processing request"
                        );
                    },
                )
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(address = %addr, "football_insight_service_backend_rs listening");
    axum::serve(listener, app).await?;
    Ok(())
}
