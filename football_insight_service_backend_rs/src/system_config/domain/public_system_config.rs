use serde::Deserialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HomeBriefingMarquees {
    pub leader: Vec<String>,
    pub scorer: Vec<String>,
    pub assist: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipTierRuleConfig {
    pub code: String,
    pub kind: String,
    pub min_referrals: Option<i32>,
    pub ticket_watch_poll_interval_seconds: i32,
}

impl MembershipTierRuleConfig {
    pub fn new(
        code: impl Into<String>,
        kind: impl Into<String>,
        min_referrals: Option<i32>,
        ticket_watch_poll_interval_seconds: i32,
    ) -> Self {
        Self {
            code: code.into(),
            kind: kind.into(),
            min_referrals,
            ticket_watch_poll_interval_seconds,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct MembershipTierRulePayload {
    code: String,
    kind: String,
    min_referrals: Option<i32>,
    ticket_watch_poll_interval_seconds: Option<i32>,
}

fn clean_membership_tier_rule(
    payload: MembershipTierRulePayload,
) -> Option<MembershipTierRuleConfig> {
    let code = payload.code.trim().to_uppercase();
    let kind = payload.kind.trim().to_ascii_lowercase();
    let poll_interval_seconds = payload.ticket_watch_poll_interval_seconds?;

    if code.is_empty() || kind.is_empty() || poll_interval_seconds <= 0 {
        return None;
    }

    Some(MembershipTierRuleConfig {
        code,
        kind,
        min_referrals: payload.min_referrals.filter(|value| *value >= 0),
        ticket_watch_poll_interval_seconds: poll_interval_seconds,
    })
}

pub fn default_membership_tier_rules() -> Vec<MembershipTierRuleConfig> {
    vec![
        MembershipTierRuleConfig::new("V1", "standard", None, 600),
        MembershipTierRuleConfig::new("V2", "standard", Some(2), 300),
        MembershipTierRuleConfig::new("V3", "invite", Some(4), 120),
        MembershipTierRuleConfig::new("V4", "referral", Some(10), 60),
        MembershipTierRuleConfig::new("V5", "referral", Some(15), 30),
        MembershipTierRuleConfig::new("V6", "referral", Some(20), 15),
        MembershipTierRuleConfig::new("V7", "referral", Some(30), 7),
        MembershipTierRuleConfig::new("V8", "referral", Some(50), 3),
        MembershipTierRuleConfig::new("V9", "referral", Some(80), 1),
    ]
}

pub fn parse_membership_tier_rules(value: Option<&str>) -> Vec<MembershipTierRuleConfig> {
    let Some(value) = value else {
        return default_membership_tier_rules();
    };

    let Ok(payloads) = serde_json::from_str::<Vec<MembershipTierRulePayload>>(value) else {
        return default_membership_tier_rules();
    };

    let rules = payloads
        .into_iter()
        .filter_map(clean_membership_tier_rule)
        .collect::<Vec<_>>();

    if rules.is_empty() {
        return default_membership_tier_rules();
    }

    rules
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiChatMode {
    #[default]
    BackendProxy,
    FrontendDirect,
}

impl AiChatMode {
    pub fn from_config_value(value: Option<&str>) -> Self {
        match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
            Some("frontend_direct") => Self::FrontendDirect,
            Some("backend_proxy") => Self::BackendProxy,
            _ => Self::BackendProxy,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BackendProxy => "backend_proxy",
            Self::FrontendDirect => "frontend_direct",
        }
    }
}

impl Display for AiChatMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicSystemConfig {
    pub wechat_login_enabled: bool,
    pub ai_chat_mode: AiChatMode,
    pub home_briefing_marquees: HomeBriefingMarquees,
    pub membership_tier_rules: Vec<MembershipTierRuleConfig>,
}

impl PublicSystemConfig {
    pub fn new(
        wechat_login_enabled: bool,
        ai_chat_mode: AiChatMode,
        home_briefing_marquees: HomeBriefingMarquees,
        membership_tier_rules: Vec<MembershipTierRuleConfig>,
    ) -> Self {
        Self {
            wechat_login_enabled,
            ai_chat_mode,
            home_briefing_marquees,
            membership_tier_rules,
        }
    }
}
