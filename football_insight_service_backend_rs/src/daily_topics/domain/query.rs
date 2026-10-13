#[derive(Debug, Clone, Copy)]
pub struct DailyTopicsQuery {
    team_id: i64,
    limit: usize,
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum InvalidDailyTopicsQuery {
    #[error("team_id must be a positive integer")]
    TeamId,
    #[error("limit must be between 1 and 5")]
    Limit,
}

impl DailyTopicsQuery {
    pub fn new(team_id: i64, limit: usize) -> Result<Self, InvalidDailyTopicsQuery> {
        if team_id <= 0 {
            return Err(InvalidDailyTopicsQuery::TeamId);
        }
        if !(1..=5).contains(&limit) {
            return Err(InvalidDailyTopicsQuery::Limit);
        }
        Ok(Self { team_id, limit })
    }

    pub fn team_id(self) -> i64 {
        self.team_id
    }

    pub fn limit(self) -> usize {
        self.limit
    }
}
