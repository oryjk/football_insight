use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::director::{
    application::{IncomingPlayerInput, SavePlanInput},
    domain::{
        CandidateItem, CircleConsensus, CirclePlanSummary, ConsensusForeignItem,
        ConsensusLeaveItem, DirectorAuthor, DirectorComment, DirectorContext, DirectorPlan,
        DirectorPlanDetail, DirectorTeam, DirectorWindow, LeaveDisagreement, PlanComparison,
        PlanPlayerItem, PlayerSource, ReactionSummary, SquadPlayer,
    },
};

#[derive(Debug, Deserialize)]
pub struct ContextQuery {
    pub team_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CandidatesQuery {
    pub team_id: Option<i64>,
    #[serde(default = "default_is_foreign")]
    pub is_foreign: bool,
    pub league: Option<String>,
    pub q: Option<String>,
}

fn default_is_foreign() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct IncomingPlayerBody {
    pub source: String,
    #[serde(default)]
    pub candidate_code: Option<String>,
    #[serde(default)]
    pub player_id: Option<i64>,
    #[serde(default)]
    pub player_name: Option<String>,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub league_tag: Option<String>,
    #[serde(default)]
    pub current_club: Option<String>,
}

impl From<IncomingPlayerBody> for IncomingPlayerInput {
    fn from(value: IncomingPlayerBody) -> Self {
        Self {
            source: value.source,
            candidate_code: value.candidate_code,
            player_id: value.player_id,
            player_name: value.player_name,
            position: value.position,
            league_tag: value.league_tag,
            current_club: value.current_club,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SavePlanBody {
    pub team_id: i64,
    #[serde(default)]
    pub circle_code: Option<String>,
    #[serde(default)]
    pub parent_share_code: Option<String>,
    #[serde(default)]
    pub foreign_stay_ids: Vec<i64>,
    #[serde(default)]
    pub foreign_leave_ids: Vec<i64>,
    #[serde(default)]
    pub foreign_ins: Vec<IncomingPlayerBody>,
    #[serde(default)]
    pub domestic_ins: Vec<IncomingPlayerBody>,
    #[serde(default)]
    pub domestic_outs: Vec<i64>,
    #[serde(default)]
    pub manifesto: String,
}

impl From<SavePlanBody> for SavePlanInput {
    fn from(value: SavePlanBody) -> Self {
        Self {
            team_id: value.team_id,
            circle_code: value.circle_code,
            parent_share_code: value.parent_share_code,
            foreign_stay_ids: value.foreign_stay_ids,
            foreign_leave_ids: value.foreign_leave_ids,
            foreign_ins: value.foreign_ins.into_iter().map(Into::into).collect(),
            domestic_ins: value.domestic_ins.into_iter().map(Into::into).collect(),
            domestic_outs: value.domestic_outs,
            manifesto: value.manifesto,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReactionBody {
    pub value: i16,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentBody {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct DirectorWindowDto {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub opens_at: DateTime<Utc>,
    pub closes_at: DateTime<Utc>,
}

impl From<DirectorWindow> for DirectorWindowDto {
    fn from(w: DirectorWindow) -> Self {
        Self {
            id: w.id,
            code: w.code,
            name: w.name,
            opens_at: w.opens_at,
            closes_at: w.closes_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DirectorTeamDto {
    pub team_id: i64,
    pub team_name: String,
    pub logo_url: Option<String>,
}

impl From<DirectorTeam> for DirectorTeamDto {
    fn from(t: DirectorTeam) -> Self {
        Self {
            team_id: t.team_id,
            team_name: t.team_name,
            logo_url: t.logo_url,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SquadPlayerDto {
    pub player_id: i64,
    pub player_name: String,
    pub shirt_number: Option<i32>,
    pub position: Option<String>,
    pub country: Option<String>,
    pub avatar_url: Option<String>,
    pub market_value_eur: Option<i32>,
    pub birth_date: Option<String>,
    pub contract_until: Option<String>,
}

impl From<SquadPlayer> for SquadPlayerDto {
    fn from(p: SquadPlayer) -> Self {
        Self {
            player_id: p.player_id,
            player_name: p.player_name,
            shirt_number: p.shirt_number,
            position: p.position,
            country: p.country,
            avatar_url: p.avatar_url,
            market_value_eur: p.market_value_eur,
            birth_date: p.birth_date,
            contract_until: p.contract_until,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DirectorAuthorDto {
    pub user_id: Uuid,
    pub nickname: String,
    pub avatar_url: Option<String>,
}

impl From<DirectorAuthor> for DirectorAuthorDto {
    fn from(a: DirectorAuthor) -> Self {
        Self {
            user_id: a.user_id,
            nickname: a.nickname,
            avatar_url: a.avatar_url,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DirectorPlanDto {
    pub id: Uuid,
    pub window_code: String,
    pub window_name: String,
    pub team_id: i64,
    pub team_name: String,
    pub share_code: String,
    pub circle_code: String,
    pub author: DirectorAuthorDto,
    pub is_mine: bool,
    pub foreign_stays: Vec<PlanPlayerItem>,
    pub foreign_leaves: Vec<PlanPlayerItem>,
    pub foreign_ins: Vec<PlanPlayerItem>,
    pub domestic_ins: Vec<PlanPlayerItem>,
    pub domestic_outs: Vec<PlanPlayerItem>,
    pub style_tag: String,
    pub manifesto: String,
    pub likes_count: i64,
    pub dislikes_count: i64,
    pub my_reaction: i16,
    pub comments_count: i64,
    pub updated_at: DateTime<Utc>,
}

impl From<DirectorPlan> for DirectorPlanDto {
    fn from(p: DirectorPlan) -> Self {
        Self {
            id: p.id,
            window_code: p.window_code,
            window_name: p.window_name,
            team_id: p.team_id,
            team_name: p.team_name,
            share_code: p.share_code,
            circle_code: p.circle_code,
            author: p.author.into(),
            is_mine: p.is_mine,
            foreign_stays: p.foreign_stays,
            foreign_leaves: p.foreign_leaves,
            foreign_ins: p.foreign_ins,
            domestic_ins: p.domestic_ins,
            domestic_outs: p.domestic_outs,
            style_tag: p.style_tag,
            manifesto: p.manifesto,
            likes_count: p.likes_count,
            dislikes_count: p.dislikes_count,
            my_reaction: p.my_reaction,
            comments_count: p.comments_count,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DirectorContextDto {
    pub window: DirectorWindowDto,
    pub team: DirectorTeamDto,
    pub foreign_quota: usize,
    pub max_domestic_ins: usize,
    pub max_domestic_outs: usize,
    pub squad_foreign: Vec<SquadPlayerDto>,
    pub squad_domestic: Vec<SquadPlayerDto>,
    pub my_plan: Option<DirectorPlanDto>,
    pub team_plan_count: i64,
}

impl From<DirectorContext> for DirectorContextDto {
    fn from(c: DirectorContext) -> Self {
        Self {
            window: c.window.into(),
            team: c.team.into(),
            foreign_quota: c.foreign_quota,
            max_domestic_ins: c.max_domestic_ins,
            max_domestic_outs: c.max_domestic_outs,
            squad_foreign: c.squad_foreign.into_iter().map(Into::into).collect(),
            squad_domestic: c.squad_domestic.into_iter().map(Into::into).collect(),
            my_plan: c.my_plan.map(Into::into),
            team_plan_count: c.team_plan_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CandidateItemDto {
    pub key: String,
    pub source: PlayerSource,
    pub player_id: Option<i64>,
    pub candidate_code: Option<String>,
    pub player_name: String,
    pub position: String,
    pub country: String,
    pub is_foreign: bool,
    pub league_tag: String,
    pub current_club: String,
    pub tags: Vec<String>,
    pub avatar_url: Option<String>,
    pub market_value_eur: Option<i32>,
    pub birth_date: Option<String>,
    pub contract_until: Option<String>,
}

impl From<CandidateItem> for CandidateItemDto {
    fn from(c: CandidateItem) -> Self {
        Self {
            key: c.key,
            source: c.source,
            player_id: c.player_id,
            candidate_code: c.candidate_code,
            player_name: c.player_name,
            position: c.position,
            country: c.country,
            is_foreign: c.is_foreign,
            league_tag: c.league_tag,
            current_club: c.current_club,
            tags: c.tags,
            avatar_url: c.avatar_url,
            market_value_eur: c.market_value_eur,
            birth_date: c.birth_date,
            contract_until: c.contract_until,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CandidatesResponseDto {
    pub items: Vec<CandidateItemDto>,
}

#[derive(Debug, Serialize)]
pub struct DirectorCommentDto {
    pub id: Uuid,
    pub author: DirectorAuthorDto,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl From<DirectorComment> for DirectorCommentDto {
    fn from(c: DirectorComment) -> Self {
        Self {
            id: c.id,
            author: c.author.into(),
            content: c.content,
            created_at: c.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LeaveDisagreementDto {
    pub player: PlanPlayerItem,
    pub target_action: String,
    pub my_action: String,
}

impl From<LeaveDisagreement> for LeaveDisagreementDto {
    fn from(d: LeaveDisagreement) -> Self {
        Self {
            player: d.player,
            target_action: d.target_action,
            my_action: d.my_action,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PlanComparisonDto {
    pub my_share_code: String,
    pub overlap_percent: u8,
    pub common_foreign: Vec<PlanPlayerItem>,
    pub only_in_target: Vec<PlanPlayerItem>,
    pub only_in_mine: Vec<PlanPlayerItem>,
    pub leave_disagreements: Vec<LeaveDisagreementDto>,
    pub summary_line: String,
}

impl From<PlanComparison> for PlanComparisonDto {
    fn from(c: PlanComparison) -> Self {
        Self {
            my_share_code: c.my_share_code,
            overlap_percent: c.overlap_percent,
            common_foreign: c.common_foreign,
            only_in_target: c.only_in_target,
            only_in_mine: c.only_in_mine,
            leave_disagreements: c.leave_disagreements.into_iter().map(Into::into).collect(),
            summary_line: c.summary_line,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DirectorPlanDetailDto {
    pub plan: DirectorPlanDto,
    pub comments: Vec<DirectorCommentDto>,
    pub comparison: Option<PlanComparisonDto>,
}

impl From<DirectorPlanDetail> for DirectorPlanDetailDto {
    fn from(d: DirectorPlanDetail) -> Self {
        Self {
            plan: d.plan.into(),
            comments: d.comments.into_iter().map(Into::into).collect(),
            comparison: d.comparison.map(Into::into),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConsensusForeignItemDto {
    pub player: PlanPlayerItem,
    pub pick_count: usize,
    pub pick_rate: u8,
}

impl From<ConsensusForeignItem> for ConsensusForeignItemDto {
    fn from(i: ConsensusForeignItem) -> Self {
        Self {
            player: i.player,
            pick_count: i.pick_count,
            pick_rate: i.pick_rate,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConsensusLeaveItemDto {
    pub player: PlanPlayerItem,
    pub leave_count: usize,
    pub leave_rate: u8,
}

impl From<ConsensusLeaveItem> for ConsensusLeaveItemDto {
    fn from(i: ConsensusLeaveItem) -> Self {
        Self {
            player: i.player,
            leave_count: i.leave_count,
            leave_rate: i.leave_rate,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CirclePlanSummaryDto {
    pub share_code: String,
    pub author: DirectorAuthorDto,
    pub style_tag: String,
    pub manifesto: String,
    pub foreign_count: usize,
    pub likes_count: i64,
    pub dislikes_count: i64,
    pub updated_at: DateTime<Utc>,
}

impl From<CirclePlanSummary> for CirclePlanSummaryDto {
    fn from(s: CirclePlanSummary) -> Self {
        Self {
            share_code: s.share_code,
            author: s.author.into(),
            style_tag: s.style_tag,
            manifesto: s.manifesto,
            foreign_count: s.foreign_count,
            likes_count: s.likes_count,
            dislikes_count: s.dislikes_count,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CircleConsensusDto {
    pub circle_code: String,
    pub window_name: String,
    pub team_id: i64,
    pub team_name: String,
    pub member_count: usize,
    pub unlocked: bool,
    pub top_foreign: Vec<ConsensusForeignItemDto>,
    pub top_leaves: Vec<ConsensusLeaveItemDto>,
    pub plans: Vec<CirclePlanSummaryDto>,
}

impl From<CircleConsensus> for CircleConsensusDto {
    fn from(c: CircleConsensus) -> Self {
        Self {
            circle_code: c.circle_code,
            window_name: c.window_name,
            team_id: c.team_id,
            team_name: c.team_name,
            member_count: c.member_count,
            unlocked: c.unlocked,
            top_foreign: c.top_foreign.into_iter().map(Into::into).collect(),
            top_leaves: c.top_leaves.into_iter().map(Into::into).collect(),
            plans: c.plans.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReactionSummaryDto {
    pub likes_count: i64,
    pub dislikes_count: i64,
    pub my_reaction: i16,
}

impl From<ReactionSummary> for ReactionSummaryDto {
    fn from(r: ReactionSummary) -> Self {
        Self {
            likes_count: r.likes_count,
            dislikes_count: r.dislikes_count,
            my_reaction: r.my_reaction,
        }
    }
}
