use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::admin::{
    domain::admin_user::{
        AdminCreateUserInput, AdminInviter, AdminUpdateUserInput, AdminUser, AdminUserList,
        AdminUserSearch,
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
        let display_name_pattern = search
            .display_name
            .as_ref()
            .map(|value| format!("%{}%", value));

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
              FROM f_i_users
             WHERE status = 'active'
               AND ($1::TEXT IS NULL OR display_name ILIKE $1)
            "#,
        )
        .bind(display_name_pattern.as_deref())
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
             WHERE users.status = 'active'
               AND ($1::TEXT IS NULL OR users.display_name ILIKE $1)
             ORDER BY users.created_at DESC, users.id DESC
             LIMIT $2 OFFSET $3
            "#,
        )
        .bind(display_name_pattern.as_deref())
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
}

fn map_unique_user_error(error: sqlx::Error) -> anyhow::Error {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.is_unique_violation() {
            return anyhow::anyhow!("account identifier already exists");
        }
    }

    error.into()
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
