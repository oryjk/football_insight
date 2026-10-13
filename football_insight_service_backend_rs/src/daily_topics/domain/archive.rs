use chrono::{DateTime, FixedOffset};

use super::edition::DailyTopic;

#[derive(Debug, Clone)]
pub struct TopicPosition {
    pub published_at: DateTime<FixedOffset>,
    pub id: String,
}

impl TopicPosition {
    pub fn follows(&self, topic: &DailyTopic) -> bool {
        topic.published_at < self.published_at
            || (topic.published_at == self.published_at && topic.id > self.id)
    }
}

#[derive(Debug, Clone)]
pub struct TopicHistoryQuery {
    team_id: i64,
    limit: usize,
    after: Option<TopicPosition>,
}

#[derive(Debug, Clone)]
pub struct TopicDetailQuery {
    team_id: i64,
    id: String,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid topic archive parameters")]
pub struct InvalidTopicArchiveQuery;

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

impl TopicHistoryQuery {
    pub fn new(
        team_id: i64,
        limit: usize,
        after: Option<TopicPosition>,
    ) -> Result<Self, InvalidTopicArchiveQuery> {
        if team_id <= 0
            || !(1..=50).contains(&limit)
            || after.as_ref().is_some_and(|cursor| !valid_id(&cursor.id))
        {
            return Err(InvalidTopicArchiveQuery);
        }
        Ok(Self {
            team_id,
            limit,
            after,
        })
    }

    pub fn team_id(&self) -> i64 {
        self.team_id
    }
    pub fn limit(&self) -> usize {
        self.limit
    }
    pub fn after(&self) -> Option<&TopicPosition> {
        self.after.as_ref()
    }
}

impl TopicDetailQuery {
    pub fn new(team_id: i64, id: String) -> Result<Self, InvalidTopicArchiveQuery> {
        if team_id <= 0 || !valid_id(&id) {
            return Err(InvalidTopicArchiveQuery);
        }
        Ok(Self { team_id, id })
    }
    pub fn team_id(&self) -> i64 {
        self.team_id
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}

pub struct TopicHistoryPage {
    pub team_id: i64,
    pub items: Vec<DailyTopic>,
    pub next: Option<TopicPosition>,
}
