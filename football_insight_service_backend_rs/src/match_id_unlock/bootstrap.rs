use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    auth::ports::{token_port::TokenPort, user_membership_port::UserMembershipPort},
    match_id_unlock::{
        adapters::{
            integration::ticket_monitor_match_id_source::TicketMonitorMatchIdSource,
            persistence::postgres_match_id_unlock_repository::PostgresMatchIdUnlockRepository,
            web::{handlers::MatchIdUnlockWebState, routes::match_id_unlock_routes},
        },
        application::{
            create_match_id_order::CreateMatchIdOrderUseCase,
            get_match_id_entitlement::GetMatchIdEntitlementUseCase,
        },
    },
    payment::ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
    ticket_watch::ports::ticket_monitor_port::TicketMonitorPort,
};

pub fn build_match_id_unlock_routes(
    pool: PgPool,
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
    order_repository: Arc<dyn OrderRepository>,
    user_membership_port: Arc<dyn UserMembershipPort>,
    wechat_pay_port: Arc<dyn WechatPayPort>,
    token_port: Arc<dyn TokenPort>,
) -> Router {
    let repository = Arc::new(PostgresMatchIdUnlockRepository::new(pool));
    let match_id_source = Arc::new(TicketMonitorMatchIdSource::new(ticket_monitor_port));
    let state = Arc::new(MatchIdUnlockWebState {
        get_entitlement_use_case: Arc::new(GetMatchIdEntitlementUseCase::new(
            repository.clone(),
            match_id_source.clone(),
            user_membership_port.clone(),
        )),
        create_order_use_case: Arc::new(CreateMatchIdOrderUseCase::new(
            repository,
            match_id_source,
            order_repository,
            user_membership_port,
            wechat_pay_port,
        )),
        token_port,
    });

    match_id_unlock_routes(state)
}
