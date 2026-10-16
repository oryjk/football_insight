use crate::ticket_watch::domain::best_watch_windows::RestockProfile;
use anyhow::{bail, Context};
use chrono::DateTime;
use serde::Deserialize;
#[derive(Deserialize)]
pub(super) struct RestockResponse {
    pub success: bool,
    pub data: RestockDto,
}
#[derive(Deserialize)]
pub(super) struct RestockDto {
    pub schema_version: u8,
    pub stat_unit: String,
    pub match_id: i64,
    pub since: String,
    pub total_events: u64,
    pub appear_events: u64,
    pub up_events: u64,
    pub seats_with_events: u64,
    pub hourly: Vec<RestockHourDto>,
}
#[derive(Deserialize)]
pub(super) struct RestockHourDto {
    pub hour: u8,
    pub events: u64,
}
impl RestockResponse {
    pub fn into_profile(self, match_id: i64, since: &str) -> anyhow::Result<RestockProfile> {
        let dto = self.data;
        let expected = DateTime::parse_from_rfc3339(since).context("invalid restock since")?;
        let actual = DateTime::parse_from_rfc3339(&dto.since).context("invalid snapshot since")?;
        let sum = dto
            .hourly
            .iter()
            .try_fold(0u64, |sum, item| sum.checked_add(item.events));
        if !self.success
            || dto.schema_version
                != crate::ticket_watch::domain::best_watch_windows::RESTOCK_PROFILE_SCHEMA_VERSION
            || dto.stat_unit
                != crate::ticket_watch::domain::best_watch_windows::RESTOCK_PROFILE_STAT_UNIT
            || dto.match_id != match_id
            || expected != actual
            || dto.hourly.len() != 24
            || dto
                .hourly
                .iter()
                .enumerate()
                .any(|(hour, item)| usize::from(item.hour) != hour)
            || sum != Some(dto.total_events)
            || dto.appear_events.checked_add(dto.up_events) != Some(dto.total_events)
            || dto.seats_with_events > dto.total_events
        {
            bail!("invalid restock snapshot");
        }
        let mut hourly = [0; 24];
        for item in dto.hourly {
            hourly[usize::from(item.hour)] = item.events;
        }
        Ok(RestockProfile {
            total_events: dto.total_events,
            appear_events: dto.appear_events,
            up_events: dto.up_events,
            seats_with_events: dto.seats_with_events,
            hourly,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn payload() -> serde_json::Value {
        serde_json::json!({"success":true,"data":{"schema_version":2,"stat_unit":"seat_restock_event","match_id":583,"since":"2026-10-08T14:10:00+08:00","total_events":24,"appear_events":20,"up_events":4,"seats_with_events":2,"hourly":(0..24).map(|hour|serde_json::json!({"hour":hour,"events":1})).collect::<Vec<_>>()}})
    }
    fn parse(value: serde_json::Value) -> anyhow::Result<RestockProfile> {
        serde_json::from_value::<RestockResponse>(value)?
            .into_profile(583, "2026-10-08T14:10:00+08:00")
    }
    #[test]
    fn best_windows_snapshot_validates_every_hour_totals_identity_and_since() {
        assert_eq!(parse(payload()).unwrap().hourly, [1; 24]);
        for kind in 0..9 {
            let mut p = payload();
            match kind {
                0 => p["success"] = false.into(),
                1 => p["data"]["match_id"] = 72.into(),
                2 => p["data"]["since"] = "2026-10-08T14:11:00+08:00".into(),
                3 => {
                    p["data"]["hourly"].as_array_mut().unwrap().pop();
                }
                4 => p["data"]["hourly"][23]["hour"] = 0.into(),
                5 => p["data"]["total_events"] = 25.into(),
                6 => p["data"]["up_events"] = 5.into(),
                7 => p["data"]["schema_version"] = 1.into(),
                _ => p["data"]["stat_unit"] = "price_group_event".into(),
            };
            assert!(parse(p).is_err(), "kind={kind}");
        }
    }
}
