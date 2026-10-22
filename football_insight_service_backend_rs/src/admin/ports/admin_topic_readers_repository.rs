use async_trait::async_trait;

use crate::admin::domain::admin_topic_readers::{
    AdminTopicReadersError, TopicReader, TopicReaderTopic,
};

#[async_trait]
pub trait AdminTopicReadersRepository: Send + Sync {
    async fn recent_topics(
        &self,
        limit: i64,
    ) -> Result<Vec<TopicReaderTopic>, AdminTopicReadersError>;
    async fn readers(
        &self,
        team_id: i64,
        topic_id: &str,
    ) -> Result<Vec<TopicReader>, AdminTopicReadersError>;
}
