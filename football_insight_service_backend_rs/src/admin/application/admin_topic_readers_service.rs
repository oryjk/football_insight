use std::sync::Arc;

use crate::admin::{
    domain::admin_topic_readers::{AdminTopicReadersError, TopicReader, TopicReaderTopic},
    ports::admin_topic_readers_repository::AdminTopicReadersRepository,
};

const TOPIC_LIMIT: i64 = 60;

pub struct AdminTopicReadersService {
    repository: Arc<dyn AdminTopicReadersRepository>,
}

impl AdminTopicReadersService {
    pub fn new(repository: Arc<dyn AdminTopicReadersRepository>) -> Self {
        Self { repository }
    }

    pub async fn recent_topics(&self) -> Result<Vec<TopicReaderTopic>, AdminTopicReadersError> {
        self.repository.recent_topics(TOPIC_LIMIT).await
    }

    pub async fn readers(
        &self,
        team_id: i64,
        topic_id: &str,
    ) -> Result<Vec<TopicReader>, AdminTopicReadersError> {
        if team_id <= 0 || topic_id.trim().is_empty() || topic_id.len() > 200 {
            return Err(AdminTopicReadersError::InvalidInput(
                "invalid team_id or topic_id".into(),
            ));
        }
        self.repository.readers(team_id, topic_id).await
    }
}
