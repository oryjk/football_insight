use crate::auth::{
    domain::acquisition::AcquisitionSource, ports::acquisition_repository::AcquisitionRepository,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub const INSERT_ACQUISITION_SQL: &str = r#"
INSERT INTO f_i_user_acquisitions (user_id, src, cid, landing_scene)
VALUES ($1, $2, $3, $4)
ON CONFLICT (user_id) DO NOTHING
"#;

pub struct PostgresAcquisitionRepository {
    pool: PgPool,
}
impl PostgresAcquisitionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
#[async_trait]
impl AcquisitionRepository for PostgresAcquisitionRepository {
    async fn insert_first(&self, user_id: Uuid, source: AcquisitionSource) -> anyhow::Result<()> {
        sqlx::query(INSERT_ACQUISITION_SQL)
            .bind(user_id)
            .bind(source.src)
            .bind(source.cid)
            .bind(source.landing_scene)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
