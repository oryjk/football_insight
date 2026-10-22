use async_trait::async_trait;
use football_insight_service_backend_rs::{
    payment::{
        application::get_membership_product::GetMembershipProductUseCase,
        domain::{
            product_kind::ProductSpec,
            virtual_payment::{PrivateMembershipProduct, VirtualPaymentError, VirtualProduct},
        },
        ports::virtual_payment_port::VirtualProductRepository,
    },
    system_config::{
        domain::{ai_chat_config::AiChatSystemConfig, public_system_config::PublicSystemConfig},
        ports::system_config_port::SystemConfigPort,
    },
};
use serde_json::json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
struct Config;
#[async_trait]
impl SystemConfigPort for Config {
    async fn get_public_config(&self) -> anyhow::Result<PublicSystemConfig> {
        unreachable!()
    }
    async fn get_ai_chat_config(&self) -> anyhow::Result<AiChatSystemConfig> {
        unreachable!()
    }
    async fn get_config_value(&self, key: &str) -> anyhow::Result<Option<String>> {
        assert_eq!(key, "membership_product");
        Ok(None)
    }
}
struct Products(Mutex<Vec<ProductSpec>>);
#[async_trait]
impl VirtualProductRepository for Products {
    async fn find_enabled(
        &self,
        _: &str,
        _: Uuid,
    ) -> Result<Option<VirtualProduct>, VirtualPaymentError> {
        Ok(None)
    }
    async fn has_enabled(&self) -> Result<bool, VirtualPaymentError> {
        Ok(true)
    }
    async fn private_membership_products(
        &self,
        _: Uuid,
    ) -> Result<Vec<PrivateMembershipProduct>, VirtualPaymentError> {
        Ok(vec![])
    }
    async fn membership_catalog_products(
        &self,
        _: Uuid,
    ) -> Result<Vec<ProductSpec>, VirtualPaymentError> {
        Ok(self.0.lock().unwrap().clone())
    }
}
fn spec(tier: &str, cents: i32) -> ProductSpec {
    serde_json::from_value(json!({"catalog_product_id":Uuid::new_v4(),"product_key":format!("membership:{tier}@{cents}"),"kind":"membership","price_cents":cents,"membership_duration_months":1,"grant_spec":{"tier":tier,"expiry":{"rule":"end_of_next_day","tz":"Asia/Shanghai"}},"pricing_rule":{"type":"fixed"},"eligibility":{"max_current_tier":"V3"},"requires_capability":"catalog-v2","title":format!("{tier} 场次体验卡"),"subtitle":"次日有效","description":"按数据授予","group_key":"pass","sort_order":10})).unwrap()
}
fn service() -> GetMembershipProductUseCase {
    GetMembershipProductUseCase::new(
        Arc::new(Config),
        Arc::new(Products(Mutex::new(vec![
            spec("V4", 900),
            spec("V5", 1900),
        ]))),
    ).with_kind_registry(Arc::new(football_insight_service_backend_rs::payment::domain::product_kind::ProductKindRegistry::default()))
}

#[tokio::test]
async fn capabilities_and_current_tier_gate_catalog_while_annual_fields_stay_legacy() {
    let service = service();
    let uid = Some(Uuid::new_v4());
    let old = service
        .execute_catalog_for_user(uid, Some("V3"), &[])
        .await
        .unwrap();
    assert_eq!(
        old.products
            .iter()
            .map(|p| p.target_tier.as_str())
            .collect::<Vec<_>>(),
        ["V6", "V7", "V8", "V9"]
    );
    let json=serde_json::to_value(football_insight_service_backend_rs::payment::adapters::web::dto::MembershipProductResponse::from_product_for_tier(old,Some("V3"))).unwrap();
    assert!(json["products"][0].get("duration_label").is_none());
    assert!(json["products"][0].get("is_pass").is_none());
    assert_eq!(json["products"][0]["price_cents"], 3900);
    let caps = vec!["catalog-v2".to_owned()];
    let new = service
        .execute_catalog_for_user(uid, Some("V3"), &caps)
        .await
        .unwrap();
    let pass = new.find_option("V4").unwrap();
    assert_eq!(pass.price_cents, 900);
    assert_eq!(pass.presentation.is_pass, Some(true));
    assert!(pass.membership_duration_months.is_none());
    assert_eq!(
        pass.presentation.duration_label.as_deref(),
        Some("购买次日 23:59（北京时间）")
    );
    for tier in ["V4", "V5", "V6", "V7", "V8", "V9"] {
        assert!(
            service
                .execute_catalog_for_user(uid, Some(tier), &caps)
                .await
                .unwrap()
                .find_option("V4")
                .is_none()
        );
    }
}
#[tokio::test]
async fn purchase_rechecks_capability_and_audience_without_relying_on_listing() {
    let service = service();
    let uid = Uuid::new_v4();
    assert_eq!(
        service
            .execute_purchase_catalog(uid, Some("V3"), &[], "V4")
            .await
            .unwrap_err()
            .to_string(),
        "请更新小程序后购买"
    );
    let caps = vec!["catalog-v2".to_owned()];
    for tier in ["V4", "V6", "V9"] {
        assert_eq!(
            service
                .execute_purchase_catalog(uid, Some(tier), &caps, "V5")
                .await
                .unwrap_err()
                .to_string(),
            "当前会员等级不可购买该商品"
        );
    }
    assert!(
        service
            .execute_purchase_catalog(uid, Some("V3"), &caps, "V5")
            .await
            .unwrap()
            .find_option("V5")
            .is_some()
    );
}

use football_insight_service_backend_rs::{
    auth::ports::user_membership_port::UserMembershipPort,
    payment::{
        application::create_membership_order::{
            CreateMembershipOrderInput, CreateMembershipOrderUseCase,
        },
        domain::order::{NewPaymentOrder, OrderStatus, PaymentOrder, WxPayParams},
        domain::payment_event::NewPaymentEvent,
        ports::{
            order_repository::OrderRepository, payment_event_recorder::PaymentEventRecorder,
            wechat_pay_port::WechatPayPort,
        },
    },
};
struct User(&'static str);
#[async_trait]
impl UserMembershipPort for User {
    async fn get_user_open_id(&self, _: Uuid) -> anyhow::Result<Option<String>> {
        Ok(Some("fake-openid".into()))
    }
    async fn get_user_membership_tier(&self, _: Uuid) -> anyhow::Result<Option<String>> {
        Ok(Some(self.0.into()))
    }
    async fn update_user_membership_tier(&self, _: Uuid, _: &str) -> anyhow::Result<()> {
        unreachable!()
    }
    async fn is_seat_swap_notice_enabled(&self, _: Uuid) -> anyhow::Result<Option<bool>> {
        Ok(Some(false))
    }
}
#[derive(Default)]
struct Orders(Mutex<Vec<NewPaymentOrder>>);
#[async_trait]
impl OrderRepository for Orders {
    async fn create_order(&self, o: NewPaymentOrder) -> anyhow::Result<PaymentOrder> {
        self.0.lock().unwrap().push(o.clone());
        Ok(PaymentOrder {
            id: Uuid::new_v4(),
            order_no: o.order_no,
            user_id: o.user_id,
            amount_cents: o.amount_cents,
            status: OrderStatus::Pending,
            prepay_id: None,
            transaction_id: None,
            product_type: o.product_type,
            paid_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn find_order_by_no(&self, _: &str) -> anyhow::Result<Option<PaymentOrder>> {
        unreachable!()
    }
}
struct Pay(bool);
#[async_trait]
impl WechatPayPort for Pay {
    async fn unified_order(
        &self,
        _: &str,
        _: &str,
        cents: i32,
        _: &str,
    ) -> anyhow::Result<WxPayParams> {
        assert_eq!(cents, 900);
        if self.0 {
            return Err(VirtualPaymentError::Provider.into());
        }
        Ok(WxPayParams {
            virtual_payment: None,
            time_stamp: "".into(),
            nonce_str: "".into(),
            package: "".into(),
            sign_type: "".into(),
            pay_sign: "".into(),
        })
    }
}
#[derive(Default)]
struct Events(Mutex<Vec<NewPaymentEvent>>);
#[async_trait]
impl PaymentEventRecorder for Events {
    async fn record(&self, event: NewPaymentEvent) {
        self.0.lock().unwrap().push(event);
    }
}
#[tokio::test]
async fn fresh_order_check_blocks_stale_audience_and_preserves_observability() {
    let uid = Uuid::new_v4();
    let caps = vec!["catalog-v2".into()];
    let catalog = service()
        .execute_catalog_for_user(Some(uid), Some("V3"), &caps)
        .await
        .unwrap();
    for (tier, capabilities, want_error) in [
        ("V3", vec![], Some("请更新小程序后购买")),
        ("V4", caps.clone(), Some("当前会员等级不可购买该商品")),
        ("V3", caps.clone(), None),
    ] {
        for fails in [false, true] {
            let orders = Arc::new(Orders::default());
            let events = Arc::new(Events::default());
            let use_case = CreateMembershipOrderUseCase::new(
                orders.clone(),
                Arc::new(User(tier)),
                Arc::new(Pay(fails)),
            )
            .with_catalog_repository(Arc::new(Products(Mutex::new(vec![
                catalog
                    .find_option("V4")
                    .unwrap()
                    .presentation
                    .spec
                    .clone()
                    .unwrap(),
            ]))))
            .with_event_recorder(events.clone());
            let result = use_case
                .execute_with_capabilities(
                    CreateMembershipOrderInput {
                        user_id: uid,
                        target_tier: "V4".into(),
                        products: catalog.products.clone(),
                    },
                    Some(Uuid::new_v4()),
                    &capabilities,
                )
                .await;
            for _ in 0..20 {
                tokio::task::yield_now().await;
            }
            if let Some(message) = want_error {
                assert_eq!(result.err().unwrap().to_string(), message);
                assert!(orders.0.lock().unwrap().is_empty());
                assert!(events.0.lock().unwrap().is_empty());
            } else {
                assert_eq!(orders.0.lock().unwrap()[0].product_type, "membership:V4");
                let emitted = events.0.lock().unwrap();
                assert_eq!(emitted[0].event.as_str(), "order_created");
                assert_eq!(
                    emitted[1].event.as_str(),
                    if fails {
                        "pay_params_failed"
                    } else {
                        "pay_params_issued"
                    }
                );
                if fails {
                    assert!(matches!(
                        result.err().unwrap().downcast_ref(),
                        Some(VirtualPaymentError::Provider)
                    ));
                } else {
                    assert!(result.is_ok());
                }
            }
        }
    }
}

#[tokio::test]
async fn checkout_uses_catalog_identity_and_eligible_sibling_cannot_change_price() {
    let uid = Uuid::new_v4();
    let caps = vec!["catalog-v2".into()];
    let cheap = spec("V4", 900);
    let expensive = spec("V4", 1900);
    let products = Arc::new(Products(Mutex::new(vec![expensive, cheap])));
    let catalog = GetMembershipProductUseCase::new(Arc::new(Config), products.clone())
        .execute_purchase_catalog(uid, Some("V3"), &caps, "V4")
        .await
        .unwrap();
    assert_eq!(catalog.find_option("V4").unwrap().price_cents, 900);
    let orders = Arc::new(Orders::default());
    let checkout = CreateMembershipOrderUseCase::new(
        orders.clone(),
        Arc::new(User("V3")),
        Arc::new(Pay(false)),
    )
    .with_catalog_repository(products.clone());
    checkout
        .execute_with_capabilities(
            CreateMembershipOrderInput {
                user_id: uid,
                target_tier: "V4".into(),
                products: catalog.products.clone(),
            },
            None,
            &caps,
        )
        .await
        .unwrap();
    assert_eq!(orders.0.lock().unwrap()[0].amount_cents, 900);
    // A gated/ineligible sibling must not veto the already selected eligible offer.
    let mut sibling = spec("V4", 2900);
    sibling.eligibility = json!({"max_current_tier":"V3","min_current_tier":"V3"});
    products.0.lock().unwrap().push(sibling);
    assert!(
        GetMembershipProductUseCase::new(Arc::new(Config), products.clone())
            .execute_purchase_catalog(uid, Some("V2"), &caps, "V4")
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn table_offer_never_displaces_a_legacy_annual_option_of_the_same_tier() {
    let service = GetMembershipProductUseCase::new(
        Arc::new(Config),
        Arc::new(Products(Mutex::new(vec![spec("V6", 900), spec("V5", 1900)]))),
    )
    .with_kind_registry(Arc::new(
        football_insight_service_backend_rs::payment::domain::product_kind::ProductKindRegistry::default(),
    ));
    let caps = vec!["catalog-v2".to_owned()];
    let catalog = service
        .execute_catalog_for_user(Some(Uuid::new_v4()), Some("V3"), &caps)
        .await
        .unwrap();
    let annual = catalog.find_option("V6").expect("annual V6 stays visible");
    assert_eq!(annual.price_cents, 3900, "annual price is untouched");
    assert!(
        annual.presentation.spec.is_none(),
        "V6 is still the legacy annual option"
    );
    assert_eq!(
        catalog
            .products
            .iter()
            .filter(|p| p.target_tier == "V6")
            .count(),
        1,
        "one option per tier"
    );
    assert_eq!(catalog.find_option("V5").unwrap().price_cents, 1900);
}
