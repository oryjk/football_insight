use async_trait::async_trait;
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchTrackedInterest,
    ports::tracked_interest_cache_port::TrackedInterestCachePort,
};

pub struct RedisTrackedInterestCachePort {
    client: redis::Client,
    ttl_seconds: u64,
}

impl RedisTrackedInterestCachePort {
    pub fn new(redis_url: &str, ttl_seconds: u64) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            ttl_seconds,
        })
    }

    fn build_key(match_id: i64, user_id: Uuid) -> String {
        format!("ticket_watch:tracked_interests:{match_id}:{user_id}")
    }
}

#[async_trait]
impl TrackedInterestCachePort for RedisTrackedInterestCachePort {
    async fn get(
        &self,
        match_id: i64,
        user_id: Uuid,
    ) -> anyhow::Result<Option<Vec<TicketWatchTrackedInterest>>> {
        let key = Self::build_key(match_id, user_id);
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let payload: Option<String> = conn.get(key).await?;

        payload
            .map(|value| serde_json::from_str::<Vec<TicketWatchTrackedInterest>>(&value))
            .transpose()
            .map_err(Into::into)
    }

    async fn set(
        &self,
        match_id: i64,
        user_id: Uuid,
        tracked_interests: &[TicketWatchTrackedInterest],
    ) -> anyhow::Result<()> {
        let key = Self::build_key(match_id, user_id);
        let value = serde_json::to_string(tracked_interests)?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.set_ex(key, value, self.ttl_seconds).await?;
        Ok(())
    }

    async fn delete(&self, match_id: i64, user_id: Uuid) -> anyhow::Result<()> {
        let key = Self::build_key(match_id, user_id);
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: usize = conn.del(key).await?;
        Ok(())
    }
}
