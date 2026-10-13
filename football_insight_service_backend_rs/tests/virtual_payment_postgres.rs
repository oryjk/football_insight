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
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::sync::Arc;
use uuid::Uuid;

struct Database {
    admin: PgPool,
    pool: PgPool,
    schema: String,
    user: Uuid,
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
