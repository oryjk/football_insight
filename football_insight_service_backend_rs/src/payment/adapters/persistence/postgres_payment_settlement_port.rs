use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    auth::domain::membership::membership_tier_rank,
    auth::domain::user::resolve_effective_membership_tier,
    payment::{
        domain::order::RefluxSubscriptionProductType,
        ports::payment_settlement_port::PaymentSettlementPort,
    },
    reflux_subscription::domain::subscription::normalize_team_code,
};

pub struct PostgresPaymentSettlementPort {
    pool: PgPool,
}

struct MembershipSettlement {
    effective_tier: String,
    should_refresh_paid_expiration: bool,
}

impl PostgresPaymentSettlementPort {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn resolve_membership_settlement(
    current_tier: &str,
    current_expires_at: Option<DateTime<Utc>>,
    paid_tier: &str,
    now: DateTime<Utc>,
) -> MembershipSettlement {
    let current_effective_tier =
        resolve_effective_membership_tier(current_tier, current_expires_at, now);
    let paid_tier_rank = membership_tier_rank(paid_tier);
    let current_effective_tier_rank = membership_tier_rank(&current_effective_tier);
    let is_paid_membership_tier = paid_tier_rank >= membership_tier_rank("V6");
    let should_apply_paid_tier =
        is_paid_membership_tier && paid_tier_rank > current_effective_tier_rank;
    let should_refresh_paid_expiration =
        is_paid_membership_tier && paid_tier_rank >= current_effective_tier_rank;
    let effective_tier = if should_apply_paid_tier {
        paid_tier.to_string()
    } else {
        current_effective_tier
    };

    MembershipSettlement {
        effective_tier,
        should_refresh_paid_expiration,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::resolve_membership_settlement;

    #[test]
    fn paid_membership_refreshes_expiration_even_when_target_tier_matches_current_tier() {
        let now = Utc
            .with_ymd_and_hms(2026, 4, 24, 12, 0, 0)
            .single()
            .expect("valid now");
        let existing_expires_at = Utc
            .with_ymd_and_hms(2026, 5, 1, 12, 0, 0)
            .single()
            .expect("valid expiration");

        let settlement =
            resolve_membership_settlement("V9", Some(existing_expires_at), "V9", now);

        assert_eq!(settlement.effective_tier, "V9");
        assert!(settlement.should_refresh_paid_expiration);
    }

    #[test]
    fn non_paid_membership_tier_does_not_create_expiration() {
        let now = Utc
            .with_ymd_and_hms(2026, 4, 24, 12, 0, 0)
            .single()
            .expect("valid now");

        let settlement = resolve_membership_settlement("V3", None, "V3", now);

        assert_eq!(settlement.effective_tier, "V3");
        assert!(!settlement.should_refresh_paid_expiration);
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
        let (current_tier, current_expires_at) = current_user
            .map(|row| {
                let membership_tier: String = row.get("membership_tier");
                let membership_expires_at: Option<DateTime<Utc>> =
                    row.get("membership_expires_at");

                (membership_tier, membership_expires_at)
            })
            .unwrap_or_else(|| ("V1".to_string(), None));
        let settlement =
            resolve_membership_settlement(&current_tier, current_expires_at, tier, Utc::now());

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
        .bind(&settlement.effective_tier)
        .bind(settlement.should_refresh_paid_expiration)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn settle_reflux_subscription_order(
        &self,
        order_no: &str,
        transaction_id: &str,
        user_id: Uuid,
        product: RefluxSubscriptionProductType,
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

        let team_code = normalize_team_code(&product.team_code);
        let plan = sqlx::query(
            r#"
            SELECT code, scope, season, title, expires_at
              FROM f_i_reflux_subscription_plans
             WHERE code = $1
               AND enabled = TRUE
               AND team_code IN ($2, 'global')
             ORDER BY CASE WHEN team_code = $2 THEN 0 ELSE 1 END,
                      sort_order ASC
             LIMIT 1
            "#,
        )
        .bind(&product.plan_code)
        .bind(&team_code)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("提醒套餐已下架"))?;
        let plan_code: String = plan.get("code");
        let scope: String = plan.get("scope");
        let season: Option<i32> = plan.get("season");
        let title: String = plan.get("title");
        let expires_at: Option<DateTime<Utc>> = plan.get("expires_at");
        let subscription_match_id = if scope == "single_match" {
            product.match_id
        } else {
            None
        };

        let target = sqlx::query(
            r#"
            SELECT id, target
              FROM f_i_user_notification_targets
             WHERE user_id = $1
               AND channel = 'email'
               AND is_active = TRUE
             LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("请先填写通知邮箱"))?;
        let target_id: Uuid = target.get("id");

        sqlx::query(
            r#"
            INSERT INTO f_i_user_reflux_subscriptions (
                user_id, plan_code, scope, team_code, season, match_id,
                order_no, starts_at, expires_at, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), $8, 'active')
            "#,
        )
        .bind(user_id)
        .bind(&plan_code)
        .bind(&scope)
        .bind(&team_code)
        .bind(season)
        .bind(subscription_match_id)
        .bind(order_no)
        .bind(resolve_reflux_subscription_expires_at(
            &scope,
            expires_at,
            Utc::now(),
        ))
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO f_i_reflux_notification_jobs (
                user_id, target_id, team_code, match_id,
                subject, body_html, payload_json, status, next_attempt_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', NOW())
            "#,
        )
        .bind(user_id)
        .bind(target_id)
        .bind(&team_code)
        .bind(subscription_match_id)
        .bind("[回流提醒] 订阅已开通")
        .bind(build_reflux_subscription_welcome_email(&title, expires_at))
        .bind(serde_json::json!({
            "kind": "reflux_subscription_welcome",
            "plan_code": plan_code,
            "order_no": order_no,
        }))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

fn resolve_reflux_subscription_expires_at(
    scope: &str,
    plan_expires_at: Option<DateTime<Utc>>,
    starts_at: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    match scope {
        "single_match" => plan_expires_at.or(Some(starts_at + chrono::Duration::days(7))),
        "season" => plan_expires_at,
        "lifetime" => None,
        _ => plan_expires_at,
    }
}

fn build_reflux_subscription_welcome_email(
    title: &str,
    expires_at: Option<DateTime<Utc>>,
) -> String {
    let expiration = expires_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "长期有效".to_string());

    format!(
        "<p>你的「{}」已经开通。</p><p>有效期：{}</p><p>监控到新增回流后，我们会按分钟聚合发送邮件提醒。</p>",
        title, expiration
    )
}
