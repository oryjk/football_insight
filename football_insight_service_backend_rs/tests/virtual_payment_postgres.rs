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

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn duration_changes_apply_to_new_orders_and_existing_orders_keep_their_snapshot() {
    let db = Database::new().await;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM information_schema.columns WHERE table_schema=current_schema() AND table_name='f_i_virtual_payment_products' AND column_name='membership_duration_months')").fetch_one(&db.pool).await.unwrap();
    assert!(exists, "membership duration must be a product field");
    let service = catalog_service(&db);
    catalog_checkout(&db, &service, "durationoriginal", "membership:V6", 3900)
        .await
        .unwrap();
    sqlx::query("UPDATE f_i_virtual_payment_products SET membership_duration_months=1 WHERE product_key='membership:V6@3900'").execute(&db.pool).await.unwrap();
    catalog_checkout(&db, &service, "durationchanged", "membership:V6", 3900)
        .await
        .unwrap();
    for (order_no, expected) in [("durationoriginal", 12), ("durationchanged", 1)] {
        let months: i32 = sqlx::query_scalar(
            "SELECT membership_duration_months FROM f_i_virtual_payment_orders WHERE order_no=$1",
        )
        .bind(order_no)
        .fetch_one(&db.pool)
        .await
        .unwrap();
        assert_eq!(months, expected);
    }
    sqlx::query("UPDATE f_i_virtual_payment_products SET enabled=FALSE")
        .execute(&db.pool)
        .await
        .unwrap();
    for (order_no, expected) in [("durationoriginal", 12), ("durationchanged", 1)] {
        service.reconcile(order_no, Some(db.user)).await.unwrap();
        let correct: bool = sqlx::query_scalar("SELECT abs(EXTRACT(EPOCH FROM (membership_expires_at-(NOW()+make_interval(months => $2))))) < 10 FROM f_i_users WHERE id=$1").bind(db.user).bind(expected).fetch_one(&db.pool).await.unwrap();
        assert!(
            correct,
            "fulfillment must use order duration even after disabling the catalog"
        );
    }
    db.refund("durationchanged", "durationchangedrefund", 3900)
        .await
        .unwrap();
    let restored: bool = sqlx::query_scalar(
        "SELECT membership_expires_at > NOW()+INTERVAL '11 months' FROM f_i_users WHERE id=$1",
    )
    .bind(db.user)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert!(restored);
    db.refund("durationoriginal", "durationoriginalrefund", 3900)
        .await
        .unwrap();
    assert_eq!(db.membership().await["tier"], "V1");
    assert!(db.membership().await["expires_at"].is_null());
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn catalog_duration_uses_the_actual_upgrade_sku_and_refreshes_after_database_edit() {
    let db = Database::new().await;
    sqlx::query("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents,enabled,membership_duration_months) VALUES('membership:V7@5900','V7 年会员','v7full',5900,TRUE,12),('membership:V7@3500','V7 升级','v7upgrade',3500,TRUE,3)").execute(&db.pool).await.unwrap();
    let catalog = football_insight_service_backend_rs::payment::application::get_membership_product::GetMembershipProductUseCase::new(
        Arc::new(football_insight_service_backend_rs::system_config::adapters::persistence::postgres_system_config_port::PostgresSystemConfigPort::new(db.pool.clone())),
        Arc::new(football_insight_service_backend_rs::payment::adapters::persistence::postgres_virtual_product_repository::PostgresVirtualProductRepository::new(db.pool.clone())),
    );
    let upgrade = catalog
        .with_duration_for_tier(catalog.execute().await.unwrap(), Some("V6"), Some(db.user))
        .await
        .unwrap();
    assert_eq!(
        upgrade
            .find_option("V7")
            .unwrap()
            .membership_duration_months,
        Some(3)
    );
    let full = catalog
        .with_duration_for_tier(catalog.execute().await.unwrap(), Some("V1"), Some(db.user))
        .await
        .unwrap();
    assert_eq!(
        full.find_option("V7").unwrap().membership_duration_months,
        Some(12)
    );
    sqlx::query("UPDATE f_i_virtual_payment_products SET membership_duration_months=1 WHERE product_key='membership:V7@3500'").execute(&db.pool).await.unwrap();
    let changed = catalog
        .with_duration_for_tier(catalog.execute().await.unwrap(), Some("V6"), Some(db.user))
        .await
        .unwrap();
    assert_eq!(
        changed
            .find_option("V7")
            .unwrap()
            .membership_duration_months,
        Some(1)
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn legacy_v3_expiry_backfill_preserves_later_changes_and_repairs_refund_snapshots() {
    for scenario in ["original", "manual", "refunded", "successor"] {
        let db = Database::with_duration(false).await;
        db.repos()
            .0
            .create_order(NewPaymentOrder {
                order_no: "legacyv3paid".into(),
                user_id: db.user,
                amount_cents: 1,
                product_type: "membership:V3".into(),
            })
            .await
            .unwrap();
        let before = serde_json::json!({"tier":"V1","expires_at":null});
        let after = serde_json::json!({"tier":"V3","expires_at":null});
        sqlx::query("UPDATE f_i_payment_orders SET status=$1,paid_at='2026-10-13T12:00:00Z' WHERE order_no='legacyv3paid'").bind(if scenario=="refunded"{"refunded"}else{"paid"}).execute(&db.pool).await.unwrap();
        sqlx::query("INSERT INTO f_i_virtual_payment_orders(order_no,open_id,product_id,amount_cents,wx_order_id,grant_before,grant_after) VALUES('legacyv3paid','buyer','test',1,'WXlegacyv3paid',$1,$2)").bind(&before).bind(&after).execute(&db.pool).await.unwrap();
        sqlx::query("UPDATE f_i_users SET membership_tier=$2 WHERE id=$1")
            .bind(db.user)
            .bind(if scenario == "manual" { "V9" } else { "V3" })
            .execute(&db.pool)
            .await
            .unwrap();
        if scenario == "successor" {
            db.repos()
                .0
                .create_order(NewPaymentOrder {
                    order_no: "legacysuccessor".into(),
                    user_id: db.user,
                    amount_cents: 3900,
                    product_type: "membership:V6".into(),
                })
                .await
                .unwrap();
            sqlx::query("UPDATE f_i_users SET membership_tier='V6',membership_expires_at='2027-10-13T13:00:00Z' WHERE id=$1").bind(db.user).execute(&db.pool).await.unwrap();
            let successor = db.membership().await;
            sqlx::query("UPDATE f_i_payment_orders SET status='paid',paid_at='2026-10-13T13:00:00Z' WHERE order_no='legacysuccessor'").execute(&db.pool).await.unwrap();
            sqlx::query("INSERT INTO f_i_virtual_payment_orders(order_no,open_id,product_id,amount_cents,wx_order_id,grant_before,grant_after) VALUES('legacysuccessor','buyer','fi_v6_year_39',3900,'WXlegacysuccessor',$1,$2)").bind(&after).bind(successor).execute(&db.pool).await.unwrap();
        }
        let current = db.membership().await;
        sqlx::raw_sql(include_str!(
            "../migrations/20261013230000_add_membership_duration.sql"
        ))
        .execute(&db.pool)
        .await
        .unwrap();
        sqlx::raw_sql(include_str!(
            "../migrations/20261013230100_backfill_paid_v3_expiration.sql"
        ))
        .execute(&db.pool)
        .await
        .unwrap();
        if scenario == "original" {
            let correct: bool = sqlx::query_scalar("SELECT membership_expires_at='2027-10-13T12:00:00Z'::timestamptz FROM f_i_users WHERE id=$1").bind(db.user).fetch_one(&db.pool).await.unwrap();
            assert!(correct, "use original paid time, not migration time");
        } else {
            assert_eq!(
                db.membership().await,
                current,
                "preserve later or refunded membership state"
            );
        }
        if scenario == "successor" {
            db.refund("legacysuccessor", "legacysuccessorrefund", 3900)
                .await
                .unwrap();
            let correct: bool = sqlx::query_scalar("SELECT membership_tier='V3' AND membership_expires_at='2027-10-13T12:00:00Z'::timestamptz FROM f_i_users WHERE id=$1").bind(db.user).fetch_one(&db.pool).await.unwrap();
            assert!(correct, "refund successor restores the repaired V3 expiry");
        }
        if scenario != "refunded" {
            db.refund("legacyv3paid", "legacyv3refund", 1)
                .await
                .unwrap();
            assert_eq!(
                db.membership().await["tier"],
                if scenario == "manual" { "V9" } else { "V1" }
            );
            assert!(db.membership().await["expires_at"].is_null());
        }
        db.close().await;
    }
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn legacy_multiple_v3_orders_keep_latest_expiry_and_chronological_refund_chain() {
    let db = Database::with_duration(false).await;
    let initial = serde_json::json!({"tier":"V1","expires_at":null});
    let legacy = serde_json::json!({"tier":"V3","expires_at":null});
    for (order, paid_at, before) in [
        ("legacyv3first", "2026-10-13T12:00:00Z", &initial),
        ("legacyv3last", "2026-10-13T13:00:00Z", &legacy),
    ] {
        db.repos()
            .0
            .create_order(NewPaymentOrder {
                order_no: order.into(),
                user_id: db.user,
                amount_cents: 1,
                product_type: "membership:V3".into(),
            })
            .await
            .unwrap();
        sqlx::query(
            "UPDATE f_i_payment_orders SET status='paid',paid_at=$2::timestamptz WHERE order_no=$1",
        )
        .bind(order)
        .bind(paid_at)
        .execute(&db.pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO f_i_virtual_payment_orders(order_no,open_id,product_id,amount_cents,wx_order_id,grant_before,grant_after) VALUES($1,'buyer','test',1,'WX'||$1,$2,$3)")
            .bind(order).bind(before).bind(&legacy).execute(&db.pool).await.unwrap();
    }
    sqlx::query("UPDATE f_i_users SET membership_tier='V3' WHERE id=$1")
        .bind(db.user)
        .execute(&db.pool)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261013230000_add_membership_duration.sql"
    ))
    .execute(&db.pool)
    .await
    .unwrap();
    for _ in 0..2 {
        sqlx::raw_sql(include_str!(
            "../migrations/20261013230100_backfill_paid_v3_expiration.sql"
        ))
        .execute(&db.pool)
        .await
        .unwrap();
        let matches: bool = sqlx::query_scalar("SELECT membership_expires_at='2027-10-13T13:00:00Z'::timestamptz FROM f_i_users WHERE id=$1").bind(db.user).fetch_one(&db.pool).await.unwrap();
        assert!(matches, "latest paid grant wins and backfill is idempotent");
    }
    db.refund("legacyv3last", "legacyv3lastrefund", 1)
        .await
        .unwrap();
    let matches: bool = sqlx::query_scalar("SELECT membership_tier='V3' AND membership_expires_at='2027-10-13T12:00:00Z'::timestamptz FROM f_i_users WHERE id=$1").bind(db.user).fetch_one(&db.pool).await.unwrap();
    assert!(matches, "latest refund restores the earlier repaired grant");
    db.refund("legacyv3first", "legacyv3firstrefund", 1)
        .await
        .unwrap();
    assert_eq!(db.membership().await, initial);
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
        !db.membership().await["expires_at"].is_null(),
        "paid V3 must receive the configured membership expiry"
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
        Self::with_duration(true).await
    }
    async fn with_duration(include_duration: bool) -> Self {
        Self::with_migrations(include_duration, true).await
    }
    async fn with_migrations(include_duration: bool, include_identity: bool) -> Self {
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
        if include_duration {
            sqlx::raw_sql(include_str!(
                "../migrations/20261013230000_add_membership_duration.sql"
            ))
            .execute(&pool)
            .await
            .unwrap();
            sqlx::raw_sql(include_str!(
                "../migrations/20261013230100_backfill_paid_v3_expiration.sql"
            ))
            .execute(&pool)
            .await
            .unwrap();
        }
        if include_identity {
            sqlx::raw_sql(include_str!(
                "../migrations/20261014093000_add_virtual_product_purchase_identity.sql"
            ))
            .execute(&pool)
            .await
            .unwrap();
        }
        sqlx::raw_sql(include_str!(
            "../migrations/20261014110000_add_membership_origin.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
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
                catalog_product_id: None,
                order_no: no.into(),
                open_id: "buyer".into(),
                product_id: product.into(),
                amount_cents: amount,
                membership_duration_months: 12,
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

fn membership_catalog(db:&Database)->football_insight_service_backend_rs::payment::application::get_membership_product::GetMembershipProductUseCase{
    football_insight_service_backend_rs::payment::application::get_membership_product::GetMembershipProductUseCase::new(Arc::new(football_insight_service_backend_rs::system_config::adapters::persistence::postgres_system_config_port::PostgresSystemConfigPort::new(db.pool.clone())),Arc::new(football_insight_service_backend_rs::payment::adapters::persistence::postgres_virtual_product_repository::PostgresVirtualProductRepository::new(db.pool.clone())))
}
#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn private_membership_offers_cover_all_tiers_and_refresh_reassignment_and_enablement() {
    use football_insight_service_backend_rs::payment::adapters::web::dto::MembershipProductResponse;
    use football_insight_service_backend_rs::payment::ports::virtual_payment_port::VirtualProductRepository;
    let db = Database::new().await;
    let catalog = membership_catalog(&db);
    let repo=football_insight_service_backend_rs::payment::adapters::persistence::postgres_virtual_product_repository::PostgresVirtualProductRepository::new(db.pool.clone());
    let other = Uuid::new_v4();
    sqlx::query("INSERT INTO f_i_users(id,wx_open_id,account_identifier,display_name,password_hash) VALUES($1,'other','other','Other','unused')").bind(other).execute(&db.pool).await.unwrap();
    for tier in ["V3", "V6", "V7", "V8", "V9"] {
        sqlx::query("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents,enabled,allowed_user_ids,membership_duration_months) VALUES($1,$2,$3,55,TRUE,$4,2)").bind(format!("membership:{tier}@55")).bind(format!("Private {tier}")).bind(format!("private_{tier}")).bind(vec![db.user]).execute(&db.pool).await.unwrap();
    }
    // Same tier supports multiple mapped amounts; the lowest eligible direct offer is selected.
    sqlx::query("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents,enabled,allowed_user_ids,membership_duration_months) VALUES('membership:V7@99','More expensive V7','private_v7_99',99,TRUE,$1,12)").bind(vec![db.user]).execute(&db.pool).await.unwrap();
    let allowed = repo.private_membership_products(db.user).await.unwrap();
    assert_eq!(allowed.len(), 6);
    let view = catalog
        .with_duration_for_tier(
            catalog.execute_for_user(Some(db.user)).await.unwrap(),
            Some("V1"),
            Some(db.user),
        )
        .await
        .unwrap();
    let response = serde_json::to_value(MembershipProductResponse::from_product_for_tier(
        view,
        Some("V1"),
    ))
    .unwrap();
    for tier in ["V3", "V6", "V7", "V8", "V9"] {
        let option = response["products"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["target_tier"] == tier)
            .unwrap();
        assert_eq!(option["pay_price_cents"], 55);
        assert_eq!(option["price_cents"], 55);
        assert_eq!(option["membership_duration_months"], 2);
        assert_eq!(option["is_private_offer"], true);
        assert!(
            repo.find_enabled(&format!("membership:{tier}@55"), other)
                .await
                .unwrap()
                .is_none()
        );
    }
    assert!(catalog.execute().await.unwrap().find_option("V3").is_none());
    let ordinary = serde_json::to_value(MembershipProductResponse::from_product_for_tier(
        catalog.execute_for_user(Some(other)).await.unwrap(),
        Some("V1"),
    ))
    .unwrap();
    assert!(
        ordinary["products"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["is_private_offer"] == false)
    );
    assert_eq!(ordinary["products"][1]["pay_price_cents"], 5900);
    sqlx::query("UPDATE f_i_virtual_payment_products SET enabled=FALSE WHERE product_key='membership:V7@55'").execute(&db.pool).await.unwrap();
    let fallback = catalog.execute_for_user(Some(db.user)).await.unwrap();
    assert_eq!(
        fallback.find_option("V7").unwrap().private_price_cents,
        Some(99)
    );
    let fallback = catalog
        .with_duration_for_tier(fallback, Some("V6"), Some(db.user))
        .await
        .unwrap();
    assert_eq!(
        fallback
            .find_option("V7")
            .unwrap()
            .membership_duration_months,
        Some(12)
    );
    sqlx::query(
        "UPDATE f_i_virtual_payment_products SET enabled=TRUE WHERE product_key='membership:V7@55'",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    // Reassignment edits the actual data source; old user loses the offer immediately.
    sqlx::query("UPDATE f_i_virtual_payment_products SET allowed_user_ids=$1 WHERE product_key LIKE 'membership:%@55' OR product_key='membership:V7@99'").bind(vec![other]).execute(&db.pool).await.unwrap();
    assert!(
        repo.private_membership_products(db.user)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        repo.private_membership_products(other).await.unwrap().len(),
        6
    );
    assert!(
        catalog
            .execute_for_user(Some(db.user))
            .await
            .unwrap()
            .find_option("V7")
            .unwrap()
            .private_price_cents
            .is_none()
    );
    assert_eq!(
        catalog
            .execute_for_user(Some(other))
            .await
            .unwrap()
            .find_option("V7")
            .unwrap()
            .private_price_cents,
        Some(55)
    );
    let service = catalog_service(&db);
    assert!(
        catalog_checkout(&db, &service, "old_private_buyer", "membership:V7", 55)
            .await
            .is_err()
    );
    db.repos()
        .0
        .create_order(NewPaymentOrder {
            order_no: "new_private_buyer".into(),
            user_id: other,
            amount_cents: 55,
            product_type: "membership:V7".into(),
        })
        .await
        .unwrap();
    let params = service
        .unified_order("new_private_buyer", "test", 55, "other")
        .await
        .unwrap();
    let sign: serde_json::Value =
        serde_json::from_str(&params.virtual_payment.unwrap().sign_data).unwrap();
    assert_eq!(sign["productId"], "private_V7");
    sqlx::query("UPDATE f_i_virtual_payment_products SET enabled=FALSE WHERE cardinality(allowed_user_ids)>0").execute(&db.pool).await.unwrap();
    assert!(
        repo.private_membership_products(other)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        catalog
            .execute_for_user(Some(other))
            .await
            .unwrap()
            .find_option("V7")
            .unwrap()
            .private_price_cents
            .is_none()
    );
    db.close().await;
}
#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn private_upgrade_display_and_created_order_use_the_same_direct_amount_and_key() {
    use football_insight_service_backend_rs::payment::{
        adapters::web::dto::MembershipProductResponse,
        application::create_membership_order::{
            CreateMembershipOrderInput, CreateMembershipOrderUseCase,
        },
    };
    let db = Database::new().await;
    sqlx::query("UPDATE f_i_users SET membership_tier='V6' WHERE id=$1")
        .bind(db.user)
        .execute(&db.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents,enabled,allowed_user_ids,membership_duration_months) VALUES('membership:V7@55','Assigned V7','assigned_v7',55,TRUE,$1,1)").bind(vec![db.user]).execute(&db.pool).await.unwrap();
    let catalog = membership_catalog(&db);
    let products = catalog.execute_for_user(Some(db.user)).await.unwrap();
    let view = catalog
        .with_duration_for_tier(products.clone(), Some("V6"), Some(db.user))
        .await
        .unwrap();
    let response = MembershipProductResponse::from_product_for_tier(view, Some("V6"));
    let option = response
        .products
        .iter()
        .find(|p| p.target_tier == "V7")
        .unwrap();
    assert_eq!(option.pay_price_cents, 55);
    assert_eq!(option.upgrade_fee_cents, 0);
    assert_eq!(option.membership_duration_months, Some(1));
    let use_case = CreateMembershipOrderUseCase::new(
        db.repos().0,
        Arc::new(PostgresAuthRepository::new(db.pool.clone())),
        Arc::new(catalog_service(&db)),
    );
    let output = use_case
        .execute(CreateMembershipOrderInput {
            user_id: db.user,
            target_tier: "V7".into(),
            products: products.products,
        })
        .await
        .unwrap();
    let order = db
        .repos()
        .0
        .find_order_by_no(&output.order_no)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.amount_cents, 55);
    assert_eq!(order.product_type, "membership:V7");
    let snapshot = db.repos().1.find(&output.order_no).await.unwrap().unwrap();
    assert_eq!(snapshot.product_id, "assigned_v7");
    assert_eq!(snapshot.membership_duration_months, 1);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn restricting_standard_sku_does_not_leak_private_metadata_or_allow_unassigned_purchase() {
    use football_insight_service_backend_rs::payment::adapters::web::dto::MembershipProductResponse;
    use football_insight_service_backend_rs::payment::ports::virtual_payment_port::VirtualProductRepository;
    let db = Database::new().await;
    let other = Uuid::new_v4();
    sqlx::query("INSERT INTO f_i_users(id,wx_open_id,account_identifier,display_name,password_hash) VALUES($1,'unassigned','unassigned','Unassigned','unused')").bind(other).execute(&db.pool).await.unwrap();
    sqlx::query("UPDATE f_i_virtual_payment_products SET name='Private standard-price offer',allowed_user_ids=$1 WHERE product_key='membership:V6@3900'").bind(vec![db.user]).execute(&db.pool).await.unwrap();
    let catalog = membership_catalog(&db);
    for user in [None, Some(other)] {
        let view = catalog
            .with_duration_for_tier(
                catalog.execute_for_user(user).await.unwrap(),
                Some("V1"),
                user,
            )
            .await
            .unwrap();
        let response = serde_json::to_value(MembershipProductResponse::from_product_for_tier(
            view,
            Some("V1"),
        ))
        .unwrap();
        let serialized = response.to_string();
        assert!(!serialized.contains("Private standard-price offer"));
        assert!(!serialized.contains(&db.user.to_string()));
        assert!(
            response["products"]
                .as_array()
                .unwrap()
                .iter()
                .all(|p| p["is_private_offer"] == false)
        );
        assert!(response["products"][0]["membership_duration_months"].is_null());
    }
    let repo=football_insight_service_backend_rs::payment::adapters::persistence::postgres_virtual_product_repository::PostgresVirtualProductRepository::new(db.pool.clone());
    assert!(
        repo.find_enabled("membership:V6@3900", other)
            .await
            .unwrap()
            .is_none()
    );
    db.repos()
        .0
        .create_order(NewPaymentOrder {
            order_no: "unassigned_standard_sku".into(),
            user_id: other,
            amount_cents: 3900,
            product_type: "membership:V6".into(),
        })
        .await
        .unwrap();
    let failure = catalog_service(&db)
        .unified_order("unassigned_standard_sku", "test", 3900, "unassigned")
        .await
        .unwrap_err();
    assert!(matches!(
        failure.downcast_ref::<VirtualPaymentError>(),
        Some(VirtualPaymentError::ProductUnavailable)
    ));
    assert!(
        db.repos()
            .1
            .find("unassigned_standard_sku")
            .await
            .unwrap()
            .is_none()
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn catalog_uuid_is_saved_atomically_with_virtual_order_and_never_reassigned() {
    let db = Database::new().await;
    let id: Uuid = sqlx::query_scalar(
        "SELECT id FROM f_i_virtual_payment_products WHERE product_key='membership:V6@3900'",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    let service = catalog_service(&db);
    catalog_checkout(&db, &service, "identity_order", "membership:V6", 3900)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT catalog_product_id FROM f_i_payment_orders WHERE order_no='identity_order'"
        )
        .fetch_one(&db.pool)
        .await
        .unwrap(),
        Some(id)
    );
    assert_eq!(
        db.repos()
            .1
            .find("identity_order")
            .await
            .unwrap()
            .unwrap()
            .catalog_product_id,
        Some(id)
    );
    db.repos()
        .0
        .create_order(NewPaymentOrder {
            order_no: "identity_rollback".into(),
            user_id: db.user,
            amount_cents: 3900,
            product_type: "membership:V6".into(),
        })
        .await
        .unwrap();
    sqlx::query("INSERT INTO f_i_virtual_payment_orders(order_no,open_id,product_id,amount_cents) VALUES('identity_rollback','buyer','existing_snapshot',3900)").execute(&db.pool).await.unwrap();
    let error = db
        .repos()
        .1
        .record(VirtualOrder {
            catalog_product_id: Some(id),
            order_no: "identity_rollback".into(),
            open_id: "buyer".into(),
            product_id: "different_snapshot".into(),
            amount_cents: 3900,
            membership_duration_months: 12,
            wx_order_id: None,
        })
        .await;
    assert!(error.is_err());
    assert!(
        sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT catalog_product_id FROM f_i_payment_orders WHERE order_no='identity_rollback'"
        )
        .fetch_one(&db.pool)
        .await
        .unwrap()
        .is_none()
    );
    let error = db
        .repos()
        .1
        .record(VirtualOrder {
            catalog_product_id: Some(Uuid::new_v4()),
            order_no: "identity_order".into(),
            open_id: "buyer".into(),
            product_id: "different_snapshot".into(),
            amount_cents: 3900,
            membership_duration_months: 12,
            wx_order_id: None,
        })
        .await;
    assert!(error.is_err());
    assert_eq!(
        sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT catalog_product_id FROM f_i_payment_orders WHERE order_no='identity_order'"
        )
        .fetch_one(&db.pool)
        .await
        .unwrap(),
        Some(id)
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable database"]
async fn purchase_identity_backfill_requires_unique_typed_amount_and_matching_virtual_snapshot() {
    let db = Database::with_migrations(true, false).await;
    sqlx::query("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents) VALUES('membership:V7@5900','V7','wechat_v7',5900),('membership:V8@7900','V8','wechat_v8',7900)").execute(&db.pool).await.unwrap();
    sqlx::query("ALTER TABLE f_i_virtual_payment_products DROP CONSTRAINT virtual_product_key_matches_price").execute(&db.pool).await.unwrap();
    sqlx::query("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents) VALUES('membership:V8@07900','ambiguous V8','wechat_v8_duplicate',7900)").execute(&db.pool).await.unwrap();
    for (no, kind, amount) in [
        ("backfill_typed", "membership:V6", 3900),
        ("backfill_virtual", "membership:V7", 5900),
        ("backfill_wrong_id", "membership:V7", 5900),
        ("backfill_wrong_amount", "membership:V6", 3900),
        ("backfill_generic", "membership", 3900),
        ("backfill_missing", "membership:V9", 9900),
        ("backfill_ambiguous", "membership:V8", 7900),
    ] {
        db.repos()
            .0
            .create_order(NewPaymentOrder {
                order_no: no.into(),
                user_id: db.user,
                amount_cents: amount,
                product_type: kind.into(),
            })
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO f_i_virtual_payment_orders(order_no,open_id,product_id,amount_cents) VALUES('backfill_virtual','buyer','wechat_v7',5900),('backfill_wrong_id','buyer','wrong_wechat_id',5900),('backfill_wrong_amount','buyer','fi_v6_year_39',3901)").execute(&db.pool).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261014093000_add_virtual_product_purchase_identity.sql"
    ))
    .execute(&db.pool)
    .await
    .unwrap();
    for (no, key) in [
        ("backfill_typed", "membership:V6@3900"),
        ("backfill_virtual", "membership:V7@5900"),
    ] {
        let matching:bool=sqlx::query_scalar("SELECT p.catalog_product_id=c.id FROM f_i_payment_orders p CROSS JOIN f_i_virtual_payment_products c WHERE p.order_no=$1 AND c.product_key=$2").bind(no).bind(key).fetch_one(&db.pool).await.unwrap();
        assert!(matching);
    }
    for no in [
        "backfill_wrong_id",
        "backfill_wrong_amount",
        "backfill_generic",
        "backfill_missing",
        "backfill_ambiguous",
    ] {
        assert!(
            sqlx::query_scalar::<_, Option<Uuid>>(
                "SELECT catalog_product_id FROM f_i_payment_orders WHERE order_no=$1"
            )
            .bind(no)
            .fetch_one(&db.pool)
            .await
            .unwrap()
            .is_none()
        );
    }
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn membership_origin_follows_actual_grants_and_refund_preserves_referral_origin() {
    let db = Database::new().await;
    let mut tx = db.pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('football.membership_origin','referral',true)")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE f_i_users SET membership_tier='V6',membership_expires_at=NOW()+INTERVAL '1 year' WHERE id=$1").bind(db.user).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    async fn origin(db: &Database) -> String {
        sqlx::query_scalar("SELECT membership_origin FROM f_i_users WHERE id=$1")
            .bind(db.user)
            .fetch_one(&db.pool)
            .await
            .unwrap()
    }
    assert_eq!(origin(&db).await, "referral");
    db.order("SOURCE001", "membership:V7", 5900).await;
    db.pay("SOURCE001").await;
    assert_eq!(origin(&db).await, "purchase");
    db.refund("SOURCE001", "SOURCE_REFUND", 5900).await.unwrap();
    assert_eq!(origin(&db).await, "referral");
    sqlx::query("UPDATE f_i_users SET display_name='new name' WHERE id=$1")
        .bind(db.user)
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(origin(&db).await, "referral");
    sqlx::query("UPDATE f_i_users SET membership_tier='V9' WHERE id=$1")
        .bind(db.user)
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(origin(&db).await, "unknown");
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn historical_origin_backfill_requires_exact_grant_and_keeps_uncertain_members_unknown() {
    let db = Database::new().await;
    sqlx::raw_sql("DROP TRIGGER f_i_users_membership_origin ON f_i_users; DROP FUNCTION f_i_track_membership_origin(); ALTER TABLE f_i_users DROP COLUMN membership_origin; ALTER TABLE f_i_virtual_payment_orders DROP COLUMN grant_before_origin,DROP COLUMN grant_after_origin;").execute(&db.pool).await.unwrap();
    let uncertain = Uuid::new_v4();
    sqlx::query("INSERT INTO f_i_users(id,membership_tier,membership_expires_at) VALUES($1,'V9','2030-01-01T00:00:00Z')").bind(uncertain).execute(&db.pool).await.unwrap();
    sqlx::query("UPDATE f_i_users SET membership_tier='V6',membership_expires_at='2027-10-14T01:00:00Z' WHERE id=$1").bind(db.user).execute(&db.pool).await.unwrap();
    sqlx::query("INSERT INTO f_i_payment_orders(order_no,user_id,product_type,amount_cents,status,paid_at) VALUES('HISTORY_EXACT',$1,'membership:V6',3900,'paid','2026-10-14T01:00:00Z'),('HISTORY_UNCERTAIN',$2,'membership:V9',9900,'paid','2026-10-14T01:00:00Z')").bind(db.user).bind(uncertain).execute(&db.pool).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261014110000_add_membership_origin.sql"
    ))
    .execute(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT membership_origin FROM f_i_users WHERE id=$1")
            .bind(db.user)
            .fetch_one(&db.pool)
            .await
            .unwrap(),
        "purchase"
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT membership_origin FROM f_i_users WHERE id=$1")
            .bind(uncertain)
            .fetch_one(&db.pool)
            .await
            .unwrap(),
        "unknown"
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn refund_preserves_later_unknown_origin_even_when_admin_restores_same_tier_and_expiry() {
    let db = Database::new().await;
    db.order("ORIGIN_ADMIN", "membership:V6", 3900).await;
    db.pay("ORIGIN_ADMIN").await;
    let granted = db.membership().await;
    sqlx::query("UPDATE f_i_users SET membership_expires_at=membership_expires_at+INTERVAL '1 day' WHERE id=$1").bind(db.user).execute(&db.pool).await.unwrap();
    sqlx::query("UPDATE f_i_users SET membership_expires_at=$2::text::timestamptz WHERE id=$1")
        .bind(db.user)
        .bind(granted["expires_at"].as_str())
        .execute(&db.pool)
        .await
        .unwrap();
    db.refund("ORIGIN_ADMIN", "ORIGIN_ADMIN_REFUND", 3900)
        .await
        .unwrap();
    assert_eq!(db.membership().await, granted);
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT membership_origin FROM f_i_users WHERE id=$1")
            .bind(db.user)
            .fetch_one(&db.pool)
            .await
            .unwrap(),
        "unknown"
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL database"]
async fn real_referral_registration_records_origin_and_successive_refunds_restore_it() {
    use football_insight_service_backend_rs::auth::ports::auth_repository::AuthRepository;
    let db = Database::new().await;
    sqlx::raw_sql("ALTER TABLE f_i_system_configs ALTER COLUMN config_value TYPE TEXT USING config_value::text; ALTER TABLE f_i_users ADD COLUMN avatar_url TEXT, ADD COLUMN official_wechat_open_id TEXT; CREATE TABLE f_i_invite_codes(code TEXT PRIMARY KEY,is_active BOOLEAN DEFAULT TRUE,used_by_user_id UUID,used_at TIMESTAMPTZ,issued_for_wechat_open_id TEXT); CREATE TABLE f_i_wechat_followers(open_id TEXT PRIMARY KEY,unsubscribed_at TIMESTAMPTZ);").execute(&db.pool).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20260411103000_add_user_referrals.sql"
    ))
    .execute(&db.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO f_i_invite_codes(code,used_by_user_id,issued_for_wechat_open_id) VALUES('ORIGIN_REFERRER',$1,'referrer_follower'),('ORIGIN_INVITE',NULL,'new_follower')").bind(db.user).execute(&db.pool).await.unwrap();
    sqlx::query("INSERT INTO f_i_system_configs(config_key,config_value) VALUES('membership_tier_rules',$1)").bind(serde_json::json!([{"code":"V6","kind":"referral","min_referrals":1,"ticket_watch_poll_interval_seconds":15}]).to_string()).execute(&db.pool).await.unwrap();
    PostgresAuthRepository::new(db.pool.clone())
        .create_user_with_invite_with_referral(
            "ORIGIN_INVITE",
            Some("ORIGIN_REFERRER"),
            "origin_registered",
            "unused",
        )
        .await
        .unwrap();
    let origin: String = sqlx::query_scalar("SELECT membership_origin FROM f_i_users WHERE id=$1")
        .bind(db.user)
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(origin, "referral");
    let before = db.membership().await;
    db.order("REFERRAL_A", "membership:V7", 5900).await;
    db.pay("REFERRAL_A").await;
    db.order("REFERRAL_B", "membership:V8", 7900).await;
    db.pay("REFERRAL_B").await;
    db.refund("REFERRAL_A", "REFERRAL_A_REFUND", 5900)
        .await
        .unwrap();
    db.refund("REFERRAL_B", "REFERRAL_B_REFUND", 7900)
        .await
        .unwrap();
    assert_eq!(db.membership().await, before);
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT membership_origin FROM f_i_users WHERE id=$1")
            .bind(db.user)
            .fetch_one(&db.pool)
            .await
            .unwrap(),
        "referral"
    );
    db.close().await;
}
