use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::push_notification::domain::device_token::DeviceToken;
use crate::push_notification::ports::device_token_repository::DeviceTokenRepository;

pub struct PostgresDeviceTokenRepository {
    pool: PgPool,
}

impl PostgresDeviceTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct TokenRow {
    id: i64,
    user_id: Uuid,
    device_token: String,
    platform: String,
}

impl From<TokenRow> for DeviceToken {
    fn from(row: TokenRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            device_token: row.device_token,
            platform: row.platform,
        }
    }
}

#[async_trait]
impl DeviceTokenRepository for PostgresDeviceTokenRepository {
    async fn upsert(
        &self,
        user_id: Uuid,
        device_token: &str,
        platform: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"INSERT INTO f_i_user_device_tokens (user_id, device_token, platform) VALUES ($1, $2, $3) ON CONFLICT (device_token) DO UPDATE SET user_id = EXCLUDED.user_id, platform = EXCLUDED.platform, updated_at = now()"#,
        )
        .bind(user_id)
        .bind(device_token)
        .bind(platform)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<DeviceToken>> {
        let rows = sqlx::query_as::<_, TokenRow>(
            "SELECT id, user_id, device_token, platform FROM f_i_user_device_tokens WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_by_users(&self, user_ids: &[Uuid]) -> anyhow::Result<Vec<DeviceToken>> {
        if user_ids.is_empty() {
            return Ok(vec![]);
        }
        let rows = sqlx::query_as::<_, TokenRow>(
            "SELECT id, user_id, device_token, platform FROM f_i_user_device_tokens WHERE user_id = ANY($1)",
        )
        .bind(user_ids)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete(&self, device_token: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM f_i_user_device_tokens WHERE device_token = $1")
            .bind(device_token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
