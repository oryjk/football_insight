//! HTTP contracts belong to the inbound adapter, independently of domain and JSONB storage.
use crate::lineup::{
    application::Squad,
    domain::{Lineup, LineupSlot, Position, SquadPlayer, SquadTeam},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(super) struct SquadResponse {
    team: TeamDto,
    active: Vec<PlayerDto>,
    former: Vec<PlayerDto>,
}

#[derive(Serialize)]
pub(super) struct TeamDto {
    team_id: i64,
    team_name: String,
    avatar_url: Option<String>,
}

#[derive(Serialize)]
pub(super) struct PlayerDto {
    player_id: i64,
    name: String,
    shirt_number: Option<i32>,
    position: Option<&'static str>,
    avatar_url: Option<String>,
    last_season: i32,
}

impl From<SquadTeam> for TeamDto {
    fn from(team: SquadTeam) -> Self {
        Self {
            team_id: team.team_id,
            team_name: team.team_name,
            avatar_url: team.avatar_url,
        }
    }
}

impl From<SquadPlayer> for PlayerDto {
    fn from(player: SquadPlayer) -> Self {
        Self {
            player_id: player.player_id,
            name: player.name,
            shirt_number: player.shirt_number,
            position: player.position.map(|p| match p {
                Position::GK => "GK",
                Position::DF => "DF",
                Position::MF => "MF",
                Position::FW => "FW",
            }),
            avatar_url: player.avatar_url,
            last_season: player.last_season,
        }
    }
}

impl From<Squad> for SquadResponse {
    fn from(squad: Squad) -> Self {
        Self {
            team: squad.team.into(),
            active: squad.active.into_iter().map(Into::into).collect(),
            former: squad.former.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
pub(super) struct LineupResponse {
    pub lineup: Option<LineupDto>,
}

#[derive(Serialize)]
pub(super) struct LineupDto {
    formation: String,
    slots: Vec<SlotDto>,
    share_code: String,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub(super) struct SaveRequest {
    pub formation: String,
    pub slots: Vec<SlotDto>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct SlotDto {
    slot: u8,
    player_id: Option<i64>,
    x: f64,
    y: f64,
}

impl From<SlotDto> for LineupSlot {
    fn from(slot: SlotDto) -> Self {
        Self {
            slot: slot.slot,
            player_id: slot.player_id,
            x: slot.x,
            y: slot.y,
        }
    }
}

impl From<LineupSlot> for SlotDto {
    fn from(slot: LineupSlot) -> Self {
        Self {
            slot: slot.slot,
            player_id: slot.player_id,
            x: slot.x,
            y: slot.y,
        }
    }
}

impl From<Lineup> for LineupDto {
    fn from(lineup: Lineup) -> Self {
        Self {
            formation: lineup.formation,
            slots: lineup.slots.into_iter().map(Into::into).collect(),
            share_code: lineup.share_code,
            updated_at: lineup.updated_at,
        }
    }
}
