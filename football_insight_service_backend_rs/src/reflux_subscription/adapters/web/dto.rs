use serde::{Deserialize, Serialize};

use crate::{
    payment::adapters::web::dto::WxPayParamsDto,
    reflux_subscription::domain::subscription::{
        NotificationTarget, RefluxSubscriptionPlan, RefluxSubscriptionScope, UserRefluxSubscription,
    },
};

#[derive(Debug, Serialize)]
pub struct RefluxSubscriptionPlanDto {
    pub code: String,
    pub scope: String,
    pub team_code: String,
    pub season: Option<i32>,
    pub title: String,
    pub description: String,
    pub price_cents: i32,
    pub expires_at: Option<String>,
}

impl From<RefluxSubscriptionPlan> for RefluxSubscriptionPlanDto {
    fn from(value: RefluxSubscriptionPlan) -> Self {
        Self {
            code: value.code,
            scope: scope_to_string(&value.scope),
            team_code: value.team_code,
            season: value.season,
            title: value.title,
            description: value.description,
            price_cents: value.price_cents,
            expires_at: value.expires_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RefluxSubscriptionSummaryDto {
    pub scope: String,
    pub team_code: String,
    pub season: Option<i32>,
    pub match_id: Option<i64>,
    pub starts_at: String,
    pub expires_at: Option<String>,
}

impl From<UserRefluxSubscription> for RefluxSubscriptionSummaryDto {
    fn from(value: UserRefluxSubscription) -> Self {
        Self {
            scope: scope_to_string(&value.scope),
            team_code: value.team_code,
            season: value.season,
            match_id: value.match_id,
            starts_at: value.starts_at.to_rfc3339(),
            expires_at: value.expires_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RefluxNotificationTargetDto {
    pub channel: String,
    pub target: String,
}

impl From<NotificationTarget> for RefluxNotificationTargetDto {
    fn from(value: NotificationTarget) -> Self {
        Self {
            channel: value.channel,
            target: value.target,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RefluxSubscriptionPlansResponse {
    pub plans: Vec<RefluxSubscriptionPlanDto>,
    pub active_subscriptions: Vec<RefluxSubscriptionSummaryDto>,
    pub email_target: Option<RefluxNotificationTargetDto>,
}

#[derive(Debug, Serialize)]
pub struct RefluxSubscriptionStatusResponse {
    pub subscribed: bool,
    pub active_subscriptions: Vec<RefluxSubscriptionSummaryDto>,
    pub email_target: Option<RefluxNotificationTargetDto>,
}

#[derive(Debug, Deserialize)]
pub struct RefluxSubscriptionQuery {
    pub team_code: String,
    pub season: Option<i32>,
    pub match_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRefluxSubscriptionOrderRequest {
    pub plan_code: String,
    pub team_code: String,
    pub match_id: Option<i64>,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct CreateRefluxSubscriptionOrderResponse {
    pub order_no: String,
    pub params: WxPayParamsDto,
}

#[derive(Debug, Serialize)]
pub struct RefluxNotificationEmailResponse {
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRefluxNotificationEmailRequest {
    pub email: String,
}

fn scope_to_string(scope: &RefluxSubscriptionScope) -> String {
    match scope {
        RefluxSubscriptionScope::SingleMatch => "single_match",
        RefluxSubscriptionScope::Season => "season",
        RefluxSubscriptionScope::Lifetime => "lifetime",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    use super::RefluxSubscriptionPlanDto;
    use crate::reflux_subscription::domain::subscription::{
        RefluxSubscriptionPlan, RefluxSubscriptionScope,
    };

    #[test]
    fn plan_dto_serializes_scope_and_expiration() {
        let dto = RefluxSubscriptionPlanDto::from(RefluxSubscriptionPlan {
            code: "season_2026".to_string(),
            scope: RefluxSubscriptionScope::Season,
            team_code: "global".to_string(),
            season: Some(2026),
            title: "赛季".to_string(),
            description: "desc".to_string(),
            price_cents: 5000,
            enabled: true,
            sort_order: 20,
            expires_at: Some(Utc.with_ymd_and_hms(2026, 12, 31, 15, 59, 59).unwrap()),
        });

        assert_eq!(
            serde_json::to_value(dto).expect("serialize"),
            json!({
                "code": "season_2026",
                "scope": "season",
                "team_code": "global",
                "season": 2026,
                "title": "赛季",
                "description": "desc",
                "price_cents": 5000,
                "expires_at": "2026-12-31T15:59:59+00:00"
            })
        );
    }
}
