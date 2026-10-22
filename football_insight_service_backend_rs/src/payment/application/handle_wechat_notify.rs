use std::{collections::HashMap, sync::Arc};

use crate::payment::{
    application::payment_events::PaymentEvents,
    domain::order::{
        membership_tier_from_product_type, parse_match_id_unlock_product_type,
        parse_reflux_subscription_product_type,
    },
    domain::payment_event::PaymentEventKind,
    ports::{
        order_repository::OrderRepository, payment_event_recorder::PaymentEventRecorder,
        payment_settlement_port::PaymentSettlementPort,
    },
};

pub struct HandleWechatNotifyUseCase {
    order_repository: Arc<dyn OrderRepository>,
    payment_settlement_port: Arc<dyn PaymentSettlementPort>,
    events: PaymentEvents,
}

impl HandleWechatNotifyUseCase {
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        payment_settlement_port: Arc<dyn PaymentSettlementPort>,
    ) -> Self {
        Self {
            order_repository,
            payment_settlement_port,
            events: PaymentEvents::default(),
        }
    }

    pub fn with_event_recorder(mut self, recorder: Arc<dyn PaymentEventRecorder>) -> Self {
        self.events = PaymentEvents::new(recorder);
        self
    }

    pub async fn execute(
        &self,
        notify_data: HashMap<String, String>,
    ) -> anyhow::Result<NotifyHandleResult> {
        let return_code = notify_data.get("return_code").cloned().unwrap_or_default();
        let result_code = notify_data.get("result_code").cloned().unwrap_or_default();

        if return_code != "SUCCESS" || result_code != "SUCCESS" {
            return Ok(NotifyHandleResult::ProtocolFailure);
        }

        let out_trade_no = notify_data.get("out_trade_no").cloned().unwrap_or_default();
        let transaction_id = notify_data
            .get("transaction_id")
            .cloned()
            .unwrap_or_default();
        let total_fee = notify_data.get("total_fee").cloned().unwrap_or_default();

        if self
            .order_repository
            .is_virtual_order(&out_trade_no)
            .await?
        {
            return Ok(NotifyHandleResult::ProtocolFailure);
        }

        let order = self
            .order_repository
            .find_order_by_no(&out_trade_no)
            .await?;

        let Some(order) = order else {
            return Ok(NotifyHandleResult::OrderNotFound);
        };

        let total_fee: i32 = total_fee.parse().unwrap_or(0);
        if total_fee != order.amount_cents {
            return Ok(NotifyHandleResult::AmountMismatch);
        }

        self.settle_verified_order(&order, &transaction_id).await?;
        Ok(NotifyHandleResult::Success)
    }

    pub async fn settle_verified_order(
        &self,
        order: &crate::payment::domain::order::PaymentOrder,
        transaction_id: &str,
    ) -> anyhow::Result<()> {
        if order.status == crate::payment::domain::order::OrderStatus::Paid {
            return Ok(());
        }

        let out_trade_no = &order.order_no;

        if let Some(code) =
            crate::student::domain::ServiceCode::from_product_type(&order.product_type)
        {
            if order.amount_cents != code.price() {
                anyhow::bail!("invalid student service price");
            }
            self.payment_settlement_port
                .settle_student_service_order(out_trade_no, transaction_id, order.user_id, code)
                .await?;
        } else if let Some(product) = parse_reflux_subscription_product_type(&order.product_type) {
            self.payment_settlement_port
                .settle_reflux_subscription_order(
                    out_trade_no,
                    transaction_id,
                    order.user_id,
                    product,
                )
                .await?;
        } else if let Some(match_id) = parse_match_id_unlock_product_type(&order.product_type) {
            self.payment_settlement_port
                .settle_match_id_unlock_order(out_trade_no, transaction_id, order.user_id, match_id)
                .await?;
        } else {
            let target_tier = membership_tier_from_product_type(&order.product_type)?;
            self.payment_settlement_port
                .settle_membership_order(out_trade_no, transaction_id, order.user_id, &target_tier)
                .await?;
        }

        // Both legacy callbacks and virtual delivery converge here. Record once only
        // after settlement succeeds; already-paid retries return above without an event.
        self.events
            .record_order(order.clone(), PaymentEventKind::Delivered);
        Ok(())
    }
}

pub enum NotifyHandleResult {
    Success,
    ProtocolFailure,
    OrderNotFound,
    AmountMismatch,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    use super::{HandleWechatNotifyUseCase, NotifyHandleResult};
    use crate::payment::{
        domain::order::{OrderStatus, PaymentOrder},
        ports::{
            order_repository::OrderRepository, payment_settlement_port::PaymentSettlementPort,
        },
    };

    struct FakeOrderRepository {
        order: Mutex<Option<PaymentOrder>>,
    }

    #[async_trait]
    impl OrderRepository for FakeOrderRepository {
        async fn create_order(
            &self,
            _order: crate::payment::domain::order::NewPaymentOrder,
        ) -> anyhow::Result<PaymentOrder> {
            unreachable!()
        }

        async fn find_order_by_no(&self, _order_no: &str) -> anyhow::Result<Option<PaymentOrder>> {
            Ok(self.order.lock().expect("order").clone())
        }
    }

    struct FailingPaymentSettlementPort;

    #[async_trait]
    impl PaymentSettlementPort for FailingPaymentSettlementPort {
        async fn settle_membership_order(
            &self,
            _order_no: &str,
            _transaction_id: &str,
            _user_id: Uuid,
            _tier: &str,
        ) -> anyhow::Result<()> {
            anyhow::bail!("payment settlement failed")
        }

        async fn settle_reflux_subscription_order(
            &self,
            _order_no: &str,
            _transaction_id: &str,
            _user_id: Uuid,
            _product: crate::payment::domain::order::RefluxSubscriptionProductType,
        ) -> anyhow::Result<()> {
            anyhow::bail!("payment settlement failed")
        }

        async fn settle_match_id_unlock_order(
            &self,
            _order_no: &str,
            _transaction_id: &str,
            _user_id: Uuid,
            _match_id: i64,
        ) -> anyhow::Result<()> {
            anyhow::bail!("payment settlement failed")
        }
    }

    #[derive(Default)]
    struct CountingPaymentSettlementPort {
        calls: Mutex<usize>,
        tiers: Mutex<Vec<String>>,
        reflux_products: Mutex<Vec<crate::payment::domain::order::RefluxSubscriptionProductType>>,
        unlocked_match_ids: Mutex<Vec<i64>>,
    }

    #[async_trait]
    impl PaymentSettlementPort for CountingPaymentSettlementPort {
        async fn settle_membership_order(
            &self,
            _order_no: &str,
            _transaction_id: &str,
            _user_id: Uuid,
            tier: &str,
        ) -> anyhow::Result<()> {
            *self.calls.lock().expect("calls") += 1;
            self.tiers.lock().expect("tiers").push(tier.to_string());
            Ok(())
        }

        async fn settle_reflux_subscription_order(
            &self,
            _order_no: &str,
            _transaction_id: &str,
            _user_id: Uuid,
            product: crate::payment::domain::order::RefluxSubscriptionProductType,
        ) -> anyhow::Result<()> {
            *self.calls.lock().expect("calls") += 1;
            self.reflux_products
                .lock()
                .expect("reflux products")
                .push(product);
            Ok(())
        }

        async fn settle_match_id_unlock_order(
            &self,
            _order_no: &str,
            _transaction_id: &str,
            _user_id: Uuid,
            match_id: i64,
        ) -> anyhow::Result<()> {
            *self.calls.lock().expect("calls") += 1;
            self.unlocked_match_ids
                .lock()
                .expect("unlocked match ids")
                .push(match_id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn execute_does_not_leave_paid_order_when_membership_grant_fails() {
        let user_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
        let repository = Arc::new(FakeOrderRepository {
            order: Mutex::new(Some(PaymentOrder {
                id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                order_no: "202604230001".to_string(),
                user_id,
                amount_cents: 4900,
                status: OrderStatus::Pending,
                prepay_id: None,
                transaction_id: None,
                product_type: "membership".to_string(),
                paid_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })),
        });
        let use_case = HandleWechatNotifyUseCase::new(
            repository.clone(),
            Arc::new(FailingPaymentSettlementPort),
        );

        let result = use_case
            .execute(
                [
                    ("return_code".to_string(), "SUCCESS".to_string()),
                    ("result_code".to_string(), "SUCCESS".to_string()),
                    ("out_trade_no".to_string(), "202604230001".to_string()),
                    ("transaction_id".to_string(), "wx_txn_1".to_string()),
                    ("total_fee".to_string(), "4900".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .await;

        let error = match result {
            Ok(_) => panic!("settlement failure should surface"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("payment settlement failed"));
        assert_eq!(
            repository
                .order
                .lock()
                .expect("order")
                .as_ref()
                .expect("existing order")
                .status,
            OrderStatus::Pending
        );
    }

    #[tokio::test]
    async fn execute_skips_settlement_when_order_is_already_paid() {
        let user_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
        let repository = Arc::new(FakeOrderRepository {
            order: Mutex::new(Some(PaymentOrder {
                id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                order_no: "202604230002".to_string(),
                user_id,
                amount_cents: 4900,
                status: OrderStatus::Paid,
                prepay_id: None,
                transaction_id: Some("wx_txn_existing".to_string()),
                product_type: "membership".to_string(),
                paid_at: Some(Utc::now()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })),
        });
        let settlement_port = Arc::new(CountingPaymentSettlementPort::default());
        let use_case = HandleWechatNotifyUseCase::new(repository, settlement_port.clone());

        let result = use_case
            .execute(
                [
                    ("return_code".to_string(), "SUCCESS".to_string()),
                    ("result_code".to_string(), "SUCCESS".to_string()),
                    ("out_trade_no".to_string(), "202604230002".to_string()),
                    ("transaction_id".to_string(), "wx_txn_existing".to_string()),
                    ("total_fee".to_string(), "4900".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .await
            .expect("duplicate paid notify should be treated as success");

        assert!(matches!(result, NotifyHandleResult::Success));
        assert_eq!(*settlement_port.calls.lock().expect("calls"), 0);
    }

    #[tokio::test]
    async fn execute_settles_membership_order_to_target_tier_from_product_type() {
        let user_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
        let repository = Arc::new(FakeOrderRepository {
            order: Mutex::new(Some(PaymentOrder {
                id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                order_no: "202604240001".to_string(),
                user_id,
                amount_cents: 5900,
                status: OrderStatus::Pending,
                prepay_id: None,
                transaction_id: None,
                product_type: "membership:V7".to_string(),
                paid_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })),
        });
        let settlement_port = Arc::new(CountingPaymentSettlementPort::default());
        let use_case = HandleWechatNotifyUseCase::new(repository, settlement_port.clone());

        let result = use_case
            .execute(
                [
                    ("return_code".to_string(), "SUCCESS".to_string()),
                    ("result_code".to_string(), "SUCCESS".to_string()),
                    ("out_trade_no".to_string(), "202604240001".to_string()),
                    ("transaction_id".to_string(), "wx_txn_v7".to_string()),
                    ("total_fee".to_string(), "5900".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .await
            .expect("v7 notify should settle");

        assert!(matches!(result, NotifyHandleResult::Success));
        assert_eq!(*settlement_port.calls.lock().expect("calls"), 1);
        assert_eq!(
            settlement_port.tiers.lock().expect("tiers").as_slice(),
            ["V7"]
        );
    }

    #[tokio::test]
    async fn execute_dispatches_reflux_subscription_order() {
        let user_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
        let repository = Arc::new(FakeOrderRepository {
            order: Mutex::new(Some(PaymentOrder {
                id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                order_no: "202605180001".to_string(),
                user_id,
                amount_cents: 500,
                status: OrderStatus::Pending,
                prepay_id: None,
                transaction_id: None,
                product_type: "reflux_subscription:single_match:chengdu:571".to_string(),
                paid_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })),
        });
        let settlement_port = Arc::new(CountingPaymentSettlementPort::default());
        let use_case = HandleWechatNotifyUseCase::new(repository, settlement_port.clone());

        let result = use_case
            .execute(
                [
                    ("return_code".to_string(), "SUCCESS".to_string()),
                    ("result_code".to_string(), "SUCCESS".to_string()),
                    ("out_trade_no".to_string(), "202605180001".to_string()),
                    ("transaction_id".to_string(), "wx_txn_reflux".to_string()),
                    ("total_fee".to_string(), "500".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .await
            .expect("reflux notify should settle");

        assert!(matches!(result, NotifyHandleResult::Success));
        assert_eq!(*settlement_port.calls.lock().expect("calls"), 1);
        assert_eq!(settlement_port.tiers.lock().expect("tiers").len(), 0);
        let products = settlement_port
            .reflux_products
            .lock()
            .expect("reflux products");
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].plan_code, "single_match");
        assert_eq!(products[0].team_code, "chengdu");
        assert_eq!(products[0].match_id, Some(571));
    }

    #[tokio::test]
    async fn execute_dispatches_match_id_unlock_order() {
        let user_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
        let repository = Arc::new(FakeOrderRepository {
            order: Mutex::new(Some(PaymentOrder {
                id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                order_no: "20260826140001".to_string(),
                user_id,
                amount_cents: 500,
                status: OrderStatus::Pending,
                prepay_id: None,
                transaction_id: None,
                product_type: "match_id_unlock:571".to_string(),
                paid_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })),
        });
        let settlement_port = Arc::new(CountingPaymentSettlementPort::default());
        let use_case = HandleWechatNotifyUseCase::new(repository, settlement_port.clone());

        let result = use_case
            .execute(
                [
                    ("return_code".to_string(), "SUCCESS".to_string()),
                    ("result_code".to_string(), "SUCCESS".to_string()),
                    ("out_trade_no".to_string(), "20260826140001".to_string()),
                    ("transaction_id".to_string(), "wx_txn_match_id".to_string()),
                    ("total_fee".to_string(), "500".to_string()),
                ]
                .into_iter()
                .collect(),
            )
            .await
            .expect("match id unlock notify should settle");

        assert!(matches!(result, NotifyHandleResult::Success));
        assert_eq!(*settlement_port.calls.lock().expect("calls"), 1);
        assert_eq!(
            settlement_port
                .unlocked_match_ids
                .lock()
                .expect("unlocked match ids")
                .as_slice(),
            [571]
        );
    }

    struct RecordingEvents(
        tokio::sync::mpsc::UnboundedSender<crate::payment::domain::payment_event::NewPaymentEvent>,
    );

    #[async_trait]
    impl crate::payment::ports::payment_event_recorder::PaymentEventRecorder for RecordingEvents {
        async fn record(&self, event: crate::payment::domain::payment_event::NewPaymentEvent) {
            self.0.send(event).expect("event receiver");
        }
    }

    fn observable_order(product: &str, status: OrderStatus) -> PaymentOrder {
        PaymentOrder {
            id: Uuid::new_v4(),
            order_no: "OBSERVABILITY_UNIT_ONLY".into(),
            user_id: Uuid::new_v4(),
            amount_cents: 500,
            status,
            prepay_id: None,
            transaction_id: None,
            product_type: product.into(),
            paid_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn successful_settlement_records_delivered_for_every_product() {
        use crate::payment::domain::payment_event::PaymentEventKind;
        for (product, scenario) in [
            ("membership:V7", "membership"),
            ("match_id_unlock:571", "match_id_unlock"),
            (
                "reflux_subscription:single_match:chengdu:571",
                "reflux_subscription",
            ),
        ] {
            let order = observable_order(product, OrderStatus::Pending);
            let repository = Arc::new(FakeOrderRepository {
                order: Mutex::new(Some(order.clone())),
            });
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
            let use_case = HandleWechatNotifyUseCase::new(
                repository,
                Arc::new(CountingPaymentSettlementPort::default()),
            )
            .with_event_recorder(Arc::new(RecordingEvents(sender)));
            use_case
                .settle_verified_order(&order, "fake_tx")
                .await
                .unwrap();
            let event = tokio::time::timeout(std::time::Duration::from_secs(1), receiver.recv())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(event.event, PaymentEventKind::Delivered);
            assert_eq!(event.scenario, scenario);
            assert_eq!(event.user_id, order.user_id);
            assert_eq!(event.order_no.as_deref(), Some(order.order_no.as_str()));
            assert!(receiver.try_recv().is_err());
        }
    }

    #[tokio::test]
    async fn failed_or_already_paid_settlement_emits_no_delivered() {
        for status in [OrderStatus::Pending, OrderStatus::Paid] {
            let order = observable_order("membership:V7", status);
            let repository = Arc::new(FakeOrderRepository {
                order: Mutex::new(Some(order.clone())),
            });
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
            let use_case =
                HandleWechatNotifyUseCase::new(repository, Arc::new(FailingPaymentSettlementPort))
                    .with_event_recorder(Arc::new(RecordingEvents(sender)));
            let result = use_case.settle_verified_order(&order, "fake_tx").await;
            if status == OrderStatus::Pending {
                assert_eq!(result.unwrap_err().to_string(), "payment settlement failed");
            } else {
                result.unwrap();
            }
            tokio::task::yield_now().await;
            assert!(receiver.try_recv().is_err());
        }
    }

    struct BlockedEvents;

    #[async_trait]
    impl crate::payment::ports::payment_event_recorder::PaymentEventRecorder for BlockedEvents {
        async fn record(&self, _: crate::payment::domain::payment_event::NewPaymentEvent) {
            std::future::pending::<()>().await;
        }
    }

    #[tokio::test]
    async fn blocked_recorder_does_not_delay_settlement() {
        let order = observable_order("membership:V7", OrderStatus::Pending);
        let repository = Arc::new(FakeOrderRepository {
            order: Mutex::new(Some(order.clone())),
        });
        let use_case = HandleWechatNotifyUseCase::new(
            repository,
            Arc::new(CountingPaymentSettlementPort::default()),
        )
        .with_event_recorder(Arc::new(BlockedEvents));
        tokio::time::timeout(
            std::time::Duration::from_millis(50),
            use_case.settle_verified_order(&order, "fake_tx"),
        )
        .await
        .expect("recorder cannot block settlement")
        .unwrap();
    }
}
