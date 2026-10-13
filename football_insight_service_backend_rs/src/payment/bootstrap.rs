use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    auth::ports::{token_port::TokenPort, user_membership_port::UserMembershipPort},
    config::AppConfig,
    payment::{
        adapters::{
            integration::{
                virtual_payment_gateway::HttpVirtualPaymentGateway,
                virtual_payment_webhook::VirtualWebhookCodec,
            },
            persistence::{
                postgres_order_repository::PostgresOrderRepository,
                postgres_payment_settlement_port::PostgresPaymentSettlementPort,
                postgres_virtual_order_repository::PostgresVirtualOrderRepository,
            },
            web::{handlers::PaymentWebState, routes::payment_routes},
        },
        application::{
            create_membership_order::CreateMembershipOrderUseCase,
            get_membership_product::GetMembershipProductUseCase,
            get_order_status::GetOrderStatusUseCase,
            handle_wechat_notify::HandleWechatNotifyUseCase,
            virtual_payment::VirtualPaymentService,
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
    let payment_settlement_port = Arc::new(PostgresPaymentSettlementPort::new(pool.clone()));
    let settlement = Arc::new(HandleWechatNotifyUseCase::new(
        order_repository.clone(),
        payment_settlement_port,
    ));
    let gateway = Arc::new(HttpVirtualPaymentGateway::new(
        config.virtual_payment.clone(),
        config.wechat_mini_app_id.clone(),
        config.wechat_mini_app_secret.clone(),
    ));
    let virtual_payment = Arc::new(VirtualPaymentService::new(
        gateway,
        order_repository.clone(),
        Arc::new(PostgresVirtualOrderRepository::new(pool)),
        user_membership_port.clone(),
        settlement.clone(),
        config.virtual_payment.products.clone(),
    ));
    let wechat_pay_port: Arc<dyn WechatPayPort> = virtual_payment.clone();
    let virtual_webhook = if config.virtual_payment.available() {
        VirtualWebhookCodec::new(
            config.virtual_payment.webhook_token.clone(),
            &config.virtual_payment.encoding_aes_key,
            config.wechat_mini_app_id.clone(),
        )
        .ok()
        .map(|codec| {
            Arc::new(codec)
                as Arc<dyn crate::payment::ports::virtual_payment_port::VirtualWebhookPort>
        })
    } else {
        None
    };
    if virtual_payment.available() {
        let worker = virtual_payment.clone();
        tokio::spawn(async move {
            let mut clock = tokio::time::interval(std::time::Duration::from_secs(60));
            clock.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                clock.tick().await;
                if let Err(error) = worker.reconcile_pending().await {
                    tracing::warn!(error=%error,"virtual payment compensation failed");
                }
            }
        });
    }

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
        handle_wechat_notify_use_case: settlement,
        virtual_payment,
        virtual_webhook,
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
