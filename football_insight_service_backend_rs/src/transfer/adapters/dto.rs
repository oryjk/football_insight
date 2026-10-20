use crate::transfer::{
    application::{
        BoardSlice, CandidateGroup, Candidates, CardView, NominateResult, TransferBoard,
        TransferSummary, WindowView,
    },
    domain::{CandidatePlayer, NominationCard, TransferTeam, VoteTally},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WindowQuery {
    pub window: Option<String>,
}
#[derive(Deserialize)]
pub struct BoardQuery {
    pub intent: String,
    #[serde(default = "first_page")]
    pub page: i64,
    pub window: Option<String>,
}
fn first_page() -> i64 {
    1
}
#[derive(Deserialize)]
pub struct IntentQuery {
    pub intent: String,
}
#[derive(Deserialize)]
pub struct NominateBody {
    pub player_id: i64,
    pub intent: String,
}
#[derive(Deserialize)]
pub struct VoteBody {
    pub value: i16,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct TeamDto {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_url: Option<String>,
}
impl From<TransferTeam> for TeamDto {
    fn from(t: TransferTeam) -> Self {
        Self {
            team_id: t.team_id,
            team_name: t.team_name,
            avatar_url: t.avatar_url,
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct WindowDto {
    pub code: String,
    pub name: String,
    pub opens_at: DateTime<Utc>,
    pub closes_at: DateTime<Utc>,
    pub open: bool,
}
impl From<WindowView> for WindowDto {
    fn from(v: WindowView) -> Self {
        Self {
            code: v.window.code,
            name: v.window.name,
            opens_at: v.window.opens_at,
            closes_at: v.window.closes_at,
            open: v.open,
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct PlayerDto {
    pub player_id: i64,
    pub name: String,
    pub avatar_url: Option<String>,
    pub position: Option<String>,
    pub shirt_number: Option<i32>,
    pub current_team: Option<TeamDto>,
    pub departed: bool,
}
#[derive(Serialize, Debug, PartialEq)]
pub struct CardDto {
    pub share_code: String,
    pub intent: &'static str,
    pub returning: bool,
    pub player: PlayerDto,
    pub up: i64,
    pub down: i64,
    pub my_vote: i16,
    pub rank: Option<i64>,
    pub created_at: DateTime<Utc>,
}
impl From<NominationCard> for CardDto {
    fn from(c: NominationCard) -> Self {
        Self {
            share_code: c.share_code,
            intent: c.intent.as_str(),
            returning: c.returning,
            player: PlayerDto {
                player_id: c.player.player_id,
                name: c.player.name,
                avatar_url: c.player.avatar_url,
                position: c.player.position,
                shirt_number: c.player.shirt_number,
                current_team: c.player.current_team.map(Into::into),
                departed: c.player.departed,
            },
            up: c.up,
            down: c.down,
            my_vote: c.my_vote,
            rank: c.rank,
            created_at: c.created_at,
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CurrentWindowDto {
    pub window: Option<WindowDto>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct SliceDto {
    pub intent: &'static str,
    pub total: i64,
    pub items: Vec<CardDto>,
}
impl From<BoardSlice> for SliceDto {
    fn from(s: BoardSlice) -> Self {
        Self {
            intent: s.intent.as_str(),
            total: s.total,
            items: s.items.into_iter().map(Into::into).collect(),
        }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct SummaryDto {
    pub team: TeamDto,
    pub window: Option<WindowDto>,
    pub can_vote: bool,
    pub boards: Vec<SliceDto>,
}
impl From<TransferSummary> for SummaryDto {
    fn from(s: TransferSummary) -> Self {
        Self {
            team: s.team.into(),
            window: s.window.map(Into::into),
            can_vote: s.can_vote,
            boards: s.boards.into_iter().map(Into::into).collect(),
        }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct BoardDto {
    pub team: TeamDto,
    pub window: Option<WindowDto>,
    pub intent: &'static str,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub can_vote: bool,
    pub items: Vec<CardDto>,
}
impl From<TransferBoard> for BoardDto {
    fn from(b: TransferBoard) -> Self {
        Self {
            team: b.team.into(),
            window: b.window.map(Into::into),
            intent: b.intent.as_str(),
            page: b.page,
            page_size: b.page_size,
            total: b.total,
            can_vote: b.can_vote,
            items: b.items.into_iter().map(Into::into).collect(),
        }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct CardViewDto {
    pub card: CardDto,
    pub team: TeamDto,
    pub window: WindowDto,
    pub can_vote: bool,
}
impl From<CardView> for CardViewDto {
    fn from(v: CardView) -> Self {
        Self {
            card: v.card.into(),
            team: v.team.into(),
            window: v.window.into(),
            can_vote: v.can_vote,
        }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct NominateDto {
    pub created: bool,
    pub view: CardViewDto,
}
impl From<NominateResult> for NominateDto {
    fn from(r: NominateResult) -> Self {
        Self {
            created: r.created,
            view: r.view.into(),
        }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct CandidateDto {
    pub player_id: i64,
    pub name: String,
    pub shirt_number: Option<i32>,
    pub position: Option<String>,
    pub avatar_url: Option<String>,
    pub current_team: Option<TeamDto>,
}
impl From<CandidatePlayer> for CandidateDto {
    fn from(p: CandidatePlayer) -> Self {
        Self {
            player_id: p.player_id,
            name: p.name,
            shirt_number: p.shirt_number,
            position: p.position,
            avatar_url: p.avatar_url,
            current_team: p.current_team.map(Into::into),
        }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct GroupDto {
    pub key: String,
    pub title: String,
    pub players: Vec<CandidateDto>,
}
impl From<CandidateGroup> for GroupDto {
    fn from(g: CandidateGroup) -> Self {
        Self {
            key: g.key,
            title: g.title,
            players: g.players.into_iter().map(Into::into).collect(),
        }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct CandidatesDto {
    pub intent: &'static str,
    pub groups: Vec<GroupDto>,
}
impl From<Candidates> for CandidatesDto {
    fn from(c: Candidates) -> Self {
        Self {
            intent: c.intent.as_str(),
            groups: c.groups.into_iter().map(Into::into).collect(),
        }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct TallyDto {
    pub up: i64,
    pub down: i64,
    pub my_vote: i16,
}
impl From<VoteTally> for TallyDto {
    fn from(t: VoteTally) -> Self {
        Self {
            up: t.up,
            down: t.down,
            my_vote: t.my_vote,
        }
    }
}
