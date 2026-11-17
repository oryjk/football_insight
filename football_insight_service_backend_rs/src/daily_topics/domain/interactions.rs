use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelegraphReactionKind {
    Like,
    Dislike,
}

impl TelegraphReactionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Like => "like",
            Self::Dislike => "dislike",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionMutation {
    Set(TelegraphReactionKind),
    Clear(TelegraphReactionKind),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntryInteractionStats {
    pub likes_count: u64,
    pub dislikes_count: u64,
    pub comments_count: u64,
    pub liked_by_me: bool,
    pub disliked_by_me: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegraphReactionSummary {
    pub entry_id: String,
    pub likes_count: u64,
    pub dislikes_count: u64,
    pub liked_by_me: bool,
    pub disliked_by_me: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegraphCommentAuthor {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegraphCommentReplyRef {
    pub comment_id: Uuid,
    pub author_display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegraphCommentLikeSummary {
    pub comment_id: Uuid,
    pub likes_count: u64,
    pub liked_by_me: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegraphComment {
    pub id: Uuid,
    pub topic_id: String,
    pub entry_id: String,
    pub author: TelegraphCommentAuthor,
    pub reply_to: Option<TelegraphCommentReplyRef>,
    pub content: String,
    pub likes_count: u64,
    pub liked_by_me: bool,
    pub replies_count: u64,
    pub is_mine: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegraphCommentsPosition {
    pub created_at: DateTime<Utc>,
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegraphCommentsPage {
    pub comments_enabled: bool,
    pub total_count: u64,
    pub items: Vec<TelegraphComment>,
    pub next_cursor: Option<TelegraphCommentsPosition>,
    pub has_more: bool,
}

#[derive(Debug, Clone)]
pub struct CreateTelegraphCommentInput {
    pub team_id: i64,
    pub topic_id: String,
    pub entry_id: String,
    pub user_id: Uuid,
    pub content: String,
    pub reply_to_comment_id: Option<Uuid>,
}

#[derive(Debug, thiserror::Error)]
pub enum TelegraphInteractionError {
    #[error("电报条目不存在")]
    TopicOrEntryNotFound,
    #[error("请先登录")]
    Unauthorized,
    #[error("发表评论前请先上传头像并设置昵称")]
    ProfileIncomplete,
    #[error("评论功能暂时关闭")]
    CommentsDisabled,
    #[error("发言太快啦，请稍后再试")]
    RateLimited,
    #[error("评论包含敏感或违规内容，请修改后再试")]
    ContentRejected,
    #[error("{0}")]
    InvalidInput(&'static str),
    #[error("内容安全校验服务暂不可用，请稍后重试")]
    ModerationUnavailable,
    #[error("互动服务暂不可用，请稍后重试")]
    Unavailable,
}

pub fn valid_telegraph_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

pub fn validate_comment_content(raw: &str) -> Result<String, TelegraphInteractionError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(TelegraphInteractionError::InvalidInput("请输入评论内容"));
    }
    if trimmed.contains('\0') {
        return Err(TelegraphInteractionError::InvalidInput(
            "评论内容包含不支持的字符",
        ));
    }
    if trimmed.chars().count() > 200 {
        return Err(TelegraphInteractionError::InvalidInput(
            "评论不能超过 200 个字",
        ));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::{TelegraphInteractionError, valid_telegraph_id, validate_comment_content};

    #[test]
    fn comment_content_validation_trims_and_enforces_200_chars() {
        assert_eq!(validate_comment_content("  好球！ ").unwrap(), "好球！");
        assert!(matches!(
            validate_comment_content("   "),
            Err(TelegraphInteractionError::InvalidInput(_))
        ));
        assert!(validate_comment_content(&"球".repeat(200)).is_ok());
        assert!(matches!(
            validate_comment_content(&"球".repeat(201)),
            Err(TelegraphInteractionError::InvalidInput(_))
        ));
        assert!(matches!(
            validate_comment_content("abc\0def"),
            Err(TelegraphInteractionError::InvalidInput(_))
        ));
    }

    #[test]
    fn valid_telegraph_id_allows_sha256_and_blocks_slashes() {
        assert!(valid_telegraph_id(&"a".repeat(64)));
        assert!(!valid_telegraph_id(""));
        assert!(!valid_telegraph_id("../secret"));
    }
}
