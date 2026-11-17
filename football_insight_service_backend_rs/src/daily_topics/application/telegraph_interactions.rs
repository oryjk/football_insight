use std::sync::Arc;

use uuid::Uuid;

use crate::{
    content_security::{ContentSecurityError, ContentSecurityPort, ModerationScene},
    daily_topics::{
        domain::interactions::{
            CreateTelegraphCommentInput, ReactionMutation, TelegraphComment,
            TelegraphCommentLikeSummary, TelegraphCommentThreadPage, TelegraphCommentsPage,
            TelegraphCommentsPosition, TelegraphInteractionError, TelegraphReactionSummary,
            valid_telegraph_id, validate_comment_content,
        },
        ports::daily_topics_repository::TelegraphInteractionsRepository,
    },
};

pub struct TelegraphInteractionsService {
    repository: Arc<dyn TelegraphInteractionsRepository>,
    content_security: Arc<dyn ContentSecurityPort>,
}

impl TelegraphInteractionsService {
    pub fn new(
        repository: Arc<dyn TelegraphInteractionsRepository>,
        content_security: Arc<dyn ContentSecurityPort>,
    ) -> Self {
        Self {
            repository,
            content_security,
        }
    }

    pub async fn set_reaction(
        &self,
        team_id: i64,
        topic_id: &str,
        entry_id: &str,
        user_id: Uuid,
        mutation: ReactionMutation,
    ) -> Result<TelegraphReactionSummary, TelegraphInteractionError> {
        validate_scope(team_id, topic_id, entry_id)?;
        self.repository
            .set_entry_reaction(team_id, topic_id, entry_id, user_id, mutation)
            .await
    }

    pub async fn list_comments(
        &self,
        team_id: i64,
        topic_id: &str,
        entry_id: &str,
        viewer_user_id: Option<Uuid>,
        after: Option<&TelegraphCommentsPosition>,
        limit: usize,
        root_only: bool,
    ) -> Result<TelegraphCommentsPage, TelegraphInteractionError> {
        validate_scope(team_id, topic_id, entry_id)?;
        if !(1..=50).contains(&limit) {
            return Err(TelegraphInteractionError::InvalidInput("分页大小无效"));
        }
        self.repository
            .list_entry_comments(
                team_id,
                topic_id,
                entry_id,
                viewer_user_id,
                after,
                limit,
                root_only,
            )
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_comment_thread(
        &self,
        team_id: i64,
        topic_id: &str,
        entry_id: &str,
        comment_id: Uuid,
        viewer_user_id: Option<Uuid>,
        after: Option<&TelegraphCommentsPosition>,
        limit: usize,
    ) -> Result<TelegraphCommentThreadPage, TelegraphInteractionError> {
        validate_scope(team_id, topic_id, entry_id)?;
        if !(1..=50).contains(&limit) {
            return Err(TelegraphInteractionError::InvalidInput("分页大小无效"));
        }
        self.repository
            .get_comment_thread(
                team_id,
                topic_id,
                entry_id,
                comment_id,
                viewer_user_id,
                after,
                limit,
            )
            .await
    }

    pub async fn create_comment(
        &self,
        mut input: CreateTelegraphCommentInput,
    ) -> Result<(TelegraphComment, u64), TelegraphInteractionError> {
        validate_scope(input.team_id, &input.topic_id, &input.entry_id)?;
        input.content = validate_comment_content(&input.content)?;

        let open_id = self
            .repository
            .check_comment_eligibility(
                input.team_id,
                &input.topic_id,
                &input.entry_id,
                input.user_id,
                input.reply_to_comment_id,
            )
            .await?;

        self.content_security
            .check_text(
                ModerationScene::Comment,
                open_id.as_deref(),
                &input.content,
            )
            .await
            .map_err(|error| match error {
                ContentSecurityError::Rejected => TelegraphInteractionError::ContentRejected,
                ContentSecurityError::Unavailable => {
                    TelegraphInteractionError::ModerationUnavailable
                }
            })?;

        self.repository.insert_entry_comment(&input).await
    }

    pub async fn delete_comment(
        &self,
        team_id: i64,
        topic_id: &str,
        entry_id: &str,
        comment_id: Uuid,
        user_id: Uuid,
    ) -> Result<u64, TelegraphInteractionError> {
        validate_scope(team_id, topic_id, entry_id)?;
        self.repository
            .delete_entry_comment(team_id, topic_id, entry_id, comment_id, user_id)
            .await
    }

    pub async fn set_comment_like(
        &self,
        team_id: i64,
        topic_id: &str,
        entry_id: &str,
        comment_id: Uuid,
        user_id: Uuid,
        active: bool,
    ) -> Result<TelegraphCommentLikeSummary, TelegraphInteractionError> {
        validate_scope(team_id, topic_id, entry_id)?;
        self.repository
            .set_comment_like(team_id, topic_id, entry_id, comment_id, user_id, active)
            .await
    }
}

fn validate_scope(
    team_id: i64,
    topic_id: &str,
    entry_id: &str,
) -> Result<(), TelegraphInteractionError> {
    if team_id <= 0 || !valid_telegraph_id(topic_id) || !valid_telegraph_id(entry_id) {
        return Err(TelegraphInteractionError::InvalidInput("电报参数无效"));
    }
    Ok(())
}
