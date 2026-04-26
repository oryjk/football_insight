use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::team_board::domain::team_board::{
    TeamBoardComment, TeamBoardComposerPreset, TeamBoardInsightKind, TeamBoardLikeSummary,
    TeamBoardPost, TeamBoardSnapshot, TeamBoardSnapshotItem, TeamBoardSnapshotSection,
    TeamBoardTeam, TeamBoardView,
};

#[derive(Debug, Deserialize)]
pub struct CreateTeamBoardPostRequest {
    pub insight_kind: TeamBoardInsightKind,
    pub title: String,
    pub commentary: String,
}

#[derive(Debug, Deserialize)]
pub struct AddTeamBoardCommentRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct TeamBoardViewDto {
    pub current_season: i32,
    pub round_number: Option<i32>,
    pub team: TeamBoardTeamDto,
    pub composer_presets: Vec<TeamBoardComposerPresetDto>,
    pub posts: Vec<TeamBoardPostDto>,
}

#[derive(Debug, Serialize)]
pub struct TeamBoardTeamDto {
    pub team_id: i64,
    pub team_name: String,
    pub rank_no: i32,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TeamBoardComposerPresetDto {
    pub insight_kind: TeamBoardInsightKind,
    pub label: String,
    pub snapshot: TeamBoardSnapshotDto,
}

#[derive(Debug, Serialize)]
pub struct TeamBoardSnapshotDto {
    pub current_season: i32,
    pub round_number: Option<i32>,
    pub team_id: i64,
    pub team_name: String,
    pub rank_no: i32,
    pub avatar_storage_url: Option<String>,
    pub insight_kind: TeamBoardInsightKind,
    pub insight_label: String,
    pub summary_label: String,
    pub summary_value: i32,
    pub sections: Vec<TeamBoardSnapshotSectionDto>,
}

#[derive(Debug, Serialize)]
pub struct TeamBoardSnapshotSectionDto {
    pub title: String,
    pub metric_label: String,
    pub items: Vec<TeamBoardSnapshotItemDto>,
}

#[derive(Debug, Serialize)]
pub struct TeamBoardSnapshotItemDto {
    pub item_id: Option<i64>,
    pub name: String,
    pub avatar_storage_url: Option<String>,
    pub value: i32,
    pub share: f64,
}

#[derive(Debug, Serialize)]
pub struct TeamBoardPostDto {
    pub post_id: Uuid,
    pub team_id: i64,
    pub insight_kind: TeamBoardInsightKind,
    pub insight_label: String,
    pub title: String,
    pub commentary: String,
    pub author: TeamBoardAuthorDto,
    pub snapshot: TeamBoardSnapshotDto,
    pub like_count: i64,
    pub comment_count: i64,
    pub liked_by_viewer: bool,
    pub created_at: String,
    pub comments: Vec<TeamBoardCommentDto>,
}

#[derive(Debug, Serialize)]
pub struct TeamBoardAuthorDto {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TeamBoardCommentDto {
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub author: TeamBoardAuthorDto,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TeamBoardLikeSummaryDto {
    pub post_id: Uuid,
    pub liked_by_viewer: bool,
    pub like_count: i64,
}

impl From<TeamBoardView> for TeamBoardViewDto {
    fn from(value: TeamBoardView) -> Self {
        Self {
            current_season: value.current_season,
            round_number: value.round_number,
            team: value.team.into(),
            composer_presets: value.composer_presets.into_iter().map(Into::into).collect(),
            posts: value.posts.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TeamBoardTeam> for TeamBoardTeamDto {
    fn from(value: TeamBoardTeam) -> Self {
        Self {
            team_id: value.team_id,
            team_name: value.team_name,
            rank_no: value.rank_no,
            avatar_storage_url: value.avatar_storage_url,
        }
    }
}

impl From<TeamBoardComposerPreset> for TeamBoardComposerPresetDto {
    fn from(value: TeamBoardComposerPreset) -> Self {
        Self {
            insight_kind: value.insight_kind,
            label: value.label,
            snapshot: value.snapshot.into(),
        }
    }
}

impl From<TeamBoardSnapshot> for TeamBoardSnapshotDto {
    fn from(value: TeamBoardSnapshot) -> Self {
        Self {
            current_season: value.current_season,
            round_number: value.round_number,
            team_id: value.team_id,
            team_name: value.team_name,
            rank_no: value.rank_no,
            avatar_storage_url: value.avatar_storage_url,
            insight_kind: value.insight_kind,
            insight_label: value.insight_label,
            summary_label: value.summary_label,
            summary_value: value.summary_value,
            sections: value.sections.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TeamBoardSnapshotSection> for TeamBoardSnapshotSectionDto {
    fn from(value: TeamBoardSnapshotSection) -> Self {
        Self {
            title: value.title,
            metric_label: value.metric_label,
            items: value.items.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TeamBoardSnapshotItem> for TeamBoardSnapshotItemDto {
    fn from(value: TeamBoardSnapshotItem) -> Self {
        Self {
            item_id: value.item_id,
            name: value.name,
            avatar_storage_url: value.avatar_storage_url,
            value: value.value,
            share: value.share,
        }
    }
}

impl From<TeamBoardPost> for TeamBoardPostDto {
    fn from(value: TeamBoardPost) -> Self {
        Self {
            post_id: value.post_id,
            team_id: value.team_id,
            insight_kind: value.insight_kind,
            insight_label: value.insight_label,
            title: value.title,
            commentary: value.commentary,
            author: value.author.into(),
            snapshot: value.snapshot.into(),
            like_count: value.like_count,
            comment_count: value.comment_count,
            liked_by_viewer: value.liked_by_viewer,
            created_at: value.created_at.to_rfc3339(),
            comments: value.comments.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<crate::team_board::domain::team_board::TeamBoardPostAuthor> for TeamBoardAuthorDto {
    fn from(value: crate::team_board::domain::team_board::TeamBoardPostAuthor) -> Self {
        Self {
            user_id: value.user_id,
            display_name: value.display_name,
            avatar_url: value.avatar_url,
        }
    }
}

impl From<TeamBoardComment> for TeamBoardCommentDto {
    fn from(value: TeamBoardComment) -> Self {
        Self {
            comment_id: value.comment_id,
            post_id: value.post_id,
            author: value.author.into(),
            content: value.content,
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

impl From<TeamBoardLikeSummary> for TeamBoardLikeSummaryDto {
    fn from(value: TeamBoardLikeSummary) -> Self {
        Self {
            post_id: value.post_id,
            liked_by_viewer: value.liked_by_viewer,
            like_count: value.like_count,
        }
    }
}
