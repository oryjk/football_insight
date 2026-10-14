use crate::payment::{
    domain::virtual_payment::{Delivery, Refund, VirtualOrder, VirtualPaymentError},
    ports::virtual_payment_port::VirtualOrderRepository,
};
use sqlx::{PgPool, Row};

pub struct PostgresVirtualOrderRepository {
    pool: PgPool,
}
impl PostgresVirtualOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
fn map(row: &sqlx::postgres::PgRow) -> VirtualOrder {
    VirtualOrder {
        catalog_product_id: row.get("catalog_product_id"),
        order_no: row.get("order_no"),
        open_id: row.get("open_id"),
        product_id: row.get("product_id"),
        amount_cents: row.get("amount_cents"),
        membership_duration_months: row.get("membership_duration_months"),
        wx_order_id: row.get("wx_order_id"),
    }
}

#[async_trait::async_trait]
impl VirtualOrderRepository for PostgresVirtualOrderRepository {
    async fn record(&self, order: VirtualOrder) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        let existing=sqlx::query_as::<_,(Option<uuid::Uuid>,i32)>("SELECT catalog_product_id,amount_cents FROM f_i_payment_orders WHERE order_no=$1 FOR UPDATE")
            .bind(&order.order_no).fetch_optional(&mut *tx).await?.ok_or(VirtualPaymentError::InvalidOrder)?;
        if existing.1 != order.amount_cents {
            return Err(VirtualPaymentError::InvalidOrder.into());
        }
        if let Some(id) = order.catalog_product_id {
            if existing.0.is_some_and(|existing| existing != id) {
                return Err(VirtualPaymentError::InvalidOrder.into());
            }
            sqlx::query("UPDATE f_i_payment_orders SET catalog_product_id=$2 WHERE order_no=$1 AND catalog_product_id IS NULL").bind(&order.order_no).bind(id).execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO f_i_virtual_payment_orders(order_no,open_id,product_id,amount_cents,membership_duration_months) VALUES($1,$2,$3,$4,$5)")
            .bind(order.order_no).bind(order.open_id).bind(order.product_id).bind(order.amount_cents).bind(order.membership_duration_months).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
    async fn find(&self, order_no: &str) -> anyhow::Result<Option<VirtualOrder>> {
        Ok(sqlx::query("SELECT v.order_no,v.open_id,v.product_id,v.amount_cents,v.wx_order_id,v.membership_duration_months,p.catalog_product_id FROM f_i_virtual_payment_orders v JOIN f_i_payment_orders p ON p.order_no=v.order_no WHERE v.order_no=$1").bind(order_no).fetch_optional(&self.pool).await?.as_ref().map(map))
    }
    async fn record_delivery(&self, delivery: &Delivery) -> anyhow::Result<()> {
        let result = sqlx::query("UPDATE f_i_virtual_payment_orders SET wx_order_id=$2 WHERE order_no=$1 AND (wx_order_id IS NULL OR wx_order_id=$2)")
            .bind(&delivery.order_no).bind(&delivery.wx_order_id).execute(&self.pool).await?;
        if result.rows_affected() != 1 {
            return Err(VirtualPaymentError::InvalidOrder.into());
        }
        Ok(())
    }
    async fn acknowledge_delivery(&self, order_no: &str) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE f_i_virtual_payment_orders SET delivery_acknowledged=TRUE WHERE order_no=$1",
        )
        .bind(order_no)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn refund(&self, refund: &Refund) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        let order = sqlx::query("SELECT user_id,product_type,status,amount_cents FROM f_i_payment_orders WHERE order_no=$1 FOR UPDATE")
            .bind(&refund.order_no).fetch_one(&mut *tx).await?;
        let user_id: uuid::Uuid = order.get("user_id");
        if order
            .get::<String, _>("product_type")
            .starts_with("membership:")
        {
            // All grants/refunds take the user lock before metadata; this avoids cross-order deadlocks.
            sqlx::query("SELECT id FROM f_i_users WHERE id=$1 FOR UPDATE")
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?;
        }
        let metadata = sqlx::query("SELECT open_id,wx_order_id,grant_before,grant_after,grant_before_origin,grant_after_origin FROM f_i_virtual_payment_orders WHERE order_no=$1 FOR UPDATE")
            .bind(&refund.order_no).fetch_one(&mut *tx).await?;
        if metadata.get::<String, _>("open_id") != refund.open_id
            || metadata
                .get::<Option<String>, _>("wx_order_id")
                .as_ref()
                .is_some_and(|id| id != &refund.wx_order_id)
            || refund.wx_order_id.is_empty()
        {
            return Err(VirtualPaymentError::InvalidOrder.into());
        }
        let existing = sqlx::query(
            "SELECT order_no,amount_cents FROM f_i_virtual_payment_refunds WHERE refund_id=$1",
        )
        .bind(&refund.refund_id)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(existing) = existing {
            if existing.get::<String, _>("order_no") != refund.order_no
                || existing.get::<i32, _>("amount_cents") != refund.amount_cents
            {
                return Err(VirtualPaymentError::InvalidOrder.into());
            }
            tx.commit().await?;
            return Ok(());
        }
        let was_pending = order.get::<String, _>("status") == "pending";
        if !matches!(
            order.get::<String, _>("status").as_str(),
            "pending" | "paid" | "refunded"
        ) {
            return Err(VirtualPaymentError::InvalidOrder.into());
        }
        sqlx::query("UPDATE f_i_virtual_payment_orders SET wx_order_id=$2 WHERE order_no=$1 AND (wx_order_id IS NULL OR wx_order_id=$2)").bind(&refund.order_no).bind(&refund.wx_order_id).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO f_i_virtual_payment_refunds(refund_id,order_no,amount_cents) VALUES($1,$2,$3)")
            .bind(&refund.refund_id).bind(&refund.order_no).bind(refund.amount_cents).execute(&mut *tx).await?;
        let total: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(amount_cents),0)::bigint FROM f_i_virtual_payment_refunds WHERE order_no=$1")
            .bind(&refund.order_no).fetch_one(&mut *tx).await?;
        let amount = i64::from(order.get::<i32, _>("amount_cents"));
        if total > amount {
            return Err(VirtualPaymentError::InvalidOrder.into());
        }
        if total == amount || was_pending {
            let user_id: uuid::Uuid = order.get("user_id");
            let product_type: String = order.get("product_type");
            if was_pending
                || (order.get::<String, _>("status") == "refunded"
                    && metadata
                        .get::<Option<serde_json::Value>, _>("grant_before")
                        .is_none())
            {
                // No entitlement was delivered. Any successful refund prevents later delivery.
            } else if product_type.starts_with("membership:") {
                let before: Option<serde_json::Value> = metadata.get("grant_before");
                let after: Option<serde_json::Value> = metadata.get("grant_after");
                let (before, after) = before.zip(after).ok_or(VirtualPaymentError::InvalidOrder)?;
                // Lock the user: preserve any later purchase/admin adjustment that differs from our grant.
                let current: serde_json::Value = sqlx::query_scalar("SELECT jsonb_build_object('tier',membership_tier,'expires_at',membership_expires_at) FROM f_i_users WHERE id=$1 FOR UPDATE")
                    .bind(user_id).fetch_one(&mut *tx).await?;
                let current_origin: String =
                    sqlx::query_scalar("SELECT membership_origin FROM f_i_users WHERE id=$1")
                        .bind(user_id)
                        .fetch_one(&mut *tx)
                        .await?;
                let after_origin: String = metadata.get("grant_after_origin");
                if current == after && current_origin == after_origin {
                    let origin: String = metadata.get("grant_before_origin");
                    sqlx::query("SELECT set_config('football.membership_origin',$1,true)")
                        .bind(&origin)
                        .execute(&mut *tx)
                        .await?;
                    sqlx::query("UPDATE f_i_users SET membership_tier=$2,membership_expires_at=$3::text::timestamptz,updated_at=NOW() WHERE id=$1")
                        .bind(user_id).bind(before["tier"].as_str().ok_or(VirtualPaymentError::InvalidOrder)?)
                        .bind(before["expires_at"].as_str()).execute(&mut *tx).await?;
                    sqlx::query("SELECT set_config('football.membership_origin','',true)")
                        .execute(&mut *tx)
                        .await?;
                }
                // Remove this grant from a successor's rollback chain, preventing later refunds resurrecting it.
                sqlx::query("UPDATE f_i_virtual_payment_orders v SET grant_before=$2,grant_before_origin=$4 WHERE v.grant_before=$3 AND v.grant_before_origin=$5 AND EXISTS(SELECT 1 FROM f_i_payment_orders p WHERE p.order_no=v.order_no AND p.user_id=$1 AND p.status='paid')")
                    .bind(user_id).bind(&before).bind(&after).bind(metadata.get::<String,_>("grant_before_origin")).bind(after_origin).execute(&mut *tx).await?;
            } else if product_type.starts_with("reflux_subscription:") {
                sqlx::query(
                    "UPDATE f_i_user_reflux_subscriptions SET status='cancelled' WHERE order_no=$1",
                )
                .bind(&refund.order_no)
                .execute(&mut *tx)
                .await?;
            } else if product_type.starts_with("match_id_unlock:") {
                sqlx::query("DELETE FROM f_i_user_match_id_unlocks WHERE order_no=$1")
                    .bind(&refund.order_no)
                    .execute(&mut *tx)
                    .await?;
                if let Some(match_id) =
                    crate::payment::domain::order::parse_match_id_unlock_product_type(&product_type)
                {
                    sqlx::query("INSERT INTO f_i_user_match_id_unlocks(user_id,match_id,order_no) SELECT user_id,$2,order_no FROM f_i_payment_orders WHERE user_id=$1 AND product_type=$3 AND status='paid' AND order_no<>$4 ORDER BY paid_at DESC LIMIT 1 ON CONFLICT(user_id,match_id) DO NOTHING")
                        .bind(user_id).bind(match_id).bind(&product_type).bind(&refund.order_no).execute(&mut *tx).await?;
                }
            } else {
                return Err(VirtualPaymentError::InvalidOrder.into());
            }
            sqlx::query("UPDATE f_i_payment_orders SET status='refunded',updated_at=NOW() WHERE order_no=$1").bind(&refund.order_no).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
    async fn pending_orders(&self) -> anyhow::Result<Vec<VirtualOrder>> {
        let rows = sqlx::query("WITH candidates AS (SELECT v.order_no FROM f_i_virtual_payment_orders v JOIN f_i_payment_orders p USING(order_no) WHERE p.status IN ('pending','paid') AND NOT v.delivery_acknowledged AND v.created_at>NOW()-INTERVAL '30 days' AND (v.last_checked_at IS NULL OR v.last_checked_at<NOW()-INTERVAL '60 seconds') ORDER BY v.last_checked_at NULLS FIRST,v.created_at LIMIT 20) UPDATE f_i_virtual_payment_orders v SET last_checked_at=NOW() FROM candidates c WHERE v.order_no=c.order_no RETURNING v.order_no,v.open_id,v.product_id,v.amount_cents,v.wx_order_id,v.membership_duration_months,(SELECT p.catalog_product_id FROM f_i_payment_orders p WHERE p.order_no=v.order_no) AS catalog_product_id")
            .fetch_all(&self.pool).await?;
        // The pending query returns metadata snapshots, including duration, for reconciliation.
        Ok(rows.iter().map(map).collect())
    }
}

pub async fn snapshot_membership(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    order_no: &str,
    user_id: uuid::Uuid,
    before: bool,
) -> anyhow::Result<()> {
    let snapshot: serde_json::Value = sqlx::query_scalar("SELECT jsonb_build_object('tier',membership_tier,'expires_at',membership_expires_at) FROM f_i_users WHERE id=$1 FOR UPDATE")
        .bind(user_id).fetch_one(&mut **tx).await?;
    let query = if before {
        "UPDATE f_i_virtual_payment_orders SET grant_before=$2,grant_before_origin=(SELECT membership_origin FROM f_i_users WHERE id=$3) WHERE order_no=$1"
    } else {
        "UPDATE f_i_virtual_payment_orders SET grant_after=$2,grant_after_origin=(SELECT membership_origin FROM f_i_users WHERE id=$3) WHERE order_no=$1"
    };
    sqlx::query(query)
        .bind(order_no)
        .bind(snapshot)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
