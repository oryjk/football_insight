use std::{
    collections::HashMap,
    panic::AssertUnwindSafe,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chrono::Utc;
use futures_util::FutureExt;
use serde_json::Value;
use uuid::Uuid;

use crate::activity::{
    domain::paywall::{NewPaywallEvent, PaywallAction, normalize_tier, sanitize_detail, valid_surface},
    ports::paywall_event_repository::PaywallEventRepository,
};

const MAX_EVENTS_PER_REQUEST: usize = 10;
const MAX_EVENTS_PER_MINUTE: usize = 60;
const IN_FLIGHT_LIMIT: usize = 64;
const WRITE_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Clone)]
pub struct PaywallEventInput {
    pub surface: String,
    pub action: String,
    pub membership_tier: Option<String>,
    pub detail: Value,
}

/// Records paywall impressions/taps. `execute` is synchronous and infallible by design:
/// validation drops bad input, the write runs detached with a timeout and a concurrency cap,
/// so measurement can never delay or break the request that triggered it.
pub struct RecordPaywallEvents {
    repository: Arc<dyn PaywallEventRepository>,
    slots: Arc<tokio::sync::Semaphore>,
    limits: Mutex<HashMap<Uuid, (Instant, usize)>>,
}

impl RecordPaywallEvents {
    pub fn new(repository: Arc<dyn PaywallEventRepository>) -> Self {
        Self {
            repository,
            slots: Arc::new(tokio::sync::Semaphore::new(IN_FLIGHT_LIMIT)),
            limits: Mutex::new(HashMap::new()),
        }
    }

    pub fn execute(&self, user_id: Uuid, inputs: Vec<PaywallEventInput>) {
        if inputs.is_empty() || inputs.len() > MAX_EVENTS_PER_REQUEST {
            return;
        }
        let now = Utc::now();
        let mut events = Vec::new();
        for input in inputs {
            let Some(action) = PaywallAction::parse(&input.action) else {
                continue;
            };
            if !valid_surface(&input.surface) || !self.allow(user_id, Instant::now()) {
                continue;
            }
            events.push(NewPaywallEvent {
                user_id,
                surface: input.surface,
                action,
                membership_tier: normalize_tier(input.membership_tier.as_deref()),
                detail: sanitize_detail(&input.detail),
                occurred_at: now,
            });
        }
        if events.is_empty() {
            return;
        }
        let Ok(permit) = self.slots.clone().try_acquire_owned() else {
            tracing::warn!("paywall telemetry capacity exceeded");
            return;
        };
        let repository = self.repository.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let write = AssertUnwindSafe(async move { repository.record(events).await }).catch_unwind();
            match tokio::time::timeout(WRITE_TIMEOUT, write).await {
                Ok(Ok(())) => {}
                Ok(Err(_)) => tracing::warn!("paywall telemetry write panicked"),
                Err(_) => tracing::warn!("paywall telemetry write timed out"),
            }
        });
    }

    fn allow(&self, user_id: Uuid, now: Instant) -> bool {
        let Ok(mut limits) = self.limits.lock() else {
            return false;
        };
        limits.retain(|_, (start, _)| now.duration_since(*start) < Duration::from_secs(60));
        let (_, count) = limits.entry(user_id).or_insert((now, 0));
        if *count >= MAX_EVENTS_PER_MINUTE {
            return false;
        }
        *count += 1;
        true
    }
}
