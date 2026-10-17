use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AdminTopicReadersError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("topic readers query failed")]
    Internal,
}

#[derive(Debug, Clone)]
pub struct TopicReaderTopic {
    pub team_id: i64,
    pub topic_id: String,
    pub title: String,
    pub edition_date: NaiveDate,
    pub reader_count: i64,
}

/// `first_read_at` is the first time this user opened the topic, not the latest.
#[derive(Debug, Clone)]
pub struct TopicReader {
    pub user_id: Uuid,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub first_read_at: chrono::DateTime<chrono::Utc>,
}
