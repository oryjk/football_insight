use crate::admin::{
    domain::admin_payment::*, ports::admin_payment_repository::AdminPaymentRepository,
};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PostgresAdminPaymentRepository {
    pool: PgPool,
}
impl PostgresAdminPaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const FUNNEL_SQL: &str = r#"
WITH scoped AS (
    SELECT *, (occurred_at AT TIME ZONE 'UTC')::date AS day
    FROM f_i_payment_attempt_events
    WHERE occurred_at >= $1 AND occurred_at < $2 AND ($3::text IS NULL OR scenario=$3)
), counts AS (
    SELECT day, scenario, product_ref, event, COUNT(DISTINCT attempt_id)::bigint AS attempt_count,
           COUNT(DISTINCT user_id)::bigint AS user_count, ARRAY_AGG(DISTINCT attempt_id) AS attempts
    FROM scoped GROUP BY day, scenario, product_ref, event
), adjacent(event, previous_event) AS (
    VALUES ('order_create_requested','purchase_tapped'), ('order_created','order_create_requested'),
           ('pay_params_issued','order_created'), ('pay_sheet_invoked','pay_params_issued'),
           ('pay_success_client','pay_sheet_invoked'), ('delivered','pay_success_client')
)
SELECT c.day,c.scenario,c.product_ref,c.event,c.attempt_count,c.user_count,a.previous_event,
       CASE WHEN a.previous_event IS NOT NULL THEN COALESCE(p.attempt_count,0) END AS previous_attempt_count,
       CASE WHEN a.previous_event IS NOT NULL THEN
           (SELECT COUNT(*) FROM UNNEST(c.attempts) attempt WHERE attempt=ANY(COALESCE(p.attempts,ARRAY[]::uuid[]))) END AS converted_attempt_count
FROM counts c LEFT JOIN adjacent a ON a.event=c.event
LEFT JOIN counts p ON p.day=c.day AND p.scenario=c.scenario
    AND p.product_ref IS NOT DISTINCT FROM c.product_ref AND p.event=a.previous_event
ORDER BY c.day,c.scenario,c.product_ref,c.event
"#;

const TERMINALS_SQL: &str = r#"
WITH attempts AS (
    SELECT attempt_id,user_id,scenario,product_ref,(MIN(occurred_at) AT TIME ZONE 'UTC')::date AS day,
           ARRAY_AGG(event ORDER BY occurred_at,id) AS events
    FROM f_i_payment_attempt_events
    WHERE occurred_at >= $1 AND occurred_at < $2 AND ($3::text IS NULL OR scenario=$3)
    GROUP BY attempt_id,user_id,scenario,product_ref
)
SELECT day,scenario,product_ref,events,COUNT(*)::bigint AS attempt_count,ARRAY_AGG(DISTINCT user_id) AS user_ids
FROM attempts GROUP BY day,scenario,product_ref,events ORDER BY day,scenario,product_ref
"#;

const ERRORS_SQL: &str = r#"
WITH errors AS (
    SELECT * FROM f_i_payment_attempt_events
    WHERE occurred_at >= $1 AND occurred_at < $2 AND ($3::text IS NULL OR scenario=$3)
      AND (error_code IS NOT NULL OR event IN ('session_prepare_failed','order_create_failed','pay_params_failed','pay_cancelled','pay_failed','confirm_timeout'))
), grouped AS (
    SELECT event,error_code,COUNT(*)::bigint AS count,
           (ARRAY_AGG(error_msg ORDER BY occurred_at DESC,id DESC))[1] AS error_msg,
           MAX(occurred_at) AS last_occurred_at FROM errors GROUP BY event,error_code
)
SELECT *,SUM(count) OVER ()::bigint AS total_error_count FROM grouped
ORDER BY count DESC,last_occurred_at DESC,event,error_code LIMIT 20
"#;

const TIMELINE_SQL: &str = r#"
SELECT e.* FROM f_i_payment_attempt_events e
WHERE e.order_no=$1 OR EXISTS (
    SELECT 1 FROM f_i_payment_attempt_events linked
    WHERE linked.order_no=$1 AND linked.attempt_id=e.attempt_id AND linked.user_id=e.user_id
)
ORDER BY e.occurred_at,e.id
"#;

const STUCK_SQL: &str = r#"
SELECT p.order_no,p.user_id,p.product_type,p.created_at,history.events,history.last_event,history.last_event_at
FROM f_i_payment_orders p
LEFT JOIN LATERAL (
    SELECT ARRAY_AGG(e.event ORDER BY e.occurred_at,e.id) AS events,
           (ARRAY_AGG(e.event ORDER BY e.occurred_at DESC,e.id DESC))[1] AS last_event,
           MAX(e.occurred_at) AS last_event_at
    FROM f_i_payment_attempt_events e
    WHERE e.user_id=p.user_id AND (e.order_no=p.order_no OR EXISTS (
        SELECT 1 FROM f_i_payment_attempt_events linked
        WHERE linked.order_no=p.order_no AND linked.attempt_id=e.attempt_id AND linked.user_id=p.user_id
    ))
) history ON TRUE
WHERE p.status='pending' AND p.created_at < NOW() - $1::bigint * INTERVAL '1 minute'
ORDER BY p.created_at,p.order_no
"#;

#[async_trait]
impl AdminPaymentRepository for PostgresAdminPaymentRepository {
    async fn funnel(&self, range: &PaymentRange) -> Result<Vec<FunnelCount>, AdminPaymentError> {
        let rows = sqlx::query(FUNNEL_SQL)
            .bind(range.from)
            .bind(range.to)
            .bind(&range.scenario)
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.into_iter()
            .map(|r| {
                Ok(FunnelCount {
                    day: r.try_get("day")?,
                    scenario: r.try_get("scenario")?,
                    product_ref: r.try_get("product_ref")?,
                    event: r.try_get("event")?,
                    attempt_count: r.try_get("attempt_count")?,
                    user_count: r.try_get("user_count")?,
                    previous_event: r.try_get("previous_event")?,
                    previous_attempt_count: r.try_get("previous_attempt_count")?,
                    converted_attempt_count: r.try_get("converted_attempt_count")?,
                })
            })
            .collect::<Result<_, sqlx::Error>>()
            .map_err(map_error)
    }
    async fn terminals(
        &self,
        range: &PaymentRange,
    ) -> Result<Vec<AttemptTerminalCount>, AdminPaymentError> {
        let rows = sqlx::query(TERMINALS_SQL)
            .bind(range.from)
            .bind(range.to)
            .bind(&range.scenario)
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.into_iter()
            .map(|r| {
                Ok(AttemptTerminalCount {
                    day: r.try_get::<NaiveDate, _>("day")?,
                    scenario: r.try_get("scenario")?,
                    product_ref: r.try_get("product_ref")?,
                    events: r.try_get("events")?,
                    attempt_count: r.try_get("attempt_count")?,
                    user_ids: r.try_get::<Vec<Uuid>, _>("user_ids")?,
                })
            })
            .collect::<Result<_, sqlx::Error>>()
            .map_err(map_error)
    }
    async fn errors(
        &self,
        range: &PaymentRange,
    ) -> Result<Vec<PaymentErrorCount>, AdminPaymentError> {
        let rows = sqlx::query(ERRORS_SQL)
            .bind(range.from)
            .bind(range.to)
            .bind(&range.scenario)
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.into_iter()
            .map(|r| {
                Ok(PaymentErrorCount {
                    event: r.try_get("event")?,
                    error_code: r.try_get("error_code")?,
                    error_msg: r.try_get("error_msg")?,
                    count: r.try_get("count")?,
                    total_error_count: r.try_get("total_error_count")?,
                    last_occurred_at: r.try_get::<DateTime<Utc>, _>("last_occurred_at")?,
                })
            })
            .collect::<Result<_, sqlx::Error>>()
            .map_err(map_error)
    }
    async fn timeline(
        &self,
        order_no: &str,
    ) -> Result<Vec<PaymentTimelineEvent>, AdminPaymentError> {
        let rows = sqlx::query(TIMELINE_SQL)
            .bind(order_no)
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.into_iter()
            .map(|r| {
                Ok(PaymentTimelineEvent {
                    id: r.try_get("id")?,
                    attempt_id: r.try_get("attempt_id")?,
                    order_no: r.try_get("order_no")?,
                    user_id: r.try_get("user_id")?,
                    scenario: r.try_get("scenario")?,
                    product_ref: r.try_get("product_ref")?,
                    event: r.try_get("event")?,
                    source: r.try_get("source")?,
                    error_code: r.try_get("error_code")?,
                    error_msg: r.try_get("error_msg")?,
                    detail: r.try_get("detail")?,
                    occurred_at: r.try_get("occurred_at")?,
                })
            })
            .collect::<Result<_, sqlx::Error>>()
            .map_err(map_error)
    }
    async fn stuck(
        &self,
        older_than_minutes: i64,
    ) -> Result<Vec<StuckPaymentOrder>, AdminPaymentError> {
        let rows = sqlx::query(STUCK_SQL)
            .bind(older_than_minutes)
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.into_iter()
            .map(|r| {
                Ok(StuckPaymentOrder {
                    order_no: r.try_get("order_no")?,
                    user_id: r.try_get("user_id")?,
                    product_type: r.try_get("product_type")?,
                    created_at: r.try_get("created_at")?,
                    last_event: r.try_get("last_event")?,
                    last_event_at: r.try_get("last_event_at")?,
                    events: r
                        .try_get::<Option<Vec<String>>, _>("events")?
                        .unwrap_or_default(),
                })
            })
            .collect::<Result<_, sqlx::Error>>()
            .map_err(map_error)
    }
}
fn map_error(error: sqlx::Error) -> AdminPaymentError {
    tracing::warn!(error=%error,"admin payment observability query failed");
    AdminPaymentError::Internal
}
