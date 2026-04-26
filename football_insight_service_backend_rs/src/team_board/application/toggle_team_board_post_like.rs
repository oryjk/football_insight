use std::sync::Arc;

use uuid::Uuid;

use crate::team_board::{
    domain::team_board::TeamBoardLikeSummary, ports::team_board_repository::TeamBoardRepository,
};

pub struct ToggleTeamBoardPostLikeUseCase {
    repository: Arc<dyn TeamBoardRepository>,
}

impl ToggleTeamBoardPostLikeUseCase {
    pub fn new(repository: Arc<dyn TeamBoardRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        post_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<TeamBoardLikeSummary> {
        self.repository.toggle_like(post_id, user_id).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::ToggleTeamBoardPostLikeUseCase;
    use crate::team_board::{
        domain::team_board::{
            NewTeamBoardComment, NewTeamBoardPost, TeamBoardComment, TeamBoardLikeSummary,
            TeamBoardPost,
        },
        ports::team_board_repository::TeamBoardRepository,
    };

    #[derive(Default)]
    struct FakeRepository {
        toggles: Mutex<Vec<(Uuid, Uuid)>>,
    }

    #[async_trait]
    impl TeamBoardRepository for FakeRepository {
        async fn list_posts(
            &self,
            _team_id: i64,
            _viewer_user_id: Uuid,
        ) -> anyhow::Result<Vec<TeamBoardPost>> {
            unreachable!()
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
            post_id: Uuid,
            user_id: Uuid,
        ) -> anyhow::Result<TeamBoardLikeSummary> {
            self.toggles.lock().unwrap().push((post_id, user_id));
            Ok(TeamBoardLikeSummary {
                post_id,
                liked_by_viewer: true,
                like_count: 3,
            })
        }
    }

    #[tokio::test]
    async fn execute_toggles_like() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = ToggleTeamBoardPostLikeUseCase::new(repository.clone());
        let post_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let user_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();

        let summary = use_case.execute(post_id, user_id).await.unwrap();

        assert!(summary.liked_by_viewer);
        assert_eq!(summary.like_count, 3);
        assert_eq!(repository.toggles.lock().unwrap().len(), 1);
    }
}
