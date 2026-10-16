use crate::ticket_watch::domain::{
    best_watch_windows::BEST_WINDOWS_MAX_SAMPLES, ticket_watch::TicketWatchMatchSummary,
};
use chrono::{DateTime, Utc};

pub const BEST_WINDOWS_STATS_DELAY_MINUTES: i64 = 10;
#[derive(Debug, Clone)]
pub struct BestWindowsCandidate {
    pub match_id: i64,
    pub fallback_match_id: Option<i64>,
    pub since: String,
}
pub fn select_best_windows_matches(
    matches: &[TicketWatchMatchSummary],
    now: DateTime<Utc>,
) -> Vec<BestWindowsCandidate> {
    let mut selected = Vec::new();
    for item in matches {
        let kickoff = DateTime::parse_from_rfc3339(&item.kickoff_at).ok();
        if !item.include_in_reflux_stats
            || item.home_team_name != "成都蓉城"
            || !(item.ticket_sale_ended || kickoff.is_some_and(|at| at <= now))
        {
            continue;
        }
        let Some(sale) = item
            .sale_start_at
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        else {
            tracing::warn!(
                match_id = item.match_id,
                "skipping best windows match without valid sale_start_at"
            );
            continue;
        };
        let Some(since) =
            sale.checked_add_signed(chrono::Duration::minutes(BEST_WINDOWS_STATS_DELAY_MINUTES))
        else {
            continue;
        };
        selected.push((
            kickoff,
            BestWindowsCandidate {
                match_id: item.match_id,
                fallback_match_id: item.external_match_id.parse().ok(),
                since: since.to_rfc3339(),
            },
        ));
    }
    selected.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then(right.1.match_id.cmp(&left.1.match_id))
    });
    selected
        .into_iter()
        .take(BEST_WINDOWS_MAX_SAMPLES)
        .map(|(_, candidate)| candidate)
        .collect()
}

use crate::{
    auth::{
        domain::membership::membership_tier_rank, ports::user_membership_port::UserMembershipPort,
    },
    ticket_watch::{
        domain::best_watch_windows::{
            BestWindowsAggregate, RestockProfile, RestockProfileNotReady,
            RESTOCK_PROFILE_SCHEMA_VERSION,
        },
        ports::ticket_monitor_port::TicketMonitorPort,
    },
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;
pub const BEST_WINDOWS_CACHE_SECONDS: i64 = 600;
pub const BEST_WINDOWS_RETRY_SECONDS: i64 = 60;
pub struct GetBestWatchWindowsUseCase {
    monitor: Arc<dyn TicketMonitorPort>,
    membership: Arc<dyn UserMembershipPort>,
    cache: RwLock<Option<(DateTime<Utc>, BestWindowsAggregate)>>,
    profiles: Mutex<HashMap<(i64, u8), RestockProfile>>,
    refresh_lock: Mutex<Option<DateTime<Utc>>>,
}
impl GetBestWatchWindowsUseCase {
    pub fn new(
        monitor: Arc<dyn TicketMonitorPort>,
        membership: Arc<dyn UserMembershipPort>,
    ) -> Self {
        Self {
            monitor,
            membership,
            cache: RwLock::new(None),
            profiles: Mutex::new(HashMap::new()),
            refresh_lock: Mutex::new(None),
        }
    }
    /// Foreground: only aggregate cache and effective membership. No monitor calls.
    pub async fn execute(&self, viewer: Option<Uuid>) -> (bool, BestWindowsAggregate) {
        let aggregate = self
            .cache
            .read()
            .await
            .as_ref()
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| BestWindowsAggregate::unavailable(Utc::now().to_rfc3339()));
        let full = if let Some(viewer) = viewer {
            match self.membership.get_user_membership_tier(viewer).await {
                Ok(Some(tier)) => membership_tier_rank(&tier) >= membership_tier_rank("V6"),
                Ok(None) => false,
                Err(error) => {
                    tracing::warn!(%error,"best windows membership unavailable; returning preview");
                    false
                }
            }
        } else {
            false
        };
        (full, aggregate.for_access(full))
    }
    /// Background-only, single flight. Existing complete aggregate stays readable while refreshing.
    pub async fn refresh_at(&self, now: DateTime<Utc>) {
        let mut next_refresh_at = self.refresh_lock.lock().await;
        if next_refresh_at.is_some_and(|next| now < next) {
            return;
        }
        match self.build_aggregate(now).await {
            Ok(aggregate) => {
                *self.cache.write().await = Some((now, aggregate));
                *next_refresh_at =
                    Some(now + chrono::Duration::seconds(BEST_WINDOWS_CACHE_SECONDS));
            }
            Err(error) => {
                tracing::warn!(%error,"offline best windows refresh failed; retaining previous aggregate");
                *next_refresh_at =
                    Some(now + chrono::Duration::seconds(BEST_WINDOWS_RETRY_SECONDS));
            }
        }
    }
    async fn build_aggregate(&self, now: DateTime<Utc>) -> anyhow::Result<BestWindowsAggregate> {
        let matches = self.monitor.fetch_all_matches().await?;
        let candidates = select_best_windows_matches(&matches, now);
        let mut results = Vec::with_capacity(candidates.len());
        for item in candidates {
            let key = (item.match_id, RESTOCK_PROFILE_SCHEMA_VERSION);
            let cached = { self.profiles.lock().await.get(&key).cloned() };
            let profile = if let Some(value) = cached {
                value
            } else {
                let value = match self
                    .monitor
                    .fetch_restock_profile(item.match_id, item.fallback_match_id, &item.since)
                    .await
                {
                    Ok(value) => value,
                    Err(error) if error.is::<RestockProfileNotReady>() => {
                        tracing::warn!(
                            match_id = item.match_id,
                            "offline seat snapshot not ready; skipping selected sample"
                        );
                        continue;
                    }
                    Err(error) => return Err(error),
                };
                self.profiles.lock().await.insert(key, value.clone());
                value
            };
            results.push(profile);
        }
        Ok(BestWindowsAggregate::from_profiles(
            &results,
            now.to_rfc3339(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-16T12:00:00+08:00")
            .unwrap()
            .with_timezone(&Utc)
    }
    fn fixture(id: i64) -> TicketWatchMatchSummary {
        TicketWatchMatchSummary {
            match_id: id,
            external_match_id: (id + 100).to_string(),
            round_number: 1,
            sale_start_at: Some("2026-10-01T14:00:00+08:00".into()),
            match_date: "".into(),
            match_time: "".into(),
            kickoff_at: format!("2026-10-{:02}T19:35:00+08:00", id),
            home_team_name: "成都蓉城".into(),
            away_team_name: "客队".into(),
            is_current: false,
            include_in_reflux_stats: true,
            ticket_sale_ended: false,
        }
    }
    #[test]
    fn best_windows_selection_enforces_ended_included_exact_home_and_sale_start() {
        let base = fixture(1);
        assert_eq!(select_best_windows_matches(&[base.clone()], now()).len(), 1);
        for kind in 0..6 {
            let mut item = base.clone();
            match kind {
                0 => item.kickoff_at = "2026-11-01T19:35:00+08:00".into(),
                1 => item.include_in_reflux_stats = false,
                2 => item.home_team_name = "成都蓉城希拉谷".into(),
                3 => item.home_team_name = "成都蓉城U21".into(),
                4 => item.sale_start_at = None,
                _ => item.sale_start_at = Some("invalid".into()),
            }
            assert!(
                select_best_windows_matches(&[item], now()).is_empty(),
                "kind={kind}"
            );
        }
        let mut sold = base;
        sold.kickoff_at = "2026-11-01T19:35:00+08:00".into();
        sold.ticket_sale_ended = true;
        let result = select_best_windows_matches(&[sold], now());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].since, "2026-10-01T14:10:00+08:00");
        assert_eq!(result[0].fallback_match_id, Some(101));
    }
    #[test]
    fn best_windows_selection_takes_latest_eight_after_metadata_filters() {
        let mut items: Vec<_> = (1..=12).map(fixture).collect();
        items[11].sale_start_at = None;
        let result = select_best_windows_matches(&items, now());
        assert_eq!(
            result.iter().map(|m| m.match_id).collect::<Vec<_>>(),
            (4..=11).rev().collect::<Vec<_>>()
        );
    }
    use crate::ticket_watch::domain::{best_watch_windows::BestWindowsStatus, ticket_watch::*};
    use async_trait::async_trait;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex as StdMutex,
    };
    struct Fake {
        calls: AtomicUsize,
        profiles: AtomicUsize,
        tier: StdMutex<Option<String>>,
        fail: std::sync::atomic::AtomicBool,
        match_count: usize,
        missing: StdMutex<Vec<i64>>,
        fetched_ids: StdMutex<Vec<i64>>,
    }
    impl Fake {
        fn new(fail: bool) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                profiles: AtomicUsize::new(0),
                tier: StdMutex::new(Some("V6".into())),
                fail: std::sync::atomic::AtomicBool::new(fail),
                match_count: 3,
                missing: StdMutex::new(vec![]),
                fetched_ids: StdMutex::new(vec![]),
            }
        }
    }
    #[async_trait]
    impl UserMembershipPort for Fake {
        async fn get_user_open_id(&self, _: Uuid) -> anyhow::Result<Option<String>> {
            unreachable!()
        }
        async fn get_user_membership_tier(&self, _: Uuid) -> anyhow::Result<Option<String>> {
            if self.fail.load(Ordering::SeqCst) {
                anyhow::bail!("identity failure")
            }
            Ok(self.tier.lock().unwrap().clone())
        }
        async fn update_user_membership_tier(&self, _: Uuid, _: &str) -> anyhow::Result<()> {
            unreachable!()
        }
        async fn is_seat_swap_notice_enabled(&self, _: Uuid) -> anyhow::Result<Option<bool>> {
            unreachable!()
        }
    }
    #[async_trait]
    impl TicketMonitorPort for Fake {
        async fn fetch_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
            unreachable!()
        }
        async fn fetch_all_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail.load(Ordering::SeqCst) {
                anyhow::bail!("offline snapshot missing")
            }
            Ok((1..=self.match_count as i64).map(fixture).collect())
        }
        async fn fetch_restock_profile(
            &self,
            id: i64,
            _: Option<i64>,
            _: &str,
        ) -> anyhow::Result<RestockProfile> {
            self.fetched_ids.lock().unwrap().push(id);
            if self.missing.lock().unwrap().contains(&id) {
                return Err(
                    crate::ticket_watch::domain::best_watch_windows::RestockProfileNotReady.into(),
                );
            }
            self.profiles.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
            let mut hourly = [0; 24];
            hourly[12] = 20;
            Ok(RestockProfile {
                total_events: 20,
                appear_events: 20,
                up_events: 0,
                seats_with_events: 1,
                hourly,
            })
        }
        async fn fetch_regions(&self) -> anyhow::Result<Vec<TicketWatchRegion>> {
            unreachable!()
        }
        async fn fetch_inventory(
            &self,
            _: i64,
            _: Option<i64>,
            _: Option<&str>,
        ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
            unreachable!()
        }
        async fn fetch_block_interests(
            &self,
            _: i64,
            _: Option<Uuid>,
        ) -> anyhow::Result<Vec<TicketWatchBlockInterest>> {
            unreachable!()
        }
        async fn fetch_tracked_interests(
            &self,
            _: i64,
            _: Uuid,
        ) -> anyhow::Result<Vec<TicketWatchTrackedInterest>> {
            unreachable!()
        }
        async fn toggle_block_interest(
            &self,
            _: i64,
            _: Uuid,
            _: &str,
        ) -> anyhow::Result<TicketWatchBlockInterest> {
            unreachable!()
        }
        async fn fetch_yukun_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
            unreachable!()
        }
        async fn fetch_yukun_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
            unreachable!()
        }
        async fn fetch_yukun_reflux(
            &self,
            _: i64,
            _: Option<&str>,
        ) -> anyhow::Result<(Vec<TicketWatchInventoryEntry>, Vec<TicketWatchRegion>)> {
            unreachable!()
        }
    }
    #[tokio::test]
    async fn best_windows_foreground_never_fetches_and_background_is_single_flight() {
        let fake = Arc::new(Fake::new(false));
        let uc = GetBestWatchWindowsUseCase::new(fake.clone(), fake.clone());
        assert_eq!(
            uc.execute(None).await.1.status,
            BestWindowsStatus::Unavailable
        );
        assert_eq!(fake.calls.load(Ordering::SeqCst), 0);
        tokio::join!(uc.refresh_at(now()), uc.refresh_at(now()));
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
        assert_eq!(fake.profiles.load(Ordering::SeqCst), 3);
        let (full, result) = uc.execute(Some(Uuid::nil())).await;
        assert!(full);
        assert_eq!(result.hours.len(), 24);
        *fake.tier.lock().unwrap() = Some("V3".into());
        let (full, result) = uc.execute(Some(Uuid::nil())).await;
        assert!(!full);
        assert!(result.hours.is_empty());
        assert_eq!(result.top_windows.len(), 1);
        assert!(uc.execute(None).await.1.hours.is_empty());
        *fake.tier.lock().unwrap() = None;
        assert!(!uc.execute(Some(Uuid::nil())).await.0);
        uc.refresh_at(now() + chrono::Duration::seconds(599)).await;
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
        uc.refresh_at(now() + chrono::Duration::seconds(600)).await;
        assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
        assert_eq!(fake.profiles.load(Ordering::SeqCst), 3);
    }
    #[tokio::test]
    async fn best_windows_monitor_failure_is_unavailable_and_identity_failure_is_preview() {
        let monitor = Arc::new(Fake::new(true));
        let membership = Arc::new(Fake::new(false));
        let uc = GetBestWatchWindowsUseCase::new(monitor, membership);
        uc.refresh_at(now()).await;
        assert_eq!(
            uc.execute(None).await.1.status,
            BestWindowsStatus::Unavailable
        );
        let uc =
            GetBestWatchWindowsUseCase::new(Arc::new(Fake::new(false)), Arc::new(Fake::new(true)));
        uc.refresh_at(now()).await;
        let (full, result) = uc.execute(Some(Uuid::nil())).await;
        assert!(!full);
        assert_eq!(result.status, BestWindowsStatus::Ok);
        assert!(result.hours.is_empty());
        assert_eq!(result.top_windows.len(), 1);
    }

    #[tokio::test]
    async fn best_windows_skips_not_ready_within_fixed_latest_eight_without_backfill() {
        for (count, missing, expected, status) in [
            (9, vec![9], 7, BestWindowsStatus::Ok),
            (4, vec![4], 3, BestWindowsStatus::Ok),
            (3, vec![3], 2, BestWindowsStatus::Insufficient),
            (3, vec![1, 2, 3], 0, BestWindowsStatus::Insufficient),
        ] {
            let fake = Arc::new(Fake {
                match_count: count,
                missing: StdMutex::new(missing),
                ..Fake::new(false)
            });
            let uc = GetBestWatchWindowsUseCase::new(fake.clone(), fake.clone());
            uc.refresh_at(now()).await;
            let (_, result) = uc.execute(None).await;
            assert_eq!(result.status, status);
            assert_eq!(result.sample_size, expected);
            if count == 9 {
                assert_eq!(
                    *fake.fetched_ids.lock().unwrap(),
                    (2..=9).rev().collect::<Vec<_>>()
                );
            }
        }
    }
    #[tokio::test]
    async fn best_windows_failed_refresh_keeps_previous_result_and_retries_after_one_minute() {
        let monitor = Arc::new(Fake::new(false));
        let uc = GetBestWatchWindowsUseCase::new(monitor.clone(), Arc::new(Fake::new(false)));
        uc.refresh_at(now()).await;
        let previous = uc.execute(Some(Uuid::nil())).await;
        monitor.fail.store(true, Ordering::SeqCst);
        uc.refresh_at(now() + chrono::Duration::seconds(600)).await;
        assert_eq!(uc.execute(Some(Uuid::nil())).await, previous);
        assert_eq!(monitor.calls.load(Ordering::SeqCst), 2);
        monitor.fail.store(false, Ordering::SeqCst);
        uc.refresh_at(now() + chrono::Duration::seconds(659)).await;
        assert_eq!(monitor.calls.load(Ordering::SeqCst), 2);
        uc.refresh_at(now() + chrono::Duration::seconds(660)).await;
        assert_eq!(monitor.calls.load(Ordering::SeqCst), 3);
        assert_eq!(
            uc.execute(None).await.1.generated_at,
            (now() + chrono::Duration::seconds(660)).to_rfc3339()
        );
        uc.refresh_at(now() + chrono::Duration::seconds(1259)).await;
        assert_eq!(monitor.calls.load(Ordering::SeqCst), 3);
        uc.refresh_at(now() + chrono::Duration::seconds(1260)).await;
        assert_eq!(monitor.calls.load(Ordering::SeqCst), 4);
    }
}
