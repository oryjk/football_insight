use crate::payment::{
    domain::payment_event::{NewPaymentEvent, sanitize_detail, sanitize_message},
    ports::payment_event_recorder::PaymentEventRecorder,
};
use async_trait::async_trait;
use sqlx::PgPool;

pub struct PostgresPaymentEventRecorder {
    pool: PgPool,
}
impl PostgresPaymentEventRecorder {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentEventRecorder for PostgresPaymentEventRecorder {
    async fn record(&self, event: NewPaymentEvent) {
        let result=sqlx::query("INSERT INTO f_i_payment_attempt_events (attempt_id,order_no,user_id,scenario,product_ref,event,source,error_code,error_msg,detail,occurred_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)")
            .bind(event.attempt_id).bind(event.order_no).bind(event.user_id).bind(event.scenario).bind(event.product_ref)
            .bind(event.event.as_str()).bind(event.event.source()).bind(event.error_code)
            .bind(event.error_msg.map(|v|sanitize_message(&v))).bind(sanitize_detail(&event.detail)).bind(event.occurred_at)
            .execute(&self.pool).await;
        if let Err(error) = result {
            tracing::warn!(%error,"payment event insert failed");
        }
    }
    async fn attempt_for_order(&self, order_no: &str) -> Option<uuid::Uuid> {
        match sqlx::query_scalar("SELECT attempt_id FROM f_i_payment_attempt_events WHERE order_no=$1 ORDER BY (source='server') DESC,occurred_at,id LIMIT 1").bind(order_no).fetch_optional(&self.pool).await {
            Ok(id)=>id,Err(error)=>{tracing::warn!(%error,"payment attempt lookup failed");None}
        }
    }
}
