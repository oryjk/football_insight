use crate::lineup::{
    domain::{
        Lineup, LineupDraft, LineupEditor, LineupSaveError, LineupSlot, PublishedLineup,
        validate_lineup_input, validate_publish_input,
    },
    ports::{LineupRepository, LineupUserAccess, SquadRepository, StoreError},
};
use std::sync::Arc;
use uuid::Uuid;

pub struct MyLineupService {
    squads: Arc<dyn SquadRepository>,
    lineups: Arc<dyn LineupRepository>,
    users: Arc<dyn LineupUserAccess>,
}

pub fn generate_share_code() -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::rng();
    (0..8)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect()
}

fn store_error(_: StoreError) -> LineupSaveError {
    LineupSaveError::Unavailable
}

const SHARE_CODE_ATTEMPTS: usize = 5;

impl MyLineupService {
    pub fn new(
        squads: Arc<dyn SquadRepository>,
        lineups: Arc<dyn LineupRepository>,
        users: Arc<dyn LineupUserAccess>,
    ) -> Self {
        Self {
            squads,
            lineups,
            users,
        }
    }

    async fn authorize(&self, user_id: Uuid) -> Result<(), LineupSaveError> {
        if !self.users.is_active(user_id).await.map_err(store_error)? {
            return Err(LineupSaveError::Unauthorized);
        }
        Ok(())
    }

    pub async fn get(
        &self,
        user_id: Uuid,
        team_id: i64,
    ) -> Result<Option<Lineup>, LineupSaveError> {
        self.authorize(user_id).await?;
        if team_id <= 0 {
            return Err(LineupSaveError::TeamNotFound);
        }
        self.lineups
            .find(user_id, team_id)
            .await
            .map_err(store_error)
    }

    pub async fn editor(
        &self,
        user_id: Uuid,
        team_id: i64,
    ) -> Result<LineupEditor, LineupSaveError> {
        let lineup = self
            .get(user_id, team_id)
            .await?
            .filter(|lineup| validate_publish_input(&lineup.formation, &lineup.slots).is_ok());
        let draft = self
            .lineups
            .find_draft(user_id, team_id)
            .await
            .map_err(store_error)?;
        Ok(LineupEditor { lineup, draft })
    }

    async fn validate_save(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: &[LineupSlot],
    ) -> Result<(), LineupSaveError> {
        self.authorize(user_id).await?;
        validate_lineup_input(formation, slots)?;
        self.squads
            .find_team(team_id)
            .await
            .map_err(|_| LineupSaveError::Unavailable)?
            .ok_or(LineupSaveError::TeamNotFound)?;
        let ids: Vec<i64> = slots.iter().filter_map(|s| s.player_id).collect();
        if !ids.is_empty() {
            let members = self
                .lineups
                .squad_members(team_id, &ids)
                .await
                .map_err(store_error)?;
            if members.len() != ids.len() {
                return Err(LineupSaveError::PlayerNotInSquad);
            }
        }
        Ok(())
    }

    pub async fn save_draft(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: Vec<LineupSlot>,
    ) -> Result<LineupDraft, LineupSaveError> {
        self.validate_save(user_id, team_id, formation, &slots)
            .await?;
        self.lineups
            .save_draft(user_id, team_id, formation, &slots)
            .await
            .map_err(store_error)
    }

    pub async fn publish(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: Vec<LineupSlot>,
    ) -> Result<PublishedLineup, LineupSaveError> {
        self.validate_save(user_id, team_id, formation, &slots)
            .await?;
        validate_publish_input(formation, &slots)?;
        for _ in 0..SHARE_CODE_ATTEMPTS {
            match self
                .lineups
                .publish(user_id, team_id, formation, &slots, &generate_share_code())
                .await
            {
                Ok(result) => return Ok(result),
                Err(StoreError::ShareCodeTaken) => continue,
                Err(error) => return Err(store_error(error)),
            }
        }
        Err(LineupSaveError::Unavailable)
    }

    pub async fn save(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: Vec<LineupSlot>,
    ) -> Result<Lineup, LineupSaveError> {
        self.validate_save(user_id, team_id, formation, &slots)
            .await?;
        for _ in 0..SHARE_CODE_ATTEMPTS {
            match self
                .lineups
                .upsert(user_id, team_id, formation, &slots, &generate_share_code())
                .await
            {
                Ok(lineup) => return Ok(lineup),
                Err(StoreError::ShareCodeTaken) => continue,
                Err(e) => return Err(store_error(e)),
            }
        }
        Err(LineupSaveError::Unavailable)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::lineup::application::tests::FakeSquads;
    use async_trait::async_trait;
    use std::sync::Mutex;

    pub(crate) struct FakeUserAccess {
        pub active: bool,
        pub unavailable: bool,
    }

    impl Default for FakeUserAccess {
        fn default() -> Self {
            Self {
                active: true,
                unavailable: false,
            }
        }
    }

    #[async_trait]
    impl LineupUserAccess for FakeUserAccess {
        async fn is_active(&self, _: Uuid) -> Result<bool, StoreError> {
            if self.unavailable {
                Err(StoreError::Unavailable)
            } else {
                Ok(self.active)
            }
        }
    }

    #[derive(Default)]
    pub(crate) struct FakeLineups {
        pub members: Vec<i64>,
        pub stored: Mutex<std::collections::HashMap<(Uuid, i64), Lineup>>,
        pub drafts: Mutex<std::collections::HashMap<(Uuid, i64), LineupDraft>>,
        pub collisions: Mutex<u32>,
        pub upserts: Mutex<Vec<String>>,
        pub fail: bool,
    }

    #[async_trait]
    impl LineupRepository for FakeLineups {
        async fn find_draft(&self, u: Uuid, t: i64) -> Result<Option<LineupDraft>, StoreError> {
            Ok(self.drafts.lock().unwrap().get(&(u, t)).cloned())
        }
        async fn save_draft(
            &self,
            u: Uuid,
            t: i64,
            formation: &str,
            slots: &[LineupSlot],
        ) -> Result<LineupDraft, StoreError> {
            if self.fail {
                return Err(StoreError::Unavailable);
            }
            let draft = LineupDraft {
                formation: formation.into(),
                slots: slots.to_vec(),
                updated_at: chrono::Utc::now(),
            };
            self.drafts.lock().unwrap().insert((u, t), draft.clone());
            Ok(draft)
        }
        async fn publish(
            &self,
            u: Uuid,
            t: i64,
            formation: &str,
            slots: &[LineupSlot],
            code: &str,
        ) -> Result<PublishedLineup, StoreError> {
            let lineup = self.upsert(u, t, formation, slots, code).await?;
            let draft = self.find_draft(u, t).await?.unwrap();
            Ok(PublishedLineup { lineup, draft })
        }
        async fn find(&self, u: Uuid, t: i64) -> Result<Option<Lineup>, StoreError> {
            if self.fail {
                return Err(StoreError::Unavailable);
            }
            Ok(self.stored.lock().unwrap().get(&(u, t)).cloned())
        }
        async fn squad_members(&self, _t: i64, ids: &[i64]) -> Result<Vec<i64>, StoreError> {
            Ok(ids
                .iter()
                .copied()
                .filter(|id| self.members.contains(id))
                .collect())
        }
        async fn upsert(
            &self,
            u: Uuid,
            t: i64,
            formation: &str,
            slots: &[LineupSlot],
            code: &str,
        ) -> Result<Lineup, StoreError> {
            if self.fail {
                return Err(StoreError::Unavailable);
            }
            let mut left = self.collisions.lock().unwrap();
            if *left > 0 {
                *left -= 1;
                return Err(StoreError::ShareCodeTaken);
            }
            self.upserts.lock().unwrap().push(code.to_string());
            let mut stored = self.stored.lock().unwrap();
            // 与真实 SQL 一致：已存在时 share_code 不变
            let share_code = stored
                .get(&(u, t))
                .map(|l| l.share_code.clone())
                .unwrap_or_else(|| code.to_string());
            let lineup = Lineup {
                formation: formation.into(),
                slots: slots.to_vec(),
                share_code,
                updated_at: chrono::Utc::now(),
            };
            stored.insert((u, t), lineup.clone());
            self.drafts.lock().unwrap().insert(
                (u, t),
                LineupDraft {
                    formation: formation.into(),
                    slots: slots.to_vec(),
                    updated_at: lineup.updated_at,
                },
            );
            Ok(lineup)
        }
    }

    fn slots(players: &[Option<i64>]) -> Vec<LineupSlot> {
        (0..11u8)
            .map(|i| LineupSlot {
                slot: i,
                player_id: players.get(i as usize).copied().flatten(),
                x: 0.5,
                y: 0.5,
            })
            .collect()
    }

    fn service(members: Vec<i64>) -> (MyLineupService, Arc<FakeLineups>) {
        let lineups = Arc::new(FakeLineups {
            members,
            ..Default::default()
        });
        let svc = MyLineupService::new(
            Arc::new(FakeSquads {
                players: vec![],
                missing_table: false,
            }),
            lineups.clone(),
            Arc::new(FakeUserAccess::default()),
        );
        (svc, lineups)
    }

    const TEAM: i64 = 77680; // FakeSquads 只认识这支队
    fn user() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn share_codes_are_eight_base62_characters() {
        for _ in 0..50 {
            let code = generate_share_code();
            assert_eq!(code.len(), 8);
            assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
        }
    }

    #[tokio::test]
    async fn saves_a_valid_lineup_with_current_and_former_players() {
        let (svc, repo) = service(vec![1, 2, 3]);
        let lineup = svc
            .save(user(), TEAM, "4-3-3", slots(&[Some(1), Some(2), Some(3)]))
            .await
            .unwrap();
        assert_eq!(lineup.formation, "4-3-3");
        assert_eq!(lineup.share_code.len(), 8);
        assert_eq!(repo.upserts.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn rejects_players_that_are_not_in_the_squad() {
        let (svc, _) = service(vec![1]);
        let err = svc
            .save(user(), TEAM, "4-3-3", slots(&[Some(1), Some(99)]))
            .await
            .unwrap_err();
        assert_eq!(err, LineupSaveError::PlayerNotInSquad);
    }

    #[tokio::test]
    async fn rejects_duplicates_bad_formation_and_unknown_team_without_touching_the_store() {
        let (svc, repo) = service(vec![1, 2]);
        assert_eq!(
            svc.save(user(), TEAM, "4-3-3", slots(&[Some(1), Some(1)]))
                .await
                .unwrap_err(),
            LineupSaveError::DuplicatePlayer
        );
        assert_eq!(
            svc.save(user(), TEAM, "1-1-1", slots(&[]))
                .await
                .unwrap_err(),
            LineupSaveError::InvalidFormation
        );
        assert_eq!(
            svc.save(user(), 5, "4-3-3", slots(&[])).await.unwrap_err(),
            LineupSaveError::TeamNotFound
        );
        assert!(repo.upserts.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn saving_again_overwrites_and_keeps_the_share_code() {
        let (svc, _) = service(vec![1, 2]);
        let u = user();
        let first = svc.save(u, TEAM, "4-3-3", slots(&[Some(1)])).await.unwrap();
        let second = svc.save(u, TEAM, "4-4-2", slots(&[Some(2)])).await.unwrap();
        assert_eq!(second.share_code, first.share_code);
        assert_eq!(second.formation, "4-4-2");
    }

    #[tokio::test]
    async fn retries_on_share_code_collision_then_succeeds() {
        let (svc, repo) = service(vec![]);
        *repo.collisions.lock().unwrap() = 2;
        assert!(svc.save(user(), TEAM, "4-3-3", slots(&[])).await.is_ok());
    }

    #[tokio::test]
    async fn gives_up_after_repeated_collisions_as_unavailable() {
        let (svc, repo) = service(vec![]);
        *repo.collisions.lock().unwrap() = 50;
        assert_eq!(
            svc.save(user(), TEAM, "4-3-3", slots(&[]))
                .await
                .unwrap_err(),
            LineupSaveError::Unavailable
        );
    }

    #[tokio::test]
    async fn get_returns_none_then_the_saved_lineup_and_maps_store_failure() {
        let (svc, _) = service(vec![]);
        let u = user();
        assert!(svc.get(u, TEAM).await.unwrap().is_none());
        svc.save(u, TEAM, "3-5-2", slots(&[])).await.unwrap();
        assert_eq!(svc.get(u, TEAM).await.unwrap().unwrap().formation, "3-5-2");
        let broken = MyLineupService::new(
            Arc::new(FakeSquads {
                players: vec![],
                missing_table: false,
            }),
            Arc::new(FakeLineups {
                fail: true,
                ..Default::default()
            }),
            Arc::new(FakeUserAccess::default()),
        );
        assert_eq!(
            broken.get(u, TEAM).await.unwrap_err(),
            LineupSaveError::Unavailable
        );
    }

    #[tokio::test]
    async fn disabled_or_deleted_users_cannot_read_or_save_even_with_a_verified_user_id() {
        let repo = Arc::new(FakeLineups::default());
        let svc = MyLineupService::new(
            Arc::new(FakeSquads {
                players: vec![],
                missing_table: false,
            }),
            repo.clone(),
            Arc::new(FakeUserAccess {
                active: false,
                unavailable: false,
            }),
        );
        assert_eq!(
            svc.get(user(), TEAM).await.unwrap_err(),
            LineupSaveError::Unauthorized
        );
        assert_eq!(
            svc.save(user(), TEAM, "4-3-3", slots(&[]))
                .await
                .unwrap_err(),
            LineupSaveError::Unauthorized
        );
        assert!(repo.upserts.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn account_lookup_failure_denies_read_and_save_as_unavailable() {
        let svc = MyLineupService::new(
            Arc::new(FakeSquads {
                players: vec![],
                missing_table: false,
            }),
            Arc::new(FakeLineups::default()),
            Arc::new(FakeUserAccess {
                active: true,
                unavailable: true,
            }),
        );
        assert_eq!(
            svc.get(user(), TEAM).await.unwrap_err(),
            LineupSaveError::Unavailable
        );
        assert_eq!(
            svc.save(user(), TEAM, "4-3-3", slots(&[]))
                .await
                .unwrap_err(),
            LineupSaveError::Unavailable
        );
    }
    #[tokio::test]
    async fn private_edits_leave_published_snapshot_unchanged_and_editor_separates_both() {
        let (svc, repo) = service((1..=11).collect());
        let u = user();
        assert_eq!(
            svc.editor(u, TEAM).await.unwrap(),
            LineupEditor {
                lineup: None,
                draft: None
            }
        );
        let full = slots(&(1..=11).map(Some).collect::<Vec<_>>());
        let first = svc.publish(u, TEAM, "4-3-3", full.clone()).await.unwrap();
        svc.save_draft(u, TEAM, "4-4-2", slots(&[Some(3)]))
            .await
            .unwrap();
        let editor = svc.editor(u, TEAM).await.unwrap();
        assert_eq!(editor.lineup, Some(first.lineup.clone()));
        assert_eq!(editor.draft.unwrap().formation, "4-4-2");
        let second = svc.publish(u, TEAM, "3-5-2", full).await.unwrap();
        assert_eq!(second.lineup.share_code, first.lineup.share_code);
        assert_eq!(second.draft.formation, second.lineup.formation);
        assert_eq!(repo.upserts.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn incomplete_drafts_are_private_and_publish_rejects_without_writes() {
        let (svc, repo) = service(vec![1]);
        let a = user();
        svc.save_draft(a, TEAM, "4-3-3", slots(&[Some(1)]))
            .await
            .unwrap();
        assert!(svc.editor(a, TEAM).await.unwrap().lineup.is_none());
        let other = svc.editor(user(), TEAM).await.unwrap();
        assert!(other.draft.is_none());
        assert!(other.lineup.is_none());
        assert_eq!(
            svc.publish(a, TEAM, "4-3-3", slots(&[Some(1)]))
                .await
                .unwrap_err(),
            LineupSaveError::IncompleteLineup
        );
        assert!(repo.upserts.lock().unwrap().is_empty());
        // A legacy incomplete public-table row must never be presented as a published version.
        svc.save(a, TEAM, "4-4-2", slots(&[Some(1)])).await.unwrap();
        let editor = svc.editor(a, TEAM).await.unwrap();
        assert!(editor.lineup.is_none());
        assert_eq!(editor.draft.unwrap().formation, "4-4-2");
    }

    #[tokio::test]
    async fn new_workflows_reject_disabled_accounts_and_invalid_drafts_before_any_write() {
        let repo = Arc::new(FakeLineups::default());
        let svc = MyLineupService::new(
            Arc::new(FakeSquads {
                players: vec![],
                missing_table: false,
            }),
            repo.clone(),
            Arc::new(FakeUserAccess {
                active: false,
                unavailable: false,
            }),
        );
        let u = user();
        assert_eq!(
            svc.editor(u, TEAM).await.unwrap_err(),
            LineupSaveError::Unauthorized
        );
        assert_eq!(
            svc.save_draft(u, TEAM, "4-3-3", slots(&[]))
                .await
                .unwrap_err(),
            LineupSaveError::Unauthorized
        );
        assert_eq!(
            svc.publish(u, TEAM, "4-3-3", slots(&[])).await.unwrap_err(),
            LineupSaveError::Unauthorized
        );
        assert!(repo.drafts.lock().unwrap().is_empty());
        let (svc, repo) = service(vec![1]);
        assert_eq!(
            svc.save_draft(u, TEAM, "4-3-3", slots(&[Some(1), Some(1)]))
                .await
                .unwrap_err(),
            LineupSaveError::DuplicatePlayer
        );
        assert_eq!(
            svc.save_draft(u, TEAM, "4-3-3", slots(&[Some(9)]))
                .await
                .unwrap_err(),
            LineupSaveError::PlayerNotInSquad
        );
        assert!(repo.drafts.lock().unwrap().is_empty());
    }
}
