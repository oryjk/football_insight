use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::admin::{
    domain::admin_brand::{AdminBrandError, BrandSubtitle},
    ports::admin_brand_repository::AdminBrandRepository,
};

const CONFIG_KEY: &str = "brand_subtitle";

pub struct PostgresAdminBrandRepository {
    pool: PgPool,
}

impl PostgresAdminBrandRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_error(error: sqlx::Error) -> AdminBrandError {
    tracing::warn!(error = %error, "admin brand config query failed");
    AdminBrandError::Internal
}

async fn read_current(
    executor: impl sqlx::PgExecutor<'_>,
) -> Result<Option<String>, AdminBrandError> {
    let row = sqlx::query("SELECT config_value FROM f_i_system_configs WHERE config_key = $1")
        .bind(CONFIG_KEY)
        .fetch_optional(executor)
        .await
        .map_err(map_error)?;
    match row {
        Some(row) => Ok(Some(row.try_get("config_value").map_err(map_error)?)),
        None => Ok(None),
    }
}

#[async_trait]
impl AdminBrandRepository for PostgresAdminBrandRepository {
    async fn get_subtitle(&self) -> Result<BrandSubtitle, AdminBrandError> {
        let current = read_current(&self.pool).await?;
        BrandSubtitle::new(current.as_deref().unwrap_or(""))
    }

    async fn set_subtitle(
        &self,
        subtitle: BrandSubtitle,
        admin_id: Uuid,
    ) -> Result<BrandSubtitle, AdminBrandError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let before = read_current(&mut *tx).await?;
        match subtitle.value() {
            Some(value) => {
                sqlx::query(
                    "INSERT INTO f_i_system_configs (config_key, config_value, description) \
                     VALUES ($1, $2, '首页品牌栏副标题，后台可配置；留空使用客户端默认文案') \
                     ON CONFLICT (config_key) DO UPDATE \
                     SET config_value = EXCLUDED.config_value, updated_at = NOW()",
                )
                .bind(CONFIG_KEY)
                .bind(value)
                .execute(&mut *tx)
                .await
                .map_err(map_error)?;
            }
            None => {
                sqlx::query("DELETE FROM f_i_system_configs WHERE config_key = $1")
                    .bind(CONFIG_KEY)
                    .execute(&mut *tx)
                    .await
                    .map_err(map_error)?;
            }
        }
        sqlx::query(
            "INSERT INTO f_i_admin_audit_logs(id,admin_user_id,action,target_type,target_id,before_json,after_json) \
             VALUES($1,$2,'brand.subtitle.set','system_config',$3,$4,$5)",
        )
        .bind(Uuid::new_v4())
        .bind(admin_id)
        .bind(CONFIG_KEY)
        .bind(serde_json::json!({ "subtitle": before }))
        .bind(serde_json::json!({ "subtitle": subtitle.value() }))
        .execute(&mut *tx)
        .await
        .map_err(map_error)?;
        tx.commit().await.map_err(map_error)?;
        Ok(subtitle)
    }
}
