use async_trait::async_trait;
use chrono::Utc;
use football_insight_service_backend_rs::{
    auth::ports::user_membership_port::UserMembershipPort,
    payment::{
        domain::{
            order::{NewPaymentOrder, OrderStatus, PaymentOrder, WxPayParams},
            virtual_payment::VirtualPayParams,
        },
        ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
    },
    student::{
        application::{StudentConfigService, StudentService},
        domain::{ServiceCode, StudentConfig, StudentError, StudentOrderPage},
        ports::StudentRepository,
    },
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

// Application tests replace ports; the PostgreSQL suite covers their real transaction effects.
struct Fixture {
    config: Mutex<StudentConfig>,
    saved: Mutex<Vec<(StudentConfig, Uuid)>>,
    created: Mutex<Vec<NewPaymentOrder>>,
    statuses: Mutex<HashMap<String, OrderStatus>>,
    open_id: Option<String>,
    virtual_params: bool,
    provider_error: bool,
}
impl Fixture {
    fn new(config: StudentConfig) -> Arc<Self> {
        Arc::new(Self {
            config: Mutex::new(config),
            saved: Mutex::new(vec![]),
            created: Mutex::new(vec![]),
            statuses: Mutex::new(HashMap::new()),
            open_id: Some("test-buyer".into()),
            virtual_params: true,
            provider_error: false,
        })
    }
    fn service(self: &Arc<Self>) -> StudentService {
        StudentService::new(self.clone(), self.clone(), self.clone(), self.clone())
    }
}
#[async_trait]
impl StudentRepository for Fixture {
    async fn config(&self) -> Result<StudentConfig, StudentError> {
        Ok(self.config.lock().unwrap().clone())
    }
    async fn save_config(
        &self,
        config: StudentConfig,
        admin: Uuid,
    ) -> Result<StudentConfig, StudentError> {
        self.saved.lock().unwrap().push((config.clone(), admin));
        Ok(config)
    }
    async fn orders(&self, _: Uuid, _: i64, _: i64) -> Result<StudentOrderPage, StudentError> {
        panic!("invalid pagination must not reach persistence")
    }
    async fn fail_unissued_order(&self, user: Uuid, order_no: &str) -> Result<(), StudentError> {
        let created = self.created.lock().unwrap();
        let order = created
            .iter()
            .find(|order| order.order_no == order_no)
            .expect("cannot fail an unknown order");
        assert_eq!(order.user_id, user);
        self.statuses
            .lock()
            .unwrap()
            .insert(order_no.into(), OrderStatus::Failed);
        Ok(())
    }
}
#[async_trait]
impl OrderRepository for Fixture {
    async fn create_order(&self, input: NewPaymentOrder) -> anyhow::Result<PaymentOrder> {
        self.created.lock().unwrap().push(input.clone());
        self.statuses
            .lock()
            .unwrap()
            .insert(input.order_no.clone(), OrderStatus::Pending);
        Ok(PaymentOrder {
            id: Uuid::new_v4(),
            order_no: input.order_no,
            user_id: input.user_id,
            amount_cents: input.amount_cents,
            product_type: input.product_type,
            status: OrderStatus::Pending,
            prepay_id: None,
            transaction_id: None,
            paid_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    async fn find_order_by_no(&self, _: &str) -> anyhow::Result<Option<PaymentOrder>> {
        unreachable!()
    }
}
#[async_trait]
impl UserMembershipPort for Fixture {
    async fn get_user_open_id(&self, _: Uuid) -> anyhow::Result<Option<String>> {
        Ok(self.open_id.clone())
    }
    async fn get_user_membership_tier(&self, _: Uuid) -> anyhow::Result<Option<String>> {
        panic!("student checkout does not depend on membership")
    }
    async fn update_user_membership_tier(&self, _: Uuid, _: &str) -> anyhow::Result<()> {
        panic!("student checkout must not grant membership")
    }
    async fn is_seat_swap_notice_enabled(&self, _: Uuid) -> anyhow::Result<Option<bool>> {
        unreachable!()
    }
}
#[async_trait]
impl WechatPayPort for Fixture {
    async fn unified_order(
        &self,
        order_no: &str,
        title: &str,
        price: i32,
        open_id: &str,
    ) -> anyhow::Result<WxPayParams> {
        let created = self.created.lock().unwrap();
        let input = created
            .iter()
            .find(|input| input.order_no == order_no)
            .expect("persist order before requesting payment");
        assert_eq!(input.amount_cents, price);
        assert_eq!(open_id, "test-buyer");
        assert!(matches!(
            (input.product_type.as_str(), title, price),
            ("student_service:certificate_renewal", "证书续期", 9900)
                | ("student_service:tool_upgrade", "工具升级", 69900)
        ));
        if self.provider_error {
            anyhow::bail!("test provider cannot issue payment parameters");
        }
        Ok(WxPayParams {
            virtual_payment: self.virtual_params.then(|| VirtualPayParams {
                mode: "short_series_goods".into(),
                sign_data: "test-sign-data".into(),
                pay_sig: "test-pay-signature".into(),
                signature: "test-session-signature".into(),
            }),
            time_stamp: String::new(),
            nonce_str: String::new(),
            package: String::new(),
            sign_type: String::new(),
            pay_sign: String::new(),
        })
    }
}
fn enabled_config() -> StudentConfig {
    let mut config = StudentConfig::default();
    config.contact_qr_url = "https://cdn.example.test/contact-v1.png".into();
    for service in &mut config.services {
        service.description = "付款后请联系负责人办理".into();
        service.product_id = format!("published-{}", service.code.as_str());
        service.enabled = true;
    }
    config
}

#[tokio::test]
async fn unconfigured_disabled_or_corrupt_config_never_creates_an_order() {
    let mut corrupt = enabled_config();
    corrupt.services[0].price_cents = 1;
    let mut incomplete = enabled_config();
    incomplete.services[0].product_id.clear();
    for config in [StudentConfig::default(), corrupt, incomplete] {
        let fixture = Fixture::new(config);
        assert!(
            fixture
                .service()
                .create_order(Uuid::new_v4(), ServiceCode::CertificateRenewal)
                .await
                .is_err()
        );
        assert!(fixture.created.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn both_checkouts_use_fixed_prices_and_the_current_buyer_without_reading_membership() {
    let fixture = Fixture::new(enabled_config());
    let buyer = Uuid::new_v4();
    for (code, price, product_type) in [
        (
            ServiceCode::CertificateRenewal,
            9900,
            "student_service:certificate_renewal",
        ),
        (
            ServiceCode::ToolUpgrade,
            69900,
            "student_service:tool_upgrade",
        ),
    ] {
        let checkout = fixture.service().create_order(buyer, code).await.unwrap();
        assert!(checkout.params.virtual_payment.is_some());
        assert!(checkout.order_no.starts_with('S'));
        assert_eq!(checkout.order_no.len(), 32);
        let orders = fixture.created.lock().unwrap();
        let actual = orders.last().unwrap();
        assert_eq!(actual.order_no, checkout.order_no);
        assert_eq!(actual.user_id, buyer);
        assert_eq!(actual.amount_cents, price);
        assert_eq!(actual.product_type, product_type);
    }
    let orders = fixture.created.lock().unwrap();
    assert_ne!(orders[0].order_no, orders[1].order_no);
}

#[tokio::test]
async fn password_only_account_cannot_create_student_payment_order() {
    let mut fixture = Fixture::new(enabled_config());
    Arc::get_mut(&mut fixture).unwrap().open_id = None;
    assert!(matches!(
        fixture
            .service()
            .create_order(Uuid::new_v4(), ServiceCode::ToolUpgrade)
            .await,
        Err(StudentError::WechatRequired)
    ));
    assert!(fixture.created.lock().unwrap().is_empty());
}

#[tokio::test]
async fn checkout_rejects_legacy_payment_parameters_instead_of_claiming_success() {
    let mut fixture = Fixture::new(enabled_config());
    Arc::get_mut(&mut fixture).unwrap().virtual_params = false;
    assert!(matches!(
        fixture
            .service()
            .create_order(Uuid::new_v4(), ServiceCode::ToolUpgrade)
            .await,
        Err(StudentError::PaymentFailed)
    ));
    let orders = fixture.created.lock().unwrap();
    assert_eq!(
        orders.len(),
        1,
        "legacy failure cannot fall back to a second checkout"
    );
    assert_eq!(
        fixture.statuses.lock().unwrap().get(&orders[0].order_no),
        Some(&OrderStatus::Failed)
    );
}

#[tokio::test]
async fn provider_parameter_failure_marks_the_own_unissued_order_failed_without_fallback() {
    let mut fixture = Fixture::new(enabled_config());
    Arc::get_mut(&mut fixture).unwrap().provider_error = true;
    let buyer = Uuid::new_v4();
    assert!(matches!(
        fixture
            .service()
            .create_order(buyer, ServiceCode::CertificateRenewal)
            .await,
        Err(StudentError::PaymentFailed)
    ));
    let orders = fixture.created.lock().unwrap();
    assert_eq!(
        orders.len(),
        1,
        "provider failure cannot issue an ordinary payment fallback"
    );
    assert_eq!(orders[0].user_id, buyer);
    assert_eq!(orders[0].amount_cents, 9900);
    assert_eq!(
        fixture.statuses.lock().unwrap().get(&orders[0].order_no),
        Some(&OrderStatus::Failed)
    );
}

#[tokio::test]
async fn config_validation_happens_before_repository_write_and_preserves_actor() {
    let fixture = Fixture::new(StudentConfig::default());
    let service = StudentConfigService::new(fixture.clone());
    let actor = Uuid::new_v4();
    let mut invalid_configs = vec![];
    let mut bad = enabled_config();
    bad.services[0].price_cents = 1;
    invalid_configs.push(bad);
    let mut bad = enabled_config();
    bad.services[0].title = "自定义低价服务".into();
    invalid_configs.push(bad);
    let mut bad = enabled_config();
    bad.services[0].unit = "月".into();
    invalid_configs.push(bad);
    let mut bad = enabled_config();
    bad.services[1] = bad.services[0].clone();
    invalid_configs.push(bad);
    for missing in ["description", "product_id", "contact_qr_url"] {
        let mut bad = enabled_config();
        match missing {
            "description" => bad.services[0].description.clear(),
            "product_id" => bad.services[0].product_id.clear(),
            _ => bad.contact_qr_url.clear(),
        }
        invalid_configs.push(bad);
    }
    let mut bad = enabled_config();
    bad.services[0].description = "⚽".repeat(4001);
    invalid_configs.push(bad);
    for invalid_id in ["two words".into(), "⚽".repeat(201)] {
        let mut bad = enabled_config();
        bad.services[0].product_id = invalid_id;
        invalid_configs.push(bad);
    }
    for invalid_qr in [
        "http://cdn.example.test/qr.png",
        "https://user:password@cdn.example.test/qr.png",
    ] {
        let mut bad = enabled_config();
        bad.contact_qr_url = invalid_qr.into();
        invalid_configs.push(bad);
    }
    for bad in invalid_configs {
        assert!(matches!(
            service.save(bad, actor).await,
            Err(StudentError::InvalidInput(_))
        ));
    }
    assert!(fixture.saved.lock().unwrap().is_empty());
    let mut draft = StudentConfig::default();
    draft.services[0].description = "  一行\n二行  ".into();
    let saved = service.save(draft, actor).await.unwrap();
    assert_eq!(saved.services[0].description, "一行\n二行");
    let writes = fixture.saved.lock().unwrap();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].1, actor);
    assert_eq!(writes[0].0.services[0].description, "一行\n二行");
}

#[tokio::test]
async fn invalid_order_page_boundaries_never_reach_persistence() {
    let fixture = Fixture::new(StudentConfig::default());
    for (page, size) in [(0, 20), (-1, 20), (1_000_001, 20), (1, 0), (1, 101)] {
        assert!(matches!(
            fixture.service().orders(Uuid::new_v4(), page, size).await,
            Err(StudentError::InvalidInput(_))
        ));
    }
}
