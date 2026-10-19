use super::domain::{Lineup, LineupReadError, LineupSlot, SquadPlayer, SquadTeam};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait SquadRepository: Send + Sync {
    async fn find_team(&self, team_id: i64) -> Result<Option<SquadTeam>, LineupReadError>;
    async fn list_squad(&self, team_id: i64) -> Result<Vec<SquadPlayer>, LineupReadError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    ShareCodeTaken,
    Unavailable,
}

#[async_trait]
pub trait LineupRepository: Send + Sync {
    async fn find(&self, user_id: Uuid, team_id: i64) -> Result<Option<Lineup>, StoreError>;
    /// 返回 `player_ids` 里属于该队名单（现役或前队员）的那些 id。
    async fn squad_members(&self, team_id: i64, player_ids: &[i64])
    -> Result<Vec<i64>, StoreError>;
    /// 首次保存插入并写入 `share_code`；已存在则覆盖 formation 与 slots，`share_code` 保持不变。
    async fn upsert(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: &[LineupSlot],
        share_code: &str,
    ) -> Result<Lineup, StoreError>;
}
