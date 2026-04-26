use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use football_insight_service_backend_rs::activity::{
    application::record_page_activity::{RecordPageActivityInput, RecordPageActivityUseCase},
    ports::user_activity_repository::UserActivityRepository,
};
use uuid::Uuid;

#[derive(Default)]
struct FakeUserActivityRepository {
    calls: Mutex<Vec<(Uuid, String)>>,
}

#[async_trait]
impl UserActivityRepository for FakeUserActivityRepository {
    async fn record_page_activity(&self, user_id: Uuid, page_key: &str) -> anyhow::Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push((user_id, page_key.to_string()));
        Ok(())
    }
}

#[tokio::test]
async fn records_allowed_page_activity_for_user() {
    let repository = Arc::new(FakeUserActivityRepository::default());
    let use_case = RecordPageActivityUseCase::new(repository.clone());
    let user_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();

    use_case
        .execute(RecordPageActivityInput {
            user_id,
            page_key: "ticket_watch".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(
        repository.calls.lock().unwrap().as_slice(),
        [(user_id, "ticket_watch".to_string())]
    );
}

#[tokio::test]
async fn rejects_unknown_page_key() {
    let repository = Arc::new(FakeUserActivityRepository::default());
    let use_case = RecordPageActivityUseCase::new(repository.clone());

    let error = use_case
        .execute(RecordPageActivityInput {
            user_id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
            page_key: "button_click".to_string(),
        })
        .await
        .unwrap_err();

    assert!(error.to_string().contains("unsupported activity page key"));
    assert!(repository.calls.lock().unwrap().is_empty());
}
