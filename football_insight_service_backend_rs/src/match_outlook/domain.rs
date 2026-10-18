use chrono::{DateTime, Utc};
use serde::Serialize;
#[derive(Debug, Clone, Serialize)]
pub struct Outlook {
    pub match_id: i64,
    pub home_win: f64,
    pub draw: f64,
    pub away_win: f64,
    pub tau: Option<f64>,
    pub model_version: String,
    pub generated_at: DateTime<Utc>,
    pub kickoff_at: DateTime<Utc>,
    pub frozen: bool,
    pub outcome: Option<i16>,
    pub schedule_changed: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct Record {
    pub season: i32,
    pub matches: i64,
    pub hits: i64,
    pub accuracy: Option<f64>,
    pub rps_model: Option<f64>,
    pub rps_baseline: Option<f64>,
    pub rps_improvement: Option<f64>,
}
#[derive(Debug, Clone, Serialize)]
pub struct OutlookResponse {
    pub status: &'static str,
    pub outlook: Option<Outlook>,
}
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ReadError {
    #[error("参数无效")]
    Invalid,
    #[error("数据暂不可用，请稍后再试")]
    Unavailable,
}
