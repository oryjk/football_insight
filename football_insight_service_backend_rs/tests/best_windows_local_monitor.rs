//! Explicit local-only integration: real monitor file router/HTTP adapter, fixture identities, no DB.
use async_trait::async_trait;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{DateTime, Utc};
use football_insight_service_backend_rs::{
    auth::ports::{token_port::*, user_membership_port::UserMembershipPort},
    ticket_watch::{
        adapters::{
            integration::{
                http_ticket_monitor_port::HttpTicketMonitorPort,
                noop_tracked_interest_cache_port::NoopTrackedInterestCachePort,
            },
            web::routes::ticket_watch_routes,
        },
        bootstrap::build_ticket_watch_state,
        domain::{best_watch_windows::RestockProfile, ticket_watch::*},
        ports::ticket_monitor_port::TicketMonitorPort,
    },
};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;
struct LocalMonitor {
    upstream: HttpTicketMonitorPort,
}
#[async_trait]
impl TicketMonitorPort for LocalMonitor {
    async fn fetch_all_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
        Ok([
            (
                583,
                "2026-10-10T19:35:00+08:00",
                "2026-10-08T14:00:00+08:00",
            ),
            (
                582,
                "2026-08-29T19:35:00+08:00",
                "2026-08-27T14:00:00+08:00",
            ),
            (
                581,
                "2026-08-22T19:35:00+08:00",
                "2026-08-20T14:00:00+08:00",
            ),
        ]
        .into_iter()
        .map(|(id, kickoff, sale)| TicketWatchMatchSummary {
            match_id: id,
            external_match_id: id.to_string(),
            round_number: 1,
            sale_start_at: Some(sale.into()),
            kickoff_at: kickoff.into(),
            match_date: "".into(),
            match_time: "".into(),
            home_team_name: "成都蓉城".into(),
            away_team_name: "测试客队".into(),
            is_current: false,
            include_in_reflux_stats: true,
            ticket_sale_ended: true,
        })
        .collect())
    }
    async fn fetch_restock_profile(
        &self,
        id: i64,
        fallback: Option<i64>,
        since: &str,
    ) -> anyhow::Result<RestockProfile> {
        self.upstream
            .fetch_restock_profile(id, fallback, since)
            .await
    }
    async fn fetch_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
        unreachable!()
    }
    async fn fetch_regions(&self) -> anyhow::Result<Vec<TicketWatchRegion>> {
        unreachable!()
    }
    async fn fetch_inventory(
        &self,
        _: i64,
        _: Option<i64>,
        _: Option<&str>,
    ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
        unreachable!()
    }
    async fn fetch_block_interests(
        &self,
        _: i64,
        _: Option<Uuid>,
    ) -> anyhow::Result<Vec<TicketWatchBlockInterest>> {
        unreachable!()
    }
    async fn fetch_tracked_interests(
        &self,
        _: i64,
        _: Uuid,
    ) -> anyhow::Result<Vec<TicketWatchTrackedInterest>> {
        unreachable!()
    }
    async fn toggle_block_interest(
        &self,
        _: i64,
        _: Uuid,
        _: &str,
    ) -> anyhow::Result<TicketWatchBlockInterest> {
        unreachable!()
    }
    async fn fetch_yukun_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
        unreachable!()
    }
    async fn fetch_yukun_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
        unreachable!()
    }
    async fn fetch_yukun_reflux(
        &self,
        _: i64,
        _: Option<&str>,
    ) -> anyhow::Result<(Vec<TicketWatchInventoryEntry>, Vec<TicketWatchRegion>)> {
        unreachable!()
    }
}
struct FixtureIdentity;
#[async_trait]
impl UserMembershipPort for FixtureIdentity {
    async fn get_user_open_id(&self, _: Uuid) -> anyhow::Result<Option<String>> {
        unreachable!()
    }
    async fn get_user_membership_tier(&self, id: Uuid) -> anyhow::Result<Option<String>> {
        Ok(Some(if id.is_nil() { "V6" } else { "V3" }.into()))
    }
    async fn update_user_membership_tier(&self, _: Uuid, _: &str) -> anyhow::Result<()> {
        unreachable!()
    }
    async fn is_seat_swap_notice_enabled(&self, _: Uuid) -> anyhow::Result<Option<bool>> {
        unreachable!()
    }
}
impl TokenPort for FixtureIdentity {
    fn issue_token(&self, _: Uuid, _: &str, _: DateTime<Utc>) -> anyhow::Result<String> {
        unreachable!()
    }
    fn verify_token(&self, token: &str) -> anyhow::Result<AuthTokenClaims> {
        let sub = match token {
            "fixture-full" => Uuid::nil(),
            "fixture-expired-tier" => Uuid::from_u128(1),
            _ => anyhow::bail!("invalid fixture token"),
        };
        Ok(AuthTokenClaims {
            sub,
            account_identifier: "local-fixture".into(),
            exp: Utc::now() + chrono::Duration::hours(1),
        })
    }
    fn issue_wechat_bind_token(
        &self,
        _: WechatBindTokenPayload,
        _: DateTime<Utc>,
    ) -> anyhow::Result<String> {
        unreachable!()
    }
    fn verify_wechat_bind_token(&self, _: &str) -> anyhow::Result<WechatBindTokenClaims> {
        unreachable!()
    }
}
#[tokio::test]
#[ignore = "requires explicitly started local snapshot-only monitor on 127.0.0.1:4001 with readonly-produced 581/582/583 snapshots"]
async fn best_windows_real_local_snapshot_http_full_then_guest_and_expired_preview() {
    let url = std::env::var("BEST_WINDOWS_LOCAL_MONITOR").expect("explicit local monitor URL");
    assert_eq!(
        url, "http://127.0.0.1:4001",
        "never point this fixture test at production"
    );
    let monitor = Arc::new(LocalMonitor {
        upstream: HttpTicketMonitorPort::new(Some(url)),
    });
    let state = build_ticket_watch_state(
        monitor,
        Arc::new(NoopTrackedInterestCachePort),
        Arc::new(FixtureIdentity),
        Arc::new(FixtureIdentity),
    );
    state
        .get_best_watch_windows_use_case
        .refresh_at(Utc::now())
        .await;
    let app = ticket_watch_routes(state);
    for (token, access, hours, top) in [
        (Some("fixture-full"), "full", 24, 3),
        (None, "preview", 0, 1),
        (Some("fixture-expired-tier"), "preview", 0, 1),
        (Some("invalid"), "preview", 0, 1),
    ] {
        let mut builder = Request::builder().uri("/api/v1/ticket-watch/best-windows");
        if let Some(token) = token {
            builder = builder.header("Authorization", format!("Bearer {token}"));
        }
        let response = app
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["status"], "ok");
        assert_eq!(value["access"], access);
        assert_eq!(value["sample_size"], 3);
        assert_eq!(value["hours"].as_array().unwrap().len(), hours);
        assert_eq!(value["top_windows"].as_array().unwrap().len(), top);
        println!("HTTP200 status=ok access={access} sample_size=3 hours={hours} top_windows={top}");
    }
}
