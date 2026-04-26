use sqlx::{PgPool, Row};

use crate::payment::{
    domain::order::{NewPaymentOrder, OrderStatus, PaymentOrder},
    ports::order_repository::OrderRepository,
};

pub struct PostgresOrderRepository {
    pool: PgPool,
}

impl PostgresOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OrderRepository for PostgresOrderRepository {
    async fn create_order(&self, order: NewPaymentOrder) -> anyhow::Result<PaymentOrder> {
        let row = sqlx::query(
            r#"
            INSERT INTO f_i_payment_orders (order_no, user_id, amount_cents, product_type)
            VALUES ($1, $2, $3, $4)
            RETURNING id, order_no, user_id, amount_cents, status, prepay_id, transaction_id, product_type, paid_at, created_at, updated_at
            "#,
        )
        .bind(&order.order_no)
        .bind(order.user_id)
        .bind(order.amount_cents)
        .bind(&order.product_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(map_row_to_order(&row))
    }

    async fn find_order_by_no(&self, order_no: &str) -> anyhow::Result<Option<PaymentOrder>> {
        let row = sqlx::query(
            r#"
            SELECT id, order_no, user_id, amount_cents, status, prepay_id, transaction_id, product_type, paid_at, created_at, updated_at
            FROM f_i_payment_orders
            WHERE order_no = $1
            LIMIT 1
            "#,
        )
        .bind(order_no)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| map_row_to_order(&r)))
    }
}

fn map_row_to_order(row: &sqlx::postgres::PgRow) -> PaymentOrder {
    PaymentOrder {
        id: row.get("id"),
        order_no: row.get("order_no"),
        user_id: row.get("user_id"),
        amount_cents: row.get("amount_cents"),
        status: OrderStatus::from_str(row.get::<String, _>("status").as_str()),
        prepay_id: row.get("prepay_id"),
        transaction_id: row.get("transaction_id"),
        product_type: row.get("product_type"),
        paid_at: row.get("paid_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
