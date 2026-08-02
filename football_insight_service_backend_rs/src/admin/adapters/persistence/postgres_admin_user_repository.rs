use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::admin::{
    domain::admin_user::{
        AdminCreateUserInput, AdminInviter, AdminMembershipAdjustment, AdminPaymentOrder,
        AdminReferredUser, AdminSubscription, AdminUpdateUserInput, AdminUser, AdminUserActivity,
        AdminUserDetail, AdminUserDevice, AdminUserList, AdminUserSearch,
    },
    ports::admin_user_repository::AdminUserRepository,
};

#[derive(Clone)]
pub struct PostgresAdminUserRepository {
    pool: PgPool,
}

impl PostgresAdminUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct AdminUserRow {
    id: Uuid,
    account_identifier: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
    has_wechat_binding: bool,
    status: String,
    invite_code: Option<String>,
    invited_by_user_id: Option<Uuid>,
    invited_by_display_name: Option<String>,
    invited_by_account_identifier: Option<String>,
    referral_invite_code: Option<String>,
    membership_tier: String,
    membership_expires_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct AdminReferredUserRow {
    id: Uuid,
    account_identifier: String,
    display_name: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct AdminUserActivityRow {
    last_login_at: Option<DateTime<Utc>>,
    last_active_at: Option<DateTime<Utc>>,
    last_active_page_key: Option<String>,
}

#[derive(Debug, FromRow)]
struct AdminPaymentOrderRow {
    order_no: String,
    amount_cents: i32,
    status: String,
    product_type: String,
    paid_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct AdminSubscriptionRow {
    id: Uuid,
    plan_code: String,
    scope: String,
    team_code: String,
    season: Option<i32>,
    match_id: Option<i64>,
    status: String,
    starts_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow)]
struct AdminUserDeviceRow {
    id: i64,
    device_token: String,
    platform: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<AdminUserRow> for AdminUser {
    fn from(value: AdminUserRow) -> Self {
        Self {
            id: value.id,
            account_identifier: value.account_identifier,
            display_name: value.display_name,
            avatar_url: value.avatar_url,
            has_wechat_binding: value.has_wechat_binding,
            status: value.status,
            invite_code: value.invite_code,
            invited_by: match (
                value.invited_by_user_id,
                value.invited_by_account_identifier,
                value.referral_invite_code,
            ) {
                (Some(id), Some(account_identifier), Some(referral_invite_code)) => {
                    Some(AdminInviter {
                        id,
                        display_name: value.invited_by_display_name,
                        account_identifier,
                        referral_invite_code,
                    })
                }
                _ => None,
            },
            membership_tier: value.membership_tier,
            membership_expires_at: value.membership_expires_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[async_trait]
impl AdminUserRepository for PostgresAdminUserRepository {
    async fn list_users(&self, search: AdminUserSearch) -> anyhow::Result<AdminUserList> {
        let offset = (search.page - 1) * search.page_size;
        let query_pattern = search.query.as_ref().map(|value| format!("%{}%", value));

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
              FROM f_i_users
             WHERE ($1::TEXT IS NULL OR display_name ILIKE $1 OR account_identifier ILIKE $1)
               AND ($2::TEXT IS NULL OR status = $2)
               AND ($3::TEXT IS NULL OR membership_tier = $3)
            "#,
        )
        .bind(query_pattern.as_deref())
        .bind(search.status.as_deref())
        .bind(search.membership_tier.as_deref())
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query_as::<_, AdminUserRow>(
            r#"
            SELECT users.id,
                   users.account_identifier,
                   users.display_name,
                   users.avatar_url,
                   (users.wx_open_id IS NOT NULL) AS has_wechat_binding,
                   users.status,
                   own_invite.code AS invite_code,
                   referrer.id AS invited_by_user_id,
                   referrer.display_name AS invited_by_display_name,
                   referrer.account_identifier AS invited_by_account_identifier,
                   referrals.referral_invite_code,
                   users.membership_tier,
                   users.membership_expires_at,
                   users.created_at,
                   users.updated_at
              FROM f_i_users AS users
         LEFT JOIN LATERAL (
                   SELECT invite_codes.code
                     FROM f_i_invite_codes AS invite_codes
                    WHERE invite_codes.used_by_user_id = users.id
                    ORDER BY invite_codes.used_at DESC NULLS LAST,
                             invite_codes.created_at DESC,
                             invite_codes.id DESC
                    LIMIT 1
                   ) AS own_invite ON TRUE
         LEFT JOIN f_i_user_referrals AS referrals
                ON referrals.referred_user_id = users.id
         LEFT JOIN f_i_users AS referrer
                ON referrer.id = referrals.referrer_user_id
             WHERE ($1::TEXT IS NULL OR users.display_name ILIKE $1 OR users.account_identifier ILIKE $1)
               AND ($2::TEXT IS NULL OR users.status = $2)
               AND ($3::TEXT IS NULL OR users.membership_tier = $3)
             ORDER BY users.created_at DESC, users.id DESC
             LIMIT $4 OFFSET $5
            "#,
        )
        .bind(query_pattern.as_deref())
        .bind(search.status.as_deref())
        .bind(search.membership_tier.as_deref())
        .bind(search.page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(AdminUserList {
            total,
            page: search.page,
            page_size: search.page_size,
            items: rows.into_iter().map(Into::into).collect(),
        })
    }

    async fn get_user(&self, user_id: Uuid) -> anyhow::Result<Option<AdminUser>> {
        let row = sqlx::query_as::<_, AdminUserRow>(
            r#"
            SELECT users.id,
                   users.account_identifier,
                   users.display_name,
                   users.avatar_url,
                   (users.wx_open_id IS NOT NULL) AS has_wechat_binding,
                   users.status,
                   own_invite.code AS invite_code,
                   referrer.id AS invited_by_user_id,
                   referrer.display_name AS invited_by_display_name,
                   referrer.account_identifier AS invited_by_account_identifier,
                   referrals.referral_invite_code,
                   users.membership_tier,
                   users.membership_expires_at,
                   users.created_at,
                   users.updated_at
              FROM f_i_users AS users
         LEFT JOIN LATERAL (
                   SELECT invite_codes.code
                     FROM f_i_invite_codes AS invite_codes
                    WHERE invite_codes.used_by_user_id = users.id
                    ORDER BY invite_codes.used_at DESC NULLS LAST,
                             invite_codes.created_at DESC,
                             invite_codes.id DESC
                    LIMIT 1
                   ) AS own_invite ON TRUE
         LEFT JOIN f_i_user_referrals AS referrals
                ON referrals.referred_user_id = users.id
         LEFT JOIN f_i_users AS referrer
                ON referrer.id = referrals.referrer_user_id
             WHERE users.id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn get_user_detail(&self, user_id: Uuid) -> anyhow::Result<Option<AdminUserDetail>> {
        let Some(user) = self.get_user(user_id).await? else {
            return Ok(None);
        };
        let referrals = sqlx::query_as::<_, AdminReferredUserRow>(
            r#"
            SELECT users.id, users.account_identifier, users.display_name, users.status, users.created_at
              FROM f_i_user_referrals AS referrals
              JOIN f_i_users AS users ON users.id = referrals.referred_user_id
             WHERE referrals.referrer_user_id = $1
             ORDER BY users.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| AdminReferredUser {
            id: row.id,
            account_identifier: row.account_identifier,
            display_name: row.display_name,
            status: row.status,
            created_at: row.created_at,
        })
        .collect();
        let activity = sqlx::query_as::<_, AdminUserActivityRow>(
            r#"
            SELECT last_login_at, last_active_at, last_active_page_key
              FROM f_i_user_activity_snapshots
             WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| AdminUserActivity {
            last_login_at: row.last_login_at,
            last_active_at: row.last_active_at,
            last_active_page_key: row.last_active_page_key,
        });
        let orders = sqlx::query_as::<_, AdminPaymentOrderRow>(
            r#"
            SELECT order_no, amount_cents, status, product_type, paid_at, created_at
              FROM f_i_payment_orders
             WHERE user_id = $1
             ORDER BY created_at DESC
             LIMIT 50
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| AdminPaymentOrder {
            order_no: row.order_no,
            amount_cents: row.amount_cents,
            status: row.status,
            product_type: row.product_type,
            paid_at: row.paid_at,
            created_at: row.created_at,
        })
        .collect();
        let subscriptions = sqlx::query_as::<_, AdminSubscriptionRow>(
            r#"
            SELECT id, plan_code, scope, team_code, season, match_id, status, starts_at, expires_at
              FROM f_i_user_reflux_subscriptions
             WHERE user_id = $1
             ORDER BY created_at DESC
             LIMIT 50
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| AdminSubscription {
            id: row.id,
            plan_code: row.plan_code,
            scope: row.scope,
            team_code: row.team_code,
            season: row.season,
            match_id: row.match_id,
            status: row.status,
            starts_at: row.starts_at,
            expires_at: row.expires_at,
        })
        .collect();
        let devices = sqlx::query_as::<_, AdminUserDeviceRow>(
            r#"
            SELECT id, device_token, platform, created_at, updated_at
              FROM f_i_user_device_tokens
             WHERE user_id = $1
             ORDER BY updated_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| AdminUserDevice {
            id: row.id,
            platform: row.platform,
            masked_device_token: mask_device_token(&row.device_token),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect();

        Ok(Some(AdminUserDetail {
            user,
            referrals,
            activity,
            orders,
            subscriptions,
            devices,
        }))
    }

    async fn create_user(
        &self,
        input: AdminCreateUserInput,
        password_hash: String,
    ) -> anyhow::Result<AdminUser> {
        let row = sqlx::query_as::<_, AdminUserRow>(
            r#"
            INSERT INTO f_i_users (
                id,
                account_identifier,
                display_name,
                avatar_url,
                password_hash,
                membership_tier,
                membership_expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id,
                      account_identifier,
                      display_name,
                      avatar_url,
                      (wx_open_id IS NOT NULL) AS has_wechat_binding,
                      status,
                      NULL::TEXT AS invite_code,
                      NULL::UUID AS invited_by_user_id,
                      NULL::TEXT AS invited_by_display_name,
                      NULL::TEXT AS invited_by_account_identifier,
                      NULL::TEXT AS referral_invite_code,
                      membership_tier,
                      membership_expires_at,
                      created_at,
                      updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(input.account_identifier)
        .bind(Some(input.display_name))
        .bind(input.avatar_url)
        .bind(password_hash)
        .bind(input.membership_tier)
        .bind(input.membership_expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(map_unique_user_error)?;

        Ok(row.into())
    }

    async fn update_user(
        &self,
        user_id: Uuid,
        input: AdminUpdateUserInput,
    ) -> anyhow::Result<Option<AdminUser>> {
        let row = sqlx::query_as::<_, AdminUserRow>(
            r#"
            UPDATE f_i_users
               SET account_identifier = COALESCE($2, account_identifier),
                   display_name = COALESCE($3, display_name),
                   avatar_url = CASE WHEN $4 THEN $5 ELSE avatar_url END,
                   membership_tier = COALESCE($6, membership_tier),
                   membership_expires_at = CASE WHEN $7 THEN $8 ELSE membership_expires_at END,
                   updated_at = NOW()
             WHERE id = $1
               AND status = 'active'
            RETURNING id,
                      account_identifier,
                      display_name,
                      avatar_url,
                      (wx_open_id IS NOT NULL) AS has_wechat_binding,
                      status,
                      NULL::TEXT AS invite_code,
                      NULL::UUID AS invited_by_user_id,
                      NULL::TEXT AS invited_by_display_name,
                      NULL::TEXT AS invited_by_account_identifier,
                      NULL::TEXT AS referral_invite_code,
                      membership_tier,
                      membership_expires_at,
                      created_at,
                      updated_at
            "#,
        )
        .bind(user_id)
        .bind(input.account_identifier)
        .bind(input.display_name)
        .bind(input.avatar_url.is_some())
        .bind(input.avatar_url.unwrap_or(None))
        .bind(input.membership_tier)
        .bind(input.membership_expires_at_set)
        .bind(input.membership_expires_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_unique_user_error)?;

        Ok(row.map(Into::into))
    }

    async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            r#"
            UPDATE f_i_users
               SET status = 'disabled',
                   updated_at = NOW()
             WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() > 0 {
            sqlx::query("DELETE FROM f_i_user_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_user_status(
        &self,
        user_id: Uuid,
        status: &str,
        admin_id: Uuid,
        reason: &str,
    ) -> anyhow::Result<Option<AdminUser>> {
        let mut tx = self.pool.begin().await?;
        let before_status = sqlx::query_scalar::<_, String>(
            "SELECT status FROM f_i_users WHERE id = $1 FOR UPDATE",
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(before_status) = before_status else {
            tx.rollback().await?;
            return Ok(None);
        };

        let row = sqlx::query_as::<_, AdminUserRow>(
            r#"
            UPDATE f_i_users
               SET status = $2, updated_at = NOW()
             WHERE id = $1
            RETURNING id,
                      account_identifier,
                      display_name,
                      avatar_url,
                      (wx_open_id IS NOT NULL) AS has_wechat_binding,
                      status,
                      NULL::TEXT AS invite_code,
                      NULL::UUID AS invited_by_user_id,
                      NULL::TEXT AS invited_by_display_name,
                      NULL::TEXT AS invited_by_account_identifier,
                      NULL::TEXT AS referral_invite_code,
                      membership_tier,
                      membership_expires_at,
                      created_at,
                      updated_at
            "#,
        )
        .bind(user_id)
        .bind(status)
        .fetch_one(&mut *tx)
        .await?;

        if status == "disabled" {
            sqlx::query("DELETE FROM f_i_user_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }
        sqlx::query(
            r#"
            INSERT INTO f_i_admin_audit_logs (
                id, admin_user_id, action, target_type, target_id, reason, before_json, after_json
            ) VALUES ($1, $2, $3, 'user', $4, $5, $6, $7)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(admin_id)
        .bind(if status == "disabled" {
            "user.disable"
        } else {
            "user.restore"
        })
        .bind(user_id.to_string())
        .bind(reason)
        .bind(serde_json::json!({"status": before_status}))
        .bind(serde_json::json!({"status": status}))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(Some(row.into()))
    }

    async fn adjust_membership(
        &self,
        user_id: Uuid,
        adjustment: AdminMembershipAdjustment,
        admin_id: Uuid,
    ) -> anyhow::Result<Option<AdminUser>> {
        let mut tx = self.pool.begin().await?;
        let before = sqlx::query_as::<_, (String, Option<DateTime<Utc>>)>(
            r#"
            SELECT membership_tier, membership_expires_at
              FROM f_i_users
             WHERE id = $1
             FOR UPDATE
            "#,
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;
        let Some((before_tier, before_expires_at)) = before else {
            tx.rollback().await?;
            return Ok(None);
        };

        let row = sqlx::query_as::<_, AdminUserRow>(
            r#"
            UPDATE f_i_users
               SET membership_tier = $2,
                   membership_expires_at = CASE WHEN $3 THEN $4 ELSE membership_expires_at END,
                   updated_at = NOW()
             WHERE id = $1
            RETURNING id,
                      account_identifier,
                      display_name,
                      avatar_url,
                      (wx_open_id IS NOT NULL) AS has_wechat_binding,
                      status,
                      NULL::TEXT AS invite_code,
                      NULL::UUID AS invited_by_user_id,
                      NULL::TEXT AS invited_by_display_name,
                      NULL::TEXT AS invited_by_account_identifier,
                      NULL::TEXT AS referral_invite_code,
                      membership_tier,
                      membership_expires_at,
                      created_at,
                      updated_at
            "#,
        )
        .bind(user_id)
        .bind(&adjustment.membership_tier)
        .bind(adjustment.membership_expires_at_set)
        .bind(adjustment.membership_expires_at)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO f_i_admin_audit_logs (
                id, admin_user_id, action, target_type, target_id, reason, before_json, after_json
            ) VALUES ($1, $2, 'user.membership.adjust', 'user', $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(admin_id)
        .bind(user_id.to_string())
        .bind(&adjustment.reason)
        .bind(serde_json::json!({
            "membership_tier": before_tier,
            "membership_expires_at": before_expires_at,
        }))
        .bind(serde_json::json!({
            "membership_tier": row.membership_tier,
            "membership_expires_at": row.membership_expires_at,
        }))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(Some(row.into()))
    }
}

fn map_unique_user_error(error: sqlx::Error) -> anyhow::Error {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.is_unique_violation() {
            return anyhow::anyhow!("account identifier already exists");
        }
    }

    error.into()
}

fn mask_device_token(token: &str) -> String {
    if token.chars().count() <= 8 {
        return "********".to_string();
    }
    let prefix: String = token.chars().take(4).collect();
    let suffix: String = token
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{prefix}...{suffix}")
}

#[cfg(test)]
mod tests {
    #[test]
    fn admin_user_list_query_qualifies_self_joined_user_columns() {
        let source = include_str!("postgres_admin_user_repository.rs");
        let own_invite_marker = source
            .find("own_invite.code AS invite_code")
            .expect("admin user list query selects invite code");
        let list_query_start = source[..own_invite_marker]
            .rfind("SELECT")
            .expect("admin user list query is present");
        let list_query_end = source[list_query_start..]
            .find("\"#,")
            .expect("admin user list query ends")
            + list_query_start;
        let list_query = &source[list_query_start..list_query_end];

        assert!(
            list_query.contains("SELECT users.id,"),
            "list query must qualify f_i_users columns because it joins f_i_users again as referrer"
        );
    }

    #[test]
    fn admin_delete_uses_disabled_status_instead_of_hard_delete() {
        let source = include_str!("postgres_admin_user_repository.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("repository implementation exists");
        let delete_implementation = implementation
            .split("async fn delete_user")
            .nth(1)
            .expect("delete_user implementation exists")
            .split("fn map_unique_user_error")
            .next()
            .expect("delete_user implementation ends before error mapper");

        assert!(!delete_implementation.contains("DELETE FROM f_i_users WHERE id = $1"));
        assert!(delete_implementation.contains("SET status = 'disabled'"));
        assert!(!delete_implementation.contains("AND status = 'active'"));
        assert!(delete_implementation.contains("DELETE FROM f_i_user_sessions WHERE user_id = $1"));
    }
}
