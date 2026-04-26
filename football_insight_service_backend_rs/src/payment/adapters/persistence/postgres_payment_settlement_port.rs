use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    auth::domain::membership::membership_tier_rank,
    auth::domain::user::resolve_effective_membership_tier,
    payment::ports::payment_settlement_port::PaymentSettlementPort,
};

pub struct PostgresPaymentSettlementPort {
    pool: PgPool,
}

impl PostgresPaymentSettlementPort {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PaymentSettlementPort for PostgresPaymentSettlementPort {
    async fn settle_membership_order(
        &self,
        order_no: &str,
        transaction_id: &str,
        user_id: Uuid,
        tier: &str,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let order_update = sqlx::query(
            r#"
            UPDATE f_i_payment_orders
            SET status = 'paid',
                transaction_id = COALESCE(transaction_id, $2),
                paid_at = COALESCE(paid_at, NOW()),
                updated_at = NOW()
            WHERE order_no = $1
              AND status = 'pending'
            "#,
        )
        .bind(order_no)
        .bind(transaction_id)
        .execute(&mut *tx)
        .await?;

        if order_update.rows_affected() == 0 {
            tx.commit().await?;
            return Ok(());
        }

        let current_user = sqlx::query(
            "SELECT membership_tier, membership_expires_at FROM f_i_users WHERE id = $1 LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;
        let current_effective_tier = current_user
            .map(|row| {
                let membership_tier: String = row.get("membership_tier");
                let membership_expires_at: Option<DateTime<Utc>> =
                    row.get("membership_expires_at");

                resolve_effective_membership_tier(
                    &membership_tier,
                    membership_expires_at,
                    Utc::now(),
                )
            })
            .unwrap_or_else(|| "V1".to_string());
        let should_apply_paid_tier =
            membership_tier_rank(tier) > membership_tier_rank(&current_effective_tier);
        let effective_tier = if should_apply_paid_tier {
            tier.to_string()
        } else {
            current_effective_tier
        };

        sqlx::query(
            r#"
            UPDATE f_i_users
            SET membership_tier = $2,
                membership_expires_at = CASE
                    WHEN $3 THEN NOW() + INTERVAL '1 year'
                    ELSE membership_expires_at
                END,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(&effective_tier)
        .bind(should_apply_paid_tier)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
