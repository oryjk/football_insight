use super::archive::{InvalidTopicArchiveQuery, TopicDetailQuery};

pub const MAX_HEAT: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, Copy, Default)]
pub struct TopicHeat {
    pub base: u64,
    pub views: u64,
}

impl TopicHeat {
    pub fn total(self) -> u64 {
        self.base.saturating_add(self.views).min(MAX_HEAT)
    }
}

pub struct TopicVisit {
    pub topic: TopicDetailQuery,
    pub visitor_hash: String,
    /// Set only when the visit carried a valid login; anonymous visits stay unattributed.
    pub user_id: Option<uuid::Uuid>,
}

impl TopicVisit {
    pub fn new(
        topic: TopicDetailQuery,
        visitor_hash: String,
    ) -> Result<Self, InvalidTopicArchiveQuery> {
        if visitor_hash.len() != 64
            || !visitor_hash
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(InvalidTopicArchiveQuery);
        }
        Ok(Self {
            topic,
            visitor_hash,
            user_id: None,
        })
    }

    pub fn with_user(mut self, user_id: Option<uuid::Uuid>) -> Self {
        self.user_id = user_id;
        self
    }
}
