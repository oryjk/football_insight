use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::activity::ports::user_activity_repository::UserActivityRepository;

#[derive(Clone)]
pub struct PostgresUserActivityRepository {
    pool: PgPool,
}

impl PostgresUserActivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserActivityRepository for PostgresUserActivityRepository {
    async fn record_page_activity(&self, user_id: Uuid, page_key: &str) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO f_i_user_activity_snapshots (
                user_id,
                last_active_at,
                last_active_page_key
            ) VALUES ($1, NOW(), $2)
            ON CONFLICT (user_id) DO UPDATE
               SET last_active_at = NOW(),
                   last_active_page_key = EXCLUDED.last_active_page_key,
                   updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(page_key)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO f_i_user_page_activity_snapshots (
                user_id,
                page_key,
                last_seen_at,
                visit_count
            ) VALUES ($1, $2, NOW(), 1)
            ON CONFLICT (user_id, page_key) DO UPDATE
               SET last_seen_at = NOW(),
                   visit_count = f_i_user_page_activity_snapshots.visit_count + 1,
                   updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(page_key)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
