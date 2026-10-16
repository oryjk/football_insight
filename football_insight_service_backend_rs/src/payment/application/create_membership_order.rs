use crate::payment::{
    application::payment_events::{OrderAttempt, PaymentEvents},
    ports::payment_event_recorder::PaymentEventRecorder,
};
use std::sync::Arc;

use chrono::Utc;
use rand::Rng;

use crate::{
    auth::domain::membership::membership_tier_rank,
    auth::ports::user_membership_port::UserMembershipPort,
    payment::{
        domain::order::{
            MembershipProductOption, NewPaymentOrder, WxPayParams,
            calculate_membership_checkout_price, membership_product_type_for_tier,
        },
        ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
    },
};

pub struct CreateMembershipOrderUseCase {
    registry: Arc<crate::payment::domain::product_kind::ProductKindRegistry>,
    events: PaymentEvents,
    catalog: Option<Arc<dyn crate::payment::ports::virtual_payment_port::VirtualProductRepository>>,
    order_repository: Arc<dyn OrderRepository>,
    user_membership_port: Arc<dyn UserMembershipPort>,
    wechat_pay_port: Arc<dyn WechatPayPort>,
}

impl CreateMembershipOrderUseCase {
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        user_membership_port: Arc<dyn UserMembershipPort>,
        wechat_pay_port: Arc<dyn WechatPayPort>,
    ) -> Self {
        Self {
            registry: Arc::new(
                crate::payment::domain::product_kind::ProductKindRegistry::default(),
            ),
            events: PaymentEvents::default(),
            catalog: None,
            order_repository,
            user_membership_port,
            wechat_pay_port,
        }
    }

    pub fn with_event_recorder(mut self, recorder: Arc<dyn PaymentEventRecorder>) -> Self {
        self.events = PaymentEvents::new(recorder);
        self
    }

    pub fn with_kind_registry(
        mut self,
        registry: Arc<crate::payment::domain::product_kind::ProductKindRegistry>,
    ) -> Self {
        self.registry = registry;
        self
    }

    pub fn with_catalog_repository(
        mut self,
        catalog: Arc<dyn crate::payment::ports::virtual_payment_port::VirtualProductRepository>,
    ) -> Self {
        self.catalog = Some(catalog);
        self
    }

    pub async fn execute(
        &self,
        input: CreateMembershipOrderInput,
    ) -> anyhow::Result<CreateMembershipOrderOutput> {
        self.execute_with_attempt(input, None).await
    }

    pub async fn execute_with_attempt(
        &self,
        input: CreateMembershipOrderInput,
        attempt_id: Option<uuid::Uuid>,
    ) -> anyhow::Result<CreateMembershipOrderOutput> {
        self.execute_with_capabilities(input, attempt_id, &[]).await
    }
    pub async fn execute_with_capabilities(
        &self,
        input: CreateMembershipOrderInput,
        attempt_id: Option<uuid::Uuid>,
        capabilities: &[String],
    ) -> anyhow::Result<CreateMembershipOrderOutput> {
        let openid = self
            .user_membership_port
            .get_user_open_id(input.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("请先绑定微信"))?;
        let current_tier = self
            .user_membership_port
            .get_user_membership_tier(input.user_id)
            .await?
            .unwrap_or_else(|| "V1".to_string());

        let mut fixed_price = None;
        let selected_spec = input
            .products
            .iter()
            .find(|p| p.target_tier == input.target_tier)
            .and_then(|p| p.presentation.spec.as_ref());
        if let (Some(catalog), Some(selected)) = (&self.catalog, selected_spec) {
            use crate::payment::domain::product_kind::UserContext;
            let spec=catalog.membership_catalog_products(input.user_id).await?.into_iter()
                .find(|spec|spec.catalog_product_id==selected.catalog_product_id && spec.product_key==selected.product_key && spec.price_cents==selected.price_cents)
                .ok_or(crate::payment::domain::virtual_payment::VirtualPaymentError::ProductUnavailable)?;
            fixed_price = Some(self.registry.resolve_price(
                &spec,
                &UserContext {
                    current_tier: current_tier.clone(),
                    capabilities: capabilities.to_vec(),
                },
            )?);
        }

        let is_v9_renewal = current_tier.trim().eq_ignore_ascii_case("V9")
            && input.target_tier.trim().eq_ignore_ascii_case("V9");
        if !is_v9_renewal
            && membership_tier_rank(&input.target_tier) <= membership_tier_rank(&current_tier)
        {
            anyhow::bail!("请选择高于当前等级的会员档位");
        }
        let price = match fixed_price {
            Some(price) => crate::payment::domain::order::MembershipCheckoutPrice {
                original_price_cents: price,
                pay_price_cents: price,
                upgrade_fee_cents: 0,
            },
            None => calculate_membership_checkout_price(
                &input.products,
                &current_tier,
                &input.target_tier,
            )?,
        };
        let product = input
            .products
            .iter()
            .find(|product| {
                product
                    .target_tier
                    .trim()
                    .eq_ignore_ascii_case(&input.target_tier)
            })
            .ok_or_else(|| anyhow::anyhow!("请选择有效的会员档位"))?;

        let order_no = generate_order_no();

        self.order_repository
            .create_order(NewPaymentOrder {
                order_no: order_no.clone(),
                user_id: input.user_id,
                amount_cents: price.pay_price_cents,
                product_type: membership_product_type_for_tier(&input.target_tier),
            })
            .await?;

        let attempt = OrderAttempt::new(
            self.events.clone(),
            attempt_id.unwrap_or_else(uuid::Uuid::new_v4),
            input.user_id,
            "membership",
            input.target_tier.clone(),
            order_no.clone(),
        );
        attempt.created();

        let wx_pay_params = attempt.params(
            self.wechat_pay_port
                .unified_membership_order(
                    &order_no,
                    &product.title,
                    price.pay_price_cents,
                    &openid,
                    capabilities,
                )
                .await,
        )?;

        Ok(CreateMembershipOrderOutput {
            order_no,
            wx_pay_params,
        })
    }
}

fn generate_order_no() -> String {
    let timestamp = Utc::now().timestamp_millis();
    let random: u32 = rand::rng().random_range(1000..10000);
    format!("{}{}", timestamp, random)
}

pub struct CreateMembershipOrderInput {
    pub user_id: uuid::Uuid,
    pub target_tier: String,
    pub products: Vec<MembershipProductOption>,
}

pub struct CreateMembershipOrderOutput {
    pub order_no: String,
    pub wx_pay_params: WxPayParams,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    use super::{CreateMembershipOrderInput, CreateMembershipOrderUseCase};
    use crate::{
        auth::ports::user_membership_port::UserMembershipPort,
        payment::{
            domain::order::{
                NewPaymentOrder, OrderStatus, PaymentOrder, WxPayParams,
                default_membership_product_options,
            },
            ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
        },
    };

    struct FakeUserMembershipPort {
        open_id: Option<String>,
        membership_tier: Option<String>,
    }

    #[async_trait]
    impl UserMembershipPort for FakeUserMembershipPort {
        async fn get_user_open_id(&self, _user_id: Uuid) -> anyhow::Result<Option<String>> {
            Ok(self.open_id.clone())
        }

        async fn get_user_membership_tier(&self, _user_id: Uuid) -> anyhow::Result<Option<String>> {
            Ok(self.membership_tier.clone())
        }

        async fn update_user_membership_tier(
            &self,
            _user_id: Uuid,
            _tier: &str,
        ) -> anyhow::Result<()> {
            unreachable!()
        }

        async fn is_seat_swap_notice_enabled(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<bool>> {
            Ok(Some(false))
        }
    }

    #[derive(Default)]
    struct FakeOrderRepository {
        created_orders: Mutex<Vec<NewPaymentOrder>>,
    }

    #[async_trait]
    impl OrderRepository for FakeOrderRepository {
        async fn create_order(&self, order: NewPaymentOrder) -> anyhow::Result<PaymentOrder> {
            self.created_orders
                .lock()
                .expect("created orders")
                .push(order.clone());

            Ok(PaymentOrder {
                id: Uuid::new_v4(),
                order_no: order.order_no,
                user_id: order.user_id,
                amount_cents: order.amount_cents,
                status: OrderStatus::Pending,
                prepay_id: None,
                transaction_id: None,
                product_type: order.product_type,
                paid_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        async fn find_order_by_no(&self, _order_no: &str) -> anyhow::Result<Option<PaymentOrder>> {
            unreachable!()
        }
    }

    #[derive(Default)]
    struct FakeWechatPayPort {
        amounts: Mutex<Vec<i32>>,
    }

    #[async_trait]
    impl WechatPayPort for FakeWechatPayPort {
        async fn unified_order(
            &self,
            _order_no: &str,
            _description: &str,
            amount_cents: i32,
            _openid: &str,
        ) -> anyhow::Result<WxPayParams> {
            self.amounts
                .lock()
                .expect("wechat amounts")
                .push(amount_cents);
            Ok(WxPayParams {
                virtual_payment: None,
                time_stamp: "1".to_string(),
                nonce_str: "nonce".to_string(),
                package: "prepay_id=test".to_string(),
                sign_type: "MD5".to_string(),
                pay_sign: "sign".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn execute_rejects_target_tier_that_would_not_upgrade_user() {
        let repository = Arc::new(FakeOrderRepository::default());
        let use_case = CreateMembershipOrderUseCase::new(
            repository.clone(),
            Arc::new(FakeUserMembershipPort {
                open_id: Some("openid".to_string()),
                membership_tier: Some("V8".to_string()),
            }),
            Arc::new(FakeWechatPayPort::default()),
        );

        let result = use_case
            .execute(CreateMembershipOrderInput {
                user_id: Uuid::new_v4(),
                target_tier: "V6".to_string(),
                products: default_membership_product_options(),
            })
            .await;

        let error = match result {
            Ok(_) => panic!("lower tier should be rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("高于当前等级"));
        assert_eq!(
            repository
                .created_orders
                .lock()
                .expect("created orders")
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn execute_creates_order_when_target_tier_is_higher() {
        let repository = Arc::new(FakeOrderRepository::default());
        let wechat_pay_port = Arc::new(FakeWechatPayPort::default());
        let use_case = CreateMembershipOrderUseCase::new(
            repository.clone(),
            Arc::new(FakeUserMembershipPort {
                open_id: Some("openid".to_string()),
                membership_tier: Some("V6".to_string()),
            }),
            wechat_pay_port.clone(),
        );

        use_case
            .execute(CreateMembershipOrderInput {
                user_id: Uuid::new_v4(),
                target_tier: "V7".to_string(),
                products: default_membership_product_options(),
            })
            .await
            .expect("higher tier order should be created");

        let created_orders = repository.created_orders.lock().expect("created orders");
        assert_eq!(created_orders.len(), 1);
        assert_eq!(created_orders[0].amount_cents, 3500);
        assert_eq!(created_orders[0].product_type, "membership:V7");
        assert_eq!(
            *wechat_pay_port.amounts.lock().expect("wechat amounts"),
            vec![3500]
        );
    }

    #[tokio::test]
    async fn execute_creates_order_when_v9_user_renews_v9() {
        let repository = Arc::new(FakeOrderRepository::default());
        let wechat_pay_port = Arc::new(FakeWechatPayPort::default());
        let use_case = CreateMembershipOrderUseCase::new(
            repository.clone(),
            Arc::new(FakeUserMembershipPort {
                open_id: Some("openid".to_string()),
                membership_tier: Some("V9".to_string()),
            }),
            wechat_pay_port.clone(),
        );

        use_case
            .execute(CreateMembershipOrderInput {
                user_id: Uuid::new_v4(),
                target_tier: "V9".to_string(),
                products: default_membership_product_options(),
            })
            .await
            .expect("V9 renewal order should be created");

        let created_orders = repository.created_orders.lock().expect("created orders");
        assert_eq!(created_orders.len(), 1);
        assert_eq!(created_orders[0].amount_cents, 9900);
        assert_eq!(created_orders[0].product_type, "membership:V9");
        assert_eq!(
            *wechat_pay_port.amounts.lock().expect("wechat amounts"),
            vec![9900]
        );
    }
    #[derive(Default)]
    struct EventTestRecorder(Mutex<Vec<crate::payment::domain::payment_event::NewPaymentEvent>>);
    #[async_trait]
    impl crate::payment::ports::payment_event_recorder::PaymentEventRecorder for EventTestRecorder {
        async fn record(&self, event: crate::payment::domain::payment_event::NewPaymentEvent) {
            self.0.lock().unwrap().push(event);
        }
    }
    struct EventTestPay {
        fail: bool,
    }
    #[async_trait]
    impl WechatPayPort for EventTestPay {
        async fn unified_order(
            &self,
            _: &str,
            _: &str,
            _: i32,
            _: &str,
        ) -> anyhow::Result<WxPayParams> {
            if self.fail {
                return Err(
                    crate::payment::domain::virtual_payment::VirtualPaymentError::Provider.into(),
                );
            }
            Ok(WxPayParams {
                virtual_payment: None,
                time_stamp: "1".into(),
                nonce_str: "n".into(),
                package: "p".into(),
                sign_type: "MD5".into(),
                pay_sign: "s".into(),
            })
        }
    }
    #[tokio::test]
    async fn records_success_and_preserves_parameter_failure_error() {
        for fail in [false, true] {
            let recorder = Arc::new(EventTestRecorder::default());
            let use_case = CreateMembershipOrderUseCase::new(
                Arc::new(FakeOrderRepository::default()),
                Arc::new(FakeUserMembershipPort {
                    open_id: Some("openid".into()),
                    membership_tier: Some("V6".into()),
                }),
                Arc::new(EventTestPay { fail }),
            )
            .with_event_recorder(recorder.clone());
            let attempt_id = Uuid::new_v4();
            let result = use_case
                .execute_with_attempt(
                    CreateMembershipOrderInput {
                        user_id: Uuid::new_v4(),
                        target_tier: "V7".into(),
                        products: default_membership_product_options(),
                    },
                    Some(attempt_id),
                )
                .await;
            if fail {
                assert!(matches!(
                    result.err().unwrap().downcast_ref(),
                    Some(crate::payment::domain::virtual_payment::VirtualPaymentError::Provider)
                ));
            } else {
                assert!(result.is_ok());
            }
            for _ in 0..20 {
                tokio::task::yield_now().await;
            }
            let events = recorder.0.lock().unwrap();
            assert_eq!(
                events.iter().map(|v| v.event.as_str()).collect::<Vec<_>>(),
                [
                    "order_created",
                    if fail {
                        "pay_params_failed"
                    } else {
                        "pay_params_issued"
                    }
                ]
            );
            assert!(events.iter().all(|v| v.attempt_id == attempt_id
                && v.scenario == "membership"
                && v.product_ref.as_deref() == Some("V7")));
            assert_eq!(events[0].order_no, events[1].order_no);
        }
    }
}
