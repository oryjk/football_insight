use std::sync::Arc;

use crate::daily_topics::{
    domain::{
        archive::{TopicDetailQuery, TopicHistoryPage, TopicHistoryQuery, TopicPosition},
        edition::{AudioStatus, DailyTopicAudio, DailyTopicsView},
        query::DailyTopicsQuery,
    },
    ports::daily_topics_repository::{DailyTopicsRepository, DailyTopicsRepositoryError},
};

pub struct GetDailyTopics {
    repository: Arc<dyn DailyTopicsRepository>,
}

impl GetDailyTopics {
    pub fn new(repository: Arc<dyn DailyTopicsRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        query: DailyTopicsQuery,
    ) -> Result<DailyTopicsView, DailyTopicsRepositoryError> {
        let Some(mut edition) = self
            .repository
            .recent_for_team(query.team_id(), query.limit())
            .await?
        else {
            return Ok(DailyTopicsView {
                team_id: query.team_id(),
                edition_date: None,
                updated_at: None,
                items: Vec::new(),
            });
        };
        if edition.team_id != query.team_id()
            || edition
                .items
                .iter()
                .any(|item| item.team_id != query.team_id())
        {
            return Err(DailyTopicsRepositoryError::InvalidEdition);
        }
        edition.items.truncate(query.limit());
        for item in &mut edition.items {
            normalize_audio(&mut item.audio);
        }
        Ok(DailyTopicsView {
            team_id: edition.team_id,
            edition_date: Some(edition.edition_date),
            updated_at: Some(edition.published_at),
            items: edition.items,
        })
    }

    pub async fn history(
        &self,
        query: TopicHistoryQuery,
    ) -> Result<TopicHistoryPage, DailyTopicsRepositoryError> {
        let mut items = self
            .repository
            .history_for_team(query.team_id(), query.after(), query.limit())
            .await?;
        if items.iter().any(|item| item.team_id != query.team_id()) {
            return Err(DailyTopicsRepositoryError::InvalidEdition);
        }
        let has_more = items.len() > query.limit();
        items.truncate(query.limit());
        for item in &mut items {
            normalize_audio(&mut item.audio);
        }
        let next = if has_more {
            items.last().map(|item| TopicPosition {
                published_at: item.published_at,
                id: item.id.clone(),
            })
        } else {
            None
        };
        Ok(TopicHistoryPage {
            team_id: query.team_id(),
            items,
            next,
        })
    }

    pub async fn detail(
        &self,
        query: TopicDetailQuery,
    ) -> Result<Option<crate::daily_topics::domain::edition::DailyTopic>, DailyTopicsRepositoryError>
    {
        let mut item = self
            .repository
            .topic_for_team(query.team_id(), query.id())
            .await?;
        if let Some(item) = &mut item {
            if item.team_id != query.team_id() || item.id != query.id() {
                return Err(DailyTopicsRepositoryError::InvalidEdition);
            }
            normalize_audio(&mut item.audio);
        }
        Ok(item)
    }
}

fn normalize_audio(audio: &mut DailyTopicAudio) {
    let valid_url = audio
        .url
        .as_deref()
        .and_then(|value| url::Url::parse(value).ok())
        .is_some_and(|url| {
            url.scheme() == "https"
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        });
    if audio.status == AudioStatus::Ready && !valid_url {
        audio.status = AudioStatus::Failed;
        audio.duration_seconds = None;
    }
    if audio.status != AudioStatus::Ready {
        audio.url = None;
    }
    if audio
        .duration_seconds
        .is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        audio.duration_seconds = None;
    }
}
