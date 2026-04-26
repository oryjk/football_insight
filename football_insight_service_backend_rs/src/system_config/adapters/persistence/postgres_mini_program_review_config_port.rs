use sqlx::{PgPool, Row};

use crate::system_config::{
    domain::mini_program_review_config::MiniProgramReviewConfig,
    ports::mini_program_review_config_port::MiniProgramReviewConfigPort,
};

pub struct PostgresMiniProgramReviewConfigPort {
    pool: PgPool,
}

impl PostgresMiniProgramReviewConfigPort {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl MiniProgramReviewConfigPort for PostgresMiniProgramReviewConfigPort {
    async fn find_review_config(
        &self,
        mini_program_app_id: &str,
        mini_program_version: &str,
    ) -> anyhow::Result<Option<MiniProgramReviewConfig>> {
        let row = sqlx::query(
            r#"
            SELECT mini_program_app_id, mini_program_version, is_under_review, created_at, updated_at
            FROM f_i_mini_program_review_configs
            WHERE mini_program_app_id = $1 AND mini_program_version = $2
            LIMIT 1
            "#,
        )
        .bind(mini_program_app_id)
        .bind(mini_program_version)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| MiniProgramReviewConfig {
            mini_program_app_id: row.get("mini_program_app_id"),
            mini_program_version: row.get("mini_program_version"),
            is_under_review: row.get("is_under_review"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }
}
