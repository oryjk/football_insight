use std::sync::Arc;

use axum::{Router, routing::get};

use crate::ticket_watch::adapters::web::handlers::{
    TicketWatchWebState, get_current_ticket_watch_board_handler,
    get_current_ticket_watch_match_handler, get_match_block_interests_handler,
    get_match_ticket_inventory_handler, get_match_tracked_interests_handler,
    list_ticket_watch_matches_handler, list_ticket_watch_regions_handler,
    toggle_match_block_interest_handler,
};

pub fn ticket_watch_routes(state: Arc<TicketWatchWebState>) -> Router {
    Router::new()
        .route(
            "/api/v1/ticket-watch/current-match",
            get(get_current_ticket_watch_match_handler),
        )
        .route(
            "/api/v1/ticket-watch/current-board",
            get(get_current_ticket_watch_board_handler),
        )
        .route(
            "/api/v1/ticket-watch/matches",
            get(list_ticket_watch_matches_handler),
        )
        .route(
            "/api/v1/ticket-watch/regions",
            get(list_ticket_watch_regions_handler),
        )
        .route(
            "/api/v1/ticket-watch/matches/{match_id}/inventory",
            get(get_match_ticket_inventory_handler),
        )
        .route(
            "/api/v1/ticket-watch/matches/{match_id}/interests",
            get(get_match_block_interests_handler),
        )
        .route(
            "/api/v1/ticket-watch/matches/{match_id}/tracked-interests",
            get(get_match_tracked_interests_handler),
        )
        .route(
            "/api/v1/ticket-watch/matches/{match_id}/interests/toggle",
            axum::routing::post(toggle_match_block_interest_handler),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use chrono::{DateTime, Utc};
    use tower::util::ServiceExt;
    use uuid::Uuid;

    use super::ticket_watch_routes;
    use crate::{
        auth::ports::token_port::TokenPort,
        ticket_watch::{
            adapters::{
                integration::noop_tracked_interest_cache_port::NoopTrackedInterestCachePort,
                web::handlers::TicketWatchWebState,
            },
            application::{
                current_board_cache::CurrentTicketWatchBoardCache,
                get_current_ticket_watch_board::GetCurrentTicketWatchBoardUseCase,
                get_current_ticket_watch_match::GetCurrentTicketWatchMatchUseCase,
                get_match_block_interests::GetMatchBlockInterestsUseCase,
                get_match_ticket_inventory::GetMatchTicketInventoryUseCase,
                get_match_tracked_interests::GetMatchTrackedInterestsUseCase,
                list_ticket_watch_matches::ListTicketWatchMatchesUseCase,
                list_ticket_watch_regions::ListTicketWatchRegionsUseCase,
                toggle_match_block_interest::ToggleMatchBlockInterestUseCase,
            },
            domain::ticket_watch::{
                TicketWatchBlockInterest, TicketWatchCurrentMatchView, TicketWatchInventoryEntry,
                TicketWatchMatchSummary, TicketWatchRegion, TicketWatchTrackedInterest,
            },
            ports::ticket_monitor_port::TicketMonitorPort,
        },
    };

    struct StubTicketMonitorPort;

    #[async_trait]
    impl TicketMonitorPort for StubTicketMonitorPort {
        async fn fetch_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
            Ok(TicketWatchCurrentMatchView {
                current_match: None,
                group_ticket_active: false,
                message: String::new(),
            })
        }

        async fn fetch_all_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
            Ok(vec![])
        }

        async fn fetch_regions(&self) -> anyhow::Result<Vec<TicketWatchRegion>> {
            Ok(vec![])
        }

        async fn fetch_inventory(
            &self,
            _match_id: i64,
            _fallback_match_id: Option<i64>,
            _since: Option<&str>,
        ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
            Ok(vec![])
        }

        async fn fetch_block_interests(
            &self,
            _match_id: i64,
            _viewer_user_id: Option<Uuid>,
        ) -> anyhow::Result<Vec<TicketWatchBlockInterest>> {
            Ok(vec![])
        }

        async fn fetch_tracked_interests(
            &self,
            _match_id: i64,
            _user_id: Uuid,
        ) -> anyhow::Result<Vec<TicketWatchTrackedInterest>> {
            Ok(vec![])
        }

        async fn toggle_block_interest(
            &self,
            _match_id: i64,
            _user_id: Uuid,
            _block_name: &str,
        ) -> anyhow::Result<TicketWatchBlockInterest> {
            Ok(TicketWatchBlockInterest {
                block_name: "102".to_string(),
                interested_user_count: 1,
                viewer_interested: true,
            })
        }
    }

    struct StubTokenPort;

    impl TokenPort for StubTokenPort {
        fn issue_token(
            &self,
            _user_id: Uuid,
            _account_identifier: &str,
            _expires_at: DateTime<Utc>,
        ) -> anyhow::Result<String> {
            unreachable!()
        }

        fn verify_token(
            &self,
            _token: &str,
        ) -> anyhow::Result<crate::auth::ports::token_port::AuthTokenClaims> {
            Ok(crate::auth::ports::token_port::AuthTokenClaims {
                sub: Uuid::nil(),
                account_identifier: "user".to_string(),
                exp: Utc::now(),
            })
        }

        fn issue_wechat_bind_token(
            &self,
            _payload: crate::auth::ports::token_port::WechatBindTokenPayload,
            _expires_at: DateTime<Utc>,
        ) -> anyhow::Result<String> {
            unreachable!()
        }

        fn verify_wechat_bind_token(
            &self,
            _token: &str,
        ) -> anyhow::Result<crate::auth::ports::token_port::WechatBindTokenClaims> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn tracked_interests_route_should_exist() {
        let port = Arc::new(StubTicketMonitorPort);
        let state = Arc::new(TicketWatchWebState {
            get_current_ticket_watch_board_use_case: Arc::new(GetCurrentTicketWatchBoardUseCase::new(
                Arc::new(CurrentTicketWatchBoardCache::new(std::time::Duration::from_secs(2))),
                Arc::new(GetCurrentTicketWatchMatchUseCase::new(port.clone())),
                Arc::new(GetMatchTicketInventoryUseCase::new(port.clone())),
                Arc::new(GetMatchBlockInterestsUseCase::new(port.clone())),
                Arc::new(GetMatchTrackedInterestsUseCase::new(
                    port.clone(),
                    Arc::new(NoopTrackedInterestCachePort),
                )),
            )),
            get_current_ticket_watch_match_use_case: Arc::new(
                GetCurrentTicketWatchMatchUseCase::new(port.clone()),
            ),
            list_ticket_watch_matches_use_case: Arc::new(ListTicketWatchMatchesUseCase::new(
                port.clone(),
            )),
            list_ticket_watch_regions_use_case: Arc::new(ListTicketWatchRegionsUseCase::new(
                port.clone(),
            )),
            get_match_ticket_inventory_use_case: Arc::new(GetMatchTicketInventoryUseCase::new(
                port.clone(),
            )),
            get_match_block_interests_use_case: Arc::new(GetMatchBlockInterestsUseCase::new(
                port.clone(),
            )),
            get_match_tracked_interests_use_case: Arc::new(GetMatchTrackedInterestsUseCase::new(
                port.clone(),
                Arc::new(NoopTrackedInterestCachePort),
            )),
            toggle_match_block_interest_use_case: Arc::new(ToggleMatchBlockInterestUseCase::new(
                port,
                Arc::new(NoopTrackedInterestCachePort),
            )),
            token_port: Arc::new(StubTokenPort),
        });
        let app = ticket_watch_routes(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/ticket-watch/matches/572/tracked-interests")
                    .header("authorization", "Bearer test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn current_board_route_should_exist() {
        let port = Arc::new(StubTicketMonitorPort);
        let state = Arc::new(TicketWatchWebState {
            get_current_ticket_watch_board_use_case: Arc::new(GetCurrentTicketWatchBoardUseCase::new(
                Arc::new(CurrentTicketWatchBoardCache::new(std::time::Duration::from_secs(2))),
                Arc::new(GetCurrentTicketWatchMatchUseCase::new(port.clone())),
                Arc::new(GetMatchTicketInventoryUseCase::new(port.clone())),
                Arc::new(GetMatchBlockInterestsUseCase::new(port.clone())),
                Arc::new(GetMatchTrackedInterestsUseCase::new(
                    port.clone(),
                    Arc::new(NoopTrackedInterestCachePort),
                )),
            )),
            get_current_ticket_watch_match_use_case: Arc::new(
                GetCurrentTicketWatchMatchUseCase::new(port.clone()),
            ),
            list_ticket_watch_matches_use_case: Arc::new(ListTicketWatchMatchesUseCase::new(
                port.clone(),
            )),
            list_ticket_watch_regions_use_case: Arc::new(ListTicketWatchRegionsUseCase::new(
                port.clone(),
            )),
            get_match_ticket_inventory_use_case: Arc::new(GetMatchTicketInventoryUseCase::new(
                port.clone(),
            )),
            get_match_block_interests_use_case: Arc::new(GetMatchBlockInterestsUseCase::new(
                port.clone(),
            )),
            get_match_tracked_interests_use_case: Arc::new(GetMatchTrackedInterestsUseCase::new(
                port.clone(),
                Arc::new(NoopTrackedInterestCachePort),
            )),
            toggle_match_block_interest_use_case: Arc::new(ToggleMatchBlockInterestUseCase::new(
                port,
                Arc::new(NoopTrackedInterestCachePort),
            )),
            token_port: Arc::new(StubTokenPort),
        });
        let app = ticket_watch_routes(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/ticket-watch/current-board")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
