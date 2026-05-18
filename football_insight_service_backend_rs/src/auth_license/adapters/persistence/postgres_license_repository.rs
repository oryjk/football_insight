use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::auth_license::domain::license::UserLicense;
use crate::auth_license::ports::license_repository::LicenseRepository;

pub struct PostgresLicenseRepository {
    pool: PgPool,
}

impl PostgresLicenseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct LicenseRow {
    id: i64,
    user_id: Uuid,
    license_key: String,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl From<LicenseRow> for UserLicense {
    fn from(row: LicenseRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            license_key: row.license_key,
            used_at: row.used_at,
            created_at: row.created_at,
            expires_at: row.expires_at,
        }
    }
}

#[async_trait]
impl LicenseRepository for PostgresLicenseRepository {
    async fn create_license(
        &self,
        user_id: Uuid,
        license_key: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<UserLicense> {
        let row = sqlx::query_as::<_, LicenseRow>(
            "INSERT INTO f_i_user_licenses (user_id, license_key, expires_at) VALUES ($1, $2, $3) RETURNING id, user_id, license_key, used_at, created_at, expires_at",
        )
        .bind(user_id)
        .bind(license_key)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn find_by_key(&self, license_key: &str) -> anyhow::Result<Option<UserLicense>> {
        let row = sqlx::query_as::<_, LicenseRow>(
            "SELECT id, user_id, license_key, used_at, created_at, expires_at FROM f_i_user_licenses WHERE license_key = $1",
        )
        .bind(license_key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn mark_used(&self, license_id: i64) -> anyhow::Result<()> {
        sqlx::query("UPDATE f_i_user_licenses SET used_at = now() WHERE id = $1")
            .bind(license_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
