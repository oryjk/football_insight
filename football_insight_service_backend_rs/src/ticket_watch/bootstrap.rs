use std::sync::Arc;

use axum::Router;

use crate::{
    auth::ports::token_port::TokenPort,
    config::AppConfig,
    ticket_watch::{
        adapters::{
            integration::{
                http_ticket_monitor_port::HttpTicketMonitorPort,
                noop_tracked_interest_cache_port::NoopTrackedInterestCachePort,
                redis_tracked_interest_cache_port::RedisTrackedInterestCachePort,
            },
            persistence::PostgresRestockProfileStore,
            web::{handlers::TicketWatchWebState, routes::ticket_watch_routes},
        },
        application::{
            current_board_cache::CurrentTicketWatchBoardCache,
            get_best_watch_windows::{BEST_WINDOWS_RETRY_SECONDS, GetBestWatchWindowsUseCase},
            get_current_ticket_watch_board::GetCurrentTicketWatchBoardUseCase,
            get_current_ticket_watch_match::GetCurrentTicketWatchMatchUseCase,
            get_match_block_interests::GetMatchBlockInterestsUseCase,
            get_match_ticket_inventory::GetMatchTicketInventoryUseCase,
            get_match_tracked_interests::GetMatchTrackedInterestsUseCase,
            get_yukun_current_ticket_watch_match::GetYukunCurrentTicketWatchMatchUseCase,
            get_yukun_ticket_inventory::GetYukunTicketInventoryUseCase,
            list_ticket_watch_matches::ListTicketWatchMatchesUseCase,
            list_ticket_watch_regions::ListTicketWatchRegionsUseCase,
            list_yukun_match_ticket_regions::ListYukunMatchTicketRegionsUseCase,
            list_yukun_ticket_watch_matches::ListYukunTicketWatchMatchesUseCase,
            toggle_match_block_interest::ToggleMatchBlockInterestUseCase,
        },
        ports::{
            restock_profile_store::RestockProfileStore, ticket_monitor_port::TicketMonitorPort,
            tracked_interest_cache_port::TrackedInterestCachePort,
        },
    },
};

pub struct TicketWatchBootstrap {
    pub routes: Router,
    pub ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

pub fn build_ticket_watch(
    pool: sqlx::PgPool,
    config: &AppConfig,
    token_port: Arc<dyn TokenPort>,
    membership_port: Arc<dyn crate::auth::ports::user_membership_port::UserMembershipPort>,
) -> TicketWatchBootstrap {
    let ticket_monitor_port: Arc<dyn TicketMonitorPort> = Arc::new(
        HttpTicketMonitorPort::new(config.ticket_monitor_base_url.clone())
            .with_restock_base_url(config.ticket_restock_profile_base_url.clone()),
    );
    let tracked_interest_cache_port: Arc<dyn TrackedInterestCachePort> =
        match RedisTrackedInterestCachePort::new(&config.redis_url, 60) {
            Ok(port) => Arc::new(port),
            Err(error) => {
                tracing::warn!(error = %error, "failed to initialize tracked interest redis cache, fallback to noop");
                Arc::new(NoopTrackedInterestCachePort)
            }
        };
    let state = build_ticket_watch_state_with_store(
        ticket_monitor_port.clone(),
        Some(Arc::new(PostgresRestockProfileStore::new(pool))),
        tracked_interest_cache_port,
        token_port,
        membership_port,
    );
    let background = state.get_best_watch_windows_use_case.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            BEST_WINDOWS_RETRY_SECONDS as u64,
        ));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            background.refresh_at(chrono::Utc::now()).await;
        }
    });

    TicketWatchBootstrap {
        routes: ticket_watch_routes(state),
        ticket_monitor_port,
    }
}

pub fn build_ticket_watch_state(
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
    tracked_interest_cache_port: Arc<dyn TrackedInterestCachePort>,
    token_port: Arc<dyn TokenPort>,
    membership_port: Arc<dyn crate::auth::ports::user_membership_port::UserMembershipPort>,
) -> Arc<TicketWatchWebState> {
    build_ticket_watch_state_with_store(
        ticket_monitor_port,
        None,
        tracked_interest_cache_port,
        token_port,
        membership_port,
    )
}

pub fn build_ticket_watch_state_with_store(
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
    restock_profile_store: Option<Arc<dyn RestockProfileStore>>,
    tracked_interest_cache_port: Arc<dyn TrackedInterestCachePort>,
    token_port: Arc<dyn TokenPort>,
    membership_port: Arc<dyn crate::auth::ports::user_membership_port::UserMembershipPort>,
) -> Arc<TicketWatchWebState> {
    let mut best_windows =
        GetBestWatchWindowsUseCase::new(ticket_monitor_port.clone(), membership_port);
    if let Some(store) = restock_profile_store {
        best_windows = best_windows.with_store(store);
    }
    Arc::new(TicketWatchWebState {
        get_best_watch_windows_use_case: Arc::new(best_windows),
        get_current_ticket_watch_board_use_case: Arc::new(GetCurrentTicketWatchBoardUseCase::new(
            Arc::new(CurrentTicketWatchBoardCache::new(
                std::time::Duration::from_secs(2),
            )),
            Arc::new(GetCurrentTicketWatchMatchUseCase::new(
                ticket_monitor_port.clone(),
            )),
            Arc::new(GetMatchTicketInventoryUseCase::new(
                ticket_monitor_port.clone(),
            )),
            Arc::new(GetMatchBlockInterestsUseCase::new(
                ticket_monitor_port.clone(),
            )),
            Arc::new(GetMatchTrackedInterestsUseCase::new(
                ticket_monitor_port.clone(),
                tracked_interest_cache_port.clone(),
            )),
        )),
        get_current_ticket_watch_match_use_case: Arc::new(GetCurrentTicketWatchMatchUseCase::new(
            ticket_monitor_port.clone(),
        )),
        list_ticket_watch_matches_use_case: Arc::new(ListTicketWatchMatchesUseCase::new(
            ticket_monitor_port.clone(),
        )),
        list_ticket_watch_regions_use_case: Arc::new(ListTicketWatchRegionsUseCase::new(
            ticket_monitor_port.clone(),
        )),
        get_match_ticket_inventory_use_case: Arc::new(GetMatchTicketInventoryUseCase::new(
            ticket_monitor_port.clone(),
        )),
        get_match_block_interests_use_case: Arc::new(GetMatchBlockInterestsUseCase::new(
            ticket_monitor_port.clone(),
        )),
        get_match_tracked_interests_use_case: Arc::new(GetMatchTrackedInterestsUseCase::new(
            ticket_monitor_port.clone(),
            tracked_interest_cache_port.clone(),
        )),
        toggle_match_block_interest_use_case: Arc::new(ToggleMatchBlockInterestUseCase::new(
            ticket_monitor_port.clone(),
            tracked_interest_cache_port,
        )),
        get_yukun_ticket_inventory_use_case: Arc::new(GetYukunTicketInventoryUseCase::new(
            ticket_monitor_port.clone(),
        )),
        list_yukun_match_ticket_regions_use_case: Arc::new(
            ListYukunMatchTicketRegionsUseCase::new(ticket_monitor_port.clone()),
        ),
        list_yukun_ticket_watch_matches_use_case: Arc::new(
            ListYukunTicketWatchMatchesUseCase::new(ticket_monitor_port.clone()),
        ),
        get_yukun_current_ticket_watch_match_use_case: Arc::new(
            GetYukunCurrentTicketWatchMatchUseCase::new(ticket_monitor_port),
        ),
        token_port,
    })
}
