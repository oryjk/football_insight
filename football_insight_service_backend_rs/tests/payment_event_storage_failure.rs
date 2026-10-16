use football_insight_service_backend_rs::payment::{
    adapters::persistence::postgres_payment_event_recorder::PostgresPaymentEventRecorder,
    domain::payment_event::{NewPaymentEvent, PaymentEventKind},
    ports::payment_event_recorder::PaymentEventRecorder,
};

#[tokio::test]
async fn closed_database_is_swallowed_without_any_connection_or_write() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@127.0.0.1:1/unused")
        .unwrap();
    pool.close().await;
    let recorder = PostgresPaymentEventRecorder::new(pool);
    recorder
        .record(NewPaymentEvent {
            attempt_id: uuid::Uuid::new_v4(),
            order_no: None,
            user_id: uuid::Uuid::new_v4(),
            scenario: "membership".into(),
            product_ref: Some("V7".into()),
            event: PaymentEventKind::PurchaseTapped,
            error_code: None,
            error_msg: None,
            detail: serde_json::json!({}),
            occurred_at: chrono::Utc::now(),
        })
        .await;
    assert_eq!(recorder.attempt_for_order("unused").await, None);
}
