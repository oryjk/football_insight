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
                postgres_virtual_product_repository::PostgresVirtualProductRepository,
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
    pub event_recorder:
        Arc<dyn crate::payment::ports::payment_event_recorder::PaymentEventRecorder>,
}

fn build_virtual_webhook(
    config: &crate::config::VirtualPaymentConfig,
    app_id: &str,
) -> Option<Arc<dyn crate::payment::ports::virtual_payment_port::VirtualWebhookPort>> {
    // URL verification must work before purchases and the product catalog are enabled.
    VirtualWebhookCodec::new(
        config.webhook_token.clone(),
        &config.encoding_aes_key,
        app_id.into(),
    )
    .ok()
    .map(|codec| {
        Arc::new(codec) as Arc<dyn crate::payment::ports::virtual_payment_port::VirtualWebhookPort>
    })
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
    // Dedicated, bounded pool: telemetry cannot exhaust the business connection pool.
    let event_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_millis(100))
        .connect_lazy_with((*pool.connect_options()).clone());
    let event_recorder: Arc<dyn crate::payment::ports::payment_event_recorder::PaymentEventRecorder> = Arc::new(
        crate::payment::adapters::persistence::postgres_payment_event_recorder::PostgresPaymentEventRecorder::new(event_pool.clone()));
    let event_orders: Arc<dyn OrderRepository> = Arc::new(PostgresOrderRepository::new(event_pool));
    let report_events = Arc::new(
        crate::payment::application::report_payment_events::ReportPaymentEvents::new(
            event_orders.clone(),
            crate::payment::application::payment_events::PaymentEvents::new(event_recorder.clone()),
        ),
    );
    let payment_settlement_port = Arc::new(PostgresPaymentSettlementPort::new(pool.clone()));
    let settlement = Arc::new(
        HandleWechatNotifyUseCase::new(order_repository.clone(), payment_settlement_port)
            .with_event_recorder(event_recorder.clone()),
    );
    let gateway = Arc::new(HttpVirtualPaymentGateway::new(
        config.virtual_payment.clone(),
        config.wechat_mini_app_id.clone(),
        config.wechat_mini_app_secret.clone(),
    ));
    let virtual_products = Arc::new(PostgresVirtualProductRepository::new(pool.clone()));
    let virtual_payment = Arc::new(
        VirtualPaymentService::new(
            gateway,
            order_repository.clone(),
            Arc::new(PostgresVirtualOrderRepository::new(pool.clone())),
            user_membership_port.clone(),
            settlement.clone(),
            virtual_products.clone(),
        )
        .with_event_recorder(event_recorder.clone())
        .with_event_order_repository(event_orders),
    );
    let wechat_pay_port: Arc<dyn WechatPayPort> = virtual_payment.clone();
    let virtual_webhook =
        build_virtual_webhook(&config.virtual_payment, &config.wechat_mini_app_id);
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
        create_membership_order_use_case: Arc::new(
            CreateMembershipOrderUseCase::new(
                order_repository.clone(),
                user_membership_port.clone(),
                wechat_pay_port.clone(),
            )
            .with_event_recorder(event_recorder.clone()),
        ),
        get_membership_product_use_case: Arc::new(GetMembershipProductUseCase::new(
            system_config_port,
            virtual_products,
        )),
        get_order_status_use_case: Arc::new(GetOrderStatusUseCase::new(order_repository.clone())),
        handle_wechat_notify_use_case: settlement,
        virtual_payment,
        virtual_webhook,
        token_port: token_port.clone(),
        user_membership_port,
        wechat_pay_api_key: config.wechat_pay_api_key.clone(),
    });

    PaymentBootstrap {
        routes: payment_routes(state).merge(
            crate::payment::adapters::web::event_handlers::payment_event_routes(
                report_events,
                token_port,
            ),
        ),
        order_repository,
        wechat_pay_port,
        event_recorder,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use sha1::{Digest, Sha1};

    fn disabled_config() -> crate::config::VirtualPaymentConfig {
        crate::config::VirtualPaymentConfig {
            enabled: false,
            offer_id: String::new(),
            app_key: String::new(),
            webhook_token: "token".into(),
            encoding_aes_key: base64::engine::general_purpose::STANDARD.encode([7u8; 32]),
        }
    }

    #[test]
    fn disabled_purchase_without_catalog_still_verifies_webhook_url() {
        let config = disabled_config();
        assert!(!config.available());
        let codec = build_virtual_webhook(&config, "app").expect("webhook must remain available");
        let mut parts = ["token", "123", "nonce"];
        parts.sort_unstable();
        let signature = format!("{:x}", Sha1::digest(parts.join("").as_bytes()));
        assert_eq!(
            codec.challenge(&signature, "123", "nonce", "echo").unwrap(),
            "echo"
        );
        assert!(codec.challenge("invalid", "123", "nonce", "echo").is_err());
    }

    #[test]
    fn invalid_credentials_do_not_initialize_webhook() {
        let mut config = disabled_config();
        config.encoding_aes_key = "invalid".into();
        assert!(build_virtual_webhook(&config, "app").is_none());
        config = disabled_config();
        config.webhook_token.clear();
        assert!(build_virtual_webhook(&config, "app").is_none());
        assert!(build_virtual_webhook(&disabled_config(), "").is_none());
    }
}
