use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::ticket_watch::domain::ticket_watch::{
    TicketWatchBlockInterest, TicketWatchCurrentMatchView, TicketWatchInventoryEntry,
    TicketWatchTrackedInterest,
};

#[derive(Clone)]
pub struct CurrentBoardPublicData {
    pub current_match_view: TicketWatchCurrentMatchView,
    pub inventory: Vec<TicketWatchInventoryEntry>,
    pub block_interests: Vec<TicketWatchBlockInterest>,
}

#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct CurrentTicketWatchBoardCache {
    ttl: Duration,
    public_data: Arc<RwLock<Option<CacheEntry<CurrentBoardPublicData>>>>,
    tracked_interests:
        Arc<RwLock<HashMap<String, CacheEntry<Vec<TicketWatchTrackedInterest>>>>>,
}

impl CurrentTicketWatchBoardCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            public_data: Arc::new(RwLock::new(None)),
            tracked_interests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_public_data(&self) -> Option<CurrentBoardPublicData> {
        let mut guard = self.public_data.write().await;
        let entry = guard.clone()?;

        if entry.expires_at <= Instant::now() {
            *guard = None;
            return None;
        }

        Some(entry.value)
    }

    pub async fn set_public_data(&self, value: CurrentBoardPublicData) {
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        };
        *self.public_data.write().await = Some(entry);
    }

    pub async fn get_tracked_interests(
        &self,
        match_id: i64,
        user_id: Uuid,
    ) -> Option<Vec<TicketWatchTrackedInterest>> {
        let key = build_tracked_interest_cache_key(match_id, user_id);
        let mut guard = self.tracked_interests.write().await;
        let entry = guard.get(&key)?.clone();

        if entry.expires_at <= Instant::now() {
            guard.remove(&key);
            return None;
        }

        Some(entry.value)
    }

    pub async fn set_tracked_interests(
        &self,
        match_id: i64,
        user_id: Uuid,
        value: Vec<TicketWatchTrackedInterest>,
    ) {
        let key = build_tracked_interest_cache_key(match_id, user_id);
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        };
        self.tracked_interests.write().await.insert(key, entry);
    }
}

fn build_tracked_interest_cache_key(match_id: i64, user_id: Uuid) -> String {
    format!("{match_id}:{user_id}")
}

