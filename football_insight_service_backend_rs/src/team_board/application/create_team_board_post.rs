use std::sync::Arc;

use uuid::Uuid;

use crate::{
    insight::{
        domain::team_insight::TeamInsight, ports::insight_query_repository::InsightQueryRepository,
    },
    team_board::{
        domain::team_board::{
            NewTeamBoardPost, TeamBoardInsightKind, TeamBoardPost, build_snapshot_from_team_insight,
        },
        ports::team_board_repository::TeamBoardRepository,
    },
};

#[derive(Debug, Clone)]
pub struct CreateTeamBoardPostInput {
    pub team_id: i64,
    pub author_user_id: Uuid,
    pub insight_kind: TeamBoardInsightKind,
    pub title: String,
    pub commentary: String,
}

pub struct CreateTeamBoardPostUseCase {
    repository: Arc<dyn TeamBoardRepository>,
    insight_repository: Arc<dyn InsightQueryRepository>,
}

impl CreateTeamBoardPostUseCase {
    pub fn new(
        repository: Arc<dyn TeamBoardRepository>,
        insight_repository: Arc<dyn InsightQueryRepository>,
    ) -> Self {
        Self {
            repository,
            insight_repository,
        }
    }

    pub async fn execute(&self, input: CreateTeamBoardPostInput) -> anyhow::Result<TeamBoardPost> {
        let title = input.title.trim().to_string();
        if title.is_empty() {
            anyhow::bail!("title is required");
        }

        let commentary = input.commentary.trim().to_string();
        if commentary.is_empty() {
            anyhow::bail!("commentary is required");
        }

        let team_insights = self.insight_repository.get_live_team_insights().await?;
        let insight = find_team_insight(&team_insights, input.team_id)?;
        let snapshot = build_snapshot_from_team_insight(
            team_insights.current_season,
            team_insights.round_number,
            insight,
            input.insight_kind,
        );

        self.repository
            .create_post(NewTeamBoardPost {
                team_id: input.team_id,
                author_user_id: input.author_user_id,
                insight_kind: input.insight_kind,
                title,
                commentary,
                snapshot,
            })
            .await
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
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    use super::{CreateTeamBoardPostInput, CreateTeamBoardPostUseCase};
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

    #[derive(Default)]
    struct FakeTeamBoardRepository {
        created_posts: Mutex<Vec<NewTeamBoardPost>>,
    }

    #[async_trait]
    impl TeamBoardRepository for FakeTeamBoardRepository {
        async fn list_posts(
            &self,
            _team_id: i64,
            _viewer_user_id: Uuid,
        ) -> anyhow::Result<Vec<TeamBoardPost>> {
            unreachable!()
        }

        async fn create_post(&self, input: NewTeamBoardPost) -> anyhow::Result<TeamBoardPost> {
            self.created_posts.lock().unwrap().push(input.clone());

            Ok(TeamBoardPost {
                post_id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                team_id: input.team_id,
                insight_kind: input.insight_kind,
                insight_label: input.insight_kind.label().to_string(),
                title: input.title,
                commentary: input.commentary,
                author: TeamBoardPostAuthor {
                    user_id: input.author_user_id,
                    display_name: "测试用户".to_string(),
                    avatar_url: None,
                },
                snapshot: input.snapshot,
                like_count: 0,
                comment_count: 0,
                liked_by_viewer: false,
                created_at: Utc::now(),
                comments: Vec::new(),
            })
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
    async fn execute_creates_goal_contribution_post_with_snapshot() {
        let repository = Arc::new(FakeTeamBoardRepository::default());
        let use_case =
            CreateTeamBoardPostUseCase::new(repository.clone(), Arc::new(FakeInsightRepository));

        let result = use_case
            .execute(CreateTeamBoardPostInput {
                team_id: 1,
                author_user_id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                insight_kind: TeamBoardInsightKind::GoalsFor,
                title: "费利佩还是最稳的终结点".to_string(),
                commentary: "进球虽然分散，但头部终结点还是费利佩。".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.snapshot.summary_label, "总进球");
        assert_eq!(result.snapshot.summary_value, 14);
        assert_eq!(result.snapshot.sections.len(), 2);
        assert_eq!(result.snapshot.sections[0].title, "对手维度");
        assert_eq!(result.snapshot.sections[1].title, "球员维度");

        let created_posts = repository.created_posts.lock().unwrap().clone();
        assert_eq!(created_posts.len(), 1);
        assert_eq!(created_posts[0].snapshot.summary_value, 14);
        assert_eq!(
            created_posts[0].snapshot.sections[1].items[0].name,
            "费利佩"
        );
    }
}
