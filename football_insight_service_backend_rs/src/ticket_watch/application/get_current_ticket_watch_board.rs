use std::sync::Arc;

use chrono::{DateTime, Duration};
use uuid::Uuid;

use crate::ticket_watch::{
    application::{
        current_board_cache::{CurrentBoardPublicData, CurrentTicketWatchBoardCache},
        get_current_ticket_watch_match::GetCurrentTicketWatchMatchUseCase,
        get_match_block_interests::GetMatchBlockInterestsUseCase,
        get_match_ticket_inventory::GetMatchTicketInventoryUseCase,
        get_match_tracked_interests::GetMatchTrackedInterestsUseCase,
    },
    domain::ticket_watch::{
        TicketWatchBlockInterest, TicketWatchCurrentBoardView, TicketWatchTrackedInterest,
    },
};

pub struct GetCurrentTicketWatchBoardUseCase {
    cache: Arc<CurrentTicketWatchBoardCache>,
    get_current_ticket_watch_match_use_case: Arc<GetCurrentTicketWatchMatchUseCase>,
    get_match_ticket_inventory_use_case: Arc<GetMatchTicketInventoryUseCase>,
    get_match_block_interests_use_case: Arc<GetMatchBlockInterestsUseCase>,
    get_match_tracked_interests_use_case: Arc<GetMatchTrackedInterestsUseCase>,
}

impl GetCurrentTicketWatchBoardUseCase {
    pub fn new(
        cache: Arc<CurrentTicketWatchBoardCache>,
        get_current_ticket_watch_match_use_case: Arc<GetCurrentTicketWatchMatchUseCase>,
        get_match_ticket_inventory_use_case: Arc<GetMatchTicketInventoryUseCase>,
        get_match_block_interests_use_case: Arc<GetMatchBlockInterestsUseCase>,
        get_match_tracked_interests_use_case: Arc<GetMatchTrackedInterestsUseCase>,
    ) -> Self {
        Self {
            cache,
            get_current_ticket_watch_match_use_case,
            get_match_ticket_inventory_use_case,
            get_match_block_interests_use_case,
            get_match_tracked_interests_use_case,
        }
    }

    pub async fn execute(
        &self,
        viewer_user_id: Option<Uuid>,
    ) -> anyhow::Result<TicketWatchCurrentBoardView> {
        let public_data = match self.cache.get_public_data().await {
            Some(cached) => cached,
            None => {
                let current_match_view = self.get_current_ticket_watch_match_use_case.execute().await?;
                let Some(current_match) = current_match_view.current_match.clone() else {
                    let public_data = CurrentBoardPublicData {
                        current_match_view,
                        inventory: vec![],
                        block_interests: vec![],
                    };
                    self.cache.set_public_data(public_data.clone()).await;
                    return Ok(TicketWatchCurrentBoardView {
                        current_match: None,
                        group_ticket_active: public_data.current_match_view.group_ticket_active,
                        message: public_data.current_match_view.message,
                        inventory: vec![],
                        block_interests: vec![],
                        tracked_interests: vec![],
                    });
                };

                let since = build_inventory_since(current_match.sale_start_at.as_deref());
                let fallback_match_id = resolve_fallback_match_id(
                    current_match.match_id,
                    current_match.external_match_id.as_str(),
                );
                let (inventory, block_interests) = tokio::try_join!(
                    self.get_match_ticket_inventory_use_case.execute_with_since(
                        current_match.match_id,
                        fallback_match_id,
                        since.as_deref(),
                    ),
                    self.get_match_block_interests_use_case
                        .execute(current_match.match_id, None),
                )?;

                let public_data = CurrentBoardPublicData {
                    current_match_view,
                    inventory,
                    block_interests,
                };
                self.cache.set_public_data(public_data.clone()).await;
                public_data
            }
        };

        let Some(current_match) = public_data.current_match_view.current_match.clone() else {
            return Ok(TicketWatchCurrentBoardView {
                current_match: None,
                group_ticket_active: public_data.current_match_view.group_ticket_active,
                message: public_data.current_match_view.message,
                inventory: vec![],
                block_interests: vec![],
                tracked_interests: vec![],
            });
        };

        let tracked_interests = match viewer_user_id {
            Some(user_id) => match self
                .cache
                .get_tracked_interests(current_match.match_id, user_id)
                .await
            {
                Some(cached) => cached,
                None => {
                    let tracked = self
                        .get_match_tracked_interests_use_case
                        .execute(current_match.match_id, user_id)
                        .await?;
                    self.cache
                        .set_tracked_interests(current_match.match_id, user_id, tracked.clone())
                        .await;
                    tracked
                }
            },
            None => vec![],
        };

        let block_interests = merge_block_interests(
            public_data.block_interests.clone(),
            tracked_interests.as_slice(),
        );

        Ok(TicketWatchCurrentBoardView {
            current_match: Some(current_match),
            group_ticket_active: public_data.current_match_view.group_ticket_active,
            message: public_data.current_match_view.message,
            inventory: public_data.inventory,
            block_interests,
            tracked_interests,
        })
    }
}

fn resolve_fallback_match_id(match_id: i64, external_match_id: &str) -> Option<i64> {
    let parsed = external_match_id.parse::<i64>().ok()?;
    (parsed > 0 && parsed != match_id).then_some(parsed)
}

fn build_inventory_since(sale_start_at: Option<&str>) -> Option<String> {
    let value = sale_start_at?;
    let parsed = DateTime::parse_from_rfc3339(value).ok()?;
    Some((parsed + Duration::minutes(10)).to_rfc3339())
}

fn merge_block_interests(
    block_interests: Vec<TicketWatchBlockInterest>,
    tracked_interests: &[TicketWatchTrackedInterest],
) -> Vec<TicketWatchBlockInterest> {
    let tracked_blocks = tracked_interests
        .iter()
        .map(|item| item.block_name.as_str())
        .collect::<std::collections::HashSet<_>>();

    block_interests
        .into_iter()
        .map(|item| TicketWatchBlockInterest {
            viewer_interested: tracked_blocks.contains(item.block_name.as_str()),
            ..item
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::GetCurrentTicketWatchBoardUseCase;
    use crate::ticket_watch::{
        application::{
            current_board_cache::CurrentTicketWatchBoardCache,
            get_current_ticket_watch_match::GetCurrentTicketWatchMatchUseCase,
            get_match_block_interests::GetMatchBlockInterestsUseCase,
            get_match_ticket_inventory::GetMatchTicketInventoryUseCase,
            get_match_tracked_interests::GetMatchTrackedInterestsUseCase,
        },
        domain::ticket_watch::{
            TicketWatchBlockInterest, TicketWatchCurrentMatchView, TicketWatchInventoryEntry,
            TicketWatchMatchSummary, TicketWatchRegion, TicketWatchTrackedInterest,
        },
        ports::{
            ticket_monitor_port::TicketMonitorPort,
            tracked_interest_cache_port::TrackedInterestCachePort,
        },
    };

    #[derive(Default)]
    struct StubState {
        current_match_calls: usize,
        inventory_calls: Vec<(i64, Option<i64>, Option<String>)>,
        block_interest_calls: Vec<(i64, Option<Uuid>)>,
        tracked_calls: Vec<(i64, Uuid)>,
    }

    struct StubTicketMonitorPort {
        state: Arc<Mutex<StubState>>,
    }

    #[async_trait]
    impl TicketMonitorPort for StubTicketMonitorPort {
        async fn fetch_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
            self.state.lock().expect("state").current_match_calls += 1;
            Ok(TicketWatchCurrentMatchView {
                current_match: Some(TicketWatchMatchSummary {
                    match_id: 574,
                    external_match_id: "78".to_string(),
                    round_number: 8,
                    sale_start_at: Some("2026-04-23T14:00:00+08:00".to_string()),
                    match_date: "2026-04-25".to_string(),
                    match_time: "19:00".to_string(),
                    kickoff_at: "2026-04-25T19:00:00+08:00".to_string(),
                    home_team_name: "成都蓉城".to_string(),
                    away_team_name: "浙江俱乐部绿城".to_string(),
                    is_current: true,
                }),
                group_ticket_active: false,
                message: "ok".to_string(),
            })
        }

        async fn fetch_all_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
            unreachable!()
        }

        async fn fetch_regions(&self) -> anyhow::Result<Vec<TicketWatchRegion>> {
            unreachable!()
        }

        async fn fetch_inventory(
            &self,
            match_id: i64,
            fallback_match_id: Option<i64>,
            since: Option<&str>,
        ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
            self.state.lock().expect("state").inventory_calls.push((
                match_id,
                fallback_match_id,
                since.map(str::to_string),
            ));

            Ok(vec![TicketWatchInventoryEntry {
                block_name: "117".to_string(),
                occurrences: 31,
                latest_time: "2026-04-23T14:18:18+08:00".to_string(),
            }])
        }

        async fn fetch_block_interests(
            &self,
            match_id: i64,
            viewer_user_id: Option<Uuid>,
        ) -> anyhow::Result<Vec<TicketWatchBlockInterest>> {
            self.state
                .lock()
                .expect("state")
                .block_interest_calls
                .push((match_id, viewer_user_id));
            Ok(vec![TicketWatchBlockInterest {
                block_name: "117".to_string(),
                interested_user_count: 2,
                viewer_interested: false,
            }])
        }

        async fn fetch_tracked_interests(
            &self,
            match_id: i64,
            user_id: Uuid,
        ) -> anyhow::Result<Vec<TicketWatchTrackedInterest>> {
            self.state
                .lock()
                .expect("state")
                .tracked_calls
                .push((match_id, user_id));

            Ok(vec![TicketWatchTrackedInterest {
                block_name: "117".to_string(),
                started_at: "2026-04-23T14:17:00+08:00".to_string(),
                first_inventory_at: Some("2026-04-23T14:18:18+08:00".to_string()),
            }])
        }

        async fn toggle_block_interest(
            &self,
            _match_id: i64,
            _user_id: Uuid,
            _block_name: &str,
        ) -> anyhow::Result<TicketWatchBlockInterest> {
            unreachable!()
        }
    }

    struct NoopTrackedInterestCachePort;

    #[async_trait]
    impl TrackedInterestCachePort for NoopTrackedInterestCachePort {
        async fn get(
            &self,
            _match_id: i64,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<Vec<TicketWatchTrackedInterest>>> {
            Ok(None)
        }

        async fn set(
            &self,
            _match_id: i64,
            _user_id: Uuid,
            _tracked_interests: &[TicketWatchTrackedInterest],
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn delete(&self, _match_id: i64, _user_id: Uuid) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn aggregates_current_match_inventory_and_interest_data() {
        let state = Arc::new(Mutex::new(StubState::default()));
        let port = Arc::new(StubTicketMonitorPort {
            state: state.clone(),
        });
        let tracked_cache = Arc::new(NoopTrackedInterestCachePort);
        let use_case = GetCurrentTicketWatchBoardUseCase::new(
            Arc::new(CurrentTicketWatchBoardCache::new(std::time::Duration::from_secs(2))),
            Arc::new(GetCurrentTicketWatchMatchUseCase::new(port.clone())),
            Arc::new(GetMatchTicketInventoryUseCase::new(port.clone())),
            Arc::new(GetMatchBlockInterestsUseCase::new(port.clone())),
            Arc::new(GetMatchTrackedInterestsUseCase::new(port, tracked_cache)),
        );
        let user_id = Uuid::nil();

        let result = use_case.execute(Some(user_id)).await.expect("result");
        let state = state.lock().expect("state");

        assert_eq!(result.current_match.as_ref().map(|item| item.match_id), Some(574));
        assert_eq!(result.inventory.len(), 1);
        assert_eq!(result.block_interests.len(), 1);
        assert!(result.block_interests[0].viewer_interested);
        assert_eq!(result.tracked_interests.len(), 1);
        assert_eq!(state.current_match_calls, 1);
        assert_eq!(
            state.inventory_calls,
            vec![(
                574,
                Some(78),
                Some("2026-04-23T14:10:00+08:00".to_string()),
            )]
        );
        assert_eq!(state.block_interest_calls, vec![(574, None)]);
        assert_eq!(state.tracked_calls, vec![(574, user_id)]);
    }

    #[tokio::test]
    async fn reuses_public_cache_across_users_within_ttl_window() {
        let state = Arc::new(Mutex::new(StubState::default()));
        let port = Arc::new(StubTicketMonitorPort {
            state: state.clone(),
        });
        let tracked_cache = Arc::new(NoopTrackedInterestCachePort);
        let use_case = GetCurrentTicketWatchBoardUseCase::new(
            Arc::new(CurrentTicketWatchBoardCache::new(std::time::Duration::from_secs(2))),
            Arc::new(GetCurrentTicketWatchMatchUseCase::new(port.clone())),
            Arc::new(GetMatchTicketInventoryUseCase::new(port.clone())),
            Arc::new(GetMatchBlockInterestsUseCase::new(port.clone())),
            Arc::new(GetMatchTrackedInterestsUseCase::new(port, tracked_cache)),
        );

        let _ = use_case.execute(Some(Uuid::nil())).await.expect("first");
        let _ = use_case
            .execute(Some(Uuid::from_u128(1)))
            .await
            .expect("second");
        let state = state.lock().expect("state");

        assert_eq!(state.current_match_calls, 1);
        assert_eq!(state.inventory_calls.len(), 1);
        assert_eq!(state.block_interest_calls, vec![(574, None)]);
        assert_eq!(state.tracked_calls.len(), 2);
    }
}
