use super::{
    archive::{InvalidTopicArchiveQuery, TopicDetailQuery, TopicPosition},
    edition::{DailyTopic, DailyTopicTag},
};

pub struct TopicTimelineQuery {
    pub anchor: TopicDetailQuery,
    pub tag_team_id: i64,
    pub limit: usize,
    pub after: Option<TopicPosition>,
}

impl TopicTimelineQuery {
    pub fn new(
        anchor: TopicDetailQuery,
        tag_team_id: i64,
        limit: usize,
        after: Option<TopicPosition>,
    ) -> Result<Self, InvalidTopicArchiveQuery> {
        if tag_team_id <= 0
            || !(1..=50).contains(&limit)
            || after
                .as_ref()
                .is_some_and(|p| TopicDetailQuery::new(anchor.team_id(), p.id.clone()).is_err())
        {
            return Err(InvalidTopicArchiveQuery);
        }
        Ok(Self {
            anchor,
            tag_team_id,
            limit,
            after,
        })
    }
}

pub struct TopicTimelinePage {
    pub team_id: i64,
    pub anchor_id: String,
    pub tag: DailyTopicTag,
    pub items: Vec<DailyTopic>,
    pub next: Option<TopicPosition>,
}
