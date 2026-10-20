use super::dto::{PlayerDto, SlotDto, TeamDto};
use crate::lineup::domain::{
    LineupAuthor, LineupCodeImage, LineupConsensus, LineupLikeState, Position, PublicLineupItem,
    PublicLineupPage, SharedLineup,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(super) struct ListQuery {
    #[serde(default = "latest")]
    pub sort: String,
    #[serde(default = "one")]
    pub page: u32,
    #[serde(default = "twenty")]
    pub page_size: u32,
}
fn latest() -> String {
    "latest".into()
}
fn one() -> u32 {
    1
}
fn twenty() -> u32 {
    20
}

#[derive(Serialize)]
struct AuthorDto {
    display_name: String,
    avatar_url: Option<String>,
}
impl From<LineupAuthor> for AuthorDto {
    fn from(author: LineupAuthor) -> Self {
        Self {
            display_name: author.display_name,
            avatar_url: author.avatar_url,
        }
    }
}

#[derive(Serialize)]
pub(super) struct LikeDto {
    likes_count: i64,
    liked_by_me: bool,
    disliked_by_me: bool,
}
impl From<LineupLikeState> for LikeDto {
    fn from(likes: LineupLikeState) -> Self {
        Self {
            likes_count: likes.likes_count,
            liked_by_me: likes.liked_by_me,
            disliked_by_me: likes.disliked_by_me,
        }
    }
}

#[derive(Serialize)]
struct ItemDto {
    share_code: String,
    formation: String,
    author: AuthorDto,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde(flatten)]
    likes: LikeDto,
}
impl From<PublicLineupItem> for ItemDto {
    fn from(item: PublicLineupItem) -> Self {
        Self {
            share_code: item.share_code,
            formation: item.formation,
            author: item.author.into(),
            created_at: item.created_at,
            updated_at: item.updated_at,
            likes: item.likes.into(),
        }
    }
}

#[derive(Serialize)]
pub(super) struct ListDto {
    team: TeamDto,
    items: Vec<ItemDto>,
    total: i64,
    page: u32,
    page_size: u32,
}
impl From<PublicLineupPage> for ListDto {
    fn from(page: PublicLineupPage) -> Self {
        Self {
            team: page.team.into(),
            items: page.items.into_iter().map(Into::into).collect(),
            total: page.total,
            page: page.query.page,
            page_size: page.query.page_size,
        }
    }
}

#[derive(Serialize)]
struct SharedPlayerDto {
    #[serde(flatten)]
    player: PlayerDto,
    former: bool,
}

#[derive(Serialize)]
pub(super) struct SharedDto {
    #[serde(flatten)]
    item: ItemDto,
    team: TeamDto,
    slots: Vec<SlotDto>,
    players: Vec<SharedPlayerDto>,
}
impl From<SharedLineup> for SharedDto {
    fn from(lineup: SharedLineup) -> Self {
        Self {
            item: lineup.item.into(),
            team: lineup.team.into(),
            slots: lineup.slots.into_iter().map(Into::into).collect(),
            players: lineup
                .players
                .into_iter()
                .map(|player| SharedPlayerDto {
                    former: !player.active,
                    player: player.into(),
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub(super) struct CodeDto {
    image_base64: String,
    mime_type: String,
}
impl From<LineupCodeImage> for CodeDto {
    fn from(image: LineupCodeImage) -> Self {
        Self {
            image_base64: image.image_base64,
            mime_type: image.mime_type,
        }
    }
}

fn position_code(position: Position) -> &'static str {
    match position {
        Position::GK => "GK",
        Position::DF => "DF",
        Position::MF => "MF",
        Position::FW => "FW",
    }
}

#[derive(Serialize)]
pub(super) struct ConsensusTeamDto {
    team_id: i64,
    team_name: String,
    avatar_url: Option<String>,
}
#[derive(Serialize)]
pub(super) struct ConsensusPlayerDto {
    player_id: i64,
    name: String,
    avatar_url: Option<String>,
    shirt_number: Option<i32>,
    position: Option<&'static str>,
    group: &'static str,
    pick_rate: u8,
}
#[derive(Serialize)]
pub(super) struct ConsensusDto {
    team: ConsensusTeamDto,
    total: usize,
    threshold: usize,
    unlocked: bool,
    formation: Option<String>,
    formation_share: u8,
    players: Vec<ConsensusPlayerDto>,
}
impl From<LineupConsensus> for ConsensusDto {
    fn from(c: LineupConsensus) -> Self {
        Self {
            team: ConsensusTeamDto {
                team_id: c.team.team_id,
                team_name: c.team.team_name,
                avatar_url: c.team.avatar_url,
            },
            total: c.total,
            threshold: c.threshold,
            unlocked: c.formation.is_some(),
            formation: c.formation,
            formation_share: c.formation_share,
            players: c
                .players
                .into_iter()
                .map(|p| ConsensusPlayerDto {
                    player_id: p.player.player_id,
                    name: p.player.name,
                    avatar_url: p.player.avatar_url,
                    shirt_number: p.player.shirt_number,
                    position: p.player.position.map(position_code),
                    group: position_code(p.group),
                    pick_rate: p.pick_rate,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lineup::domain::{LineupSlot, Position, SquadPlayer, SquadTeam};

    #[test]
    fn shared_detail_serializes_public_contract_and_former_players() {
        let now = Utc::now();
        let detail = SharedLineup {
            item: PublicLineupItem {
                share_code: "ABCd1234".into(),
                formation: "4-3-3".into(),
                author: LineupAuthor {
                    display_name: "球迷".into(),
                    avatar_url: None,
                },
                created_at: now,
                updated_at: now,
                likes: LineupLikeState {
                    likes_count: 3,
                    liked_by_me: true,
                    disliked_by_me: false,
                },
            },
            team: SquadTeam {
                team_id: 1,
                team_name: "球队".into(),
                avatar_url: None,
            },
            slots: (0..11)
                .map(|slot| LineupSlot {
                    slot,
                    player_id: Some(i64::from(slot) + 1),
                    x: 0.5,
                    y: 0.5,
                })
                .collect(),
            players: vec![SquadPlayer {
                player_id: 1,
                name: "球员".into(),
                shirt_number: Some(7),
                position: Some(Position::FW),
                avatar_url: None,
                last_season: 2025,
                active: false,
            }],
        };
        let json = serde_json::to_value(SharedDto::from(detail)).unwrap();
        let mut fields = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        fields.sort();
        assert_eq!(
            fields,
            vec![
                "author",
                "created_at",
                "disliked_by_me",
                "formation",
                "liked_by_me",
                "likes_count",
                "players",
                "share_code",
                "slots",
                "team",
                "updated_at"
            ]
        );
        assert_eq!(
            json["author"],
            serde_json::json!({"display_name":"球迷", "avatar_url":null})
        );
        assert_eq!(
            json["team"],
            serde_json::json!({"team_id":1,"team_name":"球队","avatar_url":null})
        );
        assert_eq!(json["slots"].as_array().unwrap().len(), 11);
        assert_eq!(
            json["slots"][0],
            serde_json::json!({"slot":0,"player_id":1,"x":0.5,"y":0.5})
        );
        assert_eq!(
            json["players"][0],
            serde_json::json!({"player_id":1,"name":"球员","shirt_number":7,"position":"FW","avatar_url":null,"last_season":2025,"former":true})
        );
        assert_eq!(json["likes_count"], 3);
        assert_eq!(json["liked_by_me"], true);
        assert!(json["created_at"].is_string());
        assert!(json["updated_at"].is_string());
    }
}
