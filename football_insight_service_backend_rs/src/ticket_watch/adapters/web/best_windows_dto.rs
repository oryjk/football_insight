use crate::ticket_watch::domain::best_watch_windows::{BestWindowsAggregate, BestWindowsStatus};
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct BestWindowsDto {
    pub status: &'static str,
    pub access: &'static str,
    pub sample_size: usize,
    pub generated_at: String,
    pub top_windows: Vec<BestWindowDto>,
    pub hours: Vec<BestHourDto>,
}
#[derive(Debug, Serialize)]
pub struct BestWindowDto {
    pub start_hour: u8,
    pub end_hour: u8,
    pub avg_events: f64,
    pub hit_matches: usize,
}
#[derive(Debug, Serialize)]
pub struct BestHourDto {
    pub hour: u8,
    pub avg_events: f64,
    pub share: f64,
    pub hit_matches: usize,
}
impl From<(bool, BestWindowsAggregate)> for BestWindowsDto {
    fn from((full, value): (bool, BestWindowsAggregate)) -> Self {
        Self {
            status: match value.status {
                BestWindowsStatus::Ok => "ok",
                BestWindowsStatus::Insufficient => "insufficient",
                BestWindowsStatus::Unavailable => "unavailable",
            },
            access: if full { "full" } else { "preview" },
            sample_size: value.sample_size,
            generated_at: value.generated_at,
            top_windows: value
                .top_windows
                .into_iter()
                .map(|h| BestWindowDto {
                    start_hour: h.start_hour,
                    end_hour: h.end_hour,
                    avg_events: h.avg_events,
                    hit_matches: h.hit_matches,
                })
                .collect(),
            hours: value
                .hours
                .into_iter()
                .map(|h| BestHourDto {
                    hour: h.hour,
                    avg_events: h.avg_events,
                    share: h.share,
                    hit_matches: h.hit_matches,
                })
                .collect(),
        }
    }
}
