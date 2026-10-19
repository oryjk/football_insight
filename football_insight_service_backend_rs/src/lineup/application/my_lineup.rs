use crate::lineup::{
    domain::{Lineup, LineupSaveError, LineupSlot, validate_lineup_input},
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

    pub async fn save(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: Vec<LineupSlot>,
    ) -> Result<Lineup, LineupSaveError> {
        self.authorize(user_id).await?;
        validate_lineup_input(formation, &slots)?;
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
        pub stored: Mutex<Option<Lineup>>,
        pub collisions: Mutex<u32>,
        pub upserts: Mutex<Vec<String>>,
        pub fail: bool,
    }

    #[async_trait]
    impl LineupRepository for FakeLineups {
        async fn find(&self, _u: Uuid, _t: i64) -> Result<Option<Lineup>, StoreError> {
            if self.fail {
                return Err(StoreError::Unavailable);
            }
            Ok(self.stored.lock().unwrap().clone())
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
            _u: Uuid,
            _t: i64,
            formation: &str,
            slots: &[LineupSlot],
            code: &str,
        ) -> Result<Lineup, StoreError> {
            let mut left = self.collisions.lock().unwrap();
            if *left > 0 {
                *left -= 1;
                return Err(StoreError::ShareCodeTaken);
            }
            self.upserts.lock().unwrap().push(code.to_string());
            let mut stored = self.stored.lock().unwrap();
            // 与真实 SQL 一致：已存在时 share_code 不变
            let share_code = stored
                .as_ref()
                .map(|l| l.share_code.clone())
                .unwrap_or_else(|| code.to_string());
            let lineup = Lineup {
                formation: formation.into(),
                slots: slots.to_vec(),
                share_code,
                updated_at: chrono::Utc::now(),
            };
            *stored = Some(lineup.clone());
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
}
