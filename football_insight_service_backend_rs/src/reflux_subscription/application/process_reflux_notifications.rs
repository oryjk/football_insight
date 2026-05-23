use std::sync::Arc;

use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, TimeZone, Utc};

use crate::{
    reflux_subscription::{
        domain::subscription::normalize_team_code,
        ports::reflux_subscription_repository::{
            CreateNotificationJobInput, RefluxSubscriptionRepository,
        },
    },
    ticket_watch::{
        domain::ticket_watch::{TicketWatchInventoryEntry, TicketWatchMatchSummary},
        ports::ticket_monitor_port::TicketMonitorPort,
    },
};

pub struct ProcessRefluxNotificationsUseCase {
    repository: Arc<dyn RefluxSubscriptionRepository>,
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

impl ProcessRefluxNotificationsUseCase {
    pub fn new(
        repository: Arc<dyn RefluxSubscriptionRepository>,
        ticket_monitor_port: Arc<dyn TicketMonitorPort>,
    ) -> Self {
        Self {
            repository,
            ticket_monitor_port,
        }
    }

    pub async fn execute(&self) -> anyhow::Result<usize> {
        let mut created_jobs = 0usize;

        if let Some(match_info) = self
            .ticket_monitor_port
            .fetch_current_match()
            .await?
            .current_match
        {
            created_jobs += self
                .process_match("chengdurongcheng", &match_info, TeamRefluxSource::Chengdu)
                .await?;
        }

        if let Some(match_info) = self
            .ticket_monitor_port
            .fetch_yukun_current_match()
            .await?
            .current_match
        {
            created_jobs += self
                .process_match("yunnanyukun", &match_info, TeamRefluxSource::Yukun)
                .await?;
        }

        Ok(created_jobs)
    }

    async fn process_match(
        &self,
        team_code: &str,
        match_info: &TicketWatchMatchSummary,
        source: TeamRefluxSource,
    ) -> anyhow::Result<usize> {
        let sale_start_at =
            parse_ticket_datetime(match_info.sale_start_at.as_deref().ok_or_else(|| {
                anyhow::anyhow!("match {} missing sale_start_at", match_info.match_id)
            })?)?;
        let min_start_at = sale_start_at + Duration::minutes(10);
        let cursor = self
            .repository
            .get_cursor(team_code, match_info.match_id)
            .await?;
        let since = cursor.map_or(min_start_at, |cursor| cursor.max(min_start_at));
        let now = Utc::now();

        if since >= now {
            return Ok(0);
        }

        let since_text = since.to_rfc3339();
        let inventory = match source {
            TeamRefluxSource::Chengdu => {
                self.ticket_monitor_port
                    .fetch_inventory(
                        match_info.match_id,
                        resolve_fallback_match_id(match_info),
                        Some(&since_text),
                    )
                    .await?
            }
            TeamRefluxSource::Yukun => {
                self.ticket_monitor_port
                    .fetch_yukun_reflux(match_info.match_id, Some(&since_text))
                    .await?
                    .0
            }
        };

        let reflux_items = inventory
            .into_iter()
            .filter(|item| item.occurrences > 0)
            .collect::<Vec<_>>();
        if reflux_items.is_empty() {
            self.repository
                .update_cursor(team_code, match_info.match_id, now)
                .await?;
            return Ok(0);
        }

        let subscribers = self
            .repository
            .list_email_subscribers_for_match(
                team_code,
                infer_match_season(match_info).unwrap_or(2026),
                match_info.match_id,
            )
            .await?;
        if subscribers.is_empty() {
            self.repository
                .update_cursor(team_code, match_info.match_id, now)
                .await?;
            return Ok(0);
        }

        let subject = build_reflux_alert_subject(match_info);
        let body_html = build_reflux_alert_body_html(match_info, &reflux_items, since, now);
        for subscriber in subscribers {
            self.repository
                .create_notification_job(CreateNotificationJobInput {
                    user_id: subscriber.user_id,
                    target_id: subscriber.target.id,
                    team_code: normalize_team_code(team_code),
                    match_id: Some(match_info.match_id),
                    subject: subject.clone(),
                    body_html: body_html.clone(),
                    payload_json: serde_json::json!({
                        "kind": "reflux_alert",
                        "team_code": team_code,
                        "match_id": match_info.match_id,
                        "since": since.to_rfc3339(),
                        "until": now.to_rfc3339(),
                        "item_count": reflux_items.len(),
                    }),
                })
                .await?;
        }

        self.repository
            .update_cursor(team_code, match_info.match_id, now)
            .await?;
        Ok(reflux_items.len())
    }
}

#[derive(Debug, Clone, Copy)]
enum TeamRefluxSource {
    Chengdu,
    Yukun,
}

pub fn resolve_reflux_query_start(
    sale_start_at: DateTime<Utc>,
    cursor: Option<DateTime<Utc>>,
) -> DateTime<Utc> {
    let min_start_at = sale_start_at + Duration::minutes(10);
    cursor.map_or(min_start_at, |cursor| cursor.max(min_start_at))
}

fn resolve_fallback_match_id(match_info: &TicketWatchMatchSummary) -> Option<i64> {
    match_info
        .external_match_id
        .trim()
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0 && *value != match_info.match_id)
}

fn infer_match_season(match_info: &TicketWatchMatchSummary) -> Option<i32> {
    match_info
        .kickoff_at
        .get(0..4)
        .and_then(|value| value.parse::<i32>().ok())
        .or_else(|| {
            match_info
                .match_date
                .get(0..4)
                .and_then(|value| value.parse::<i32>().ok())
        })
}

fn parse_ticket_datetime(value: &str) -> anyhow::Result<DateTime<Utc>> {
    let value = value.trim();
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Ok(parsed.with_timezone(&Utc));
    }

    let shanghai = FixedOffset::east_opt(8 * 3600).expect("valid fixed offset");
    for format in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            return shanghai
                .from_local_datetime(&naive)
                .single()
                .map(|value| value.with_timezone(&Utc))
                .ok_or_else(|| anyhow::anyhow!("invalid local datetime: {value}"));
        }
    }

    anyhow::bail!("invalid datetime: {value}")
}

fn build_reflux_alert_subject(match_info: &TicketWatchMatchSummary) -> String {
    format!(
        "[回流提醒] {} vs {} 出现余票回流",
        match_info.home_team_name, match_info.away_team_name
    )
}

fn build_reflux_alert_body_html(
    match_info: &TicketWatchMatchSummary,
    items: &[TicketWatchInventoryEntry],
    since: DateTime<Utc>,
    until: DateTime<Utc>,
) -> String {
    let rows = items
        .iter()
        .map(|item| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&item.block_name),
                item.occurrences,
                html_escape(&item.latest_time)
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        "<p>{} vs {}</p><p>比赛时间：{} {}</p><p>聚合窗口：{} 至 {}</p><table><thead><tr><th>区域</th><th>次数</th><th>最新发现</th></tr></thead><tbody>{}</tbody></table><p>数据来自实时监控，余票可能随时变化。</p>",
        html_escape(&match_info.home_team_name),
        html_escape(&match_info.away_team_name),
        html_escape(&match_info.match_date),
        html_escape(&match_info.match_time),
        since.to_rfc3339(),
        until.to_rfc3339(),
        rows
    )
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::resolve_reflux_query_start;

    #[test]
    fn query_start_uses_sale_start_plus_ten_minutes_without_cursor() {
        let sale_start_at = Utc.with_ymd_and_hms(2026, 5, 18, 10, 0, 0).unwrap();

        assert_eq!(
            resolve_reflux_query_start(sale_start_at, None),
            Utc.with_ymd_and_hms(2026, 5, 18, 10, 10, 0).unwrap()
        );
    }

    #[test]
    fn query_start_uses_later_database_cursor() {
        let sale_start_at = Utc.with_ymd_and_hms(2026, 5, 18, 10, 0, 0).unwrap();
        let cursor = Utc.with_ymd_and_hms(2026, 5, 18, 10, 30, 0).unwrap();

        assert_eq!(
            resolve_reflux_query_start(sale_start_at, Some(cursor)),
            cursor
        );
    }

    #[test]
    fn query_start_never_goes_before_sale_start_plus_ten_minutes() {
        let sale_start_at = Utc.with_ymd_and_hms(2026, 5, 18, 10, 0, 0).unwrap();
        let cursor = Utc.with_ymd_and_hms(2026, 5, 18, 10, 5, 0).unwrap();

        assert_eq!(
            resolve_reflux_query_start(sale_start_at, Some(cursor)),
            Utc.with_ymd_and_hms(2026, 5, 18, 10, 10, 0).unwrap()
        );
    }
}
