use async_trait::async_trait;
use sqlx::{PgPool, Row, postgres::PgRow};
use uuid::Uuid;

use crate::admin::{
    domain::admin_review_exempt::{AdminReviewExemptError, ReviewExemptUser},
    ports::admin_review_exempt_repository::AdminReviewExemptRepository,
};

pub struct PostgresAdminReviewExemptRepository {
    pool: PgPool,
}

impl PostgresAdminReviewExemptRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const SELECT_EXEMPT: &str = r#"
SELECT e.user_id, u.display_name, u.account_identifier, u.avatar_url, e.created_at
FROM f_i_review_exempt_users e
JOIN f_i_users u ON u.id = e.user_id
"#;

fn map_error(error: sqlx::Error) -> AdminReviewExemptError {
    tracing::error!(%error, "admin review exempt persistence failed");
    AdminReviewExemptError::Internal
}

fn read(row: &PgRow) -> Result<ReviewExemptUser, sqlx::Error> {
    Ok(ReviewExemptUser {
        user_id: row.try_get("user_id")?,
        display_name: row.try_get("display_name")?,
        account_identifier: row.try_get("account_identifier")?,
        avatar_url: row.try_get("avatar_url")?,
        created_at: row.try_get("created_at")?,
    })
}

async fn audit(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    admin_id: Uuid,
    action: &str,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO f_i_admin_audit_logs(id,admin_user_id,action,target_type,target_id,before_json,after_json) \
         VALUES($1,$2,$3,'user',$4,$5,$6)",
    )
    .bind(Uuid::new_v4())
    .bind(admin_id)
    .bind(action)
    .bind(user_id.to_string())
    .bind(serde_json::json!({"review_exempt": action == "review_exempt.remove"}))
    .bind(serde_json::json!({"review_exempt": action == "review_exempt.add"}))
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[async_trait]
impl AdminReviewExemptRepository for PostgresAdminReviewExemptRepository {
    async fn list(&self) -> Result<Vec<ReviewExemptUser>, AdminReviewExemptError> {
        let rows = sqlx::query(&format!("{SELECT_EXEMPT} ORDER BY e.created_at DESC"))
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.iter()
            .map(read)
            .collect::<Result<_, _>>()
            .map_err(map_error)
    }

    async fn add(
        &self,
        user_id: Uuid,
        admin_id: Uuid,
    ) -> Result<ReviewExemptUser, AdminReviewExemptError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM f_i_users WHERE id = $1)")
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_error)?;
        if !exists {
            return Err(AdminReviewExemptError::NotFound);
        }
        let inserted = sqlx::query(
            "INSERT INTO f_i_review_exempt_users(user_id, created_by) VALUES($1,$2) ON CONFLICT (user_id) DO NOTHING",
        )
        .bind(user_id)
        .bind(admin_id)
        .execute(&mut *tx)
        .await
        .map_err(map_error)?
        .rows_affected();
        if inserted > 0 {
            audit(&mut tx, admin_id, "review_exempt.add", user_id)
                .await
                .map_err(map_error)?;
        }
        let row = sqlx::query(&format!("{SELECT_EXEMPT} WHERE e.user_id = $1"))
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_error)?;
        let item = read(&row).map_err(map_error)?;
        tx.commit().await.map_err(map_error)?;
        Ok(item)
    }

    async fn remove(&self, user_id: Uuid, admin_id: Uuid) -> Result<(), AdminReviewExemptError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let removed = sqlx::query("DELETE FROM f_i_review_exempt_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(map_error)?
            .rows_affected();
        if removed > 0 {
            audit(&mut tx, admin_id, "review_exempt.remove", user_id)
                .await
                .map_err(map_error)?;
        }
        tx.commit().await.map_err(map_error)?;
        Ok(())
    }
}
