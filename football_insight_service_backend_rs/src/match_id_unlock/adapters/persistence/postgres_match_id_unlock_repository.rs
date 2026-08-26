use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::match_id_unlock::ports::match_id_unlock_repository::MatchIdUnlockRepository;

pub struct PostgresMatchIdUnlockRepository {
    pool: PgPool,
}

impl PostgresMatchIdUnlockRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MatchIdUnlockRepository for PostgresMatchIdUnlockRepository {
    async fn find_unlock(&self, user_id: Uuid, match_id: i64) -> anyhow::Result<bool> {
        let row = sqlx::query_scalar::<_, uuid::Uuid>(
            "SELECT user_id FROM f_i_user_match_id_unlocks WHERE user_id = $1 AND match_id = $2 LIMIT 1",
        )
        .bind(user_id)
        .bind(match_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }
}
