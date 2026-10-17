use uuid::Uuid;

pub const MAX_SUPPORT_BASE: i64 = 100_000;

#[derive(Debug, thiserror::Error)]
pub enum AdminSupportBasesError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("match not found")]
    NotFound,
    #[error("support base query failed")]
    Internal,
}

#[derive(Debug, Clone)]
pub struct SupportBaseMatch {
    pub match_id: i64,
    pub round_number: i32,
    pub match_date: String,
    pub match_time: String,
    pub status: String,
    pub home_team_id: i64,
    pub home_team_name: String,
    pub away_team_id: i64,
    pub away_team_name: String,
    pub home_real_count: i64,
    pub away_real_count: i64,
    pub home_base: i64,
    pub away_base: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct SupportBaseInput {
    pub home_base: i64,
    pub away_base: i64,
}

impl SupportBaseInput {
    pub fn new(home_base: i64, away_base: i64) -> Result<Self, AdminSupportBasesError> {
        for value in [home_base, away_base] {
            if !(0..=MAX_SUPPORT_BASE).contains(&value) {
                return Err(AdminSupportBasesError::InvalidInput(format!(
                    "base count must be between 0 and {MAX_SUPPORT_BASE}"
                )));
            }
        }
        Ok(Self { home_base, away_base })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SupportBaseAudit {
    pub admin_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_must_be_within_bounds() {
        assert!(SupportBaseInput::new(0, 100_000).is_ok());
        assert!(SupportBaseInput::new(-1, 0).is_err());
        assert!(SupportBaseInput::new(0, 100_001).is_err());
    }
}
