use crate::system_config::domain::{
    mini_program_review_config::MiniProgramReviewConfigView,
    public_system_config::{HomeBriefingMarquees, PublicSystemConfig},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct MiniProgramReviewConfigQuery {
    pub version: String,
    pub app_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MiniProgramReviewConfigDto {
    pub mini_program_app_id: String,
    pub mini_program_version: String,
    pub is_under_review: bool,
    pub matched: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicSystemConfigDto {
    pub wechat_login_enabled: bool,
    pub ai_chat_mode: String,
    pub home_briefing_marquees: HomeBriefingMarqueesDto,
    pub membership_tier_rules: Vec<MembershipTierRuleDto>,
}

#[derive(Debug, Serialize)]
pub struct HomeBriefingMarqueesDto {
    pub leader: Vec<String>,
    pub scorer: Vec<String>,
    pub assist: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MembershipTierRuleDto {
    pub code: String,
    pub kind: String,
    pub min_referrals: Option<i32>,
    pub ticket_watch_poll_interval_seconds: i32,
}

impl From<HomeBriefingMarquees> for HomeBriefingMarqueesDto {
    fn from(value: HomeBriefingMarquees) -> Self {
        Self {
            leader: value.leader,
            scorer: value.scorer,
            assist: value.assist,
        }
    }
}

impl From<PublicSystemConfig> for PublicSystemConfigDto {
    fn from(value: PublicSystemConfig) -> Self {
        Self {
            wechat_login_enabled: value.wechat_login_enabled,
            ai_chat_mode: value.ai_chat_mode.to_string(),
            home_briefing_marquees: value.home_briefing_marquees.into(),
            membership_tier_rules: value
                .membership_tier_rules
                .into_iter()
                .map(|item| MembershipTierRuleDto {
                    code: item.code,
                    kind: item.kind,
                    min_referrals: item.min_referrals,
                    ticket_watch_poll_interval_seconds: item.ticket_watch_poll_interval_seconds,
                })
                .collect(),
        }
    }
}

impl From<MiniProgramReviewConfigView> for MiniProgramReviewConfigDto {
    fn from(value: MiniProgramReviewConfigView) -> Self {
        Self {
            mini_program_app_id: value.mini_program_app_id,
            mini_program_version: value.mini_program_version,
            is_under_review: value.is_under_review,
            matched: value.matched,
            created_at: value.created_at.map(|value| value.to_rfc3339()),
            updated_at: value.updated_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{MiniProgramReviewConfigDto, PublicSystemConfigDto};
    use crate::system_config::domain::{
        mini_program_review_config::MiniProgramReviewConfigView,
        public_system_config::{
            AiChatMode, HomeBriefingMarquees, MembershipTierRuleConfig, PublicSystemConfig,
        },
    };

    #[test]
    fn public_system_config_dto_exposes_ai_chat_mode() {
        let dto = PublicSystemConfigDto::from(PublicSystemConfig::new(
            true,
            AiChatMode::FrontendDirect,
            HomeBriefingMarquees {
                leader: vec!["榜首压力还在传导".to_string()],
                scorer: vec![],
                assist: vec![],
            },
            vec![MembershipTierRuleConfig::new(
                "V4",
                "referral",
                Some(10),
                60,
            )],
        ));

        let payload = serde_json::to_value(dto).expect("serialize dto");

        assert_eq!(payload["wechat_login_enabled"], true);
        assert_eq!(payload["ai_chat_mode"], "frontend_direct");
        assert_eq!(payload["membership_tier_rules"][0]["code"], "V4");
        assert_eq!(
            payload["membership_tier_rules"][0]["ticket_watch_poll_interval_seconds"],
            60
        );
    }

    #[test]
    fn mini_program_review_config_dto_serializes_review_status() {
        let now = Utc.with_ymd_and_hms(2026, 4, 24, 20, 0, 0).unwrap();
        let dto = MiniProgramReviewConfigDto::from(MiniProgramReviewConfigView {
            mini_program_app_id: "".to_string(),
            mini_program_version: "1.2.3".to_string(),
            is_under_review: true,
            matched: true,
            created_at: Some(now),
            updated_at: Some(now),
        });

        let payload = serde_json::to_value(dto).expect("serialize dto");

        assert_eq!(payload["mini_program_version"], "1.2.3");
        assert_eq!(payload["is_under_review"], true);
        assert_eq!(payload["matched"], true);
        assert_eq!(payload["created_at"], "2026-04-24T20:00:00+00:00");
    }
}
