use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum RefluxSubscriptionError {
    #[error("请输入有效的邮箱地址")]
    InvalidNotificationEmail,
    #[error("请选择有效的提醒套餐")]
    InvalidPlan,
    #[error("请先绑定微信")]
    WechatBindingRequired,
    #[error("单场订阅需要选择比赛")]
    MatchRequiredForSingleMatchPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefluxSubscriptionScope {
    SingleMatch,
    Season,
    Lifetime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefluxSubscriptionStatus {
    Active,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefluxSubscriptionPlan {
    pub code: String,
    pub scope: RefluxSubscriptionScope,
    pub team_code: String,
    pub season: Option<i32>,
    pub title: String,
    pub description: String,
    pub price_cents: i32,
    pub enabled: bool,
    pub sort_order: i32,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationTarget {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub channel: String,
    pub target: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRefluxSubscription {
    pub scope: RefluxSubscriptionScope,
    pub team_code: String,
    pub season: Option<i32>,
    pub match_id: Option<i64>,
    pub status: RefluxSubscriptionStatus,
    pub starts_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefluxNotificationJob {
    pub id: uuid::Uuid,
    pub target: NotificationTarget,
    pub subject: String,
    pub body_html: String,
    pub attempts: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefluxEmailSubscriber {
    pub user_id: uuid::Uuid,
    pub target: NotificationTarget,
    pub subscription: UserRefluxSubscription,
}

pub fn is_valid_notification_email(email: &str) -> bool {
    static EMAIL_PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let email = email.trim();
    let pattern = EMAIL_PATTERN.get_or_init(|| {
        regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("valid email regex")
    });

    !email.is_empty() && email.len() <= 254 && pattern.is_match(email)
}

pub fn normalize_team_code(team_code: &str) -> String {
    team_code.trim().to_lowercase()
}

pub fn select_effective_plans(
    plans: Vec<RefluxSubscriptionPlan>,
    requested_team_code: &str,
) -> Vec<RefluxSubscriptionPlan> {
    let team_code = normalize_team_code(requested_team_code);
    let mut selected = std::collections::BTreeMap::<String, RefluxSubscriptionPlan>::new();

    for plan in plans.into_iter().filter(|plan| plan.enabled) {
        let normalized_plan_team = normalize_team_code(&plan.team_code);
        if normalized_plan_team != "global" && normalized_plan_team != team_code {
            continue;
        }

        let key = plan.code.trim().to_string();
        match selected.get(&key) {
            Some(existing) if normalize_team_code(&existing.team_code) != "global" => {}
            _ => {
                selected.insert(key, plan);
            }
        }
    }

    let mut values = selected.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| {
        left.sort_order
            .cmp(&right.sort_order)
            .then_with(|| left.code.cmp(&right.code))
    });
    values
}

pub fn subscription_matches_current_match(
    subscription: &UserRefluxSubscription,
    team_code: &str,
    season: i32,
    match_id: i64,
    now: DateTime<Utc>,
) -> bool {
    if subscription.status != RefluxSubscriptionStatus::Active {
        return false;
    }

    if normalize_team_code(&subscription.team_code) != normalize_team_code(team_code) {
        return false;
    }

    if subscription.starts_at > now {
        return false;
    }

    if subscription
        .expires_at
        .is_some_and(|expires_at| expires_at <= now)
    {
        return false;
    }

    match subscription.scope {
        RefluxSubscriptionScope::SingleMatch => subscription.match_id == Some(match_id),
        RefluxSubscriptionScope::Season => subscription.season == Some(season),
        RefluxSubscriptionScope::Lifetime => true,
    }
}

#[cfg(test)]
pub mod tests_support {
    use super::{RefluxSubscriptionPlan, RefluxSubscriptionScope};

    pub fn test_plan(
        code: &str,
        team_code: &str,
        price_cents: i32,
        sort_order: i32,
    ) -> RefluxSubscriptionPlan {
        RefluxSubscriptionPlan {
            code: code.to_string(),
            scope: RefluxSubscriptionScope::SingleMatch,
            team_code: team_code.to_string(),
            season: Some(2026),
            title: code.to_string(),
            description: String::new(),
            price_cents,
            enabled: true,
            sort_order,
            expires_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{
        RefluxSubscriptionPlan, RefluxSubscriptionScope, RefluxSubscriptionStatus,
        UserRefluxSubscription, is_valid_notification_email, select_effective_plans,
        subscription_matches_current_match,
    };

    fn plan(
        code: &str,
        team_code: &str,
        price_cents: i32,
        sort_order: i32,
    ) -> RefluxSubscriptionPlan {
        super::tests_support::test_plan(code, team_code, price_cents, sort_order)
    }

    #[test]
    fn validates_notification_email_format() {
        assert!(is_valid_notification_email("user@example.com"));
        assert!(is_valid_notification_email(
            " user.name+tag@football.example.cn "
        ));
        assert!(!is_valid_notification_email("invalid"));
        assert!(!is_valid_notification_email("@example.com"));
        assert!(!is_valid_notification_email("user@"));
        assert!(!is_valid_notification_email("user example@example.com"));
    }

    #[test]
    fn team_specific_plans_override_global_plans() {
        let plans = select_effective_plans(
            vec![
                plan("single_match", "global", 500, 10),
                plan("season_2026", "global", 5000, 20),
                plan("single_match", "chengdu", 600, 10),
                plan("single_match", "yunnanyukun", 700, 10),
            ],
            "chengdu",
        );

        assert_eq!(
            plans
                .iter()
                .map(|plan| (
                    plan.code.as_str(),
                    plan.team_code.as_str(),
                    plan.price_cents
                ))
                .collect::<Vec<_>>(),
            vec![
                ("single_match", "chengdu", 600),
                ("season_2026", "global", 5000)
            ]
        );
    }

    #[test]
    fn single_match_subscription_only_matches_bound_match() {
        let now = Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap();
        let subscription = UserRefluxSubscription {
            scope: RefluxSubscriptionScope::SingleMatch,
            team_code: "chengdu".to_string(),
            season: Some(2026),
            match_id: Some(571),
            status: RefluxSubscriptionStatus::Active,
            starts_at: now,
            expires_at: None,
        };

        assert!(subscription_matches_current_match(
            &subscription,
            "chengdu",
            2026,
            571,
            now
        ));
        assert!(!subscription_matches_current_match(
            &subscription,
            "chengdu",
            2026,
            572,
            now
        ));
        assert!(!subscription_matches_current_match(
            &subscription,
            "yunnanyukun",
            2026,
            571,
            now
        ));
    }

    #[test]
    fn season_subscription_matches_same_team_and_season_until_expiration() {
        let now = Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap();
        let subscription = UserRefluxSubscription {
            scope: RefluxSubscriptionScope::Season,
            team_code: "chengdu".to_string(),
            season: Some(2026),
            match_id: None,
            status: RefluxSubscriptionStatus::Active,
            starts_at: now,
            expires_at: Some(now + chrono::Duration::days(30)),
        };

        assert!(subscription_matches_current_match(
            &subscription,
            "chengdu",
            2026,
            571,
            now
        ));
        assert!(!subscription_matches_current_match(
            &subscription,
            "chengdu",
            2027,
            571,
            now
        ));
        assert!(!subscription_matches_current_match(
            &subscription,
            "chengdu",
            2026,
            571,
            now + chrono::Duration::days(31),
        ));
    }
}
