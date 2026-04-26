use std::sync::Arc;

use crate::insight::ports::insight_query_repository::InsightQueryRepository;
use crate::system_config::{
    domain::{
        home_briefing_marquees_builder::{
            build_home_briefing_marquees_from_rankings, merge_home_briefing_marquees,
        },
        public_system_config::PublicSystemConfig,
    },
    ports::system_config_port::SystemConfigPort,
};

pub struct GetPublicSystemConfigUseCase {
    system_config_port: Arc<dyn SystemConfigPort>,
    insight_query_repository: Arc<dyn InsightQueryRepository>,
}

impl GetPublicSystemConfigUseCase {
    pub fn new(
        system_config_port: Arc<dyn SystemConfigPort>,
        insight_query_repository: Arc<dyn InsightQueryRepository>,
    ) -> Self {
        Self {
            system_config_port,
            insight_query_repository,
        }
    }

    pub async fn execute(&self) -> anyhow::Result<PublicSystemConfig> {
        let mut config = self.system_config_port.get_public_config().await?;

        if let Ok(rankings) = self.insight_query_repository.get_live_rankings().await {
            config.home_briefing_marquees = merge_home_briefing_marquees(
                build_home_briefing_marquees_from_rankings(&rankings),
                config.home_briefing_marquees.clone(),
            );
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::anyhow;
    use async_trait::async_trait;

    use super::GetPublicSystemConfigUseCase;
    use crate::insight::domain::{
        match_list::MatchListView, overview::InsightOverview, round_reference::RoundReference,
        team_insight::TeamInsightsView,
    };
    use crate::insight::{
        domain::rankings::{
            PlayerRankingCategory, PlayerRankingEntry, RankingsView, StandingsTable,
            StandingsTableEntry,
        },
        ports::insight_query_repository::InsightQueryRepository,
    };
    use crate::system_config::{
        domain::{
            ai_chat_config::AiChatSystemConfig,
            public_system_config::{
                AiChatMode, HomeBriefingMarquees, MembershipTierRuleConfig, PublicSystemConfig,
            },
        },
        ports::system_config_port::SystemConfigPort,
    };

    struct FakeSystemConfigPort {
        wechat_login_enabled: bool,
        ai_chat_mode: AiChatMode,
        home_briefing_marquees: HomeBriefingMarquees,
    }

    #[async_trait]
    impl SystemConfigPort for FakeSystemConfigPort {
        async fn get_public_config(&self) -> anyhow::Result<PublicSystemConfig> {
            Ok(PublicSystemConfig::new(
                self.wechat_login_enabled,
                self.ai_chat_mode,
                self.home_briefing_marquees.clone(),
                vec![MembershipTierRuleConfig::new("V3", "invite", Some(0), 300)],
            ))
        }

        async fn get_ai_chat_config(&self) -> anyhow::Result<AiChatSystemConfig> {
            Ok(AiChatSystemConfig::default())
        }

        async fn get_config_value(&self, _config_key: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }
    }

    struct FakeInsightQueryRepository {
        live_rankings: Option<RankingsView>,
        should_fail: bool,
    }

    #[async_trait]
    impl InsightQueryRepository for FakeInsightQueryRepository {
        async fn get_live_overview(&self) -> anyhow::Result<InsightOverview> {
            Err(anyhow!("unused in test"))
        }

        async fn get_round_overview(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<InsightOverview> {
            Err(anyhow!("unused in test"))
        }

        async fn list_available_rounds(&self, _season: i32) -> anyhow::Result<Vec<RoundReference>> {
            Err(anyhow!("unused in test"))
        }

        async fn get_live_rankings(&self) -> anyhow::Result<RankingsView> {
            if self.should_fail {
                return Err(anyhow!("live rankings unavailable"));
            }

            self.live_rankings
                .clone()
                .ok_or_else(|| anyhow!("missing live rankings fixture"))
        }

        async fn get_live_team_insights(&self) -> anyhow::Result<TeamInsightsView> {
            Err(anyhow!("unused in test"))
        }

        async fn get_round_rankings(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<RankingsView> {
            Err(anyhow!("unused in test"))
        }

        async fn get_live_matches(&self) -> anyhow::Result<MatchListView> {
            Err(anyhow!("unused in test"))
        }

        async fn get_round_matches(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<MatchListView> {
            Err(anyhow!("unused in test"))
        }
    }

    fn create_live_rankings() -> RankingsView {
        RankingsView {
            view_kind: "live".to_string(),
            round_number: None,
            current_season: 2026,
            standings_tables: vec![StandingsTable {
                slug: "standings".to_string(),
                label: "积分榜".to_string(),
                note: String::new(),
                entries: vec![
                    StandingsTableEntry {
                        rank_no: 1,
                        team_id: 1,
                        team_name: "成都蓉城".to_string(),
                        played: 7,
                        wins: 5,
                        draws: 1,
                        losses: 1,
                        goals_for: 14,
                        goals_against: 6,
                        goal_difference: 8,
                        points: 16,
                        points_without_penalty: 16,
                        points_adjustment: 0,
                        avatar_storage_url: None,
                    },
                    StandingsTableEntry {
                        rank_no: 2,
                        team_id: 2,
                        team_name: "上海申花".to_string(),
                        played: 7,
                        wins: 5,
                        draws: 0,
                        losses: 2,
                        goals_for: 13,
                        goals_against: 7,
                        goal_difference: 6,
                        points: 15,
                        points_without_penalty: 15,
                        points_adjustment: 0,
                        avatar_storage_url: None,
                    },
                    StandingsTableEntry {
                        rank_no: 3,
                        team_id: 3,
                        team_name: "北京国安".to_string(),
                        played: 7,
                        wins: 4,
                        draws: 2,
                        losses: 1,
                        goals_for: 12,
                        goals_against: 8,
                        goal_difference: 4,
                        points: 14,
                        points_without_penalty: 14,
                        points_adjustment: 0,
                        avatar_storage_url: None,
                    },
                ],
            }],
            team_categories: vec![],
            player_categories: vec![
                PlayerRankingCategory {
                    slug: "goals".to_string(),
                    label: "射手榜".to_string(),
                    item_id: 1,
                    entries: vec![
                        PlayerRankingEntry {
                            rank_no: 1,
                            player_id: 11,
                            player_name: "费利佩".to_string(),
                            team_id: 1,
                            team_name: "成都蓉城".to_string(),
                            score_value: "7".to_string(),
                            penalty_value: Some("1".to_string()),
                            avatar_storage_url: None,
                        },
                        PlayerRankingEntry {
                            rank_no: 2,
                            player_id: 12,
                            player_name: "莱昂纳多".to_string(),
                            team_id: 2,
                            team_name: "上海申花".to_string(),
                            score_value: "6".to_string(),
                            penalty_value: Some("0".to_string()),
                            avatar_storage_url: None,
                        },
                        PlayerRankingEntry {
                            rank_no: 3,
                            player_id: 13,
                            player_name: "法比奥".to_string(),
                            team_id: 3,
                            team_name: "北京国安".to_string(),
                            score_value: "5".to_string(),
                            penalty_value: Some("0".to_string()),
                            avatar_storage_url: None,
                        },
                    ],
                },
                PlayerRankingCategory {
                    slug: "assists".to_string(),
                    label: "助攻榜".to_string(),
                    item_id: 2,
                    entries: vec![
                        PlayerRankingEntry {
                            rank_no: 1,
                            player_id: 21,
                            player_name: "罗慕洛".to_string(),
                            team_id: 1,
                            team_name: "成都蓉城".to_string(),
                            score_value: "7".to_string(),
                            penalty_value: None,
                            avatar_storage_url: None,
                        },
                        PlayerRankingEntry {
                            rank_no: 2,
                            player_id: 22,
                            player_name: "奥斯卡".to_string(),
                            team_id: 4,
                            team_name: "上海海港".to_string(),
                            score_value: "5".to_string(),
                            penalty_value: None,
                            avatar_storage_url: None,
                        },
                        PlayerRankingEntry {
                            rank_no: 3,
                            player_id: 23,
                            player_name: "谢文能".to_string(),
                            team_id: 5,
                            team_name: "山东泰山".to_string(),
                            score_value: "4".to_string(),
                            penalty_value: None,
                            avatar_storage_url: None,
                        },
                    ],
                },
            ],
        }
    }

    #[tokio::test]
    async fn execute_prefers_generated_marquees_from_live_rankings() {
        let use_case = GetPublicSystemConfigUseCase::new(
            Arc::new(FakeSystemConfigPort {
                wechat_login_enabled: false,
                ai_chat_mode: AiChatMode::FrontendDirect,
                home_briefing_marquees: HomeBriefingMarquees {
                    leader: vec!["旧的榜首文案".to_string()],
                    scorer: vec!["旧的射手文案".to_string()],
                    assist: vec!["旧的助攻文案".to_string()],
                },
            }),
            Arc::new(FakeInsightQueryRepository {
                live_rankings: Some(create_live_rankings()),
                should_fail: false,
            }),
        );

        let config = use_case
            .execute()
            .await
            .expect("public config should resolve");

        assert!(!config.wechat_login_enabled);
        assert_eq!(config.ai_chat_mode, AiChatMode::FrontendDirect);
        assert_eq!(
            config.home_briefing_marquees.leader,
            vec![
                "成都蓉城暂时领跑积分榜",
                "榜首与第二名只差 1 分",
                "前 3 名之间只差 2 分",
            ]
        );
        assert_eq!(
            config.home_briefing_marquees.scorer,
            vec![
                "费利佩暂时领跑射手榜",
                "只领先第二名 1 球",
                "前 3 名射手只差 2 球",
            ]
        );
        assert_eq!(
            config.home_briefing_marquees.assist,
            vec![
                "罗慕洛暂时领跑助攻榜",
                "只领先第二名 2 次助攻",
                "前 3 名助攻手只差 3 次助攻",
            ]
        );
        assert_eq!(config.membership_tier_rules[0].code, "V3");
    }

    #[tokio::test]
    async fn execute_falls_back_to_configured_marquees_when_live_rankings_fail() {
        let use_case = GetPublicSystemConfigUseCase::new(
            Arc::new(FakeSystemConfigPort {
                wechat_login_enabled: false,
                ai_chat_mode: AiChatMode::FrontendDirect,
                home_briefing_marquees: HomeBriefingMarquees {
                    leader: vec!["成都这波抢分含金量很高".to_string()],
                    scorer: vec!["头名射手还在扩大优势".to_string()],
                    assist: vec!["助攻榜竞争仍然胶着".to_string()],
                },
            }),
            Arc::new(FakeInsightQueryRepository {
                live_rankings: None,
                should_fail: true,
            }),
        );

        let config = use_case
            .execute()
            .await
            .expect("public config should resolve");

        assert_eq!(
            config.home_briefing_marquees.leader,
            vec!["成都这波抢分含金量很高"]
        );
        assert_eq!(
            config.home_briefing_marquees.scorer,
            vec!["头名射手还在扩大优势"]
        );
        assert_eq!(
            config.home_briefing_marquees.assist,
            vec!["助攻榜竞争仍然胶着"]
        );
    }
}
