use crate::{
    admin::{domain::admin_payment::*, ports::admin_payment_repository::AdminPaymentRepository},
    payment::domain::payment_event::terminal_stage,
};
use chrono::{DateTime, Duration};
use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

impl PaymentRange {
    pub fn parse(
        from: &str,
        to: &str,
        scenario: Option<String>,
    ) -> Result<Self, AdminPaymentError> {
        let parse = |v: &str| {
            DateTime::parse_from_rfc3339(v)
                .map(|v| v.to_utc())
                .map_err(|_| {
                    AdminPaymentError::InvalidInput("from and to must be RFC3339 timestamps".into())
                })
        };
        let from = parse(from)?;
        let to = parse(to)?;
        if from >= to || to - from > Duration::days(180) {
            return Err(AdminPaymentError::InvalidInput(
                "time range must be positive and at most 180 days".into(),
            ));
        }
        if scenario.as_ref().is_some_and(|v| {
            !matches!(
                v.as_str(),
                "membership" | "match_id_unlock" | "reflux_subscription"
            )
        }) {
            return Err(AdminPaymentError::InvalidInput(
                "invalid payment scenario".into(),
            ));
        }
        Ok(Self { from, to, scenario })
    }
}

pub struct AdminPaymentService {
    repository: Arc<dyn AdminPaymentRepository>,
}
impl AdminPaymentService {
    pub fn new(repository: Arc<dyn AdminPaymentRepository>) -> Self {
        Self { repository }
    }
    pub async fn funnel(&self, range: PaymentRange) -> Result<PaymentFunnel, AdminPaymentError> {
        let stages = assemble_stages(self.repository.funnel(&range).await?);
        let rows = self.repository.terminals(&range).await?;
        let mut groups = BTreeMap::new();
        for row in rows {
            let key = (
                row.day,
                row.scenario,
                row.product_ref,
                terminal_stage(&row.events).to_string(),
            );
            let group = groups.entry(key).or_insert_with(|| (0i64, HashSet::new()));
            group.0 += row.attempt_count;
            group.1.extend(row.user_ids);
        }
        let terminal_stages = groups
            .into_iter()
            .map(
                |((day, scenario, product_ref, stage), (attempt_count, users))| TerminalCount {
                    day,
                    scenario,
                    product_ref,
                    stage,
                    attempt_count,
                    user_count: users.len() as i64,
                },
            )
            .collect();
        Ok(PaymentFunnel {
            stages,
            terminal_stages,
        })
    }
    pub async fn errors(
        &self,
        range: PaymentRange,
    ) -> Result<Vec<PaymentErrorCount>, AdminPaymentError> {
        self.repository.errors(&range).await
    }
    pub async fn timeline(
        &self,
        order_no: &str,
    ) -> Result<Vec<PaymentTimelineEvent>, AdminPaymentError> {
        if order_no.is_empty() || order_no.len() > 128 {
            return Err(AdminPaymentError::InvalidInput(
                "invalid order number".into(),
            ));
        }
        self.repository.timeline(order_no).await
    }
    pub async fn stuck(&self, minutes: i64) -> Result<Vec<StuckPaymentStage>, AdminPaymentError> {
        if !(1..=259200).contains(&minutes) {
            return Err(AdminPaymentError::InvalidInput(
                "older_than_minutes must be between 1 and 259200".into(),
            ));
        }
        Ok(self
            .repository
            .stuck(minutes)
            .await?
            .into_iter()
            .map(|order| StuckPaymentStage {
                stage: terminal_stage(&order.events).into(),
                order,
            })
            .collect())
    }
}

fn assemble_stages(rows: Vec<FunnelCount>) -> Vec<FunnelStage> {
    const MAIN_CHAIN: [&str; 7] = [
        "purchase_tapped",
        "order_create_requested",
        "order_created",
        "pay_params_issued",
        "pay_sheet_invoked",
        "pay_success_client",
        "delivered",
    ];
    let mut groups = BTreeMap::new();
    for row in rows {
        groups
            .entry((row.day, row.scenario.clone(), row.product_ref.clone()))
            .or_insert_with(BTreeMap::new)
            .insert(row.event.clone(), row);
    }
    let mut stages = Vec::new();
    for ((day, scenario, product_ref), mut events) in groups {
        let mut previous_count = None;
        for (index, event) in MAIN_CHAIN.iter().enumerate() {
            let mut counts = events.remove(*event).unwrap_or_else(|| FunnelCount {
                day,
                scenario: scenario.clone(),
                product_ref: product_ref.clone(),
                event: (*event).into(),
                attempt_count: 0,
                user_count: 0,
                previous_event: None,
                previous_attempt_count: None,
                converted_attempt_count: None,
            });
            counts.previous_event = index.checked_sub(1).map(|index| MAIN_CHAIN[index].into());
            counts.previous_attempt_count = previous_count;
            counts.converted_attempt_count = previous_count.map(|previous| {
                if previous == 0 || counts.attempt_count == 0 {
                    0
                } else {
                    counts.converted_attempt_count.unwrap_or(0)
                }
            });
            let rate = conversion_rate(
                counts.previous_attempt_count,
                counts.converted_attempt_count,
            );
            previous_count = Some(counts.attempt_count);
            stages.push(FunnelStage {
                counts,
                conversion_rate: rate,
            });
        }
        // Branch events retain their observed counts and do not gain inferred conversions.
        stages.extend(events.into_values().map(|counts| FunnelStage {
            conversion_rate: conversion_rate(
                counts.previous_attempt_count,
                counts.converted_attempt_count,
            ),
            counts,
        }));
    }
    stages
}

fn conversion_rate(previous: Option<i64>, converted: Option<i64>) -> Option<f64> {
    previous
        .zip(converted)
        .filter(|(previous, _)| *previous > 0)
        .map(|(previous, converted)| converted as f64 / previous as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_time_range_and_scenario_before_querying() {
        assert!(
            PaymentRange::parse(
                "2026-10-16T00:00:00Z",
                "2026-10-17T00:00:00Z",
                Some("membership".into())
            )
            .is_ok()
        );
        assert!(PaymentRange::parse("2026-10-17T00:00:00Z", "2026-10-16T00:00:00Z", None).is_err());
        assert!(PaymentRange::parse("bad", "2026-10-17T00:00:00Z", None).is_err());
        assert!(
            PaymentRange::parse(
                "2026-10-16T00:00:00Z",
                "2026-10-17T00:00:00Z",
                Some("other".into())
            )
            .is_err()
        );
    }
    #[test]
    fn assembles_adjacent_conversion_and_zero_denominators() {
        assert_eq!(conversion_rate(Some(4), Some(2)), Some(0.5));
        assert_eq!(conversion_rate(Some(0), Some(0)), None);
        assert_eq!(conversion_rate(None, None), None);
    }
}

#[cfg(test)]
mod fake_tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;
    #[derive(Default)]
    pub(super) struct Fake {
        pub(super) rows: Option<Vec<FunnelCount>>,
    }
    #[async_trait]
    impl AdminPaymentRepository for Fake {
        async fn funnel(&self, _: &PaymentRange) -> Result<Vec<FunnelCount>, AdminPaymentError> {
            Ok(self.rows.clone().unwrap_or_else(|| {
                vec![
                    FunnelCount {
                        day: Utc::now().date_naive(),
                        scenario: "membership".into(),
                        product_ref: Some("V7".into()),
                        event: "order_create_requested".into(),
                        attempt_count: 4,
                        user_count: 2,
                        previous_event: Some("purchase_tapped".into()),
                        previous_attempt_count: Some(0),
                        converted_attempt_count: Some(0),
                    },
                    FunnelCount {
                        day: Utc::now().date_naive(),
                        scenario: "membership".into(),
                        product_ref: Some("V7".into()),
                        event: "order_created".into(),
                        attempt_count: 2,
                        user_count: 1,
                        previous_event: Some("order_create_requested".into()),
                        previous_attempt_count: Some(4),
                        converted_attempt_count: Some(2),
                    },
                ]
            }))
        }
        async fn terminals(
            &self,
            _: &PaymentRange,
        ) -> Result<Vec<AttemptTerminalCount>, AdminPaymentError> {
            Ok(vec![
                vec!["purchase_tapped".into(), "pay_cancelled".into()],
                vec!["pay_cancelled".into()],
            ]
            .into_iter()
            .map(|events| AttemptTerminalCount {
                day: Utc::now().date_naive(),
                scenario: "membership".into(),
                product_ref: Some("V7".into()),
                events,
                attempt_count: 1,
                user_ids: vec![Uuid::nil()],
            })
            .collect())
        }
        async fn errors(
            &self,
            _: &PaymentRange,
        ) -> Result<Vec<PaymentErrorCount>, AdminPaymentError> {
            Ok(vec![])
        }
        async fn timeline(&self, _: &str) -> Result<Vec<PaymentTimelineEvent>, AdminPaymentError> {
            Ok(vec![])
        }
        async fn stuck(&self, _: i64) -> Result<Vec<StuckPaymentOrder>, AdminPaymentError> {
            Ok(vec![StuckPaymentOrder {
                order_no: "legacy-order".into(),
                user_id: Uuid::nil(),
                product_type: "membership:V7".into(),
                created_at: Utc::now(),
                last_event: None,
                last_event_at: None,
                events: vec![],
            }])
        }
    }
    #[tokio::test]
    async fn funnel_merges_terminal_patterns_without_double_counting_users() {
        let result = AdminPaymentService::new(Arc::new(Fake::default()))
            .funnel(
                PaymentRange::parse("2026-10-16T00:00:00Z", "2026-10-17T00:00:00Z", None).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            result
                .stages
                .iter()
                .find(|stage| stage.counts.event == "order_created")
                .unwrap()
                .conversion_rate,
            Some(0.5)
        );
        assert_eq!(result.terminal_stages.len(), 1);
        assert_eq!(result.terminal_stages[0].attempt_count, 2);
        assert_eq!(result.terminal_stages[0].user_count, 1);
        assert_eq!(result.terminal_stages[0].stage, "cancelled");
    }
    #[tokio::test]
    async fn legacy_pending_orders_without_events_are_explicitly_unknown() {
        let service = AdminPaymentService::new(Arc::new(Fake::default()));
        let rows = service.stuck(30).await.unwrap();
        assert_eq!(rows[0].stage, "unknown");
        assert!(rows[0].order.last_event.is_none());
        assert!(service.stuck(0).await.is_err());
        assert!(service.timeline("").await.is_err());
    }
}

#[cfg(test)]
mod missing_stage_tests {
    use super::fake_tests::Fake;
    use super::*;
    use chrono::NaiveDate;
    fn count(day: &str, scenario: &str, product: &str, event: &str, attempts: i64) -> FunnelCount {
        FunnelCount {
            day: day.parse::<NaiveDate>().unwrap(),
            scenario: scenario.into(),
            product_ref: Some(product.into()),
            event: event.into(),
            attempt_count: attempts,
            user_count: attempts,
            previous_event: None,
            previous_attempt_count: None,
            converted_attempt_count: None,
        }
    }
    #[tokio::test]
    async fn purchase_only_groups_include_zero_next_stage_without_cross_dimension_ratios() {
        let rows = vec![
            count("2026-10-16", "membership", "V7", "purchase_tapped", 5),
            count(
                "2026-10-16",
                "membership",
                "V8",
                "order_create_requested",
                2,
            ),
            count("2026-10-17", "membership", "V7", "purchase_tapped", 3),
            count("2026-10-16", "match_id_unlock", "V7", "purchase_tapped", 4),
            count("2026-10-16", "membership", "V7", "pay_cancelled", 1),
        ];
        let result = AdminPaymentService::new(Arc::new(Fake { rows: Some(rows) }))
            .funnel(
                PaymentRange::parse("2026-10-16T00:00:00Z", "2026-10-18T00:00:00Z", None).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(result.stages.len(), 29);
        let find = |day: &str, scenario: &str, product: &str, event: &str| {
            result
                .stages
                .iter()
                .find(|s| {
                    s.counts.day.to_string() == day
                        && s.counts.scenario == scenario
                        && s.counts.product_ref.as_deref() == Some(product)
                        && s.counts.event == event
                })
                .unwrap()
        };
        for (day, scenario, product, previous) in [
            ("2026-10-16", "membership", "V7", 5),
            ("2026-10-17", "membership", "V7", 3),
            ("2026-10-16", "match_id_unlock", "V7", 4),
        ] {
            let next = find(day, scenario, product, "order_create_requested");
            assert_eq!(next.counts.attempt_count, 0);
            assert_eq!(next.counts.user_count, 0);
            assert_eq!(
                next.counts.previous_event.as_deref(),
                Some("purchase_tapped")
            );
            assert_eq!(next.counts.previous_attempt_count, Some(previous));
            assert_eq!(next.counts.converted_attempt_count, Some(0));
            assert_eq!(next.conversion_rate, Some(0.0));
        }
        let unrelated = find("2026-10-16", "membership", "V8", "order_create_requested");
        assert_eq!(unrelated.counts.previous_attempt_count, Some(0));
        assert_eq!(unrelated.conversion_rate, None);
        let cancelled = find("2026-10-16", "membership", "V7", "pay_cancelled");
        assert_eq!(cancelled.counts.attempt_count, 1);
        assert_eq!(cancelled.conversion_rate, None);
    }
}
