use std::sync::Arc;

use uuid::Uuid;

use crate::team_board::{
    domain::team_board::{NewTeamBoardComment, TeamBoardComment},
    ports::team_board_repository::TeamBoardRepository,
};

#[derive(Debug, Clone)]
pub struct AddTeamBoardCommentInput {
    pub post_id: Uuid,
    pub author_user_id: Uuid,
    pub content: String,
}

pub struct AddTeamBoardCommentUseCase {
    repository: Arc<dyn TeamBoardRepository>,
}

impl AddTeamBoardCommentUseCase {
    pub fn new(repository: Arc<dyn TeamBoardRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        input: AddTeamBoardCommentInput,
    ) -> anyhow::Result<TeamBoardComment> {
        let content = input.content.trim().to_string();
        if content.is_empty() {
            anyhow::bail!("comment content is required");
        }

        self.repository
            .add_comment(NewTeamBoardComment {
                post_id: input.post_id,
                author_user_id: input.author_user_id,
                content,
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    use super::{AddTeamBoardCommentInput, AddTeamBoardCommentUseCase};
    use crate::team_board::{
        domain::team_board::{
            NewTeamBoardComment, NewTeamBoardPost, TeamBoardComment, TeamBoardLikeSummary,
            TeamBoardPost, TeamBoardPostAuthor,
        },
        ports::team_board_repository::TeamBoardRepository,
    };

    #[derive(Default)]
    struct FakeRepository {
        comments: Mutex<Vec<NewTeamBoardComment>>,
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
            input: NewTeamBoardComment,
        ) -> anyhow::Result<TeamBoardComment> {
            self.comments.lock().unwrap().push(input.clone());

            Ok(TeamBoardComment {
                comment_id: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap(),
                post_id: input.post_id,
                author: TeamBoardPostAuthor {
                    user_id: input.author_user_id,
                    display_name: "评论用户".to_string(),
                    avatar_url: None,
                },
                content: input.content,
                created_at: Utc::now(),
            })
        }

        async fn toggle_like(
            &self,
            _post_id: Uuid,
            _user_id: Uuid,
        ) -> anyhow::Result<TeamBoardLikeSummary> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn execute_trims_comment_before_saving() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = AddTeamBoardCommentUseCase::new(repository.clone());
        let post_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();

        let comment = use_case
            .execute(AddTeamBoardCommentInput {
                post_id,
                author_user_id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                content: "  这张图已经能说明问题了  ".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(comment.content, "这张图已经能说明问题了");

        let saved = repository.comments.lock().unwrap().clone();
        assert_eq!(saved[0].content, "这张图已经能说明问题了");
    }
}
