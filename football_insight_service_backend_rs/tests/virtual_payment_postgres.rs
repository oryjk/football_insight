//! VIRTUAL_PAYMENT_TEST_DATABASE_URL must point to a disposable database ending in _test.
use football_insight_service_backend_rs::payment::{
    adapters::persistence::{
        postgres_order_repository::PostgresOrderRepository,
        postgres_payment_settlement_port::PostgresPaymentSettlementPort,
        postgres_virtual_order_repository::PostgresVirtualOrderRepository,
    },
    application::handle_wechat_notify::{HandleWechatNotifyUseCase, NotifyHandleResult},
    domain::{
        order::NewPaymentOrder,
        virtual_payment::{Delivery, Refund, VirtualOrder},
    },
    ports::{order_repository::OrderRepository, virtual_payment_port::VirtualOrderRepository},
};
use football_insight_service_backend_rs::{
    auth::adapters::persistence::postgres_auth_repository::PostgresAuthRepository,
    payment::{
        application::virtual_payment::VirtualPaymentService,
        domain::virtual_payment::{
            VirtualPayParams, VirtualPaymentError, VirtualProduct, build_sign_data,
        },
        ports::{virtual_payment_port::VirtualPaymentGateway, wechat_pay_port::WechatPayPort},
    },
};
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::sync::Arc;
use uuid::Uuid;

// Only the external WeChat network is replaced; orders, catalog, grants and refunds use PostgreSQL.
struct CatalogTestGateway;
#[async_trait::async_trait]
impl VirtualPaymentGateway for CatalogTestGateway {
    fn available(&self) -> bool {
        true
    }
    async fn prepare_session(&self, _: &str, _: &str) -> Result<(), VirtualPaymentError> {
        Ok(())
    }
    async fn payment_params(
        &self,
        no: &str,
        _: &str,
        product: &VirtualProduct,
    ) -> Result<VirtualPayParams, VirtualPaymentError> {
        Ok(VirtualPayParams {
            mode: "short_series_goods".into(),
            sign_data: build_sign_data("test-offer", no, product)?,
            pay_sig: "test-signature".into(),
            signature: "test-session-signature".into(),
        })
    }
    async fn query_order(
        &self,
        order: &VirtualOrder,
    ) -> Result<Option<Delivery>, VirtualPaymentError> {
        Ok(Some(Delivery {
            order_no: order.order_no.clone(),
            open_id: order.open_id.clone(),
            product_id: order.product_id.clone(),
            amount_cents: order.amount_cents,
            original_price_cents: order.amount_cents,
            quantity: 1,
            env: 0,
            wx_order_id: format!("WX{}", order.order_no),
        }))
    }
    async fn confirm_delivery(&self, _: &VirtualOrder) -> Result<(), VirtualPaymentError> {
        Ok(())
    }
}

fn catalog_service(db: &Database) -> VirtualPaymentService {
    let (orders, virtual_orders, settlement) = db.repos();
    VirtualPaymentService::new(Arc::new(CatalogTestGateway), orders, virtual_orders,
        Arc::new(PostgresAuthRepository::new(db.pool.clone())), Arc::new(settlement),
        Arc::new(football_insight_service_backend_rs::payment::adapters::persistence::postgres_virtual_product_repository::PostgresVirtualProductRepository::new(db.pool.clone())))
}

async fn catalog_checkout(
    db: &Database,
    service: &VirtualPaymentService,
    no: &str,
    kind: &str,
    price: i32,
) -> anyhow::Result<serde_json::Value> {
    db.repos()
        .0
        .create_order(NewPaymentOrder {
            order_no: no.into(),
            user_id: db.user,
            amount_cents: price,
            product_type: kind.into(),
        })
        .await?;
    let params = service
        .unified_order(no, "test product", price, "buyer")
        .await?;
    Ok(serde_json::from_str(
        &params.virtual_payment.unwrap().sign_data,
    )?)
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn catalog_edits_affect_new_orders_without_rebuilding_service_and_preserve_existing_orders() {
    let db = Database::new().await;
    let service = catalog_service(&db);
    assert!(service.purchase_available().await.unwrap());
    let before = catalog_checkout(&db, &service, "catalogbefore", "membership:V6", 3900)
        .await
        .unwrap();
    assert_eq!(before["productId"], "fi_v6_year_39");
    sqlx::query("UPDATE f_i_virtual_payment_products SET product_id='v6_republished', name='新版年会员' WHERE product_key='membership:V6@3900'")
        .execute(&db.pool).await.unwrap();
    let after = catalog_checkout(&db, &service, "catalogafter", "membership:V6", 3900)
        .await
        .unwrap();
    assert_eq!(after["productId"], "v6_republished");
    let snapshot = db.repos().1.find("catalogbefore").await.unwrap().unwrap();
    assert_eq!(snapshot.product_id, "fi_v6_year_39");
    assert_eq!(snapshot.amount_cents, 3900);
    // Disable everything; the original paid order must still reconcile and grant its purchased entitlement.
    sqlx::query("UPDATE f_i_virtual_payment_products SET enabled=FALSE")
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(!service.purchase_available().await.unwrap());
    let error = catalog_checkout(&db, &service, "catalogblocked", "membership:V6", 3900)
        .await
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<VirtualPaymentError>(),
        Some(VirtualPaymentError::ProductUnavailable)
    ));
    assert!(db.repos().1.find("catalogblocked").await.unwrap().is_none());
    service.reconcile_pending().await.unwrap();
    assert_eq!(db.membership().await["tier"], "V6");
    let status: String =
        sqlx::query_scalar("SELECT status FROM f_i_payment_orders WHERE order_no='catalogbefore'")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(status, "paid");
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn catalog_matches_upgrade_match_unlock_and_reflux_at_exact_checkout_price() {
    let db = Database::new().await;
    let service = catalog_service(&db);
    sqlx::query("DELETE FROM f_i_virtual_payment_products")
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(!service.purchase_available().await.unwrap());
    sqlx::query("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents,enabled) VALUES ('membership:V7@3500','V7 升级','v7_upgrade',3500,TRUE),('match_id_unlock@500','比赛 ID','match_unlock',500,TRUE),('reflux_subscription:single_match:chengdu@500','回流提醒','reflux',500,TRUE)")
        .execute(&db.pool).await.unwrap();
    assert!(service.purchase_available().await.unwrap());
    for (no, kind, price, id) in [
        ("catalogupgrade", "membership:V7", 3500, "v7_upgrade"),
        ("catalogmatch", "match_id_unlock:571", 500, "match_unlock"),
        (
            "catalogreflux",
            "reflux_subscription:single_match:chengdu:571",
            500,
            "reflux",
        ),
    ] {
        let params = catalog_checkout(&db, &service, no, kind, price)
            .await
            .unwrap();
        assert_eq!(params["productId"], id);
        assert_eq!(params["goodsPrice"], price);
    }
    let error = catalog_checkout(&db, &service, "catalogwrongprice", "membership:V7", 5900)
        .await
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<VirtualPaymentError>(),
        Some(VirtualPaymentError::ProductUnavailable)
    ));
    let mismatch = sqlx::query("UPDATE f_i_virtual_payment_products SET price_cents=100 WHERE product_key='membership:V7@3500'").execute(&db.pool).await;
    assert!(
        mismatch.is_err(),
        "catalog key and price must remain consistent"
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn unreadable_catalog_rejects_purchase_without_creating_virtual_order() {
    let db = Database::new().await;
    let service = catalog_service(&db);
    sqlx::query("DROP TABLE f_i_virtual_payment_products")
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(matches!(
        service.purchase_available().await,
        Err(VirtualPaymentError::CatalogUnavailable)
    ));
    let error = catalog_checkout(&db, &service, "catalogdbfailed", "membership:V6", 3900)
        .await
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<VirtualPaymentError>(),
        Some(VirtualPaymentError::CatalogUnavailable)
    ));
    assert!(
        db.repos()
            .1
            .find("catalogdbfailed")
            .await
            .unwrap()
            .is_none()
    );
    db.close().await;
}

struct Database {
    admin: PgPool,
    pool: PgPool,
    schema: String,
    user: Uuid,
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn restricted_v3_product_only_allows_its_buyer_and_grants_v3_after_payment() {
    let db = Database::new().await;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM information_schema.columns WHERE table_schema=current_schema() AND table_name='f_i_virtual_payment_products' AND column_name='allowed_user_ids')").fetch_one(&db.pool).await.unwrap();
    assert!(
        exists,
        "product-level buyer restrictions must be stored in the catalog"
    );
    sqlx::query("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents,enabled,allowed_user_ids) VALUES('membership:V3@1','VIP3 测试会员','test',1,TRUE,ARRAY[$1]::uuid[])").bind(db.user).execute(&db.pool).await.unwrap();
    let service = catalog_service(&db);
    let params = catalog_checkout(&db, &service, "restrictedv3buyer", "membership:V3", 1)
        .await
        .unwrap();
    assert_eq!(params["productId"], "test");
    assert_eq!(params["goodsPrice"], 1);
    let other = Uuid::new_v4();
    sqlx::query("INSERT INTO f_i_users(id,account_identifier) VALUES($1,'other')")
        .bind(other)
        .execute(&db.pool)
        .await
        .unwrap();
    let catalog = football_insight_service_backend_rs::payment::application::get_membership_product::GetMembershipProductUseCase::new(
        Arc::new(football_insight_service_backend_rs::system_config::adapters::persistence::postgres_system_config_port::PostgresSystemConfigPort::new(db.pool.clone())),
        Arc::new(football_insight_service_backend_rs::payment::adapters::persistence::postgres_virtual_product_repository::PostgresVirtualProductRepository::new(db.pool.clone())),
    );
    let buyer_catalog = catalog.execute_for_user(Some(db.user)).await.unwrap();
    assert_eq!(
        buyer_catalog
            .find_option("V3")
            .expect("authorized buyer sees VIP3")
            .price_cents,
        1
    );
    assert!(
        catalog
            .execute_for_user(Some(other))
            .await
            .unwrap()
            .find_option("V3")
            .is_none()
    );
    assert!(catalog.execute().await.unwrap().find_option("V3").is_none());
    db.repos()
        .0
        .create_order(NewPaymentOrder {
            order_no: "restrictedv3other".into(),
            user_id: other,
            amount_cents: 1,
            product_type: "membership:V3".into(),
        })
        .await
        .unwrap();
    let error = service
        .unified_order("restrictedv3other", "test", 1, "other")
        .await
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<VirtualPaymentError>(),
        Some(VirtualPaymentError::ProductUnavailable)
    ));
    assert!(
        db.repos()
            .1
            .find("restrictedv3other")
            .await
            .unwrap()
            .is_none()
    );
    let invalid_public_v3 = sqlx::query("UPDATE f_i_virtual_payment_products SET allowed_user_ids='{}' WHERE product_key='membership:V3@1'").execute(&db.pool).await;
    assert!(
        invalid_public_v3.is_err(),
        "V3 must never accidentally become a public paid tier"
    );
    service.reconcile_pending().await.unwrap();
    assert_eq!(db.membership().await["tier"], "V3");
    assert!(
        db.membership().await["expires_at"].is_null(),
        "existing V3 uses ordinary membership rules"
    );
    db.refund("restrictedv3buyer", "restrictedv3refund", 1)
        .await
        .unwrap();
    assert_eq!(db.membership().await["tier"], "V1");
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn product_catalog_migration_seeds_published_v6() {
    let db = Database::new().await;
    let table: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('f_i_virtual_payment_products')::text")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert!(
        table.is_some(),
        "virtual payment products must live in a database table"
    );
    let row = sqlx::query("SELECT product_id, price_cents, enabled FROM f_i_virtual_payment_products WHERE product_key='membership:V6@3900'")
        .fetch_one(&db.pool).await.unwrap();
    assert_eq!(row.get::<String, _>("product_id"), "fi_v6_year_39");
    assert_eq!(row.get::<i32, _>("price_cents"), 3900);
    assert!(row.get::<bool, _>("enabled"));
    db.close().await;
}
impl Database {
    async fn new() -> Self {
        let url = std::env::var("VIRTUAL_PAYMENT_TEST_DATABASE_URL")
            .expect("explicit disposable database required");
        assert!(
            url::Url::parse(&url).unwrap().path().ends_with("_test"),
            "refusing a non-test database"
        );
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .unwrap();
        let schema = format!("virtualpay_{}", Uuid::new_v4().simple());
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
        sqlx::raw_sql("CREATE TABLE f_i_users(id UUID PRIMARY KEY,wx_open_id TEXT UNIQUE,account_identifier TEXT,display_name TEXT,password_hash TEXT,membership_tier TEXT NOT NULL DEFAULT 'V1',membership_expires_at TIMESTAMPTZ,status TEXT NOT NULL DEFAULT 'active',created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()); CREATE TABLE f_i_system_configs(config_key TEXT PRIMARY KEY,config_value JSONB,description TEXT,updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW());").execute(&pool).await.unwrap();
        for migration in [
            include_str!("../migrations/20260422150000_add_payment_orders.sql"),
            include_str!("../migrations/20260519000003_widen_product_type.sql"),
            include_str!("../migrations/20260518170000_add_reflux_email_subscriptions.sql"),
            include_str!("../migrations/20260826140000_add_match_id_unlocks.sql"),
            include_str!("../migrations/20261013000500_add_virtual_payment.sql"),
            include_str!("../migrations/20261013220000_add_virtual_payment_products.sql"),
            include_str!("../migrations/20261013223000_add_restricted_virtual_products.sql"),
        ] {
            sqlx::raw_sql(migration).execute(&pool).await.unwrap();
        }
        let user = Uuid::new_v4();
        sqlx::query("INSERT INTO f_i_users(id,wx_open_id,account_identifier,display_name,password_hash) VALUES($1,'buyer','buyer','Buyer','unused')").bind(user).execute(&pool).await.unwrap();
        Self {
            admin,
            pool,
            schema,
            user,
        }
    }
    fn repos(
        &self,
    ) -> (
        Arc<PostgresOrderRepository>,
        Arc<PostgresVirtualOrderRepository>,
        HandleWechatNotifyUseCase,
    ) {
        let orders = Arc::new(PostgresOrderRepository::new(self.pool.clone()));
        let virtual_orders = Arc::new(PostgresVirtualOrderRepository::new(self.pool.clone()));
        let settlement = HandleWechatNotifyUseCase::new(
            orders.clone(),
            Arc::new(PostgresPaymentSettlementPort::new(self.pool.clone())),
        );
        (orders, virtual_orders, settlement)
    }
    async fn order(&self, no: &str, product: &str, amount: i32) {
        let (orders, virtual_orders, _) = self.repos();
        orders
            .create_order(NewPaymentOrder {
                order_no: no.into(),
                user_id: self.user,
                amount_cents: amount,
                product_type: product.into(),
            })
            .await
            .unwrap();
        virtual_orders
            .record(VirtualOrder {
                order_no: no.into(),
                open_id: "buyer".into(),
                product_id: product.into(),
                amount_cents: amount,
                wx_order_id: None,
            })
            .await
            .unwrap();
        virtual_orders
            .record_delivery(&Delivery {
                order_no: no.into(),
                open_id: "buyer".into(),
                product_id: product.into(),
                amount_cents: amount,
                original_price_cents: amount,
                quantity: 1,
                env: 0,
                wx_order_id: format!("WX{no}"),
            })
            .await
            .unwrap();
    }
    async fn pay(&self, no: &str) {
        let (orders, _, settlement) = self.repos();
        let order = orders.find_order_by_no(no).await.unwrap().unwrap();
        settlement
            .settle_verified_order(&order, &format!("WX{no}"))
            .await
            .unwrap();
    }
    async fn refund(&self, no: &str, id: &str, amount: i32) -> anyhow::Result<()> {
        self.repos()
            .1
            .refund(&Refund {
                order_no: no.into(),
                open_id: "buyer".into(),
                wx_order_id: format!("WX{no}"),
                refund_id: id.into(),
                amount_cents: amount,
            })
            .await
    }
    async fn membership(&self) -> serde_json::Value {
        sqlx::query_scalar("SELECT jsonb_build_object('tier',membership_tier,'expires_at',membership_expires_at) FROM f_i_users WHERE id=$1").bind(self.user).fetch_one(&self.pool).await.unwrap()
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
#[ignore = "requires an isolated PostgreSQL database"]
async fn duplicate_settlement_and_refund_are_idempotent_and_restore_original_membership() {
    let db = Database::new().await;
    let before = db.membership().await;
    db.order("ORDER001", "membership:V6", 3900).await;
    db.pay("ORDER001").await;
    let granted = db.membership().await;
    assert_eq!(granted["tier"], "V6");
    db.pay("ORDER001").await;
    assert_eq!(granted, db.membership().await);
    db.refund("ORDER001", "REF001", 3900).await.unwrap();
    db.refund("ORDER001", "REF001", 3900).await.unwrap();
    assert_eq!(before, db.membership().await);
    assert_eq!(
        db.repos()
            .0
            .find_order_by_no("ORDER001")
            .await
            .unwrap()
            .unwrap()
            .status
            .as_str(),
        "refunded"
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn refund_chain_does_not_resurrect_an_earlier_refunded_membership() {
    let db = Database::new().await;
    let before = db.membership().await;
    db.order("ORDER001", "membership:V6", 3900).await;
    db.pay("ORDER001").await;
    db.order("ORDER002", "membership:V7", 3500).await;
    db.pay("ORDER002").await;
    let latest = db.membership().await;
    db.refund("ORDER001", "REF001", 3900).await.unwrap();
    assert_eq!(latest, db.membership().await);
    db.refund("ORDER002", "REF002", 3500).await.unwrap();
    assert_eq!(before, db.membership().await);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn refund_preserves_manual_membership_adjustment_and_rejects_over_refund() {
    let db = Database::new().await;
    db.order("ORDER001", "membership:V6", 3900).await;
    db.pay("ORDER001").await;
    sqlx::query("UPDATE f_i_users SET membership_tier='V9',membership_expires_at=NULL WHERE id=$1")
        .bind(db.user)
        .execute(&db.pool)
        .await
        .unwrap();
    let manual = db.membership().await;
    db.refund("ORDER001", "PARTIAL", 1000).await.unwrap();
    assert!(db.refund("ORDER001", "OVER", 3900).await.is_err());
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_virtual_payment_refunds")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    db.refund("ORDER001", "REST", 2900).await.unwrap();
    assert_eq!(manual, db.membership().await);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn ordinary_notify_cannot_settle_virtual_orders_and_provider_id_cannot_be_reused() {
    let db = Database::new().await;
    db.order("ORDER001", "membership:V6", 3900).await;
    let notify = std::collections::HashMap::from([
        ("return_code".into(), "SUCCESS".into()),
        ("result_code".into(), "SUCCESS".into()),
        ("out_trade_no".into(), "ORDER001".into()),
        ("transaction_id".into(), "FAKE".into()),
        ("total_fee".into(), "3900".into()),
    ]);
    assert!(matches!(
        db.repos().2.execute(notify).await.unwrap(),
        NotifyHandleResult::ProtocolFailure
    ));
    assert_eq!(db.membership().await["tier"], "V1");
    db.order("ORDER002", "membership:V6", 3900).await;
    let result = db
        .repos()
        .1
        .record_delivery(&Delivery {
            order_no: "ORDER002".into(),
            open_id: "buyer".into(),
            product_id: "membership:V6".into(),
            amount_cents: 3900,
            original_price_cents: 3900,
            quantity: 1,
            env: 0,
            wx_order_id: "WXORDER001".into(),
        })
        .await;
    assert!(result.is_err());
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn match_unlock_refund_preserves_another_paid_order() {
    let db = Database::new().await;
    db.order("ORDER001", "match_id_unlock:123", 500).await;
    db.pay("ORDER001").await;
    db.order("ORDER002", "match_id_unlock:123", 500).await;
    db.pay("ORDER002").await;
    db.refund("ORDER001", "REF001", 500).await.unwrap();
    let no: String = sqlx::query_scalar(
        "SELECT order_no FROM f_i_user_match_id_unlocks WHERE user_id=$1 AND match_id=123",
    )
    .bind(db.user)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(no, "ORDER002");
    db.refund("ORDER002", "REF002", 500).await.unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM f_i_user_match_id_unlocks WHERE user_id=$1")
            .bind(db.user)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn subscription_refund_only_cancels_its_own_subscription_and_settlement_rolls_back() {
    let db = Database::new().await;
    db.order("ORDER001", "reflux_subscription:missing:chengdu:123", 100)
        .await;
    let order = db
        .repos()
        .0
        .find_order_by_no("ORDER001")
        .await
        .unwrap()
        .unwrap();
    assert!(
        db.repos()
            .2
            .settle_verified_order(&order, "WXORDER001")
            .await
            .is_err()
    );
    assert_eq!(
        db.repos()
            .0
            .find_order_by_no("ORDER001")
            .await
            .unwrap()
            .unwrap()
            .status
            .as_str(),
        "pending"
    );
    sqlx::query("INSERT INTO f_i_user_notification_targets(id,user_id,channel,target,is_active) VALUES($1,$2,'email','a@example.com',TRUE)").bind(Uuid::new_v4()).bind(db.user).execute(&db.pool).await.unwrap();
    sqlx::query("INSERT INTO f_i_reflux_subscription_plans(code,scope,team_code,title,description,price_cents) VALUES('testplan','single_match','chengdu','Test','Test',100)").execute(&db.pool).await.unwrap();
    for no in ["ORDER002", "ORDER003"] {
        db.order(no, "reflux_subscription:testplan:chengdu:123", 100)
            .await;
        db.pay(no).await;
    }
    db.refund("ORDER002", "REF002", 100).await.unwrap();
    let rows =
        sqlx::query("SELECT order_no,status FROM f_i_user_reflux_subscriptions ORDER BY order_no")
            .fetch_all(&db.pool)
            .await
            .unwrap();
    assert_eq!(rows[0].get::<String, _>("status"), "cancelled");
    assert_eq!(rows[1].get::<String, _>("status"), "active");
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn successful_refund_before_delivery_blocks_a_late_entitlement_grant() {
    let db = Database::new().await;
    db.order("ORDER001", "membership:V6", 3900).await;
    db.refund("ORDER001", "REF001", 3900).await.unwrap();
    db.pay("ORDER001").await;
    assert_eq!(db.membership().await["tier"], "V1");
    assert_eq!(
        db.repos()
            .0
            .find_order_by_no("ORDER001")
            .await
            .unwrap()
            .unwrap()
            .status
            .as_str(),
        "refunded"
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn concurrent_member_refunds_serialize_without_deadlock_or_resurrecting_grants() {
    let db = Database::new().await;
    let before = db.membership().await;
    db.order("ORDER001", "membership:V6", 3900).await;
    db.pay("ORDER001").await;
    db.order("ORDER002", "membership:V7", 3500).await;
    db.pay("ORDER002").await;
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        tokio::join!(
            db.refund("ORDER001", "REF001", 3900),
            db.refund("ORDER002", "REF002", 3500)
        )
    })
    .await
    .unwrap();
    result.0.unwrap();
    result.1.unwrap();
    assert_eq!(before, db.membership().await);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn unfulfilled_partial_refund_can_finish_and_never_grants_membership() {
    let db = Database::new().await;
    db.order("ORDER001", "membership:V6", 3900).await;
    db.refund("ORDER001", "PARTIAL", 1000).await.unwrap();
    db.pay("ORDER001").await;
    db.refund("ORDER001", "REST", 2900).await.unwrap();
    assert_eq!(db.membership().await["tier"], "V1");
    db.close().await;
}
