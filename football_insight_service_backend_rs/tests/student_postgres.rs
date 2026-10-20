//! Explicitly opt in with STUDENT_TEST_DATABASE_URL: localhost, disposable database ending _test.
//! Only the external WeChat network is replaced; application, catalog, order, settlement,
//! refund, administrator audit and membership-origin persistence are production components.
use async_trait::async_trait;
use football_insight_service_backend_rs::{
    auth::adapters::persistence::postgres_auth_repository::PostgresAuthRepository,
    payment::{
        adapters::persistence::{
            postgres_order_repository::PostgresOrderRepository,
            postgres_payment_settlement_port::PostgresPaymentSettlementPort,
            postgres_virtual_order_repository::PostgresVirtualOrderRepository,
            postgres_virtual_product_repository::PostgresVirtualProductRepository,
        },
        application::{
            handle_wechat_notify::HandleWechatNotifyUseCase, virtual_payment::VirtualPaymentService,
        },
        domain::{
            order::{NewPaymentOrder, OrderStatus},
            virtual_payment::{
                Delivery, Refund, VirtualOrder, VirtualPayParams, VirtualPaymentError,
                VirtualProduct, build_sign_data,
            },
        },
        ports::{
            order_repository::OrderRepository,
            virtual_payment_port::{VirtualOrderRepository, VirtualPaymentGateway},
        },
    },
    student::{
        adapters::persistence::PostgresStudentRepository,
        application::{StudentConfigService, StudentService},
        domain::{ServiceCode, StudentConfig, StudentError},
        ports::StudentRepository,
    },
};
use serde_json::{Value, json};
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::sync::Arc;
use uuid::Uuid;

const SEEDED_QR_URL: &str = "https://oryjk.cn:82/minio/football-insight/summary/student/contact-wechat-740aed5038f2113f.jpg";

struct TestWechat;
#[async_trait]
impl VirtualPaymentGateway for TestWechat {
    fn available(&self) -> bool {
        true
    }
    async fn prepare_session(&self, _: &str, _: &str) -> Result<(), VirtualPaymentError> {
        Ok(())
    }
    async fn payment_params(
        &self,
        order: &str,
        _: &str,
        product: &VirtualProduct,
    ) -> Result<VirtualPayParams, VirtualPaymentError> {
        Ok(VirtualPayParams {
            mode: "short_series_goods".into(),
            sign_data: build_sign_data("test-student-offer", order, product)?,
            pay_sig: "test-pay-signature".into(),
            signature: "test-session-signature".into(),
        })
    }
    async fn query_order(
        &self,
        order: &VirtualOrder,
    ) -> Result<Option<Delivery>, VirtualPaymentError> {
        Ok(Some(delivery(order)))
    }
    async fn confirm_delivery(&self, _: &VirtualOrder) -> Result<(), VirtualPaymentError> {
        Ok(())
    }
}
fn delivery(order: &VirtualOrder) -> Delivery {
    Delivery {
        order_no: order.order_no.clone(),
        open_id: order.open_id.clone(),
        product_id: order.product_id.clone(),
        amount_cents: order.amount_cents,
        original_price_cents: order.amount_cents,
        quantity: 1,
        env: 0,
        wx_order_id: format!("WX{}", order.order_no),
    }
}
fn configured() -> StudentConfig {
    let mut config = StudentConfig::default();
    config.contact_qr_url = "https://cdn.example.test/student/contact-v1.png".into();
    config.services[0].description = "证书服务说明\n付款后联系负责人".into();
    config.services[0].product_id = "published-certificate-99".into();
    config.services[0].enabled = true;
    config.services[1].description = "工具服务说明".into();
    config.services[1].product_id = "published-tool-699".into();
    config.services[1].enabled = true;
    config
}
struct Database {
    admin: PgPool,
    pool: PgPool,
    schema: String,
    actor: Uuid,
    buyer: Uuid,
    other: Uuid,
}
impl Database {
    async fn new() -> Self {
        let url = std::env::var("STUDENT_TEST_DATABASE_URL")
            .expect("explicit disposable student test URL required");
        let parsed = url::Url::parse(&url).unwrap();
        assert!(
            matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")),
            "only localhost databases are permitted"
        );
        assert!(
            parsed.path().ends_with("_test"),
            "refusing a non-test database"
        );
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .unwrap();
        let schema = format!("student_{}", Uuid::new_v4().simple());
        sqlx::raw_sql(&format!("CREATE SCHEMA {schema}"))
            .execute(&admin)
            .await
            .unwrap();
        let options = url
            .parse::<PgConnectOptions>()
            .unwrap()
            .options([("search_path", schema.as_str())]);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .unwrap();
        // Same minimal auth/config base as virtual_payment_postgres; real migrations own payment schema.
        sqlx::raw_sql("CREATE TABLE f_i_users(id UUID PRIMARY KEY,wx_open_id TEXT UNIQUE,account_identifier TEXT,display_name TEXT,password_hash TEXT,membership_tier TEXT NOT NULL DEFAULT 'V1',membership_expires_at TIMESTAMPTZ,status TEXT NOT NULL DEFAULT 'active',created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()); CREATE TABLE f_i_system_configs(config_key TEXT PRIMARY KEY,config_value TEXT,description TEXT,updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW());").execute(&pool).await.unwrap();
        for migration in [
            include_str!("../migrations/20260422150000_add_payment_orders.sql"),
            include_str!("../migrations/20260519000003_widen_product_type.sql"),
            include_str!("../migrations/20260518170000_add_reflux_email_subscriptions.sql"),
            include_str!("../migrations/20260826140000_add_match_id_unlocks.sql"),
            include_str!("../migrations/20260802120000_add_admin_identity_and_audit.sql"),
            include_str!("../migrations/20261013000500_add_virtual_payment.sql"),
            include_str!("../migrations/20261013220000_add_virtual_payment_products.sql"),
            include_str!("../migrations/20261013223000_add_restricted_virtual_products.sql"),
            include_str!("../migrations/20261013230000_add_membership_duration.sql"),
            include_str!("../migrations/20261013230100_backfill_paid_v3_expiration.sql"),
            include_str!("../migrations/20261014093000_add_virtual_product_purchase_identity.sql"),
            include_str!("../migrations/20261014110000_add_membership_origin.sql"),
            include_str!("../migrations/20261016130000_add_product_catalog_specs.sql"),
            include_str!("../migrations/20261020220000_add_student_service_products.sql"),
        ] {
            sqlx::raw_sql(migration).execute(&pool).await.unwrap();
        }
        let actor = Uuid::new_v4();
        sqlx::query("INSERT INTO f_i_admin_users(id,username,password_hash,display_name,role) VALUES($1,'student_test_owner','unused','Test Owner','owner')").bind(actor).execute(&pool).await.unwrap();
        let buyer = Uuid::new_v4();
        let other = Uuid::new_v4();
        sqlx::query("INSERT INTO f_i_users(id,wx_open_id,account_identifier,display_name,password_hash) VALUES($1,'student-buyer','student-buyer','Test Buyer','unused'),($2,'student-other','student-other','Test Other','unused')").bind(buyer).bind(other).execute(&pool).await.unwrap();
        Self {
            admin,
            pool,
            schema,
            actor,
            buyer,
            other,
        }
    }
    fn repository(&self) -> Arc<PostgresStudentRepository> {
        Arc::new(PostgresStudentRepository::new(self.pool.clone()))
    }
    fn config_service(&self) -> StudentConfigService {
        StudentConfigService::new(self.repository())
    }
    fn payment_orders(&self) -> Arc<PostgresOrderRepository> {
        Arc::new(PostgresOrderRepository::new(self.pool.clone()))
    }
    fn virtual_orders(&self) -> Arc<PostgresVirtualOrderRepository> {
        Arc::new(PostgresVirtualOrderRepository::new(self.pool.clone()))
    }
    fn payment_service(&self) -> Arc<VirtualPaymentService> {
        let orders = self.payment_orders();
        Arc::new(VirtualPaymentService::new(
            Arc::new(TestWechat),
            orders.clone(),
            self.virtual_orders(),
            Arc::new(PostgresAuthRepository::new(self.pool.clone())),
            Arc::new(HandleWechatNotifyUseCase::new(
                orders,
                Arc::new(PostgresPaymentSettlementPort::new(self.pool.clone())),
            )),
            Arc::new(PostgresVirtualProductRepository::new(self.pool.clone())),
        ))
    }
    fn service(&self) -> StudentService {
        StudentService::new(
            self.repository(),
            self.payment_orders(),
            Arc::new(PostgresAuthRepository::new(self.pool.clone())),
            self.payment_service(),
        )
    }
    async fn count(&self, table: &str) -> i64 {
        assert!(matches!(
            table,
            "f_i_payment_orders" | "f_i_admin_audit_logs" | "f_i_virtual_payment_refunds"
        ));
        sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
    async fn mappings(&self) -> Value {
        sqlx::query_scalar("SELECT COALESCE(jsonb_agg(jsonb_build_object('key',product_key,'id',product_id,'price',price_cents,'enabled',enabled,'kind',kind,'description',description) ORDER BY product_key),'[]'::jsonb) FROM f_i_virtual_payment_products WHERE product_key LIKE 'student_service:%'").fetch_one(&self.pool).await.unwrap()
    }
    async fn membership(&self) -> Value {
        sqlx::query_scalar("SELECT jsonb_build_object('tier',membership_tier,'origin',membership_origin,'expiry',membership_expires_at,'updated_at',updated_at) FROM f_i_users WHERE id=$1").bind(self.buyer).fetch_one(&self.pool).await.unwrap()
    }
    async fn close(self) {
        self.pool.close().await;
        sqlx::raw_sql(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .execute(&self.admin)
            .await
            .unwrap();
        self.admin.close().await;
    }
}

#[tokio::test]
#[ignore = "requires STUDENT_TEST_DATABASE_URL pointing to a disposable localhost database"]
async fn unconfigured_service_and_disabled_draft_cannot_create_orders_or_seed_published_mappings() {
    let db = Database::new().await;
    assert_eq!(db.mappings().await, json!([]));
    let initial = db.repository().config().await.unwrap();
    assert_eq!(initial.contact_qr_url, SEEDED_QR_URL);
    assert_eq!(initial.services.len(), 2);
    assert!(
        initial
            .services
            .iter()
            .all(|item| !item.enabled && item.description.is_empty() && item.product_id.is_empty())
    );
    let service = db.service();
    for code in [ServiceCode::CertificateRenewal, ServiceCode::ToolUpgrade] {
        assert!(matches!(
            service.create_order(db.buyer, code).await,
            Err(StudentError::Unavailable)
        ));
    }
    db.config_service()
        .save(StudentConfig::default(), db.actor)
        .await
        .unwrap();
    assert_eq!(db.mappings().await, json!([]));
    assert!(matches!(
        service
            .create_order(db.buyer, ServiceCode::ToolUpgrade)
            .await,
        Err(StudentError::Unavailable)
    ));
    assert_eq!(db.count("f_i_payment_orders").await, 0);
    assert_eq!(db.count("f_i_admin_audit_logs").await, 1);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires STUDENT_TEST_DATABASE_URL pointing to a disposable localhost database"]
async fn save_config_updates_both_live_mappings_and_audit_and_clearing_product_disables_old_mapping()
 {
    let db = Database::new().await;
    let saved = db
        .config_service()
        .save(configured(), db.actor)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(db.repository().config().await.unwrap()).unwrap(),
        serde_json::to_value(&saved).unwrap()
    );
    assert_eq!(
        db.mappings().await,
        json!([
            {"key":"student_service:certificate_renewal@9900","id":"published-certificate-99","price":9900,"enabled":true,"kind":"student_service","description":"证书服务说明\n付款后联系负责人"},
            {"key":"student_service:tool_upgrade@69900","id":"published-tool-699","price":69900,"enabled":true,"kind":"student_service","description":"工具服务说明"}
        ])
    );
    let audit = sqlx::query("SELECT admin_user_id,action,target_type,target_id,before_json,after_json FROM f_i_admin_audit_logs").fetch_one(&db.pool).await.unwrap();
    assert_eq!(audit.get::<Uuid, _>("admin_user_id"), db.actor);
    assert_eq!(audit.get::<String, _>("action"), "student.config.set");
    assert_eq!(audit.get::<String, _>("target_id"), "student_features");
    assert_eq!(audit.get::<String, _>("target_type"), "system_config");
    let mut seed = StudentConfig::default();
    seed.contact_qr_url = SEEDED_QR_URL.into();
    assert_eq!(
        audit.get::<Option<Value>, _>("before_json"),
        Some(serde_json::to_value(seed).unwrap())
    );
    assert_eq!(
        audit.get::<Value, _>("after_json"),
        serde_json::to_value(&saved).unwrap()
    );
    // Rerunning the seed migration must preserve an administrator's published config
    // and live product mappings rather than reset them to the migration defaults.
    sqlx::raw_sql(include_str!(
        "../migrations/20261020220000_add_student_service_products.sql"
    ))
    .execute(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        serde_json::to_value(db.repository().config().await.unwrap()).unwrap(),
        serde_json::to_value(&saved).unwrap()
    );
    assert_eq!(db.count("f_i_admin_audit_logs").await, 1);
    assert!(
        db.mappings()
            .await
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["enabled"] == true)
    );
    let mut next = saved;
    next.services[0].product_id.clear();
    next.services[0].enabled = false;
    db.config_service().save(next, db.actor).await.unwrap();
    let enabled: bool = sqlx::query_scalar("SELECT enabled FROM f_i_virtual_payment_products WHERE product_key='student_service:certificate_renewal@9900'").fetch_one(&db.pool).await.unwrap();
    assert!(
        !enabled,
        "clearing the draft ID cannot leave the old published mapping purchasable"
    );
    assert!(matches!(
        db.service()
            .create_order(db.buyer, ServiceCode::CertificateRenewal)
            .await,
        Err(StudentError::Unavailable)
    ));
    assert_eq!(db.count("f_i_payment_orders").await, 0);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires STUDENT_TEST_DATABASE_URL pointing to a disposable localhost database"]
async fn invalid_config_mapping_failure_and_final_audit_failure_roll_back_the_whole_write() {
    let db = Database::new().await;
    let original = db
        .config_service()
        .save(configured(), db.actor)
        .await
        .unwrap();
    let before_config = serde_json::to_value(&original).unwrap();
    let before_mappings = db.mappings().await;
    let mut invalid = original.clone();
    invalid.services[0].price_cents = 1;
    assert!(matches!(
        db.config_service().save(invalid, db.actor).await,
        Err(StudentError::InvalidInput(_))
    ));
    // A real DB failure on the second mapping tests rollback of the already-updated first mapping.
    sqlx::query("ALTER TABLE f_i_virtual_payment_products ADD CONSTRAINT student_test_reject_mapping CHECK(product_id<>'test-rejected-tool')").execute(&db.pool).await.unwrap();
    let mut changed = original.clone();
    changed.services[0].product_id = "new-certificate-99".into();
    changed.services[1].product_id = "test-rejected-tool".into();
    assert!(matches!(
        db.config_service().save(changed, db.actor).await,
        Err(StudentError::Internal)
    ));
    assert_eq!(db.mappings().await, before_mappings);
    // An unknown admin violates the actual audit foreign key, after mappings and config were written.
    let mut changed = original;
    changed.contact_qr_url = "https://cdn.example.test/contact-v2.png".into();
    changed.services[0].product_id = "new-certificate-99".into();
    assert!(matches!(
        db.config_service().save(changed, Uuid::new_v4()).await,
        Err(StudentError::Internal)
    ));
    assert_eq!(db.mappings().await, before_mappings);
    assert_eq!(
        serde_json::to_value(db.repository().config().await.unwrap()).unwrap(),
        before_config
    );
    assert_eq!(db.count("f_i_admin_audit_logs").await, 1);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires STUDENT_TEST_DATABASE_URL pointing to a disposable localhost database"]
async fn order_pages_are_account_scoped_paginated_and_exclude_membership_purchases() {
    let db = Database::new().await;
    let orders = db.payment_orders();
    for (no, user, kind, amount) in [
        (
            "student-own-a",
            db.buyer,
            "student_service:certificate_renewal",
            9900,
        ),
        (
            "student-own-b",
            db.buyer,
            "student_service:tool_upgrade",
            69900,
        ),
        (
            "student-other",
            db.other,
            "student_service:tool_upgrade",
            69900,
        ),
        ("membership-own", db.buyer, "membership:V6", 3900),
    ] {
        orders
            .create_order(NewPaymentOrder {
                order_no: no.into(),
                user_id: user,
                product_type: kind.into(),
                amount_cents: amount,
            })
            .await
            .unwrap();
    }
    sqlx::query("UPDATE f_i_payment_orders SET created_at='2026-10-20T01:00:00Z'")
        .execute(&db.pool)
        .await
        .unwrap();
    let first = db.service().orders(db.buyer, 1, 1).await.unwrap();
    assert_eq!((first.total, first.page, first.page_size), (2, 1, 1));
    assert_eq!(first.items.len(), 1);
    assert_eq!(first.items[0].order_no, "student-own-b");
    assert_eq!(first.items[0].amount_cents, 69900);
    assert_eq!(first.items[0].code, ServiceCode::ToolUpgrade);
    let second = db.service().orders(db.buyer, 2, 1).await.unwrap();
    assert_eq!(second.total, 2);
    assert_eq!(second.items[0].order_no, "student-own-a");
    let other = db.service().orders(db.other, 1, 20).await.unwrap();
    assert_eq!(other.total, 1);
    assert_eq!(other.items[0].order_no, "student-other");
    let empty = db.service().orders(Uuid::new_v4(), 1, 20).await.unwrap();
    assert_eq!(empty.total, 0);
    assert!(empty.items.is_empty());
    db.close().await;
}

#[tokio::test]
#[ignore = "requires STUDENT_TEST_DATABASE_URL pointing to a disposable localhost database"]
async fn both_services_use_fixed_checkout_prices_and_duplicate_delivery_refund_never_change_membership()
 {
    let db = Database::new().await;
    db.config_service()
        .save(configured(), db.actor)
        .await
        .unwrap();
    // Use the real origin trigger: a referral grant must survive these unrelated purchases.
    let mut tx = db.pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('football.membership_origin','referral',true)")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE f_i_users SET membership_tier='V7',membership_expires_at='2030-10-20T12:34:56Z' WHERE id=$1").bind(db.buyer).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    let membership = db.membership().await;
    assert_eq!(membership["origin"], "referral");
    let service = db.service();
    let payment = db.payment_service();
    for (code, amount, published_id) in [
        (
            ServiceCode::CertificateRenewal,
            9900,
            "published-certificate-99",
        ),
        (ServiceCode::ToolUpgrade, 69900, "published-tool-699"),
    ] {
        let checkout = service.create_order(db.buyer, code).await.unwrap();
        let params: Value =
            serde_json::from_str(&checkout.params.virtual_payment.unwrap().sign_data).unwrap();
        assert_eq!(params["productId"], published_id);
        assert_eq!(params["goodsPrice"], amount);
        let order = db
            .payment_orders()
            .find_order_by_no(&checkout.order_no)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(order.amount_cents, amount);
        assert_eq!(order.user_id, db.buyer);
        assert_eq!(order.status, OrderStatus::Pending);
        let snapshot = db
            .virtual_orders()
            .find(&checkout.order_no)
            .await
            .unwrap()
            .unwrap();
        assert!(snapshot.catalog_product_id.is_some());
        assert_eq!(snapshot.product_id, published_id);
        assert!(
            payment
                .reconcile(&checkout.order_no, Some(db.other))
                .await
                .is_err()
        );
        let mut bad = delivery(&snapshot);
        bad.amount_cents = 1;
        assert!(payment.deliver(&bad).await.is_err());
        bad = delivery(&snapshot);
        bad.open_id = "wrong-buyer".into();
        assert!(payment.deliver(&bad).await.is_err());
        assert_eq!(
            db.payment_orders()
                .find_order_by_no(&checkout.order_no)
                .await
                .unwrap()
                .unwrap()
                .status,
            OrderStatus::Pending
        );
        let mut callback = delivery(&snapshot);
        callback.wx_order_id = "callback-untrusted-id".into();
        payment.deliver_callback(callback).await.unwrap();
        let paid = db
            .payment_orders()
            .find_order_by_no(&checkout.order_no)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(paid.status, OrderStatus::Paid);
        assert_eq!(
            paid.transaction_id.as_deref(),
            Some(format!("WX{}", checkout.order_no).as_str())
        );
        assert!(paid.paid_at.is_some());
        payment.deliver_callback(delivery(&snapshot)).await.unwrap();
        let duplicate = db
            .payment_orders()
            .find_order_by_no(&checkout.order_no)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(duplicate.paid_at, paid.paid_at);
        assert_eq!(duplicate.transaction_id, paid.transaction_id);
        assert_eq!(db.membership().await, membership);
        let mut refund = Refund {
            order_no: checkout.order_no.clone(),
            open_id: "student-buyer".into(),
            wx_order_id: format!("WX{}", checkout.order_no),
            refund_id: format!("refund-{}", checkout.order_no),
            amount_cents: amount,
        };
        refund.open_id = "wrong-buyer".into();
        assert!(payment.refund(&refund).await.is_err());
        refund.open_id = "student-buyer".into();
        payment.refund(&refund).await.unwrap();
        payment.refund(&refund).await.unwrap();
        payment.deliver_callback(delivery(&snapshot)).await.unwrap();
        assert_eq!(
            db.payment_orders()
                .find_order_by_no(&checkout.order_no)
                .await
                .unwrap()
                .unwrap()
                .status,
            OrderStatus::Refunded
        );
        assert_eq!(db.membership().await, membership);
        let list = service.orders(db.buyer, 1, 20).await.unwrap();
        assert_eq!(
            list.items
                .iter()
                .find(|item| item.order_no == checkout.order_no)
                .unwrap()
                .status,
            "refunded"
        );
    }
    assert_eq!(db.count("f_i_payment_orders").await, 2);
    assert_eq!(db.count("f_i_virtual_payment_refunds").await, 2);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires STUDENT_TEST_DATABASE_URL pointing to a disposable localhost database"]
async fn checkout_parameter_failure_leaves_a_failed_record_instead_of_an_unpayable_pending_order() {
    let db = Database::new().await;
    db.config_service()
        .save(configured(), db.actor)
        .await
        .unwrap();
    // Catalog can be disabled independently after student config was published.
    // The real payment application must reject issuance; no external provider is contacted.
    sqlx::query("UPDATE f_i_virtual_payment_products SET enabled=FALSE WHERE product_key='student_service:certificate_renewal@9900'").execute(&db.pool).await.unwrap();
    assert!(matches!(
        db.service()
            .create_order(db.buyer, ServiceCode::CertificateRenewal)
            .await,
        Err(StudentError::PaymentFailed)
    ));
    let page = db.service().orders(db.buyer, 1, 20).await.unwrap();
    assert_eq!(
        page.total, 1,
        "no ordinary-payment fallback order should be created"
    );
    assert_eq!(page.items[0].status, "failed");
    assert_eq!(page.items[0].amount_cents, 9900);
    assert!(
        db.virtual_orders()
            .find(&page.items[0].order_no)
            .await
            .unwrap()
            .is_none()
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires STUDENT_TEST_DATABASE_URL pointing to a disposable localhost database"]
async fn failing_unissued_order_is_account_scoped_and_never_invalidates_issued_or_settled_payments()
{
    let db = Database::new().await;
    for (no, user) in [
        ("student-unissued", db.buyer),
        ("student-issued", db.buyer),
        ("student-other-pending", db.other),
        ("student-already-paid", db.buyer),
        ("student-already-refunded", db.buyer),
    ] {
        db.payment_orders()
            .create_order(NewPaymentOrder {
                order_no: no.into(),
                user_id: user,
                product_type: "student_service:certificate_renewal".into(),
                amount_cents: 9900,
            })
            .await
            .unwrap();
    }
    db.virtual_orders()
        .record(VirtualOrder {
            catalog_product_id: None,
            order_no: "student-issued".into(),
            open_id: "student-buyer".into(),
            product_id: "published-certificate-99".into(),
            amount_cents: 9900,
            membership_duration_months: 12,
            wx_order_id: None,
        })
        .await
        .unwrap();
    sqlx::query("UPDATE f_i_payment_orders SET status='paid',paid_at=NOW() WHERE order_no='student-already-paid'").execute(&db.pool).await.unwrap();
    sqlx::query(
        "UPDATE f_i_payment_orders SET status='refunded' WHERE order_no='student-already-refunded'",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    let membership = db.membership().await;
    // A request under another account cannot invalidate the buyer's unissued order.
    db.repository()
        .fail_unissued_order(db.other, "student-unissued")
        .await
        .unwrap();
    assert_eq!(
        db.payment_orders()
            .find_order_by_no("student-unissued")
            .await
            .unwrap()
            .unwrap()
            .status,
        OrderStatus::Pending
    );
    for no in [
        "student-unissued",
        "student-issued",
        "student-other-pending",
        "student-already-paid",
        "student-already-refunded",
        "student-nonexistent",
    ] {
        db.repository()
            .fail_unissued_order(db.buyer, no)
            .await
            .unwrap();
    }
    for (no, expected) in [
        ("student-unissued", OrderStatus::Failed),
        ("student-issued", OrderStatus::Pending),
        ("student-other-pending", OrderStatus::Pending),
        ("student-already-paid", OrderStatus::Paid),
        ("student-already-refunded", OrderStatus::Refunded),
    ] {
        assert_eq!(
            db.payment_orders()
                .find_order_by_no(no)
                .await
                .unwrap()
                .unwrap()
                .status,
            expected,
            "unexpected order status for {no}"
        );
    }
    assert_eq!(db.membership().await, membership);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires STUDENT_TEST_DATABASE_URL pointing to a disposable localhost database"]
async fn disabling_or_republishing_service_preserves_existing_payment_snapshot_and_refund_before_delivery()
 {
    let db = Database::new().await;
    db.config_service()
        .save(configured(), db.actor)
        .await
        .unwrap();
    let service = db.service();
    let old = service
        .create_order(db.buyer, ServiceCode::CertificateRenewal)
        .await
        .unwrap();
    let mut config = configured();
    config.services[0].product_id = "republished-certificate-99".into();
    db.config_service()
        .save(config.clone(), db.actor)
        .await
        .unwrap();
    let new = service
        .create_order(db.buyer, ServiceCode::CertificateRenewal)
        .await
        .unwrap();
    let old_snapshot = db
        .virtual_orders()
        .find(&old.order_no)
        .await
        .unwrap()
        .unwrap();
    let new_snapshot = db
        .virtual_orders()
        .find(&new.order_no)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(old_snapshot.product_id, "published-certificate-99");
    assert_eq!(new_snapshot.product_id, "republished-certificate-99");
    config.services[0].enabled = false;
    db.config_service().save(config, db.actor).await.unwrap();
    assert!(matches!(
        service
            .create_order(db.buyer, ServiceCode::CertificateRenewal)
            .await,
        Err(StudentError::Unavailable)
    ));
    let membership = db.membership().await;
    let payment = db.payment_service();
    // A verified full refund can arrive before payment delivery; a late callback must not revive it.
    payment
        .refund(&Refund {
            order_no: old.order_no.clone(),
            open_id: "student-buyer".into(),
            wx_order_id: format!("WX{}", old.order_no),
            refund_id: "refund-before-student-delivery".into(),
            amount_cents: 9900,
        })
        .await
        .unwrap();
    payment
        .deliver_callback(delivery(&old_snapshot))
        .await
        .unwrap();
    payment
        .deliver_callback(delivery(&new_snapshot))
        .await
        .unwrap();
    assert_eq!(
        db.payment_orders()
            .find_order_by_no(&old.order_no)
            .await
            .unwrap()
            .unwrap()
            .status,
        OrderStatus::Refunded
    );
    assert_eq!(
        db.payment_orders()
            .find_order_by_no(&new.order_no)
            .await
            .unwrap()
            .unwrap()
            .status,
        OrderStatus::Paid
    );
    assert_eq!(db.count("f_i_payment_orders").await, 2);
    assert_eq!(db.membership().await, membership);
    db.close().await;
}
