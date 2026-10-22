use async_trait::async_trait;
use sqlx::PgPool;

use crate::activity::{
    domain::paywall::NewPaywallEvent, ports::paywall_event_repository::PaywallEventRepository,
};

pub struct PostgresPaywallEventRepository {
    pool: PgPool,
}

impl PostgresPaywallEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaywallEventRepository for PostgresPaywallEventRepository {
    async fn record(&self, events: Vec<NewPaywallEvent>) {
        for event in events {
            let result = sqlx::query(
                "INSERT INTO f_i_paywall_events (user_id,surface,action,membership_tier,detail,occurred_at) VALUES ($1,$2,$3,$4,$5,$6)",
            )
            .bind(event.user_id)
            .bind(&event.surface)
            .bind(event.action.as_str())
            .bind(&event.membership_tier)
            .bind(&event.detail)
            .bind(event.occurred_at)
            .execute(&self.pool)
            .await;
            if let Err(error) = result {
                tracing::warn!(%error, "paywall event insert failed");
                return;
            }
        }
    }
}
