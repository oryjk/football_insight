pub const RESTOCK_PROFILE_SCHEMA_VERSION: u8 = 2;
pub const RESTOCK_PROFILE_STAT_UNIT: &str = "seat_restock_event";

/// Upstream has no completed snapshot yet; a missing sample can be skipped.
#[derive(Debug, thiserror::Error)]
#[error("offline restock snapshot not ready")]
pub struct RestockProfileNotReady;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestockProfile {
    pub total_events: u64,
    pub appear_events: u64,
    pub up_events: u64,
    pub seats_with_events: u64,
    pub hourly: [u64; 24],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BestWindowsStatus {
    Ok,
    Insufficient,
    Unavailable,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BestWindowHour {
    pub hour: u8,
    pub avg_events: f64,
    pub share: f64,
    pub hit_matches: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BestWindow {
    pub start_hour: u8,
    pub end_hour: u8,
    pub avg_events: f64,
    pub hit_matches: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BestWindowsAggregate {
    pub status: BestWindowsStatus,
    pub sample_size: usize,
    pub generated_at: String,
    pub top_windows: Vec<BestWindow>,
    pub hours: Vec<BestWindowHour>,
}
pub const BEST_WINDOWS_MIN_EVENTS: u64 = 20;
pub const BEST_WINDOWS_MIN_SAMPLES: usize = 3;
pub const BEST_WINDOWS_MAX_SAMPLES: usize = 8;
impl BestWindowsAggregate {
    pub fn unavailable(generated_at: String) -> Self {
        Self {
            status: BestWindowsStatus::Unavailable,
            sample_size: 0,
            generated_at,
            top_windows: vec![],
            hours: vec![],
        }
    }
    pub fn from_profiles(profiles: &[RestockProfile], generated_at: String) -> Self {
        let samples: Vec<_> = profiles
            .iter()
            .filter(|p| p.total_events >= BEST_WINDOWS_MIN_EVENTS)
            .collect();
        let sample_size = samples.len();
        let mut result = Self {
            status: BestWindowsStatus::Insufficient,
            sample_size,
            generated_at,
            top_windows: vec![],
            hours: vec![],
        };
        if sample_size < BEST_WINDOWS_MIN_SAMPLES {
            return result;
        }
        result.status = BestWindowsStatus::Ok;
        let averages: Vec<_> = (0..24)
            .map(|hour| {
                (samples.iter().map(|p| p.hourly[hour] as f64).sum::<f64>() / sample_size as f64
                    * 10.0)
                    .round()
                    / 10.0
            })
            .collect();
        let sum = averages.iter().sum::<f64>();
        result.hours = averages
            .into_iter()
            .enumerate()
            .map(|(hour, avg_events)| BestWindowHour {
                hour: hour as u8,
                avg_events,
                hit_matches: samples.iter().filter(|p| p.hourly[hour] > 0).count(),
                share: if sum > 0.0 {
                    (avg_events / sum * 1000.0).round() / 1000.0
                } else {
                    0.0
                },
            })
            .collect();
        let mut ranked: Vec<_> = result.hours.iter().collect();
        ranked.sort_by(|left, right| {
            right
                .avg_events
                .total_cmp(&left.avg_events)
                .then(left.hour.cmp(&right.hour))
        });
        result.top_windows = ranked
            .into_iter()
            .take(3)
            .map(|h| BestWindow {
                start_hour: h.hour,
                end_hour: h.hour + 1,
                avg_events: h.avg_events,
                hit_matches: h.hit_matches,
            })
            .collect();
        result
    }
    pub fn for_access(&self, full: bool) -> Self {
        let mut result = self.clone();
        if !full {
            result.hours.clear();
            result.top_windows.truncate(1);
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn profile(hour: u8, events: u64) -> RestockProfile {
        let mut hourly = [0; 24];
        hourly[hour as usize] = events;
        RestockProfile {
            total_events: events,
            appear_events: events,
            up_events: 0,
            seats_with_events: 1,
            hourly,
        }
    }
    #[test]
    fn best_windows_drops_under_twenty_events_and_requires_three_samples() {
        for (events, samples, status) in [
            (19, 8, BestWindowsStatus::Insufficient),
            (20, 2, BestWindowsStatus::Insufficient),
            (20, 3, BestWindowsStatus::Ok),
        ] {
            let result = BestWindowsAggregate::from_profiles(
                &vec![profile(12, events); samples],
                "now".into(),
            );
            assert_eq!(result.status, status);
            assert_eq!(result.sample_size, if events < 20 { 0 } else { samples });
            assert_eq!(
                result.hours.len(),
                if status == BestWindowsStatus::Ok {
                    24
                } else {
                    0
                }
            );
        }
    }
    #[test]
    fn best_windows_averages_rounds_shares_and_keeps_stable_separate_hours() {
        let p = vec![profile(12, 21), profile(13, 21), profile(14, 21)];
        let result = BestWindowsAggregate::from_profiles(&p, "now".into());
        assert_eq!(result.hours[12].avg_events, 7.0);
        assert_eq!(result.hours[12].hit_matches, 1);
        assert_eq!(result.hours[12].share, 0.333);
        assert_eq!(
            result
                .top_windows
                .iter()
                .map(|w| (w.start_hour, w.end_hour))
                .collect::<Vec<_>>(),
            vec![(12, 13), (13, 14), (14, 15)]
        );
        let uneven = BestWindowsAggregate::from_profiles(
            &[profile(12, 20), profile(12, 20), profile(23, 20)],
            "now".into(),
        );
        assert_eq!(uneven.hours[12].avg_events, 13.3);
        assert_eq!(uneven.hours[23].avg_events, 6.7);
        assert_eq!(uneven.top_windows[1].end_hour, 24);
    }
    #[test]
    fn best_windows_preview_clips_each_response_without_mutating_full_cache() {
        let result = BestWindowsAggregate::from_profiles(&vec![profile(12, 20); 3], "now".into());
        let preview = result.for_access(false);
        assert!(preview.hours.is_empty());
        assert_eq!(preview.top_windows.len(), 1);
        assert_eq!(preview.sample_size, 3);
        assert_eq!(result.for_access(true).hours.len(), 24);
        assert_eq!(result.top_windows.len(), 3);
    }
}
