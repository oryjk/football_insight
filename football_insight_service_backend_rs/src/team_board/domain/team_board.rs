use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

use crate::insight::domain::team_insight::TeamInsight;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum TeamBoardInsightKind {
    GoalsFor,
    AssistsFor,
    GoalsAgainst,
}

impl TeamBoardInsightKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::GoalsFor => "进球贡献",
            Self::AssistsFor => "助攻贡献",
            Self::GoalsAgainst => "失球贡献",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamBoardSnapshotItem {
    pub item_id: Option<i64>,
    pub name: String,
    pub avatar_storage_url: Option<String>,
    pub value: i32,
    pub share: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamBoardSnapshotSection {
    pub title: String,
    pub metric_label: String,
    pub items: Vec<TeamBoardSnapshotItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamBoardSnapshot {
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
    pub sections: Vec<TeamBoardSnapshotSection>,
}

#[derive(Debug, Clone)]
pub struct TeamBoardTeam {
    pub team_id: i64,
    pub team_name: String,
    pub rank_no: i32,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TeamBoardComposerPreset {
    pub insight_kind: TeamBoardInsightKind,
    pub label: String,
    pub snapshot: TeamBoardSnapshot,
}

#[derive(Debug, Clone)]
pub struct TeamBoardPostAuthor {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TeamBoardComment {
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub author: TeamBoardPostAuthor,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TeamBoardPost {
    pub post_id: Uuid,
    pub team_id: i64,
    pub insight_kind: TeamBoardInsightKind,
    pub insight_label: String,
    pub title: String,
    pub commentary: String,
    pub author: TeamBoardPostAuthor,
    pub snapshot: TeamBoardSnapshot,
    pub like_count: i64,
    pub comment_count: i64,
    pub liked_by_viewer: bool,
    pub created_at: DateTime<Utc>,
    pub comments: Vec<TeamBoardComment>,
}

#[derive(Debug, Clone)]
pub struct TeamBoardView {
    pub current_season: i32,
    pub round_number: Option<i32>,
    pub team: TeamBoardTeam,
    pub composer_presets: Vec<TeamBoardComposerPreset>,
    pub posts: Vec<TeamBoardPost>,
}

#[derive(Debug, Clone)]
pub struct NewTeamBoardPost {
    pub team_id: i64,
    pub author_user_id: Uuid,
    pub insight_kind: TeamBoardInsightKind,
    pub title: String,
    pub commentary: String,
    pub snapshot: TeamBoardSnapshot,
}

#[derive(Debug, Clone)]
pub struct NewTeamBoardComment {
    pub post_id: Uuid,
    pub author_user_id: Uuid,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct TeamBoardLikeSummary {
    pub post_id: Uuid,
    pub liked_by_viewer: bool,
    pub like_count: i64,
}

pub fn build_snapshot_from_team_insight(
    current_season: i32,
    round_number: Option<i32>,
    insight: &TeamInsight,
    insight_kind: TeamBoardInsightKind,
) -> TeamBoardSnapshot {
    match insight_kind {
        TeamBoardInsightKind::GoalsFor => TeamBoardSnapshot {
            current_season,
            round_number,
            team_id: insight.team_id,
            team_name: insight.team_name.clone(),
            rank_no: insight.rank_no,
            avatar_storage_url: insight.avatar_storage_url.clone(),
            insight_kind,
            insight_label: insight_kind.label().to_string(),
            summary_label: "总进球".to_string(),
            summary_value: insight.goals_for_total,
            sections: vec![
                TeamBoardSnapshotSection {
                    title: "对手维度".to_string(),
                    metric_label: "球".to_string(),
                    items: insight
                        .goals_for_by_opponent
                        .iter()
                        .map(|item| TeamBoardSnapshotItem {
                            item_id: Some(item.opponent_team_id),
                            name: item.opponent_team_name.clone(),
                            avatar_storage_url: item.opponent_avatar_storage_url.clone(),
                            value: item.goals,
                            share: item.share,
                        })
                        .collect(),
                },
                TeamBoardSnapshotSection {
                    title: "球员维度".to_string(),
                    metric_label: "球".to_string(),
                    items: insight
                        .goals_for_by_player
                        .iter()
                        .map(|item| TeamBoardSnapshotItem {
                            item_id: item.player_id,
                            name: item.player_name.clone(),
                            avatar_storage_url: item.avatar_storage_url.clone(),
                            value: item.goals,
                            share: item.share,
                        })
                        .collect(),
                },
            ],
        },
        TeamBoardInsightKind::AssistsFor => TeamBoardSnapshot {
            current_season,
            round_number,
            team_id: insight.team_id,
            team_name: insight.team_name.clone(),
            rank_no: insight.rank_no,
            avatar_storage_url: insight.avatar_storage_url.clone(),
            insight_kind,
            insight_label: insight_kind.label().to_string(),
            summary_label: "总助攻".to_string(),
            summary_value: insight
                .assists_for_by_player
                .iter()
                .map(|item| item.assists)
                .sum(),
            sections: vec![TeamBoardSnapshotSection {
                title: "球员维度".to_string(),
                metric_label: "次".to_string(),
                items: insight
                    .assists_for_by_player
                    .iter()
                    .map(|item| TeamBoardSnapshotItem {
                        item_id: item.player_id,
                        name: item.player_name.clone(),
                        avatar_storage_url: item.avatar_storage_url.clone(),
                        value: item.assists,
                        share: item.share,
                    })
                    .collect(),
            }],
        },
        TeamBoardInsightKind::GoalsAgainst => TeamBoardSnapshot {
            current_season,
            round_number,
            team_id: insight.team_id,
            team_name: insight.team_name.clone(),
            rank_no: insight.rank_no,
            avatar_storage_url: insight.avatar_storage_url.clone(),
            insight_kind,
            insight_label: insight_kind.label().to_string(),
            summary_label: "总失球".to_string(),
            summary_value: insight.goals_against_total,
            sections: vec![TeamBoardSnapshotSection {
                title: "对手维度".to_string(),
                metric_label: "球".to_string(),
                items: insight
                    .goals_against_by_opponent
                    .iter()
                    .map(|item| TeamBoardSnapshotItem {
                        item_id: Some(item.opponent_team_id),
                        name: item.opponent_team_name.clone(),
                        avatar_storage_url: item.opponent_avatar_storage_url.clone(),
                        value: item.goals,
                        share: item.share,
                    })
                    .collect(),
            }],
        },
    }
}
