use super::domain::{
    CandidatePlayer, NominationCard, TransferError, TransferIntent, TransferTeam, TransferWindow,
    VoteTally, VoteValue,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfile {
    pub active: bool,
    pub favorite_team_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardContext {
    pub card: NominationCard,
    pub team: TransferTeam,
    pub window: TransferWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominateCommand {
    pub window_id: i64,
    pub team_id: i64,
    pub player_id: i64,
    pub intent: TransferIntent,
    pub user_id: Uuid,
    pub share_code: String,
    /// 新建卡的上限；卡已存在时不检查
    pub limit: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominateOutcome {
    pub share_code: String,
    pub created: bool,
}

#[async_trait]
pub trait TransferRepository: Send + Sync {
    /// 开放中的一期；没有则取最近已结束的一期；都没有为 None。
    async fn current_window(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Option<TransferWindow>, TransferError>;
    async fn window_by_code(&self, code: &str) -> Result<Option<TransferWindow>, TransferError>;
    /// 只返回有票的卡（按名次），以及有票卡的总数。
    async fn board(
        &self,
        window_id: i64,
        team_id: i64,
        intent: TransferIntent,
        offset: i64,
        limit: i64,
        viewer: Option<Uuid>,
    ) -> Result<(Vec<NominationCard>, i64), TransferError>;
    /// 无论有没有票都能按分享码读到。
    async fn card(
        &self,
        share_code: &str,
        viewer: Option<Uuid>,
    ) -> Result<Option<CardContext>, TransferError>;
    /// 事务内：锁提名人 → 卡已存在则直接加票，否则检查上限并建卡 → 记 +1。
    async fn nominate(&self, command: NominateCommand) -> Result<NominateOutcome, TransferError>;
    /// `None` 表示取消投票。
    async fn set_vote(
        &self,
        share_code: &str,
        user_id: Uuid,
        value: Option<VoteValue>,
    ) -> Result<VoteTally, TransferError>;
}

#[async_trait]
pub trait TransferCandidateDirectory: Send + Sync {
    async fn find_team(&self, team_id: i64) -> Result<Option<TransferTeam>, TransferError>;
    /// 该球员在名单表里的全部 (team_id, active)。
    async fn player_rows(&self, player_id: i64) -> Result<Vec<(i64, bool)>, TransferError>;
    /// 本队现役
    async fn team_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError>;
    /// 本队旧将（current_team 为其现役所在队，可能为空）
    async fn former_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError>;
    /// 其他队现役，排除本队旧将（他们在旧将组里）
    async fn other_active_players(
        &self,
        team_id: i64,
    ) -> Result<Vec<CandidatePlayer>, TransferError>;
}

#[async_trait]
pub trait TransferUserAccess: Send + Sync {
    async fn profile(&self, user_id: Uuid) -> Result<Option<UserProfile>, TransferError>;
}
