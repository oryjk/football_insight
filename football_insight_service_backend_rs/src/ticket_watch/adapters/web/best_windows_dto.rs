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
    /// null in preview: the hour itself is the paid content.
    pub start_hour: Option<u8>,
    pub end_hour: Option<u8>,
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
                    start_hour: full.then_some(h.start_hour),
                    end_hour: full.then_some(h.end_hour),
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ticket_watch::domain::best_watch_windows::{BestWindow, BestWindowsAggregate};
    fn aggregate() -> BestWindowsAggregate {
        let mut value = BestWindowsAggregate::unavailable("now".into());
        value.top_windows = vec![BestWindow {
            start_hour: 12,
            end_hour: 13,
            avg_events: 38.4,
            hit_matches: 7,
        }];
        value
    }
    #[test]
    fn preview_json_never_contains_the_hour_of_the_window() {
        let preview = serde_json::to_value(BestWindowsDto::from((false, aggregate()))).unwrap();
        let window = &preview["top_windows"][0];
        assert!(window["start_hour"].is_null() && window["end_hour"].is_null());
        assert_eq!(window["hit_matches"], 7);
        let full = serde_json::to_value(BestWindowsDto::from((true, aggregate()))).unwrap();
        assert_eq!(full["top_windows"][0]["start_hour"], 12);
        assert_eq!(full["top_windows"][0]["end_hour"], 13);
    }
}
