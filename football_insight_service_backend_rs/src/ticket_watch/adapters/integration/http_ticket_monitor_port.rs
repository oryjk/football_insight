use std::time::Duration;

use anyhow::{Context, anyhow};
use async_trait::async_trait;
use chrono::{FixedOffset, NaiveDateTime, TimeZone};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ticket_watch::{
    domain::ticket_watch::{
        TicketWatchBlockInterest, TicketWatchCurrentMatchView, TicketWatchInventoryEntry,
        TicketWatchMatchSummary, TicketWatchRegion, TicketWatchTrackedInterest,
    },
    ports::ticket_monitor_port::TicketMonitorPort,
};

const DEFAULT_TICKET_MONITOR_BASE_URL: &str = "http://127.0.0.1:4000";
const TICKET_MONITOR_HTTP_TIMEOUT_SECS: u64 = 5;

pub struct HttpTicketMonitorPort {
    client: Client,
    base_url: String,
}

impl HttpTicketMonitorPort {
    pub fn new(base_url: Option<String>) -> Self {
        let base_url = base_url
            .unwrap_or_else(|| DEFAULT_TICKET_MONITOR_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(TICKET_MONITOR_HTTP_TIMEOUT_SECS))
                .build()
                .expect("failed to build ticket monitor http client"),
            base_url,
        }
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);

        self.client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("failed to call ticket monitor endpoint: {url}"))?
            .error_for_status()
            .with_context(|| format!("ticket monitor endpoint returned non-success: {url}"))?
            .json::<T>()
            .await
            .with_context(|| format!("failed to parse ticket monitor response: {url}"))
    }

    async fn post_json<TRequest: Serialize, TResponse: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &TRequest,
    ) -> anyhow::Result<TResponse> {
        let url = format!("{}{}", self.base_url, path);

        self.client
            .post(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("failed to call ticket monitor endpoint: {url}"))?
            .error_for_status()
            .with_context(|| format!("ticket monitor endpoint returned non-success: {url}"))?
            .json::<TResponse>()
            .await
            .with_context(|| format!("failed to parse ticket monitor response: {url}"))
    }
}

#[async_trait]
impl TicketMonitorPort for HttpTicketMonitorPort {
    async fn fetch_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
        let payload: ExternalCurrentMatchResponse =
            self.get_json("/api/matches/current-standard").await?;

        Ok(TicketWatchCurrentMatchView {
            current_match: payload
                .match_info
                .and_then(|item| map_match_summary(item).ok()),
            group_ticket_active: payload.group_ticket.map(|item| item.show).unwrap_or(false),
            message: payload.message,
        })
    }

    async fn fetch_all_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
        let payload: ExternalMatchListResponse = self.get_json("/api/matches/all").await?;
        let mut matches = payload
            .matches
            .into_iter()
            .filter_map(|item| map_match_summary(item).ok())
            .collect::<Vec<_>>();

        matches.sort_by(|left, right| right.kickoff_at.cmp(&left.kickoff_at));
        Ok(matches)
    }

    async fn fetch_regions(&self) -> anyhow::Result<Vec<TicketWatchRegion>> {
        let payload: ExternalRegionTemplateResponse =
            self.get_json("/api/match/regions-template").await?;

        Ok(payload
            .data
            .data
            .into_iter()
            .flat_map(|group| group.list.into_iter())
            .map(|item| TicketWatchRegion {
                block_name: item.name,
                price: item.price,
                usable_count: item.usable_count,
                estate: item.estate,
            })
            .collect())
    }

    async fn fetch_inventory(
        &self,
        match_id: i64,
        fallback_match_id: Option<i64>,
        since: Option<&str>,
    ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
        let lookup_match_ids = resolve_inventory_lookup_match_ids(match_id, fallback_match_id);
        let mut last_items = Vec::new();

        for lookup_match_id in lookup_match_ids {
            let payload: ExternalInventoryResponse = self
                .get_json(&build_inventory_history_path(lookup_match_id, since)?)
                .await?;

            let mut items = payload
                .data
                .into_iter()
                .map(|item| {
                    Ok(TicketWatchInventoryEntry {
                        block_name: item.block_name,
                        occurrences: item.occurrences,
                        latest_time: normalize_datetime(&item.latest_time)?,
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            items.sort_by(|left, right| left.block_name.cmp(&right.block_name));

            if !items.is_empty() {
                return Ok(items);
            }

            last_items = items;
        }

        Ok(last_items)
    }

    async fn fetch_block_interests(
        &self,
        match_id: i64,
        viewer_user_id: Option<Uuid>,
    ) -> anyhow::Result<Vec<TicketWatchBlockInterest>> {
        let payload: ExternalBlockInterestResponse = self
            .get_json(&build_block_interest_path(match_id, viewer_user_id)?)
            .await?;

        Ok(payload
            .data
            .into_iter()
            .map(|item| TicketWatchBlockInterest {
                block_name: item.block_name,
                interested_user_count: item.interested_user_count,
                viewer_interested: item.viewer_interested,
            })
            .collect())
    }

    async fn fetch_tracked_interests(
        &self,
        match_id: i64,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<TicketWatchTrackedInterest>> {
        let payload: ExternalTrackedInterestResponse = self
            .get_json(&build_tracked_interest_path(match_id, user_id)?)
            .await?;

        Ok(payload
            .data
            .into_iter()
            .map(|item| TicketWatchTrackedInterest {
                block_name: item.block_name,
                started_at: normalize_datetime(&item.started_at).unwrap_or(item.started_at),
                first_inventory_at: item
                    .first_inventory_at
                    .map(|value| normalize_datetime(&value).unwrap_or(value)),
            })
            .collect())
    }

    async fn toggle_block_interest(
        &self,
        match_id: i64,
        user_id: Uuid,
        block_name: &str,
    ) -> anyhow::Result<TicketWatchBlockInterest> {
        let payload: ExternalBlockInterestToggleResponse = self
            .post_json(
                &build_toggle_block_interest_path(match_id),
                &ExternalToggleBlockInterestRequest {
                    block_name: block_name.to_string(),
                    user_id: user_id.to_string(),
                },
            )
            .await?;

        Ok(TicketWatchBlockInterest {
            block_name: payload.data.block_name,
            interested_user_count: payload.data.interested_user_count,
            viewer_interested: payload.data.viewer_interested,
        })
    }
}

fn map_match_summary(item: ExternalMatchDto) -> anyhow::Result<TicketWatchMatchSummary> {
    let external_match_id = item
        .match_id
        .as_deref()
        .ok_or_else(|| anyhow!("match id is missing"))?
        .to_string();
    let schedule_id = item.id;

    let home_team_name = item.home_name.unwrap_or_else(|| "未知主队".to_string());
    let away_team_name = item.away_name.unwrap_or_else(|| "未知客队".to_string());
    let sale_start_source = item.begin_date.as_deref().or(item.begin_time.as_deref());
    let sale_start_at = sale_start_source.map(normalize_datetime).transpose()?;
    let kickoff_source = item
        .match_time
        .as_deref()
        .or(item.begin_date.as_deref())
        .or(item.begin_time.as_deref())
        .ok_or_else(|| anyhow!("kickoff time is missing"))?;
    let kickoff_at = normalize_datetime(kickoff_source)?;
    let (match_date, match_time) = split_kickoff_label(kickoff_source)?;

    Ok(TicketWatchMatchSummary {
        match_id: schedule_id,
        external_match_id,
        round_number: item.round,
        sale_start_at,
        match_date,
        match_time,
        kickoff_at,
        home_team_name,
        away_team_name,
        is_current: item.is_current,
    })
}

fn build_inventory_history_path(match_id: i64, since: Option<&str>) -> anyhow::Result<String> {
    let base = format!("{DEFAULT_TICKET_MONITOR_BASE_URL}/api/match/block-info/{match_id}");
    let mut url = reqwest::Url::parse(&base).context("failed to build inventory url")?;

    if let Some(since) = since.filter(|value| !value.trim().is_empty()) {
        url.query_pairs_mut().append_pair("since", since);
    }

    let mut path = url.path().to_string();
    if let Some(query) = url.query() {
        path.push('?');
        path.push_str(query);
    }

    Ok(path)
}

fn build_tracked_interest_path(match_id: i64, user_id: Uuid) -> anyhow::Result<String> {
    let base =
        format!("{DEFAULT_TICKET_MONITOR_BASE_URL}/api/match/block-interest-tracking/{match_id}");
    let mut url = reqwest::Url::parse(&base).context("failed to build tracked interest url")?;
    url.query_pairs_mut()
        .append_pair("user_id", &user_id.to_string());

    let mut path = url.path().to_string();
    if let Some(query) = url.query() {
        path.push('?');
        path.push_str(query);
    }

    Ok(path)
}

fn resolve_inventory_lookup_match_ids(match_id: i64, fallback_match_id: Option<i64>) -> Vec<i64> {
    let mut ids = vec![match_id];

    if let Some(fallback_match_id) = fallback_match_id.filter(|value| *value != match_id) {
        ids.push(fallback_match_id);
    }

    ids
}

fn build_block_interest_path(
    match_id: i64,
    viewer_user_id: Option<Uuid>,
) -> anyhow::Result<String> {
    let base = format!("{DEFAULT_TICKET_MONITOR_BASE_URL}/api/match/block-interests/{match_id}");
    let mut url = reqwest::Url::parse(&base).context("failed to build block interest url")?;

    if let Some(viewer_user_id) = viewer_user_id {
        url.query_pairs_mut()
            .append_pair("user_id", &viewer_user_id.to_string());
    }

    let mut path = url.path().to_string();
    if let Some(query) = url.query() {
        path.push('?');
        path.push_str(query);
    }

    Ok(path)
}

fn build_toggle_block_interest_path(match_id: i64) -> String {
    format!("/api/match/block-interests/{match_id}/toggle")
}

fn normalize_datetime(value: &str) -> anyhow::Result<String> {
    let naive = parse_naive_datetime(value)?;
    let offset = FixedOffset::east_opt(8 * 3600).expect("china offset");
    let local = offset
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| anyhow!("invalid local datetime: {value}"))?;

    Ok(local.to_rfc3339())
}

fn split_kickoff_label(value: &str) -> anyhow::Result<(String, String)> {
    let naive = parse_naive_datetime(value)?;
    Ok((
        naive.format("%Y-%m-%d").to_string(),
        naive.format("%H:%M").to_string(),
    ))
}

fn parse_naive_datetime(value: &str) -> anyhow::Result<NaiveDateTime> {
    for format in [
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
    ] {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(value, format) {
            return Ok(parsed);
        }
    }

    Err(anyhow!("invalid datetime: {value}"))
}

#[derive(Debug, Deserialize)]
struct ExternalCurrentMatchResponse {
    message: String,
    #[serde(default)]
    match_info: Option<ExternalMatchDto>,
    #[serde(default, rename = "groupTicket")]
    group_ticket: Option<ExternalGroupTicketDto>,
}

#[derive(Debug, Deserialize)]
struct ExternalGroupTicketDto {
    show: bool,
}

#[derive(Debug, Deserialize)]
struct ExternalMatchListResponse {
    matches: Vec<ExternalMatchDto>,
}

#[derive(Debug, Deserialize)]
struct ExternalMatchDto {
    id: i64,
    #[serde(default)]
    away_name: Option<String>,
    #[serde(default)]
    home_name: Option<String>,
    #[serde(default)]
    match_time: Option<String>,
    #[serde(default)]
    begin_date: Option<String>,
    #[serde(default)]
    begin_time: Option<String>,
    is_current: bool,
    #[serde(default)]
    match_id: Option<String>,
    round: i32,
}

#[derive(Debug, Deserialize)]
struct ExternalRegionTemplateResponse {
    data: ExternalRegionTemplateData,
}

#[derive(Debug, Deserialize)]
struct ExternalRegionTemplateData {
    data: Vec<ExternalRegionGroup>,
}

#[derive(Debug, Deserialize)]
struct ExternalRegionGroup {
    list: Vec<ExternalRegionItem>,
}

#[derive(Debug, Deserialize)]
struct ExternalRegionItem {
    usable_count: i32,
    price: String,
    estate: i32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ExternalInventoryResponse {
    data: Vec<ExternalInventoryEntry>,
}

#[derive(Debug, Deserialize)]
struct ExternalInventoryEntry {
    block_name: String,
    occurrences: i32,
    latest_time: String,
}

#[derive(Debug, Deserialize)]
struct ExternalBlockInterestResponse {
    data: Vec<ExternalBlockInterestEntry>,
}

#[derive(Debug, Deserialize)]
struct ExternalBlockInterestToggleResponse {
    data: ExternalBlockInterestEntry,
}

#[derive(Debug, Deserialize)]
struct ExternalTrackedInterestResponse {
    data: Vec<ExternalTrackedInterestEntry>,
}

#[derive(Debug, Deserialize)]
struct ExternalBlockInterestEntry {
    block_name: String,
    interested_user_count: i32,
    viewer_interested: bool,
}

#[derive(Debug, Deserialize)]
struct ExternalTrackedInterestEntry {
    block_name: String,
    started_at: String,
    first_inventory_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExternalToggleBlockInterestRequest {
    block_name: String,
    user_id: String,
}

#[cfg(test)]
mod tests {
    use super::{
        ExternalMatchDto, build_block_interest_path, build_inventory_history_path,
        build_toggle_block_interest_path, build_tracked_interest_path, map_match_summary,
        resolve_inventory_lookup_match_ids,
    };
    use uuid::Uuid;

    #[test]
    fn inventory_history_path_should_append_since_query() {
        let path =
            build_inventory_history_path(572, Some("2026-04-10T14:10:00+08:00")).expect("path");

        assert_eq!(
            path,
            "/api/match/block-info/572?since=2026-04-10T14%3A10%3A00%2B08%3A00"
        );
    }

    #[test]
    fn inventory_lookup_should_fallback_to_external_match_id_for_legacy_history() {
        assert_eq!(
            resolve_inventory_lookup_match_ids(570, Some(72)),
            vec![570, 72]
        );
    }

    #[test]
    fn inventory_lookup_should_not_duplicate_same_match_id() {
        assert_eq!(
            resolve_inventory_lookup_match_ids(572, Some(572)),
            vec![572]
        );
    }

    #[test]
    fn block_interest_path_should_append_viewer_user_query() {
        let user_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("uuid");
        let path = build_block_interest_path(572, Some(user_id)).expect("path");

        assert_eq!(
            path,
            "/api/match/block-interests/572?user_id=11111111-1111-1111-1111-111111111111"
        );
    }

    #[test]
    fn tracked_interest_path_should_append_user_query() {
        let user_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("uuid");
        let path = build_tracked_interest_path(572, user_id).expect("path");

        assert_eq!(
            path,
            "/api/match/block-interest-tracking/572?user_id=11111111-1111-1111-1111-111111111111"
        );
    }

    #[test]
    fn toggle_block_interest_path_should_use_match_scope() {
        assert_eq!(
            build_toggle_block_interest_path(572),
            "/api/match/block-interests/572/toggle"
        );
    }

    #[test]
    fn match_summary_should_preserve_sale_start_at() {
        let summary = map_match_summary(ExternalMatchDto {
            id: 572,
            away_name: Some("青岛西海岸".to_string()),
            home_name: Some("成都蓉城".to_string()),
            match_time: Some("2026-04-03 19:35:00".to_string()),
            begin_date: Some("2026-04-03 14:00:00".to_string()),
            begin_time: None,
            is_current: true,
            match_id: Some("74".to_string()),
            round: 4,
        })
        .expect("summary");

        assert_eq!(
            summary.sale_start_at.as_deref(),
            Some("2026-04-03T14:00:00+08:00")
        );
        assert_eq!(summary.kickoff_at, "2026-04-03T19:35:00+08:00");
    }

    #[test]
    fn match_summary_should_use_begin_time_as_sale_start_at() {
        let summary = map_match_summary(ExternalMatchDto {
            id: 574,
            away_name: Some("浙江俱乐部绿城".to_string()),
            home_name: Some("成都蓉城".to_string()),
            match_time: Some("2026-04-25T19:00:00".to_string()),
            begin_date: None,
            begin_time: Some("2026-04-23T14:00:00".to_string()),
            is_current: false,
            match_id: Some("78".to_string()),
            round: 8,
        })
        .expect("summary");

        assert_eq!(
            summary.sale_start_at.as_deref(),
            Some("2026-04-23T14:00:00+08:00")
        );
        assert_eq!(summary.kickoff_at, "2026-04-25T19:00:00+08:00");
    }
}
