use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    auth::ports::{token_port::TokenPort, user_membership_port::UserMembershipPort},
    config::AppConfig,
    seat_swap::{
        adapters::{
            integration::{
                minio_evidence_storage_port::MinioEvidenceStoragePort,
                ticket_watch_current_match_port::TicketWatchCurrentSeatSwapMatchPort,
                wechat_mini_subscribe_port::OfficialWechatMiniSubscribePort,
            },
            persistence::postgres_seat_swap_repository::PostgresSeatSwapRepository,
            web::{handlers::SeatSwapWebState, routes::seat_swap_routes},
        },
        application::{
            cancel_matched_seat_swap::CancelMatchedSeatSwapUseCase,
            cancel_my_seat_swap_request::CancelMySeatSwapRequestUseCase,
            cancel_seat_swap_confirmation::CancelSeatSwapConfirmationUseCase,
            confirm_seat_swap_candidate::ConfirmSeatSwapCandidateUseCase,
            get_current_seat_swap::GetCurrentSeatSwapUseCase,
            upsert_my_seat_swap_request::UpsertMySeatSwapRequestUseCase,
        },
    },
    ticket_watch::ports::ticket_monitor_port::TicketMonitorPort,
};

pub fn build_seat_swap_routes(
    pool: PgPool,
    config: &AppConfig,
    ticket_watch_port: Arc<dyn TicketMonitorPort>,
    token_port: Arc<dyn TokenPort>,
    user_membership_port: Arc<dyn UserMembershipPort>,
) -> Router {
    seat_swap_routes(build_seat_swap_state(
        pool,
        config,
        ticket_watch_port,
        token_port,
        user_membership_port,
    ))
}

pub fn build_seat_swap_state(
    pool: PgPool,
    config: &AppConfig,
    ticket_watch_port: Arc<dyn TicketMonitorPort>,
    token_port: Arc<dyn TokenPort>,
    user_membership_port: Arc<dyn UserMembershipPort>,
) -> Arc<SeatSwapWebState> {
    let repository = Arc::new(PostgresSeatSwapRepository::new(pool));
    let current_match_port = Arc::new(TicketWatchCurrentSeatSwapMatchPort::new(ticket_watch_port));
    let evidence_storage = Arc::new(MinioEvidenceStoragePort::new(config.minio.clone()));
    let mini_subscribe_port = Arc::new(OfficialWechatMiniSubscribePort::new(
        config.wechat_mini_app_id.clone(),
        config.wechat_mini_app_secret.clone(),
        config.seat_swap_mini_subscribe_template_id.clone(),
        config.seat_swap_mini_subscribe_page.clone(),
        user_membership_port.clone(),
    ));

    Arc::new(SeatSwapWebState {
        get_current_use_case: Arc::new(GetCurrentSeatSwapUseCase::new(
            repository.clone(),
            current_match_port.clone(),
        )),
        upsert_my_request_use_case: Arc::new(UpsertMySeatSwapRequestUseCase::new(
            repository.clone(),
            current_match_port.clone(),
        )),
        cancel_my_request_use_case: Arc::new(CancelMySeatSwapRequestUseCase::new(
            repository.clone(),
            current_match_port.clone(),
        )),
        confirm_candidate_use_case: Arc::new(ConfirmSeatSwapCandidateUseCase::new(
            repository.clone(),
            current_match_port.clone(),
            user_membership_port,
            mini_subscribe_port,
        )),
        cancel_confirmation_use_case: Arc::new(CancelSeatSwapConfirmationUseCase::new(
            repository.clone(),
            current_match_port.clone(),
        )),
        cancel_matched_use_case: Arc::new(CancelMatchedSeatSwapUseCase::new(
            repository,
            current_match_port,
            evidence_storage,
        )),
        token_port,
    })
}
