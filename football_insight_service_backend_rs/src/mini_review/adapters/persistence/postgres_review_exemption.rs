use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::mini_review::ports::review_exemption_port::ReviewExemptionPort;

pub struct PostgresReviewExemption {
    pool: PgPool,
}

impl PostgresReviewExemption {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReviewExemptionPort for PostgresReviewExemption {
    async fn is_exempt(&self, user_id: Uuid) -> anyhow::Result<bool> {
        let exempt: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM f_i_review_exempt_users WHERE user_id = $1)",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(exempt)
    }
}
