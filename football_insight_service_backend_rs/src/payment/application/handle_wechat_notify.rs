use std::{collections::HashMap, sync::Arc};

use crate::payment::{
    domain::order::membership_tier_from_product_type,
    ports::{order_repository::OrderRepository, payment_settlement_port::PaymentSettlementPort},
};

pub struct HandleWechatNotifyUseCase {
    order_repository: Arc<dyn OrderRepository>,
    payment_settlement_port: Arc<dyn PaymentSettlementPort>,
}

impl HandleWechatNotifyUseCase {
    pub fn new(
        order_repository: Arc<dyn OrderRepository>,
        payment_settlement_port: Arc<dyn PaymentSettlementPort>,
    ) -> Self {
        Self {
            order_repository,
            payment_settlement_port,
        }
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

        if order.status == crate::payment::domain::order::OrderStatus::Paid {
            return Ok(NotifyHandleResult::Success);
        }

        let target_tier = membership_tier_from_product_type(&order.product_type);
        self.payment_settlement_port
            .settle_membership_order(&out_trade_no, &transaction_id, order.user_id, &target_tier)
            .await?;

        Ok(NotifyHandleResult::Success)
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
    }

    #[derive(Default)]
    struct CountingPaymentSettlementPort {
        calls: Mutex<usize>,
        tiers: Mutex<Vec<String>>,
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
}
