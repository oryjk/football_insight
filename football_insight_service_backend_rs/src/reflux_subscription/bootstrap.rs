use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    auth::ports::{token_port::TokenPort, user_membership_port::UserMembershipPort},
    payment::ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
    reflux_subscription::{
        adapters::{
            persistence::postgres_reflux_subscription_repository::PostgresRefluxSubscriptionRepository,
            web::{handlers::RefluxSubscriptionWebState, routes::reflux_subscription_routes},
        },
        application::{
            create_reflux_subscription_order::CreateRefluxSubscriptionOrderUseCase,
            get_reflux_subscription_plans::GetRefluxSubscriptionPlansUseCase,
        },
    },
};

pub fn build_reflux_subscription_routes(
    pool: PgPool,
    order_repository: Arc<dyn OrderRepository>,
    user_membership_port: Arc<dyn UserMembershipPort>,
    wechat_pay_port: Arc<dyn WechatPayPort>,
    token_port: Arc<dyn TokenPort>,
) -> Router {
    let repository = Arc::new(PostgresRefluxSubscriptionRepository::new(pool));
    let state = Arc::new(RefluxSubscriptionWebState {
        get_plans_use_case: Arc::new(GetRefluxSubscriptionPlansUseCase::new(repository.clone())),
        create_order_use_case: Arc::new(CreateRefluxSubscriptionOrderUseCase::new(
            repository.clone(),
            order_repository,
            user_membership_port,
            wechat_pay_port,
        )),
        repository,
        token_port,
    });

    reflux_subscription_routes(state)
}
