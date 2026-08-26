use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    match_id_unlock::ports::match_id_source::MatchIdSource,
    ticket_watch::ports::ticket_monitor_port::TicketMonitorPort,
};

/// 以 ticket-monitor 的比赛列表为存在性依据，60 秒内复用一次拉取结果，
/// 避免每次 entitlement 查询都打外部服务。
pub struct TicketMonitorMatchIdSource {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
    cache: Mutex<Option<CachedMatchIds>>,
}

struct CachedMatchIds {
    fetched_at: Instant,
    match_ids: HashSet<i64>,
}

const MATCH_LIST_CACHE_TTL: Duration = Duration::from_secs(60);

impl TicketMonitorMatchIdSource {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
            cache: Mutex::new(None),
        }
    }
}

#[async_trait]
impl MatchIdSource for TicketMonitorMatchIdSource {
    async fn known_match_id(&self, match_id: i64) -> anyhow::Result<bool> {
        let mut cache = self.cache.lock().await;

        let fresh = cache
            .as_ref()
            .is_some_and(|cached| cached.fetched_at.elapsed() < MATCH_LIST_CACHE_TTL);
        if !fresh {
            let match_ids = self
                .ticket_monitor_port
                .fetch_all_matches()
                .await?
                .into_iter()
                .map(|summary| summary.match_id)
                .collect::<HashSet<_>>();

            *cache = Some(CachedMatchIds {
                fetched_at: Instant::now(),
                match_ids,
            });
        }

        Ok(cache
            .as_ref()
            .expect("match id cache populated above")
            .match_ids
            .contains(&match_id))
    }
}
