use crate::lineup::{
    domain::{
        CONSENSUS_MIN_LINEUPS, ConsensusPlayer, LineupCodeImage, LineupConsensus, LineupLikeState,
        LineupPublicError, LineupReaction, PublicLineupPage, PublicLineupQuery, SharedLineup,
        SquadPlayer, compute_consensus, percent, valid_share_code,
    },
    ports::{LineupCodePort, LineupUserAccess, PublicLineupRepository, SquadRepository},
};
use std::sync::Arc;
use uuid::Uuid;

pub struct PublicLineupService {
    squads: Arc<dyn SquadRepository>,
    lineups: Arc<dyn PublicLineupRepository>,
    users: Arc<dyn LineupUserAccess>,
    codes: Arc<dyn LineupCodePort>,
}

impl PublicLineupService {
    pub fn new(
        squads: Arc<dyn SquadRepository>,
        lineups: Arc<dyn PublicLineupRepository>,
        users: Arc<dyn LineupUserAccess>,
        codes: Arc<dyn LineupCodePort>,
    ) -> Self {
        Self {
            squads,
            lineups,
            users,
            codes,
        }
    }

    async fn authorize(&self, viewer: Option<Uuid>) -> Result<(), LineupPublicError> {
        if let Some(user) = viewer {
            let active = self
                .users
                .is_active(user)
                .await
                .map_err(|_| LineupPublicError::Unavailable)?;
            if !active {
                return Err(LineupPublicError::Unauthorized);
            }
        }
        Ok(())
    }

    pub async fn list(
        &self,
        team_id: i64,
        sort: &str,
        page: u32,
        page_size: u32,
        viewer: Option<Uuid>,
    ) -> Result<PublicLineupPage, LineupPublicError> {
        self.authorize(viewer).await?;
        let query = PublicLineupQuery::parse(sort, page, page_size)?;
        if team_id <= 0 {
            return Err(LineupPublicError::Invalid);
        }
        let team = self
            .squads
            .find_team(team_id)
            .await
            .map_err(|_| LineupPublicError::Unavailable)?
            .ok_or(LineupPublicError::NotFound)?;
        let (items, total) = self.lineups.list(team_id, query, viewer).await?;
        Ok(PublicLineupPage {
            team,
            items,
            total,
            query,
        })
    }

    async fn visible(
        &self,
        code: &str,
        viewer: Option<Uuid>,
    ) -> Result<SharedLineup, LineupPublicError> {
        if !valid_share_code(code) {
            return Err(LineupPublicError::Invalid);
        }
        self.lineups
            .shared(code, viewer)
            .await?
            .ok_or(LineupPublicError::NotFound)
    }

    pub async fn shared(
        &self,
        code: &str,
        viewer: Option<Uuid>,
    ) -> Result<SharedLineup, LineupPublicError> {
        self.authorize(viewer).await?;
        let mut lineup = self.visible(code, viewer).await?;
        let squad = self
            .squads
            .list_squad(lineup.team.team_id)
            .await
            .map_err(|_| LineupPublicError::Unavailable)?;
        lineup.players = squad
            .into_iter()
            .filter(|player| {
                lineup
                    .slots
                    .iter()
                    .any(|slot| slot.player_id == Some(player.player_id))
            })
            .collect();
        // 阵容里可能有其他球队的球员：不在本队名单的按 ID 补查，带上所在球队。
        let missing: Vec<i64> = lineup
            .slots
            .iter()
            .filter_map(|slot| slot.player_id)
            .filter(|id| !lineup.players.iter().any(|p| p.player_id == *id))
            .collect();
        if !missing.is_empty() {
            let external = self
                .squads
                .find_players(&missing)
                .await
                .map_err(|_| LineupPublicError::Unavailable)?;
            lineup.players.extend(external);
        }
        Ok(lineup)
    }

    pub async fn set_like(
        &self,
        code: &str,
        user: Uuid,
        liked: bool,
    ) -> Result<LineupLikeState, LineupPublicError> {
        self.set_reaction(code, user, liked.then_some(LineupReaction::Like))
            .await
    }

    pub async fn set_reaction(
        &self,
        code: &str,
        user: Uuid,
        reaction: Option<LineupReaction>,
    ) -> Result<LineupLikeState, LineupPublicError> {
        self.authorize(Some(user)).await?;
        if !valid_share_code(code) {
            return Err(LineupPublicError::Invalid);
        }
        self.lineups.set_reaction(code, user, reaction).await
    }

    pub async fn consensus(&self, team_id: i64) -> Result<LineupConsensus, LineupPublicError> {
        if team_id <= 0 {
            return Err(LineupPublicError::Invalid);
        }
        let team = self
            .squads
            .find_team(team_id)
            .await
            .map_err(|_| LineupPublicError::Unavailable)?
            .ok_or(LineupPublicError::NotFound)?;
        let lineups = self.lineups.team_lineup_players(team_id).await?;
        let squad = self
            .squads
            .list_squad(team_id)
            .await
            .map_err(|_| LineupPublicError::Unavailable)?;
        let positions = squad.iter().map(|p| (p.player_id, p.position)).collect();
        // 共识只统计本队球员：能在其他球队名单里查到、却不在本队名单的，是外队球员，不计入。
        // 查不到的（早已离开名单的老队员）照旧计入，显示为「未知球员」。
        let outside_ids: Vec<i64> = lineups
            .iter()
            .flat_map(|(_, ids)| ids.iter().copied())
            .filter(|id| !squad.iter().any(|p| p.player_id == *id))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let outsiders: std::collections::HashSet<i64> = if outside_ids.is_empty() {
            Default::default()
        } else {
            self.squads
                .find_players(&outside_ids)
                .await
                .map_err(|_| LineupPublicError::Unavailable)?
                .into_iter()
                .map(|p| p.player_id)
                .collect()
        };
        let lineups: Vec<(String, Vec<i64>)> = lineups
            .into_iter()
            .map(|(formation, ids)| {
                (
                    formation,
                    ids.into_iter()
                        .filter(|id| !outsiders.contains(id))
                        .collect(),
                )
            })
            .collect();
        let result = compute_consensus(&lineups, &positions);
        let players = result
            .players
            .iter()
            .map(|pick| ConsensusPlayer {
                // 球员已从名单移除时仍保留位置，用"未知球员"占位，保证 11 人
                player: squad
                    .iter()
                    .find(|p| p.player_id == pick.player_id)
                    .cloned()
                    .unwrap_or(SquadPlayer {
                        player_id: pick.player_id,
                        name: "未知球员".into(),
                        shirt_number: None,
                        position: None,
                        avatar_url: None,
                        last_season: 0,
                        active: false,
                        club: None,
                    }),
                group: pick.group,
                pick_rate: percent(pick.picks, result.total),
            })
            .collect();
        Ok(LineupConsensus {
            team,
            total: result.total,
            threshold: CONSENSUS_MIN_LINEUPS,
            formation_share: percent(result.formation_picks, result.total),
            formation: result.formation,
            players,
        })
    }

    pub async fn image(
        &self,
        code: &str,
        user: Uuid,
    ) -> Result<LineupCodeImage, LineupPublicError> {
        self.authorize(Some(user)).await?;
        self.visible(code, Some(user)).await?;
        self.codes.image(code).await
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::lineup::{
        application::{my_lineup::tests::FakeUserAccess, tests::FakeSquads},
        domain::{Position, PublicLineupItem, SquadPlayer},
    };
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    pub(crate) struct FakePublicLineups {
        pub shared: Mutex<Option<SharedLineup>>,
        pub reaction: Mutex<Option<LineupReaction>>,
        pub team_lineups: Mutex<Vec<(String, Vec<i64>)>>,
    }
    #[async_trait]
    impl PublicLineupRepository for FakePublicLineups {
        async fn list(
            &self,
            _: i64,
            _: PublicLineupQuery,
            _: Option<Uuid>,
        ) -> Result<(Vec<PublicLineupItem>, i64), LineupPublicError> {
            Ok((vec![], 0))
        }
        async fn team_lineup_players(
            &self,
            _: i64,
        ) -> Result<Vec<(String, Vec<i64>)>, LineupPublicError> {
            Ok(self.team_lineups.lock().unwrap().clone())
        }
        async fn shared(
            &self,
            _: &str,
            _: Option<Uuid>,
        ) -> Result<Option<SharedLineup>, LineupPublicError> {
            Ok(self.shared.lock().unwrap().clone())
        }
        async fn set_reaction(
            &self,
            _: &str,
            _: Uuid,
            reaction: Option<LineupReaction>,
        ) -> Result<LineupLikeState, LineupPublicError> {
            if self.shared.lock().unwrap().is_none() {
                return Err(LineupPublicError::NotFound);
            }
            *self.reaction.lock().unwrap() = reaction;
            let liked = reaction == Some(LineupReaction::Like);
            Ok(LineupLikeState {
                likes_count: i64::from(liked),
                liked_by_me: liked,
                disliked_by_me: reaction == Some(LineupReaction::Dislike),
            })
        }
    }
    pub(crate) struct FakeCodes;
    #[async_trait]
    impl LineupCodePort for FakeCodes {
        async fn image(&self, _: &str) -> Result<LineupCodeImage, LineupPublicError> {
            panic!("hidden lineup must never request a code")
        }
    }
    pub(crate) fn service(active: bool) -> PublicLineupService {
        PublicLineupService::new(
            Arc::new(FakeSquads {
                players: vec![],
                missing_table: false,
            }),
            Arc::new(FakePublicLineups::default()),
            Arc::new(FakeUserAccess {
                active,
                unavailable: false,
            }),
            Arc::new(FakeCodes),
        )
    }

    #[tokio::test]
    async fn guests_can_list_but_disabled_viewers_cannot_personalize() {
        assert!(
            service(false)
                .list(77680, "latest", 1, 20, None)
                .await
                .is_ok()
        );
        assert_eq!(
            service(false)
                .list(77680, "latest", 1, 20, Some(Uuid::new_v4()))
                .await
                .unwrap_err(),
            LineupPublicError::Unauthorized
        );
    }
    #[tokio::test]
    async fn missing_and_hidden_lineups_block_details_likes_and_code_generation() {
        let svc = service(true);
        assert_eq!(
            svc.shared("ABCD1234", None).await.unwrap_err(),
            LineupPublicError::NotFound
        );
        assert_eq!(
            svc.set_like("ABCD1234", Uuid::new_v4(), true)
                .await
                .unwrap_err(),
            LineupPublicError::NotFound
        );
        assert_eq!(
            svc.image("ABCD1234", Uuid::new_v4()).await.unwrap_err(),
            LineupPublicError::NotFound
        );
    }
    #[tokio::test]
    async fn disabled_accounts_cannot_like_or_generate_images_with_old_tokens() {
        let svc = service(false);
        assert_eq!(
            svc.set_like("ABCD1234", Uuid::new_v4(), true)
                .await
                .unwrap_err(),
            LineupPublicError::Unauthorized
        );
        assert_eq!(
            svc.image("ABCD1234", Uuid::new_v4()).await.unwrap_err(),
            LineupPublicError::Unauthorized
        );
        assert_eq!(
            svc.shared("ABCD1234", Some(Uuid::new_v4()))
                .await
                .unwrap_err(),
            LineupPublicError::Unauthorized
        );
    }
    #[tokio::test]
    async fn validates_queries_codes_and_unknown_teams() {
        let svc = service(true);
        assert_eq!(
            svc.list(77680, "bad", 1, 20, None).await.unwrap_err(),
            LineupPublicError::Invalid
        );
        assert_eq!(
            svc.list(77680, "latest", 0, 20, None).await.unwrap_err(),
            LineupPublicError::Invalid
        );
        assert_eq!(
            svc.list(1, "latest", 1, 20, None).await.unwrap_err(),
            LineupPublicError::NotFound
        );
        assert_eq!(
            svc.shared("bad", None).await.unwrap_err(),
            LineupPublicError::Invalid
        );
    }

    fn consensus_service(
        lineups: Vec<(String, Vec<i64>)>,
        squad: Vec<SquadPlayer>,
    ) -> PublicLineupService {
        let repo = FakePublicLineups::default();
        *repo.team_lineups.lock().unwrap() = lineups;
        PublicLineupService::new(
            Arc::new(FakeSquads {
                players: squad,
                missing_table: false,
            }),
            Arc::new(repo),
            Arc::new(FakeUserAccess {
                active: true,
                unavailable: false,
            }),
            Arc::new(FakeCodes),
        )
    }
    fn full_squad() -> Vec<SquadPlayer> {
        use crate::lineup::application::tests::player;
        let mut squad = vec![player(1, Some(1), Some(Position::GK), 2026, true)];
        squad.extend(
            (10..=13).map(|id| player(id, Some(id as i32), Some(Position::DF), 2026, true)),
        );
        squad.extend(
            (20..=22).map(|id| player(id, Some(id as i32), Some(Position::MF), 2026, true)),
        );
        squad.extend(
            (30..=32).map(|id| player(id, Some(id as i32), Some(Position::FW), 2026, true)),
        );
        squad
    }
    const XI: [i64; 11] = [1, 10, 11, 12, 13, 20, 21, 22, 30, 31, 32];

    #[tokio::test]
    async fn consensus_reports_progress_below_the_threshold_and_players_above_it() {
        let locked = consensus_service(vec![("4-3-3".into(), XI.to_vec()); 3], full_squad())
            .consensus(77680)
            .await
            .unwrap();
        assert_eq!(
            (
                locked.total,
                locked.threshold,
                locked.formation.clone(),
                locked.players.len()
            ),
            (3, 10, None, 0)
        );
        let mut sample = vec![("4-3-3".to_string(), XI.to_vec()); 9];
        sample.push(("4-4-2".into(), XI.to_vec()));
        let unlocked = consensus_service(sample, full_squad())
            .consensus(77680)
            .await
            .unwrap();
        assert_eq!(unlocked.formation.as_deref(), Some("4-3-3"));
        assert_eq!(unlocked.formation_share, 90);
        assert_eq!(unlocked.players.len(), 11);
        assert_eq!(unlocked.players[0].player.player_id, 1);
        assert_eq!(unlocked.players[0].pick_rate, 100);
    }

    #[tokio::test]
    async fn missing_squad_rows_still_yield_eleven_players() {
        let mut squad = full_squad();
        squad.retain(|p| p.player_id != 32);
        let consensus = consensus_service(vec![("4-3-3".into(), XI.to_vec()); 10], squad)
            .consensus(77680)
            .await
            .unwrap();
        assert_eq!(consensus.players.len(), 11);
        let unknown = consensus
            .players
            .iter()
            .find(|p| p.player.player_id == 32)
            .unwrap();
        assert_eq!(unknown.player.name, "未知球员");
    }

    fn outsider(id: i64) -> SquadPlayer {
        use crate::lineup::{application::tests::player, domain::PlayerClub};
        let mut p = player(id, Some(9), Some(Position::FW), 2026, true);
        p.club = Some(PlayerClub {
            team_id: 136,
            team_name: "山东泰山".into(),
        });
        p
    }

    #[tokio::test]
    async fn consensus_counts_only_the_teams_own_players() {
        let mut squad = full_squad();
        squad.push(outsider(99));
        let with_outsider = [1, 10, 11, 12, 13, 20, 21, 22, 30, 31, 99];
        let consensus =
            consensus_service(vec![("4-3-3".into(), with_outsider.to_vec()); 10], squad)
                .consensus(77680)
                .await
                .unwrap();
        // 每套阵容只剩 10 名本队球员，共识里就只有这 10 人。
        assert_eq!(consensus.players.len(), 10);
        assert!(consensus.players.iter().all(|p| p.player.player_id != 99));
    }

    #[tokio::test]
    async fn shared_lineups_include_players_from_other_teams_with_their_club() {
        use crate::lineup::domain::{LineupSlot, PublicLineupItem, SquadTeam};
        let repo = FakePublicLineups::default();
        let slots: Vec<LineupSlot> = [1, 99]
            .iter()
            .enumerate()
            .map(|(slot, id)| LineupSlot {
                slot: slot as u8,
                player_id: Some(*id),
                x: 0.5,
                y: 0.5,
            })
            .collect();
        *repo.shared.lock().unwrap() = Some(SharedLineup {
            item: PublicLineupItem {
                share_code: "ABCD1234".into(),
                formation: "4-3-3".into(),
                author: crate::lineup::domain::LineupAuthor {
                    display_name: "球迷".into(),
                    avatar_url: None,
                },
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                likes: LineupLikeState {
                    likes_count: 0,
                    liked_by_me: false,
                    disliked_by_me: false,
                },
            },
            team: SquadTeam {
                team_id: 77680,
                team_name: "成都蓉城".into(),
                avatar_url: None,
            },
            slots,
            players: vec![],
        });
        let mut squad = full_squad();
        squad.push(outsider(99));
        let svc = PublicLineupService::new(
            Arc::new(FakeSquads {
                players: squad,
                missing_table: false,
            }),
            Arc::new(repo),
            Arc::new(FakeUserAccess {
                active: true,
                unavailable: false,
            }),
            Arc::new(FakeCodes),
        );
        let lineup = svc.shared("ABCD1234", None).await.unwrap();
        let ids: Vec<i64> = lineup.players.iter().map(|p| p.player_id).collect();
        assert_eq!(ids, vec![1, 99]);
        assert_eq!(lineup.players[0].club, None);
        assert_eq!(
            lineup.players[1].club.as_ref().unwrap().team_name,
            "山东泰山"
        );
    }

    #[tokio::test]
    async fn consensus_validates_the_team() {
        let svc = consensus_service(vec![], full_squad());
        assert_eq!(
            svc.consensus(0).await.unwrap_err(),
            LineupPublicError::Invalid
        );
        assert_eq!(
            svc.consensus(1).await.unwrap_err(),
            LineupPublicError::NotFound
        );
    }
}
