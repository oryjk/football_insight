use super::{
    domain::{
        CandidatePlayer, NOMINATION_LIMIT, NominationCard, TransferCodeImage, TransferError,
        TransferIntent, TransferTeam, TransferWindow, VoteTally, VoteValue, candidate_allowed,
        generate_share_code, membership_from, valid_share_code,
    },
    ports::{
        NominateCommand, TransferCandidateDirectory, TransferCodePort, TransferRepository,
        TransferUserAccess, UserProfile,
    },
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;
pub const BOARD_PAGE_SIZE: i64 = 20;
pub const SUMMARY_SIZE: i64 = 3;
const SHARE_CODE_ATTEMPTS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowView {
    pub window: TransferWindow,
    pub open: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardSlice {
    pub intent: TransferIntent,
    pub total: i64,
    pub items: Vec<NominationCard>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferSummary {
    pub team: TransferTeam,
    pub window: Option<WindowView>,
    pub can_vote: bool,
    pub boards: Vec<BoardSlice>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferBoard {
    pub team: TransferTeam,
    pub window: Option<WindowView>,
    pub intent: TransferIntent,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub can_vote: bool,
    pub items: Vec<NominationCard>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardView {
    pub card: NominationCard,
    pub team: TransferTeam,
    pub window: WindowView,
    pub can_vote: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGroup {
    pub key: String,
    pub title: String,
    pub players: Vec<CandidatePlayer>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidates {
    pub intent: TransferIntent,
    pub groups: Vec<CandidateGroup>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominateResult {
    pub created: bool,
    pub view: CardView,
}

pub struct TransferService {
    repo: Arc<dyn TransferRepository>,
    directory: Arc<dyn TransferCandidateDirectory>,
    users: Arc<dyn TransferUserAccess>,
    codes: Arc<dyn TransferCodePort>,
    clock: Clock,
}

impl TransferService {
    pub fn new(
        repo: Arc<dyn TransferRepository>,
        directory: Arc<dyn TransferCandidateDirectory>,
        users: Arc<dyn TransferUserAccess>,
        codes: Arc<dyn TransferCodePort>,
        clock: Clock,
    ) -> Self {
        Self {
            repo,
            directory,
            users,
            codes,
            clock,
        }
    }

    fn view(&self, window: TransferWindow) -> WindowView {
        let open = window.is_open((self.clock)());
        WindowView { window, open }
    }

    /// 浏览者：带 token 时账号必须正常（与战术板一致）；游客为 None。
    async fn viewer_profile(
        &self,
        viewer: Option<Uuid>,
    ) -> Result<Option<UserProfile>, TransferError> {
        let Some(user) = viewer else { return Ok(None) };
        match self.users.profile(user).await? {
            Some(profile) if profile.active => Ok(Some(profile)),
            _ => Err(TransferError::Unauthorized),
        }
    }

    /// 写操作：必须是正常账号、主队等于该队、该期开放。
    async fn require_voter(
        &self,
        user: Uuid,
        team_id: i64,
        window: &TransferWindow,
    ) -> Result<(), TransferError> {
        let profile = self
            .viewer_profile(Some(user))
            .await?
            .ok_or(TransferError::Unauthorized)?;
        if profile.favorite_team_id != Some(team_id) {
            return Err(TransferError::NotFavoriteTeam);
        }
        if !window.is_open((self.clock)()) {
            return Err(TransferError::WindowClosed);
        }
        Ok(())
    }

    fn can_vote(profile: &Option<UserProfile>, team_id: i64, window: Option<&WindowView>) -> bool {
        matches!(profile, Some(p) if p.favorite_team_id == Some(team_id))
            && window.is_some_and(|w| w.open)
    }

    async fn team(&self, team_id: i64) -> Result<TransferTeam, TransferError> {
        if team_id <= 0 {
            return Err(TransferError::Invalid);
        }
        self.directory
            .find_team(team_id)
            .await?
            .ok_or(TransferError::NotFound)
    }

    async fn window(&self, code: Option<&str>) -> Result<Option<TransferWindow>, TransferError> {
        match code {
            Some(code) => self
                .repo
                .window_by_code(code)
                .await?
                .map(Some)
                .ok_or(TransferError::NotFound),
            None => self.repo.current_window((self.clock)()).await,
        }
    }

    pub async fn current_window(&self) -> Result<Option<WindowView>, TransferError> {
        Ok(self
            .repo
            .current_window((self.clock)())
            .await?
            .map(|w| self.view(w)))
    }

    pub async fn summary(
        &self,
        team_id: i64,
        window_code: Option<&str>,
        viewer: Option<Uuid>,
    ) -> Result<TransferSummary, TransferError> {
        let profile = self.viewer_profile(viewer).await?;
        let team = self.team(team_id).await?;
        let window = self.window(window_code).await?.map(|w| self.view(w));
        let mut boards = Vec::with_capacity(3);
        for intent in TransferIntent::ALL {
            let (items, total) = match &window {
                Some(w) => {
                    self.repo
                        .board(w.window.id, team_id, intent, 0, SUMMARY_SIZE, viewer)
                        .await?
                }
                None => (Vec::new(), 0),
            };
            boards.push(BoardSlice {
                intent,
                total,
                items,
            });
        }
        let can_vote = Self::can_vote(&profile, team_id, window.as_ref());
        Ok(TransferSummary {
            team,
            window,
            can_vote,
            boards,
        })
    }

    pub async fn board(
        &self,
        team_id: i64,
        intent: &str,
        page: i64,
        window_code: Option<&str>,
        viewer: Option<Uuid>,
    ) -> Result<TransferBoard, TransferError> {
        let intent = TransferIntent::parse(intent).ok_or(TransferError::Invalid)?;
        if page < 1 {
            return Err(TransferError::Invalid);
        }
        let profile = self.viewer_profile(viewer).await?;
        let team = self.team(team_id).await?;
        let window = self.window(window_code).await?.map(|w| self.view(w));
        let (items, total) = match &window {
            Some(w) => {
                self.repo
                    .board(
                        w.window.id,
                        team_id,
                        intent,
                        (page - 1) * BOARD_PAGE_SIZE,
                        BOARD_PAGE_SIZE,
                        viewer,
                    )
                    .await?
            }
            None => (Vec::new(), 0),
        };
        let can_vote = Self::can_vote(&profile, team_id, window.as_ref());
        Ok(TransferBoard {
            team,
            window,
            intent,
            page,
            page_size: BOARD_PAGE_SIZE,
            total,
            can_vote,
            items,
        })
    }

    pub async fn card(
        &self,
        share_code: &str,
        viewer: Option<Uuid>,
    ) -> Result<CardView, TransferError> {
        if !valid_share_code(share_code) {
            return Err(TransferError::Invalid);
        }
        let profile = self.viewer_profile(viewer).await?;
        let context = self
            .repo
            .card(share_code, viewer)
            .await?
            .ok_or(TransferError::NotFound)?;
        let window = self.view(context.window);
        let can_vote = Self::can_vote(&profile, context.team.team_id, Some(&window));
        Ok(CardView {
            card: context.card,
            team: context.team,
            window,
            can_vote,
        })
    }

    pub async fn candidates(
        &self,
        team_id: i64,
        intent: &str,
    ) -> Result<Candidates, TransferError> {
        let intent = TransferIntent::parse(intent).ok_or(TransferError::Invalid)?;
        let team = self.team(team_id).await?;
        let groups = match intent {
            TransferIntent::Out | TransferIntent::Stay => vec![CandidateGroup {
                key: "squad".into(),
                title: team.team_name.clone(),
                players: self.directory.team_players(team_id).await?,
            }],
            TransferIntent::In => {
                let mut groups = vec![CandidateGroup {
                    key: "former".into(),
                    title: "本队旧将".into(),
                    players: self.directory.former_players(team_id).await?,
                }];
                // 目录已按队名排序，这里按出现顺序分组。
                for player in self.directory.other_active_players(team_id).await? {
                    let Some(club) = player.current_team.clone() else {
                        continue;
                    };
                    let key = format!("team:{}", club.team_id);
                    match groups.iter_mut().find(|g| g.key == key) {
                        Some(group) => group.players.push(player),
                        None => groups.push(CandidateGroup {
                            key,
                            title: club.team_name,
                            players: vec![player],
                        }),
                    }
                }
                groups
            }
        };
        Ok(Candidates { intent, groups })
    }

    pub async fn nominate(
        &self,
        user_id: Uuid,
        team_id: i64,
        player_id: i64,
        intent: &str,
    ) -> Result<NominateResult, TransferError> {
        let intent = TransferIntent::parse(intent).ok_or(TransferError::Invalid)?;
        if player_id <= 0 {
            return Err(TransferError::Invalid);
        }
        self.team(team_id).await?;
        let window = self
            .repo
            .current_window((self.clock)())
            .await?
            .ok_or(TransferError::WindowClosed)?;
        self.require_voter(user_id, team_id, &window).await?;
        let rows = self.directory.player_rows(player_id).await?;
        if !candidate_allowed(intent, membership_from(team_id, &rows)) {
            return Err(TransferError::InvalidCandidate);
        }
        for _ in 0..SHARE_CODE_ATTEMPTS {
            let command = NominateCommand {
                window_id: window.id,
                team_id,
                player_id,
                intent,
                user_id,
                share_code: generate_share_code(),
                limit: NOMINATION_LIMIT,
            };
            match self.repo.nominate(command).await {
                Ok(outcome) => {
                    let view = self.card(&outcome.share_code, Some(user_id)).await?;
                    return Ok(NominateResult {
                        created: outcome.created,
                        view,
                    });
                }
                Err(TransferError::ShareCodeTaken) => continue,
                Err(error) => return Err(error),
            }
        }
        Err(TransferError::Unavailable)
    }

    pub async fn vote(
        &self,
        user_id: Uuid,
        share_code: &str,
        value: Option<VoteValue>,
    ) -> Result<VoteTally, TransferError> {
        if !valid_share_code(share_code) {
            return Err(TransferError::Invalid);
        }
        let context = self
            .repo
            .card(share_code, Some(user_id))
            .await?
            .ok_or(TransferError::NotFound)?;
        self.require_voter(user_id, context.team.team_id, &context.window)
            .await?;
        self.repo.set_vote(share_code, user_id, value).await
    }

    /// 拉票海报的小程序码：只要求登录的正常账号和存在的提名卡；
    /// 不要求是主队、也不要求本期开放——海报只是分享，不是投票。
    pub async fn code_image(
        &self,
        user_id: Uuid,
        share_code: &str,
    ) -> Result<TransferCodeImage, TransferError> {
        if !valid_share_code(share_code) {
            return Err(TransferError::Invalid);
        }
        self.viewer_profile(Some(user_id))
            .await?
            .ok_or(TransferError::Unauthorized)?;
        self.repo
            .card(share_code, Some(user_id))
            .await?
            .ok_or(TransferError::NotFound)?;
        self.codes.image(share_code).await
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::transfer::{
        domain::{CardPlayer, NominationCard, TransferCodeImage},
        ports::{CardContext, NominateOutcome, TransferCodePort},
    };
    use chrono::TimeZone;
    use std::sync::Mutex;

    pub(crate) const FAN: Uuid = Uuid::from_u128(1);
    pub(crate) const RIVAL: Uuid = Uuid::from_u128(2);

    pub(crate) fn at(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, day, 12, 0, 0).unwrap()
    }
    pub(crate) fn window() -> TransferWindow {
        TransferWindow {
            id: 7,
            code: "2026-winter".into(),
            name: "2026 冬窗".into(),
            opens_at: at(1),
            closes_at: at(20),
        }
    }
    pub(crate) fn team(id: i64) -> TransferTeam {
        TransferTeam {
            team_id: id,
            team_name: format!("Team {id}"),
            avatar_url: None,
        }
    }
    pub(crate) fn card(code: &str, intent: TransferIntent) -> NominationCard {
        NominationCard {
            share_code: code.into(),
            intent,
            returning: false,
            player: CardPlayer {
                player_id: 10,
                name: "Player 10".into(),
                avatar_url: None,
                position: Some("FW".into()),
                shirt_number: Some(10),
                current_team: Some(team(1)),
                departed: false,
            },
            up: 1,
            down: 0,
            my_vote: 1,
            rank: Some(1),
            created_at: at(2),
        }
    }

    #[derive(Default)]
    pub(crate) struct FakeRepo {
        pub nominations: Mutex<Vec<NominateCommand>>,
        pub votes: Mutex<Vec<(String, Uuid, Option<VoteValue>)>>,
        pub existing: Mutex<bool>,
        pub taken_once: Mutex<bool>,
        pub limit_hit: Mutex<bool>,
        pub card_team: Mutex<i64>,
    }
    #[async_trait::async_trait]
    impl TransferRepository for FakeRepo {
        async fn current_window(
            &self,
            _: DateTime<Utc>,
        ) -> Result<Option<TransferWindow>, TransferError> {
            Ok(Some(window()))
        }
        async fn window_by_code(
            &self,
            code: &str,
        ) -> Result<Option<TransferWindow>, TransferError> {
            Ok((code == "2026-winter").then(window))
        }
        async fn board(
            &self,
            _: i64,
            _: i64,
            intent: TransferIntent,
            offset: i64,
            limit: i64,
            _: Option<Uuid>,
        ) -> Result<(Vec<NominationCard>, i64), TransferError> {
            let all: Vec<_> = (0..5)
                .map(|i| card(&format!("CODE000{i}"), intent))
                .collect();
            Ok((
                all.into_iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .collect(),
                5,
            ))
        }
        async fn card(
            &self,
            code: &str,
            _: Option<Uuid>,
        ) -> Result<Option<CardContext>, TransferError> {
            if code == "MISSING0" {
                return Ok(None);
            }
            let team_id = *self.card_team.lock().unwrap();
            Ok(Some(CardContext {
                card: card(code, TransferIntent::Out),
                team: team(if team_id == 0 { 1 } else { team_id }),
                window: window(),
            }))
        }
        async fn nominate(
            &self,
            command: NominateCommand,
        ) -> Result<NominateOutcome, TransferError> {
            let mut taken = self.taken_once.lock().unwrap();
            if *taken {
                *taken = false;
                return Err(TransferError::ShareCodeTaken);
            }
            if *self.limit_hit.lock().unwrap() {
                return Err(TransferError::NominationLimit);
            }
            let created = !*self.existing.lock().unwrap();
            let code = if created {
                command.share_code.clone()
            } else {
                "EXIST001".into()
            };
            self.nominations.lock().unwrap().push(command);
            Ok(NominateOutcome {
                share_code: code,
                created,
            })
        }
        async fn set_vote(
            &self,
            code: &str,
            user: Uuid,
            value: Option<VoteValue>,
        ) -> Result<VoteTally, TransferError> {
            self.votes.lock().unwrap().push((code.into(), user, value));
            Ok(VoteTally {
                up: 1,
                down: 0,
                my_vote: value.map_or(0, VoteValue::as_i16),
            })
        }
    }

    pub(crate) struct FakeDirectory;
    #[async_trait::async_trait]
    impl TransferCandidateDirectory for FakeDirectory {
        async fn find_team(&self, id: i64) -> Result<Option<TransferTeam>, TransferError> {
            Ok((id == 1 || id == 2).then(|| team(id)))
        }
        async fn player_rows(&self, player: i64) -> Result<Vec<(i64, bool)>, TransferError> {
            // 10 = 本队现役；20 = 本队旧将、现于 2 队；30 = 2 队现役；40 = 无效
            Ok(match player {
                10 => vec![(1, true)],
                20 => vec![(1, false), (2, true)],
                30 => vec![(2, true)],
                _ => vec![],
            })
        }
        async fn team_players(&self, _: i64) -> Result<Vec<CandidatePlayer>, TransferError> {
            Ok(vec![candidate(10, Some(team(1)))])
        }
        async fn former_players(&self, _: i64) -> Result<Vec<CandidatePlayer>, TransferError> {
            Ok(vec![candidate(20, Some(team(2)))])
        }
        async fn other_active_players(
            &self,
            _: i64,
        ) -> Result<Vec<CandidatePlayer>, TransferError> {
            Ok(vec![
                candidate(30, Some(team(2))),
                candidate(31, Some(team(3))),
            ])
        }
    }
    fn candidate(id: i64, current_team: Option<TransferTeam>) -> CandidatePlayer {
        CandidatePlayer {
            player_id: id,
            name: format!("Player {id}"),
            shirt_number: None,
            position: None,
            avatar_url: None,
            current_team,
        }
    }

    pub(crate) struct FakeUsers {
        pub favorite: Mutex<Option<i64>>,
        pub active: bool,
    }
    #[async_trait::async_trait]
    impl TransferUserAccess for FakeUsers {
        async fn profile(&self, user: Uuid) -> Result<Option<UserProfile>, TransferError> {
            Ok((user == FAN || user == RIVAL).then(|| UserProfile {
                active: self.active,
                favorite_team_id: if user == FAN {
                    *self.favorite.lock().unwrap()
                } else {
                    Some(2)
                },
            }))
        }
    }

    pub(crate) struct FakeCodes;
    #[async_trait::async_trait]
    impl TransferCodePort for FakeCodes {
        async fn image(&self, code: &str) -> Result<TransferCodeImage, TransferError> {
            Ok(TransferCodeImage {
                image_base64: format!("img-{code}"),
                mime_type: "image/png".into(),
            })
        }
    }

    pub(crate) fn service_at(
        now: DateTime<Utc>,
        active: bool,
    ) -> (TransferService, Arc<FakeRepo>, Arc<FakeUsers>) {
        let repo = Arc::new(FakeRepo::default());
        let users = Arc::new(FakeUsers {
            favorite: Mutex::new(Some(1)),
            active,
        });
        let service = TransferService::new(
            repo.clone(),
            Arc::new(FakeDirectory),
            users.clone(),
            Arc::new(FakeCodes),
            Arc::new(move || now),
        );
        (service, repo, users)
    }

    #[tokio::test]
    async fn code_images_require_an_active_viewer_and_an_existing_card() {
        let (service, _, _) = service_at(at(5), true);
        let image = service.code_image(FAN, "CODE0001").await.unwrap();
        assert_eq!(image.image_base64, "img-CODE0001");
        // 非主队、本期结束也可以生成海报：拉票海报只是分享，不是投票
        assert!(service.code_image(RIVAL, "CODE0001").await.is_ok());
        assert_eq!(
            service.code_image(FAN, "bad").await.unwrap_err(),
            TransferError::Invalid
        );
        assert_eq!(
            service.code_image(FAN, "MISSING0").await.unwrap_err(),
            TransferError::NotFound
        );
        assert_eq!(
            service
                .code_image(Uuid::from_u128(9), "CODE0001")
                .await
                .unwrap_err(),
            TransferError::Unauthorized
        );
        let (disabled, _, _) = service_at(at(5), false);
        assert_eq!(
            disabled.code_image(FAN, "CODE0001").await.unwrap_err(),
            TransferError::Unauthorized
        );
        let (closed, _, _) = service_at(at(25), true);
        assert!(closed.code_image(FAN, "CODE0001").await.is_ok());
    }

    #[tokio::test]
    async fn nominate_validates_candidates_per_intent() {
        let (service, repo, _) = service_at(at(5), true);
        for (player, intent, ok) in [
            (10, "out", true),
            (10, "stay", true),
            (10, "in", false),
            (20, "in", true),
            (30, "in", true),
            (30, "out", false),
            (40, "in", false),
        ] {
            let result = service.nominate(FAN, 1, player, intent).await;
            assert_eq!(result.is_ok(), ok, "{player} {intent}: {result:?}");
            if !ok {
                assert_eq!(result.unwrap_err(), TransferError::InvalidCandidate);
            }
        }
        assert_eq!(repo.nominations.lock().unwrap().len(), 4);
        assert!(
            repo.nominations
                .lock()
                .unwrap()
                .iter()
                .all(|c| c.limit == NOMINATION_LIMIT && c.window_id == 7)
        );
    }

    #[tokio::test]
    async fn nominate_reports_existing_cards_and_retries_share_code_collisions() {
        let (service, repo, _) = service_at(at(5), true);
        *repo.taken_once.lock().unwrap() = true;
        let created = service.nominate(FAN, 1, 10, "out").await.unwrap();
        assert!(created.created);
        assert_eq!(repo.nominations.lock().unwrap().len(), 1);
        *repo.existing.lock().unwrap() = true;
        let existing = service.nominate(FAN, 1, 10, "out").await.unwrap();
        assert!(!existing.created);
        assert_eq!(existing.view.card.share_code, "EXIST001");
        *repo.limit_hit.lock().unwrap() = true;
        assert_eq!(
            service.nominate(FAN, 1, 10, "stay").await.unwrap_err(),
            TransferError::NominationLimit
        );
    }

    #[tokio::test]
    async fn only_active_fans_of_the_team_can_write() {
        let (service, _, _) = service_at(at(5), true);
        assert_eq!(
            service.nominate(RIVAL, 1, 10, "out").await.unwrap_err(),
            TransferError::NotFavoriteTeam
        );
        assert_eq!(
            service
                .vote(RIVAL, "CODE0001", Some(VoteValue::Up))
                .await
                .unwrap_err(),
            TransferError::NotFavoriteTeam
        );
        assert_eq!(
            service
                .nominate(Uuid::from_u128(9), 1, 10, "out")
                .await
                .unwrap_err(),
            TransferError::Unauthorized
        );
        let (disabled, _, _) = service_at(at(5), false);
        assert_eq!(
            disabled
                .vote(FAN, "CODE0001", Some(VoteValue::Up))
                .await
                .unwrap_err(),
            TransferError::Unauthorized
        );
        assert_eq!(
            disabled.summary(1, None, Some(FAN)).await.unwrap_err(),
            TransferError::Unauthorized
        );
    }

    #[tokio::test]
    async fn closed_window_rejects_votes_and_nominations() {
        let (service, _, _) = service_at(at(25), true);
        assert_eq!(
            service.nominate(FAN, 1, 10, "out").await.unwrap_err(),
            TransferError::WindowClosed
        );
        assert_eq!(
            service
                .vote(FAN, "CODE0001", Some(VoteValue::Down))
                .await
                .unwrap_err(),
            TransferError::WindowClosed
        );
        let summary = service.summary(1, None, Some(FAN)).await.unwrap();
        assert!(!summary.can_vote);
        assert!(!summary.window.unwrap().open);
    }

    #[tokio::test]
    async fn changing_favorite_team_keeps_votes_but_blocks_new_ones() {
        let (service, repo, users) = service_at(at(5), true);
        service
            .vote(FAN, "CODE0001", Some(VoteValue::Up))
            .await
            .unwrap();
        *users.favorite.lock().unwrap() = Some(2);
        assert_eq!(
            service.vote(FAN, "CODE0001", None).await.unwrap_err(),
            TransferError::NotFavoriteTeam
        );
        assert_eq!(
            repo.votes.lock().unwrap().len(),
            1,
            "the earlier vote stays recorded"
        );
        *repo.card_team.lock().unwrap() = 2;
        assert!(service.vote(FAN, "CODE0001", None).await.is_ok());
    }

    #[tokio::test]
    async fn reads_are_public_and_validate_inputs() {
        let (service, _, _) = service_at(at(5), true);
        let summary = service.summary(1, None, None).await.unwrap();
        assert!(!summary.can_vote);
        assert_eq!(
            summary
                .boards
                .iter()
                .map(|b| (b.intent, b.items.len(), b.total))
                .collect::<Vec<_>>(),
            vec![
                (TransferIntent::Out, 3, 5),
                (TransferIntent::Stay, 3, 5),
                (TransferIntent::In, 3, 5)
            ]
        );
        assert!(service.summary(1, None, Some(FAN)).await.unwrap().can_vote);
        assert_eq!(
            service.summary(99, None, None).await.unwrap_err(),
            TransferError::NotFound
        );
        assert_eq!(
            service.summary(1, Some("nope"), None).await.unwrap_err(),
            TransferError::NotFound
        );
        let board = service
            .board(1, "in", 1, Some("2026-winter"), None)
            .await
            .unwrap();
        assert_eq!(
            (board.page, board.page_size, board.total, board.items.len()),
            (1, BOARD_PAGE_SIZE, 5, 5)
        );
        for (intent, page) in [("bad", 1), ("out", 0), ("out", -1)] {
            assert_eq!(
                service
                    .board(1, intent, page, None, None)
                    .await
                    .unwrap_err(),
                TransferError::Invalid
            );
        }
        assert_eq!(
            service.card("bad", None).await.unwrap_err(),
            TransferError::Invalid
        );
        assert_eq!(
            service.card("MISSING0", None).await.unwrap_err(),
            TransferError::NotFound
        );
        assert!(service.card("CODE0001", Some(FAN)).await.unwrap().can_vote);
    }

    #[tokio::test]
    async fn candidates_group_former_players_before_other_teams() {
        let (service, _, _) = service_at(at(5), true);
        let squad = service.candidates(1, "out").await.unwrap();
        assert_eq!(
            squad
                .groups
                .iter()
                .map(|g| g.key.as_str())
                .collect::<Vec<_>>(),
            vec!["squad"]
        );
        let buy = service.candidates(1, "in").await.unwrap();
        assert_eq!(
            buy.groups
                .iter()
                .map(|g| (g.key.as_str(), g.title.as_str(), g.players.len()))
                .collect::<Vec<_>>(),
            vec![
                ("former", "本队旧将", 1),
                ("team:2", "Team 2", 1),
                ("team:3", "Team 3", 1)
            ]
        );
        assert_eq!(
            service.candidates(1, "x").await.unwrap_err(),
            TransferError::Invalid
        );
    }
}
