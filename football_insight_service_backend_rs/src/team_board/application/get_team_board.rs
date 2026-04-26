use std::sync::Arc;

use uuid::Uuid;

use crate::{
    insight::{
        domain::team_insight::TeamInsight, ports::insight_query_repository::InsightQueryRepository,
    },
    team_board::{
        domain::team_board::{
            TeamBoardComposerPreset, TeamBoardInsightKind, TeamBoardTeam, TeamBoardView,
            build_snapshot_from_team_insight,
        },
        ports::team_board_repository::TeamBoardRepository,
    },
};

pub struct GetTeamBoardUseCase {
    repository: Arc<dyn TeamBoardRepository>,
    insight_repository: Arc<dyn InsightQueryRepository>,
}

impl GetTeamBoardUseCase {
    pub fn new(
        repository: Arc<dyn TeamBoardRepository>,
        insight_repository: Arc<dyn InsightQueryRepository>,
    ) -> Self {
        Self {
            repository,
            insight_repository,
        }
    }

    pub async fn execute(
        &self,
        team_id: i64,
        viewer_user_id: Uuid,
    ) -> anyhow::Result<TeamBoardView> {
        let team_insights = self.insight_repository.get_live_team_insights().await?;
        let insight = find_team_insight(&team_insights, team_id)?;
        let posts = self.repository.list_posts(team_id, viewer_user_id).await?;

        Ok(TeamBoardView {
            current_season: team_insights.current_season,
            round_number: team_insights.round_number,
            team: TeamBoardTeam {
                team_id: insight.team_id,
                team_name: insight.team_name.clone(),
                rank_no: insight.rank_no,
                avatar_storage_url: insight.avatar_storage_url.clone(),
            },
            composer_presets: vec![
                TeamBoardComposerPreset {
                    insight_kind: TeamBoardInsightKind::GoalsFor,
                    label: TeamBoardInsightKind::GoalsFor.label().to_string(),
                    snapshot: build_snapshot_from_team_insight(
                        team_insights.current_season,
                        team_insights.round_number,
                        insight,
                        TeamBoardInsightKind::GoalsFor,
                    ),
                },
                TeamBoardComposerPreset {
                    insight_kind: TeamBoardInsightKind::AssistsFor,
                    label: TeamBoardInsightKind::AssistsFor.label().to_string(),
                    snapshot: build_snapshot_from_team_insight(
                        team_insights.current_season,
                        team_insights.round_number,
                        insight,
                        TeamBoardInsightKind::AssistsFor,
                    ),
                },
                TeamBoardComposerPreset {
                    insight_kind: TeamBoardInsightKind::GoalsAgainst,
                    label: TeamBoardInsightKind::GoalsAgainst.label().to_string(),
                    snapshot: build_snapshot_from_team_insight(
                        team_insights.current_season,
                        team_insights.round_number,
                        insight,
                        TeamBoardInsightKind::GoalsAgainst,
                    ),
                },
            ],
            posts,
        })
    }
}

fn find_team_insight<'a>(
    team_insights: &'a crate::insight::domain::team_insight::TeamInsightsView,
    team_id: i64,
) -> anyhow::Result<&'a TeamInsight> {
    team_insights
        .insights
        .iter()
        .find(|item| item.team_id == team_id)
        .ok_or_else(|| anyhow::anyhow!("team insight not found"))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    use super::GetTeamBoardUseCase;
    use crate::{
        insight::{
            domain::{
                match_list::MatchListView,
                overview::InsightOverview,
                rankings::RankingsView,
                round_reference::RoundReference,
                team_insight::{
                    AssistContribution, OpponentContribution, PlayerContribution, TeamInsight,
                    TeamInsightTeam, TeamInsightsView,
                },
            },
            ports::insight_query_repository::InsightQueryRepository,
        },
        team_board::{
            domain::team_board::{
                NewTeamBoardComment, NewTeamBoardPost, TeamBoardComment, TeamBoardInsightKind,
                TeamBoardLikeSummary, TeamBoardPost, TeamBoardPostAuthor,
            },
            ports::team_board_repository::TeamBoardRepository,
        },
    };

    struct FakeRepository;

    #[async_trait]
    impl TeamBoardRepository for FakeRepository {
        async fn list_posts(
            &self,
            team_id: i64,
            viewer_user_id: Uuid,
        ) -> anyhow::Result<Vec<TeamBoardPost>> {
            Ok(vec![TeamBoardPost {
                post_id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                team_id,
                insight_kind: TeamBoardInsightKind::GoalsAgainst,
                insight_label: "失球贡献".to_string(),
                title: "失球太集中在武汉三镇".to_string(),
                commentary: "丢球来源太集中，说明这类对手能持续打到弱点。".to_string(),
                author: TeamBoardPostAuthor {
                    user_id: viewer_user_id,
                    display_name: "测试用户".to_string(),
                    avatar_url: None,
                },
                snapshot: crate::team_board::domain::team_board::TeamBoardSnapshot {
                    current_season: 2026,
                    round_number: None,
                    team_id,
                    team_name: "成都蓉城".to_string(),
                    rank_no: 1,
                    avatar_storage_url: Some("team://chengdu".to_string()),
                    insight_kind: TeamBoardInsightKind::GoalsAgainst,
                    insight_label: "失球贡献".to_string(),
                    summary_label: "总失球".to_string(),
                    summary_value: 5,
                    sections: vec![
                        crate::team_board::domain::team_board::TeamBoardSnapshotSection {
                            title: "对手维度".to_string(),
                            metric_label: "球".to_string(),
                            items: vec![
                                crate::team_board::domain::team_board::TeamBoardSnapshotItem {
                                    item_id: Some(21),
                                    name: "武汉三镇".to_string(),
                                    avatar_storage_url: Some("team://wuhan".to_string()),
                                    value: 2,
                                    share: 0.4,
                                },
                            ],
                        },
                    ],
                },
                like_count: 2,
                comment_count: 1,
                liked_by_viewer: true,
                created_at: Utc::now(),
                comments: Vec::new(),
            }])
        }

        async fn create_post(&self, _input: NewTeamBoardPost) -> anyhow::Result<TeamBoardPost> {
            unreachable!()
        }

        async fn add_comment(
            &self,
            _input: NewTeamBoardComment,
        ) -> anyhow::Result<TeamBoardComment> {
            unreachable!()
        }

        async fn toggle_like(
            &self,
            _post_id: Uuid,
            _user_id: Uuid,
        ) -> anyhow::Result<TeamBoardLikeSummary> {
            unreachable!()
        }
    }

    struct FakeInsightRepository;

    #[async_trait]
    impl InsightQueryRepository for FakeInsightRepository {
        async fn get_live_overview(&self) -> anyhow::Result<InsightOverview> {
            unreachable!()
        }

        async fn get_round_overview(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<InsightOverview> {
            unreachable!()
        }

        async fn list_available_rounds(&self, _season: i32) -> anyhow::Result<Vec<RoundReference>> {
            unreachable!()
        }

        async fn get_live_rankings(&self) -> anyhow::Result<RankingsView> {
            unreachable!()
        }

        async fn get_live_team_insights(&self) -> anyhow::Result<TeamInsightsView> {
            Ok(TeamInsightsView {
                view_kind: "live".to_string(),
                round_number: None,
                current_season: 2026,
                teams: vec![TeamInsightTeam {
                    team_id: 1,
                    team_name: "成都蓉城".to_string(),
                    rank_no: 1,
                    avatar_storage_url: Some("team://chengdu".to_string()),
                }],
                insights: vec![TeamInsight {
                    team_id: 1,
                    team_name: "成都蓉城".to_string(),
                    rank_no: 1,
                    avatar_storage_url: Some("team://chengdu".to_string()),
                    goals_for_total: 14,
                    goals_against_total: 5,
                    goals_for_by_opponent: vec![OpponentContribution {
                        opponent_team_id: 11,
                        opponent_team_name: "青岛西海岸".to_string(),
                        opponent_avatar_storage_url: Some("team://west".to_string()),
                        goals: 5,
                        share: 0.357,
                    }],
                    goals_for_by_player: vec![PlayerContribution {
                        player_id: Some(100),
                        player_name: "费利佩".to_string(),
                        avatar_storage_url: Some("player://felipe".to_string()),
                        goals: 4,
                        share: 0.286,
                    }],
                    assists_for_by_player: vec![AssistContribution {
                        player_id: Some(101),
                        player_name: "席尔瓦".to_string(),
                        avatar_storage_url: Some("player://silva".to_string()),
                        assists: 3,
                        share: 0.375,
                    }],
                    goals_against_by_opponent: vec![OpponentContribution {
                        opponent_team_id: 21,
                        opponent_team_name: "武汉三镇".to_string(),
                        opponent_avatar_storage_url: Some("team://wuhan".to_string()),
                        goals: 2,
                        share: 0.4,
                    }],
                }],
            })
        }

        async fn get_round_rankings(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<RankingsView> {
            unreachable!()
        }

        async fn get_live_matches(&self) -> anyhow::Result<MatchListView> {
            unreachable!()
        }

        async fn get_round_matches(
            &self,
            _season: i32,
            _round_number: i32,
        ) -> anyhow::Result<MatchListView> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn execute_returns_team_header_presets_and_posts() {
        let use_case =
            GetTeamBoardUseCase::new(Arc::new(FakeRepository), Arc::new(FakeInsightRepository));

        let view = use_case
            .execute(
                1,
                Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(view.team.team_name, "成都蓉城");
        assert_eq!(view.composer_presets.len(), 3);
        assert_eq!(view.composer_presets[0].label, "进球贡献");
        assert_eq!(view.posts.len(), 1);
        assert_eq!(view.posts[0].insight_label, "失球贡献");
    }
}
