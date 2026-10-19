use super::domain::{LineupReadError, SquadPlayer, SquadTeam};
use async_trait::async_trait;

#[async_trait]
pub trait SquadRepository: Send + Sync {
    async fn find_team(&self, team_id: i64) -> Result<Option<SquadTeam>, LineupReadError>;
    async fn list_squad(&self, team_id: i64) -> Result<Vec<SquadPlayer>, LineupReadError>;
}
