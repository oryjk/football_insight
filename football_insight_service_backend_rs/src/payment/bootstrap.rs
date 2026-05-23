use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    auth::ports::{token_port::TokenPort, user_membership_port::UserMembershipPort},
    config::AppConfig,
    payment::{
        adapters::{
            integration::wechat_pay_port::HttpWechatPayPort,
            persistence::{
                postgres_order_repository::PostgresOrderRepository,
                postgres_payment_settlement_port::PostgresPaymentSettlementPort,
            },
            web::{handlers::PaymentWebState, routes::payment_routes},
        },
        application::{
            create_membership_order::CreateMembershipOrderUseCase,
            get_membership_product::GetMembershipProductUseCase,
            get_order_status::GetOrderStatusUseCase,
            handle_wechat_notify::HandleWechatNotifyUseCase,
        },
        ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
    },
    system_config::ports::system_config_port::SystemConfigPort,
};

pub struct PaymentBootstrap {
    pub routes: Router,
    pub order_repository: Arc<dyn OrderRepository>,
    pub wechat_pay_port: Arc<dyn WechatPayPort>,
}

pub fn build_payment(
    pool: PgPool,
    config: &AppConfig,
    system_config_port: Arc<dyn SystemConfigPort>,
    user_membership_port: Arc<dyn UserMembershipPort>,
    token_port: Arc<dyn TokenPort>,
) -> PaymentBootstrap {
    let order_repository: Arc<dyn OrderRepository> =
        Arc::new(PostgresOrderRepository::new(pool.clone()));
    let payment_settlement_port = Arc::new(PostgresPaymentSettlementPort::new(pool));
    let wechat_pay_port: Arc<dyn WechatPayPort> = Arc::new(HttpWechatPayPort::new(
        config.wechat_mini_app_id.clone(),
        config.wechat_pay_mch_id.clone(),
        config.wechat_pay_api_key.clone(),
        config.public_base_url.clone(),
    ));

    let state = Arc::new(PaymentWebState {
        create_membership_order_use_case: Arc::new(CreateMembershipOrderUseCase::new(
            order_repository.clone(),
            user_membership_port.clone(),
            wechat_pay_port.clone(),
        )),
        get_membership_product_use_case: Arc::new(GetMembershipProductUseCase::new(
            system_config_port,
        )),
        get_order_status_use_case: Arc::new(GetOrderStatusUseCase::new(order_repository.clone())),
        handle_wechat_notify_use_case: Arc::new(HandleWechatNotifyUseCase::new(
            order_repository.clone(),
            payment_settlement_port,
        )),
        token_port,
        user_membership_port,
        wechat_pay_api_key: config.wechat_pay_api_key.clone(),
    });

    PaymentBootstrap {
        routes: payment_routes(state),
        order_repository,
        wechat_pay_port,
    }
}
