use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::page_layout::{
    domain::{
        PageLayoutBlock, PageLayoutError, StoredPageLayout, ValidatedPageLayoutWrite,
    },
    ports::PageLayoutRepository,
};

pub struct PostgresPageLayoutRepository {
    pool: PgPool,
}

impl PostgresPageLayoutRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn is_undefined_table(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("42P01")
    )
}

fn map_error(error: sqlx::Error) -> PageLayoutError {
    tracing::warn!(error = %error, "page layout query failed");
    PageLayoutError::Internal
}

fn row_to_stored(row: sqlx::postgres::PgRow) -> Result<StoredPageLayout, PageLayoutError> {
    let page_key: String = row.try_get("page_key").map_err(map_error)?;
    let background_image_url: Option<String> =
        row.try_get("background_image_url").map_err(map_error)?;
    let blocks_val: serde_json::Value = row.try_get("blocks_json").map_err(map_error)?;
    let blocks: Vec<PageLayoutBlock> = serde_json::from_value(blocks_val).map_err(|error| {
        tracing::warn!(page_key = %page_key, error = %error, "invalid page layout blocks_json");
        PageLayoutError::Internal
    })?;
    let version: i64 = row.try_get("version").map_err(map_error)?;
    let updated_at: DateTime<Utc> = row.try_get("updated_at").map_err(map_error)?;

    Ok(StoredPageLayout {
        page_key,
        background_image_url,
        blocks,
        version,
        updated_at,
    })
}

async fn read_current(
    executor: impl sqlx::PgExecutor<'_>,
    page_key: &str,
) -> Result<Option<StoredPageLayout>, PageLayoutError> {
    let row = sqlx::query(
        "SELECT page_key, background_image_url, blocks_json, version, updated_at \
         FROM f_i_page_layouts \
         WHERE page_key = $1",
    )
    .bind(page_key)
    .fetch_optional(executor)
    .await;

    match row {
        Ok(Some(row)) => Ok(Some(row_to_stored(row)?)),
        Ok(None) => Ok(None),
        Err(error) if is_undefined_table(&error) => Ok(None),
        Err(error) => Err(map_error(error)),
    }
}

#[async_trait]
impl PageLayoutRepository for PostgresPageLayoutRepository {
    async fn list_all(&self) -> Result<Vec<StoredPageLayout>, PageLayoutError> {
        let rows = sqlx::query(
            "SELECT page_key, background_image_url, blocks_json, version, updated_at \
             FROM f_i_page_layouts \
             ORDER BY page_key ASC",
        )
        .fetch_all(&self.pool)
        .await;

        let rows = match rows {
            Ok(rows) => rows,
            Err(error) if is_undefined_table(&error) => return Ok(Vec::new()),
            Err(error) => return Err(map_error(error)),
        };

        rows.into_iter().map(row_to_stored).collect()
    }

    async fn get_by_page_key(
        &self,
        page_key: &str,
    ) -> Result<Option<StoredPageLayout>, PageLayoutError> {
        read_current(&self.pool, page_key).await
    }

    async fn upsert(
        &self,
        write: ValidatedPageLayoutWrite,
        admin_id: Uuid,
    ) -> Result<StoredPageLayout, PageLayoutError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let before = read_current(&mut *tx, &write.page_key).await?;
        let blocks_json = serde_json::to_value(&write.blocks).map_err(|error| {
            tracing::warn!(error = %error, "failed to serialize page layout blocks");
            PageLayoutError::Internal
        })?;

        let row = sqlx::query(
            "INSERT INTO f_i_page_layouts (page_key, background_image_url, blocks_json, version, updated_by, updated_at) \
             VALUES ($1, $2, $3, 1, $4, NOW()) \
             ON CONFLICT (page_key) DO UPDATE \
             SET background_image_url = EXCLUDED.background_image_url, \
                 blocks_json = EXCLUDED.blocks_json, \
                 version = f_i_page_layouts.version + 1, \
                 updated_by = EXCLUDED.updated_by, \
                 updated_at = NOW() \
             RETURNING page_key, background_image_url, blocks_json, version, updated_at",
        )
        .bind(&write.page_key)
        .bind(&write.background_image_url)
        .bind(&blocks_json)
        .bind(admin_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_error)?;

        let saved = row_to_stored(row)?;

        let before_json = before.as_ref().map(|b| {
            json!({
                "background_image_url": b.background_image_url,
                "blocks": b.blocks,
                "version": b.version,
            })
        });
        let after_json = json!({
            "background_image_url": saved.background_image_url,
            "blocks": saved.blocks,
            "version": saved.version,
        });

        sqlx::query(
            "INSERT INTO f_i_admin_audit_logs \
             (id, admin_user_id, action, target_type, target_id, before_json, after_json) \
             VALUES ($1, $2, 'page_layout.update', 'page_layout', $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(admin_id)
        .bind(&saved.page_key)
        .bind(before_json)
        .bind(after_json)
        .execute(&mut *tx)
        .await
        .map_err(map_error)?;

        tx.commit().await.map_err(map_error)?;
        Ok(saved)
    }

    async fn reset_to_default(
        &self,
        page_key: &str,
        admin_id: Uuid,
    ) -> Result<(), PageLayoutError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let before = read_current(&mut *tx, page_key).await?;

        sqlx::query("DELETE FROM f_i_page_layouts WHERE page_key = $1")
            .bind(page_key)
            .execute(&mut *tx)
            .await
            .map_err(map_error)?;

        let before_json = before.as_ref().map(|b| {
            json!({
                "background_image_url": b.background_image_url,
                "blocks": b.blocks,
                "version": b.version,
            })
        });

        sqlx::query(
            "INSERT INTO f_i_admin_audit_logs \
             (id, admin_user_id, action, target_type, target_id, before_json, after_json) \
             VALUES ($1, $2, 'page_layout.reset', 'page_layout', $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(admin_id)
        .bind(page_key)
        .bind(before_json)
        .bind(json!({ "reset_to_default": true }))
        .execute(&mut *tx)
        .await
        .map_err(map_error)?;

        tx.commit().await.map_err(map_error)?;
        Ok(())
    }
}
