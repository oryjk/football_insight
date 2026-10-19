use crate::lineup::{
    domain::{
        LineupCodeImage, LineupLikeState, LineupPublicError, PublicLineupPage, PublicLineupQuery,
        SharedLineup, valid_share_code,
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
        Ok(lineup)
    }

    pub async fn set_like(
        &self,
        code: &str,
        user: Uuid,
        liked: bool,
    ) -> Result<LineupLikeState, LineupPublicError> {
        self.authorize(Some(user)).await?;
        if !valid_share_code(code) {
            return Err(LineupPublicError::Invalid);
        }
        self.lineups.set_like(code, user, liked).await
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
        domain::PublicLineupItem,
    };
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    pub(crate) struct FakePublicLineups {
        pub shared: Mutex<Option<SharedLineup>>,
        pub liked: Mutex<bool>,
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
        async fn shared(
            &self,
            _: &str,
            _: Option<Uuid>,
        ) -> Result<Option<SharedLineup>, LineupPublicError> {
            Ok(self.shared.lock().unwrap().clone())
        }
        async fn set_like(
            &self,
            _: &str,
            _: Uuid,
            liked: bool,
        ) -> Result<LineupLikeState, LineupPublicError> {
            if self.shared.lock().unwrap().is_none() {
                return Err(LineupPublicError::NotFound);
            }
            *self.liked.lock().unwrap() = liked;
            Ok(LineupLikeState {
                likes_count: i64::from(liked),
                liked_by_me: liked,
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
}
