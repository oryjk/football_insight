use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminTelegraphCommentStatusFilter {
    All,
    Visible,
    Hidden,
}

impl AdminTelegraphCommentStatusFilter {
    pub fn parse(raw: Option<&str>) -> Result<Self, AdminTelegraphCommentsError> {
        match raw.map(str::trim).filter(|v| !v.is_empty()) {
            None | Some("all") => Ok(Self::All),
            Some("visible") => Ok(Self::Visible),
            Some("hidden") => Ok(Self::Hidden),
            Some(_) => Err(AdminTelegraphCommentsError::InvalidInput(
                "status 必须为 all、visible 或 hidden",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminTelegraphCommentStatus {
    Visible,
    Hidden,
    Deleted,
}

impl AdminTelegraphCommentStatus {
    pub fn parse(raw: &str) -> Result<Self, AdminTelegraphCommentsError> {
        match raw.trim() {
            "visible" => Ok(Self::Visible),
            "hidden" => Ok(Self::Hidden),
            "deleted" => Ok(Self::Deleted),
            _ => Err(AdminTelegraphCommentsError::InvalidInput(
                "status 必须为 visible、hidden 或 deleted",
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
            Self::Deleted => "deleted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminTelegraphComment {
    pub id: Uuid,
    pub team_id: i64,
    pub topic_id: String,
    pub entry_id: String,
    pub entry_title: String,
    pub user_id: Uuid,
    pub author_display_name: String,
    pub author_avatar_url: Option<String>,
    pub content: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminTelegraphCommentsView {
    pub comments_enabled: bool,
    pub items: Vec<AdminTelegraphComment>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AdminTelegraphCommentsError {
    #[error("{0}")]
    InvalidInput(&'static str),
    #[error("评论不存在")]
    NotFound,
    #[error("电报评论管理服务暂不可用")]
    Internal,
}
