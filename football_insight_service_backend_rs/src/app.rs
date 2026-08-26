use std::sync::Arc;

use axum::{Router, routing::get};
use sqlx::PgPool;

use crate::{
    activity::bootstrap::build_activity_routes, admin::bootstrap::build_admin_routes,
    ai::bootstrap::build_ai_routes, auth::bootstrap::build_auth,
    auth_license::bootstrap::build_auth_license_routes, config::AppConfig,
    health::bootstrap::build_health_routes, insight::bootstrap::build_insight,
    match_id_unlock::bootstrap::build_match_id_unlock_routes,
    mini_review::bootstrap::build_mini_review_routes, payment::bootstrap::build_payment,
    push_notification::bootstrap::build_push_notification_routes,
    reflux_subscription::bootstrap::build_reflux_subscription_routes,
    seat_swap::bootstrap::build_seat_swap_routes, support::bootstrap::build_support_routes,
    system_config::bootstrap::build_system_config_routes,
    team_board::bootstrap::build_team_board_routes, ticket_watch::bootstrap::build_ticket_watch,
};

pub fn build_router(pool: PgPool, config: &AppConfig) -> Router {
    let insight = build_insight(pool.clone());
    let admin_routes = build_admin_routes(pool.clone(), config);
    let health_routes = build_health_routes(pool.clone());
    let system_config = build_system_config_routes(pool.clone(), insight.repository.clone());

    let auth = build_auth(
        pool.clone(),
        config,
        system_config.system_config_port.clone(),
    );
    let activity_routes = build_activity_routes(pool.clone(), auth.token_port.clone());
    let ai_routes = build_ai_routes(
        config,
        system_config.system_config_port.clone(),
        insight.repository.clone(),
        auth.get_current_user_use_case.clone(),
    );
    let team_board_routes = build_team_board_routes(
        pool.clone(),
        insight.repository.clone(),
        auth.token_port.clone(),
    );
    let support_routes = build_support_routes(pool.clone(), auth.token_port.clone());
    let ticket_watch = build_ticket_watch(config, auth.token_port.clone());
    let user_membership_port: Arc<
        dyn crate::auth::ports::user_membership_port::UserMembershipPort,
    > = auth.auth_repository.clone();
    let seat_swap_routes = build_seat_swap_routes(
        pool.clone(),
        config,
        ticket_watch.ticket_monitor_port.clone(),
        auth.token_port.clone(),
        user_membership_port.clone(),
    );
    let payment = build_payment(
        pool.clone(),
        config,
        system_config.system_config_port.clone(),
        user_membership_port.clone(),
        auth.token_port.clone(),
    );
    let reflux_subscription_routes = build_reflux_subscription_routes(
        pool.clone(),
        payment.order_repository.clone(),
        user_membership_port.clone(),
        payment.wechat_pay_port.clone(),
        auth.token_port.clone(),
    );
    let match_id_unlock_routes = build_match_id_unlock_routes(
        pool.clone(),
        ticket_watch.ticket_monitor_port.clone(),
        payment.order_repository.clone(),
        user_membership_port.clone(),
        payment.wechat_pay_port.clone(),
        auth.token_port.clone(),
    );

    let auth_license_routes = build_auth_license_routes(pool.clone(), auth.token_port.clone());
    let mini_review_routes = build_mini_review_routes(
        pool.clone(),
        config.mini_review_api_key.clone(),
        auth.token_port.clone(),
        config.mini_review_control_user_ids.clone(),
    );
    let push_notification_routes = build_push_notification_routes(pool, auth.token_port.clone());

    Router::new()
        .route("/", get(|| async { "football insight service" }))
        .merge(health_routes)
        .merge(mini_review_routes)
        .merge(admin_routes)
        .merge(system_config.routes)
        .merge(auth.routes)
        .merge(activity_routes)
        .merge(ai_routes)
        .merge(support_routes)
        .merge(ticket_watch.routes)
        .merge(seat_swap_routes)
        .merge(insight.routes)
        .merge(team_board_routes)
        .merge(payment.routes)
        .merge(reflux_subscription_routes)
        .merge(match_id_unlock_routes)
        .merge(auth_license_routes)
        .merge(push_notification_routes)
}
