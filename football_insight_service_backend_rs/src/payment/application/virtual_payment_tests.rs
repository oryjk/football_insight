use super::*;
use crate::payment::{
    domain::{
        order::{NewPaymentOrder, PaymentOrder},
        payment_event::{NewPaymentEvent, PaymentEventKind},
        virtual_payment::{
            PrivateMembershipProduct, VirtualOrder, VirtualPayParams, VirtualProduct,
        },
    },
    ports::{
        payment_event_recorder::PaymentEventRecorder,
        payment_settlement_port::PaymentSettlementPort,
    },
};
use async_trait::async_trait;
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

struct FakePorts {
    base: Mutex<PaymentOrder>,
    virtual_order: VirtualOrder,
    fail_refund: bool,
    fail_settlement: bool,
    refunds: AtomicUsize,
    lookups: AtomicUsize,
}

impl FakePorts {
    fn new() -> Self {
        Self {
            base: Mutex::new(PaymentOrder {
                id: Uuid::new_v4(),
                order_no: "OBSERVABILITY_UNIT_ONLY".into(),
                user_id: Uuid::new_v4(),
                amount_cents: 500,
                status: OrderStatus::Pending,
                prepay_id: None,
                transaction_id: None,
                product_type: "membership:V7".into(),
                paid_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }),
            virtual_order: VirtualOrder {
                catalog_product_id: None,
                order_no: "OBSERVABILITY_UNIT_ONLY".into(),
                open_id: "unit_open_id".into(),
                product_id: "unit_product".into(),
                amount_cents: 500,
                membership_duration_months: 1,
                wx_order_id: Some("unit_wx_order".into()),
            },
            fail_refund: false,
            fail_settlement: false,
            refunds: AtomicUsize::new(0),
            lookups: AtomicUsize::new(0),
        }
    }
    fn refund(&self) -> Refund {
        Refund {
            order_no: self.virtual_order.order_no.clone(),
            open_id: self.virtual_order.open_id.clone(),
            wx_order_id: "unit_wx_order".into(),
            refund_id: "unit_refund".into(),
            amount_cents: 500,
        }
    }
    fn delivery(&self) -> Delivery {
        Delivery {
            order_no: self.virtual_order.order_no.clone(),
            open_id: self.virtual_order.open_id.clone(),
            product_id: self.virtual_order.product_id.clone(),
            amount_cents: 500,
            original_price_cents: 500,
            quantity: 1,
            env: 0,
            wx_order_id: "unit_wx_order".into(),
        }
    }
}

#[async_trait]
impl OrderRepository for FakePorts {
    async fn create_order(&self, _: NewPaymentOrder) -> anyhow::Result<PaymentOrder> {
        unreachable!()
    }
    async fn find_order_by_no(&self, _: &str) -> anyhow::Result<Option<PaymentOrder>> {
        self.lookups.fetch_add(1, Ordering::SeqCst);
        Ok(Some(self.base.lock().unwrap().clone()))
    }
}

#[async_trait]
impl VirtualOrderRepository for FakePorts {
    async fn record(&self, _: VirtualOrder) -> anyhow::Result<()> {
        unreachable!()
    }
    async fn find(&self, _: &str) -> anyhow::Result<Option<VirtualOrder>> {
        Ok(Some(self.virtual_order.clone()))
    }
    async fn record_delivery(&self, _: &Delivery) -> anyhow::Result<()> {
        Ok(())
    }
    async fn acknowledge_delivery(&self, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn refund(&self, _: &Refund) -> anyhow::Result<()> {
        if self.fail_refund {
            anyhow::bail!("original refund failure")
        }
        self.refunds.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn pending_orders(&self) -> anyhow::Result<Vec<VirtualOrder>> {
        unreachable!()
    }
}

#[async_trait]
impl PaymentSettlementPort for FakePorts {
    async fn settle_membership_order(
        &self,
        _: &str,
        _: &str,
        _: Uuid,
        _: &str,
    ) -> anyhow::Result<()> {
        if self.fail_settlement {
            anyhow::bail!("original settlement failure")
        }
        self.base.lock().unwrap().status = OrderStatus::Paid;
        Ok(())
    }
    async fn settle_reflux_subscription_order(
        &self,
        _: &str,
        _: &str,
        _: Uuid,
        _: crate::payment::domain::order::RefluxSubscriptionProductType,
    ) -> anyhow::Result<()> {
        unreachable!()
    }
    async fn settle_match_id_unlock_order(
        &self,
        _: &str,
        _: &str,
        _: Uuid,
        _: i64,
    ) -> anyhow::Result<()> {
        unreachable!()
    }
}

#[async_trait]
impl VirtualPaymentGateway for FakePorts {
    fn available(&self) -> bool {
        true
    }
    async fn prepare_session(&self, _: &str, _: &str) -> Result<(), VirtualPaymentError> {
        unreachable!()
    }
    async fn payment_params(
        &self,
        _: &str,
        _: &str,
        _: &VirtualProduct,
    ) -> Result<VirtualPayParams, VirtualPaymentError> {
        unreachable!()
    }
    async fn query_order(&self, _: &VirtualOrder) -> Result<Option<Delivery>, VirtualPaymentError> {
        unreachable!()
    }
    async fn confirm_delivery(&self, _: &VirtualOrder) -> Result<(), VirtualPaymentError> {
        unreachable!()
    }
}

#[async_trait]
impl VirtualProductRepository for FakePorts {
    async fn find_enabled(
        &self,
        _: &str,
        _: Uuid,
    ) -> Result<Option<VirtualProduct>, VirtualPaymentError> {
        unreachable!()
    }
    async fn has_enabled(&self) -> Result<bool, VirtualPaymentError> {
        unreachable!()
    }
    async fn private_membership_products(
        &self,
        _: Uuid,
    ) -> Result<Vec<PrivateMembershipProduct>, VirtualPaymentError> {
        unreachable!()
    }
}

#[async_trait]
impl UserMembershipPort for FakePorts {
    async fn get_user_open_id(&self, _: Uuid) -> anyhow::Result<Option<String>> {
        unreachable!()
    }
    async fn get_user_membership_tier(&self, _: Uuid) -> anyhow::Result<Option<String>> {
        unreachable!()
    }
    async fn update_user_membership_tier(&self, _: Uuid, _: &str) -> anyhow::Result<()> {
        unreachable!()
    }
    async fn is_seat_swap_notice_enabled(&self, _: Uuid) -> anyhow::Result<Option<bool>> {
        unreachable!()
    }
}

struct Recorder(tokio::sync::mpsc::UnboundedSender<NewPaymentEvent>);
#[async_trait]
impl PaymentEventRecorder for Recorder {
    async fn record(&self, event: NewPaymentEvent) {
        self.0.send(event).unwrap();
    }
}

fn service(
    ports: Arc<FakePorts>,
    recorder: Arc<dyn PaymentEventRecorder>,
) -> VirtualPaymentService {
    let settlement = Arc::new(
        HandleWechatNotifyUseCase::new(ports.clone(), ports.clone())
            .with_event_recorder(recorder.clone()),
    );
    VirtualPaymentService::new(
        ports.clone(),
        ports.clone(),
        ports.clone(),
        ports.clone(),
        settlement,
        ports,
    )
    .with_event_recorder(recorder)
}

#[tokio::test]
async fn refund_records_after_success_using_dedicated_metadata_repository() {
    let ports = Arc::new(FakePorts::new());
    let metadata = Arc::new(FakePorts::new());
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let service = service(ports.clone(), Arc::new(Recorder(sender)))
        .with_event_order_repository(metadata.clone());
    service.refund(&ports.refund()).await.unwrap();
    assert_eq!(ports.refunds.load(Ordering::SeqCst), 1);
    let event = tokio::time::timeout(std::time::Duration::from_secs(1), receiver.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event.event, PaymentEventKind::Refunded);
    assert_eq!(event.order_no.as_deref(), Some("OBSERVABILITY_UNIT_ONLY"));
    assert_eq!(ports.lookups.load(Ordering::SeqCst), 0);
    assert_eq!(metadata.lookups.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn refund_failure_preserves_error_and_emits_no_event() {
    let mut ports = FakePorts::new();
    ports.fail_refund = true;
    let ports = Arc::new(ports);
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let service = service(ports.clone(), Arc::new(Recorder(sender)));
    assert_eq!(
        service
            .refund(&ports.refund())
            .await
            .unwrap_err()
            .to_string(),
        "original refund failure"
    );
    tokio::task::yield_now().await;
    assert!(receiver.try_recv().is_err());
}

#[tokio::test]
async fn virtual_delivery_records_once_and_ignores_paid_retry() {
    let ports = Arc::new(FakePorts::new());
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let service = service(ports.clone(), Arc::new(Recorder(sender)));
    service.deliver(&ports.delivery()).await.unwrap();
    let event = tokio::time::timeout(std::time::Duration::from_secs(1), receiver.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event.event, PaymentEventKind::Delivered);
    service.deliver(&ports.delivery()).await.unwrap();
    tokio::task::yield_now().await;
    assert!(
        receiver.try_recv().is_err(),
        "virtual wrapper and paid retry must not duplicate delivery"
    );
}

#[tokio::test]
async fn virtual_delivery_settlement_failure_preserves_error_and_emits_no_event() {
    let mut ports = FakePorts::new();
    ports.fail_settlement = true;
    let ports = Arc::new(ports);
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let service = service(ports.clone(), Arc::new(Recorder(sender)));
    assert_eq!(
        service
            .deliver(&ports.delivery())
            .await
            .unwrap_err()
            .to_string(),
        "original settlement failure"
    );
    tokio::task::yield_now().await;
    assert!(receiver.try_recv().is_err());
}

struct BlockedRecorder;
#[async_trait]
impl PaymentEventRecorder for BlockedRecorder {
    async fn record(&self, _: NewPaymentEvent) {
        std::future::pending::<()>().await;
    }
}

#[tokio::test]
async fn blocked_recorder_cannot_delay_successful_refund() {
    let ports = Arc::new(FakePorts::new());
    let service = service(ports.clone(), Arc::new(BlockedRecorder));
    tokio::time::timeout(
        std::time::Duration::from_millis(50),
        service.refund(&ports.refund()),
    )
    .await
    .expect("recorder cannot block refund")
    .unwrap();
}
