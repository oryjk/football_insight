use sqlx::{PgPool, Row, postgres::PgDatabaseError};

use crate::mini_review::{
    domain::mini_review_status::MiniReviewStatus,
    ports::mini_review_repository::{MiniReviewRepository, VersionConflictError},
};

pub struct PostgresMiniReviewRepository {
    pool: PgPool,
}

impl PostgresMiniReviewRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const SELECT_COLUMNS: &str =
    "id, project_code, version, version_code, is_reviewing, status_text, created_at, updated_at";

fn map_row(row: &sqlx::postgres::PgRow) -> MiniReviewStatus {
    MiniReviewStatus {
        id: row.get("id"),
        project_code: row.get("project_code"),
        version: row.get("version"),
        version_code: row.get("version_code"),
        is_reviewing: row.get("is_reviewing"),
        status_text: row.get("status_text"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(database_error) => {
            database_error.downcast_ref::<PgDatabaseError>().code() == "23505"
        }
        _ => false,
    }
}

#[async_trait::async_trait]
impl MiniReviewRepository for PostgresMiniReviewRepository {
    async fn find_latest(&self, project_code: &str) -> anyhow::Result<Option<MiniReviewStatus>> {
        let sql = format!(
            r#"
            SELECT {SELECT_COLUMNS}
            FROM f_i_mini_review_statuses
            WHERE project_code = $1
            ORDER BY version_code DESC, id DESC
            LIMIT 1
            "#
        );
        let row = sqlx::query(&sql)
            .bind(project_code)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.as_ref().map(map_row))
    }

    async fn find_by_project_and_version(
        &self,
        project_code: &str,
        version: &str,
    ) -> anyhow::Result<Option<MiniReviewStatus>> {
        let sql = format!(
            r#"
            SELECT {SELECT_COLUMNS}
            FROM f_i_mini_review_statuses
            WHERE project_code = $1 AND version = $2
            LIMIT 1
            "#
        );
        let row = sqlx::query(&sql)
            .bind(project_code)
            .bind(version)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.as_ref().map(map_row))
    }

    async fn create(&self, status: MiniReviewStatus) -> anyhow::Result<MiniReviewStatus> {
        let sql = format!(
            r#"
            INSERT INTO f_i_mini_review_statuses
                (project_code, version, version_code, is_reviewing, status_text, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING {SELECT_COLUMNS}
            "#
        );
        let row = sqlx::query(&sql)
            .bind(&status.project_code)
            .bind(&status.version)
            .bind(status.version_code)
            .bind(status.is_reviewing)
            .bind(&status.status_text)
            .bind(status.created_at)
            .bind(status.updated_at)
            .fetch_one(&self.pool)
            .await;

        match row {
            Ok(row) => Ok(map_row(&row)),
            Err(error) if is_unique_violation(&error) => Err(anyhow::Error::new(VersionConflictError)),
            Err(error) => Err(error.into()),
        }
    }

    async fn update_status(&self, status: MiniReviewStatus) -> anyhow::Result<MiniReviewStatus> {
        let sql = format!(
            r#"
            UPDATE f_i_mini_review_statuses
            SET is_reviewing = $2, status_text = $3, updated_at = $4
            WHERE id = $1
            RETURNING {SELECT_COLUMNS}
            "#
        );
        let row = sqlx::query(&sql)
            .bind(status.id)
            .bind(status.is_reviewing)
            .bind(&status.status_text)
            .bind(status.updated_at)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow::anyhow!("review status record not found: {}", status.id))?;

        Ok(map_row(&row))
    }
}
