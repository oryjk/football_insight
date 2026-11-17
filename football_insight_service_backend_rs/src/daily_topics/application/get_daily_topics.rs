use std::sync::Arc;

use crate::daily_topics::{
    domain::{
        archive::{TopicDetailQuery, TopicHistoryPage, TopicHistoryQuery, TopicPosition},
        edition::{AudioStatus, DailyTopic, DailyTopicAudio, DailyTopicsView},
        heat::TopicVisit,
        query::DailyTopicsQuery,
    },
    ports::daily_topics_repository::{DailyTopicsRepository, DailyTopicsRepositoryError},
};

pub struct GetDailyTopics {
    repository: Arc<dyn DailyTopicsRepository>,
}

#[derive(Debug, thiserror::Error)]
pub enum TopicTimelineError {
    #[error(transparent)]
    Storage(#[from] DailyTopicsRepositoryError),
    #[error("topic not found")]
    AnchorNotFound,
    #[error("club tag does not belong to the anchor topic")]
    UnrelatedTag,
}

impl GetDailyTopics {
    pub async fn timeline(
        &self,
        query: crate::daily_topics::domain::timeline::TopicTimelineQuery,
    ) -> Result<crate::daily_topics::domain::timeline::TopicTimelinePage, TopicTimelineError> {
        self.timeline_for_viewer(query, None).await
    }

    pub async fn timeline_for_viewer(
        &self,
        query: crate::daily_topics::domain::timeline::TopicTimelineQuery,
        viewer_user_id: Option<uuid::Uuid>,
    ) -> Result<crate::daily_topics::domain::timeline::TopicTimelinePage, TopicTimelineError> {
        use crate::daily_topics::domain::timeline::TopicTimelinePage;
        let anchor = self
            .detail_for_viewer(query.anchor.clone(), viewer_user_id)
            .await?
            .ok_or(TopicTimelineError::AnchorNotFound)?;
        let tag = anchor
            .team_tags
            .iter()
            .find(|tag| tag.team_id == query.tag_team_id)
            .cloned()
            .ok_or(TopicTimelineError::UnrelatedTag)?;
        let before = TopicPosition {
            published_at: anchor.published_at,
            id: anchor.id.clone(),
        };
        let mut items = self
            .repository
            .timeline_for_tag(
                query.anchor.team_id(),
                query.tag_team_id,
                &before,
                query.after.as_ref(),
                query.limit,
            )
            .await?;
        if items.iter().any(|item| {
            item.team_id != query.anchor.team_id()
                || item.published_at >= anchor.published_at
                || !item
                    .team_tags
                    .iter()
                    .any(|tag| tag.team_id == query.tag_team_id)
        }) {
            return Err(DailyTopicsRepositoryError::InvalidEdition.into());
        }
        let has_more = items.len() > query.limit;
        items.truncate(query.limit);
        for item in &mut items {
            normalize_audio(&mut item.audio);
            item.category_audio.values_mut().for_each(normalize_audio);
            normalize_images(item);
            normalize_tags(item);
        }
        self.attach_heat(query.anchor.team_id(), &mut items).await;
        self.attach_interactions(query.anchor.team_id(), viewer_user_id, &mut items)
            .await;
        let next = if has_more {
            items.last().map(|item| TopicPosition {
                published_at: item.published_at,
                id: item.id.clone(),
            })
        } else {
            None
        };
        Ok(TopicTimelinePage {
            team_id: query.anchor.team_id(),
            anchor_id: anchor.id,
            tag,
            items,
            next,
        })
    }

    pub fn new(repository: Arc<dyn DailyTopicsRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        query: DailyTopicsQuery,
    ) -> Result<DailyTopicsView, DailyTopicsRepositoryError> {
        self.execute_for_viewer(query, None).await
    }

    pub async fn execute_for_viewer(
        &self,
        query: DailyTopicsQuery,
        viewer_user_id: Option<uuid::Uuid>,
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
            item.category_audio.values_mut().for_each(normalize_audio);
            normalize_images(item);
            normalize_tags(item);
        }
        self.attach_heat(query.team_id(), &mut edition.items).await;
        self.attach_interactions(query.team_id(), viewer_user_id, &mut edition.items)
            .await;
        Ok(DailyTopicsView {
            team_id: edition.team_id,
            edition_date: Some(edition.edition_date),
            updated_at: Some(edition.published_at),
            items: edition.items,
        })
    }

    pub async fn latest(
        &self,
        team_id: i64,
    ) -> Result<DailyTopicsView, DailyTopicsRepositoryError> {
        self.latest_for_viewer(team_id, None).await
    }

    pub async fn latest_for_viewer(
        &self,
        team_id: i64,
        viewer_user_id: Option<uuid::Uuid>,
    ) -> Result<DailyTopicsView, DailyTopicsRepositoryError> {
        let Some(mut edition) = self.repository.latest_for_team(team_id).await? else {
            return Ok(DailyTopicsView {
                team_id,
                edition_date: None,
                updated_at: None,
                items: Vec::new(),
            });
        };
        if edition.team_id != team_id || edition.items.iter().any(|item| item.team_id != team_id) {
            return Err(DailyTopicsRepositoryError::InvalidEdition);
        }
        for item in &mut edition.items {
            normalize_audio(&mut item.audio);
            item.category_audio.values_mut().for_each(normalize_audio);
            normalize_images(item);
            normalize_tags(item);
        }
        self.attach_heat(team_id, &mut edition.items).await;
        self.attach_interactions(team_id, viewer_user_id, &mut edition.items)
            .await;
        Ok(DailyTopicsView {
            team_id: edition.team_id,
            edition_date: Some(edition.edition_date),
            updated_at: Some(edition.published_at),
            items: edition.items,
        })
    }

    pub async fn by_date(
        &self,
        team_id: i64,
        edition_date: chrono::NaiveDate,
    ) -> Result<Option<DailyTopicsView>, DailyTopicsRepositoryError> {
        self.by_date_for_viewer(team_id, edition_date, None).await
    }

    pub async fn by_date_for_viewer(
        &self,
        team_id: i64,
        edition_date: chrono::NaiveDate,
        viewer_user_id: Option<uuid::Uuid>,
    ) -> Result<Option<DailyTopicsView>, DailyTopicsRepositoryError> {
        let Some(mut edition) = self
            .repository
            .edition_for_date(team_id, edition_date)
            .await?
        else {
            return Ok(None);
        };
        if edition.team_id != team_id
            || edition.edition_date != edition_date
            || edition.items.iter().any(|item| item.team_id != team_id)
        {
            return Err(DailyTopicsRepositoryError::InvalidEdition);
        }
        for item in &mut edition.items {
            normalize_audio(&mut item.audio);
            item.category_audio.values_mut().for_each(normalize_audio);
            normalize_images(item);
            normalize_tags(item);
        }
        self.attach_heat(team_id, &mut edition.items).await;
        self.attach_interactions(team_id, viewer_user_id, &mut edition.items)
            .await;
        Ok(Some(DailyTopicsView {
            team_id: edition.team_id,
            edition_date: Some(edition.edition_date),
            updated_at: Some(edition.published_at),
            items: edition.items,
        }))
    }

    pub async fn history(
        &self,
        query: TopicHistoryQuery,
    ) -> Result<TopicHistoryPage, DailyTopicsRepositoryError> {
        self.history_for_viewer(query, None).await
    }

    pub async fn history_for_viewer(
        &self,
        query: TopicHistoryQuery,
        viewer_user_id: Option<uuid::Uuid>,
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
            item.category_audio.values_mut().for_each(normalize_audio);
            normalize_images(item);
            normalize_tags(item);
        }
        self.attach_heat(query.team_id(), &mut items).await;
        self.attach_interactions(query.team_id(), viewer_user_id, &mut items)
            .await;
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
        self.detail_for_viewer(query, None).await
    }

    pub async fn detail_for_viewer(
        &self,
        query: TopicDetailQuery,
        viewer_user_id: Option<uuid::Uuid>,
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
            item.category_audio.values_mut().for_each(normalize_audio);
            normalize_images(item);
            normalize_tags(item);
            self.attach_heat(query.team_id(), std::slice::from_mut(item))
                .await;
            self.attach_interactions(
                query.team_id(),
                viewer_user_id,
                std::slice::from_mut(item),
            )
            .await;
        }
        Ok(item)
    }

    async fn attach_interactions(
        &self,
        team_id: i64,
        viewer_user_id: Option<uuid::Uuid>,
        items: &mut [crate::daily_topics::domain::edition::DailyTopic],
    ) {
        if items.is_empty() {
            return;
        }
        let comments_enabled = match self.repository.comments_enabled().await {
            Ok(enabled) => enabled,
            Err(error) => {
                tracing::warn!(%error, team_id, "comments config unavailable; defaulting to enabled");
                true
            }
        };
        let ids: Vec<_> = items.iter().map(|item| item.id.clone()).collect();
        let stats = match self
            .repository
            .entry_interactions(team_id, &ids, viewer_user_id)
            .await
        {
            Ok(stats) => stats,
            Err(error) => {
                tracing::warn!(%error, team_id, "telegraph entry interactions unavailable; preserving readable content");
                Default::default()
            }
        };
        for item in items {
            item.comments_enabled = comments_enabled;
            let mut total_comments = 0u64;
            for entry in &mut item.entries {
                if let Some(entry_stats) = stats.get(&entry.id) {
                    entry.likes_count = entry_stats.likes_count;
                    entry.dislikes_count = entry_stats.dislikes_count;
                    entry.liked_by_me = entry_stats.liked_by_me;
                    entry.disliked_by_me = entry_stats.disliked_by_me;
                    entry.comments_count = if comments_enabled {
                        entry_stats.comments_count
                    } else {
                        0
                    };
                    total_comments = total_comments.saturating_add(entry.comments_count);
                } else {
                    entry.likes_count = 0;
                    entry.dislikes_count = 0;
                    entry.liked_by_me = false;
                    entry.disliked_by_me = false;
                    entry.comments_count = 0;
                }
            }
            item.comments_count = total_comments;
        }
    }

    async fn attach_heat(
        &self,
        team_id: i64,
        items: &mut [crate::daily_topics::domain::edition::DailyTopic],
    ) {
        if items.is_empty() {
            return;
        }
        let ids: Vec<_> = items.iter().map(|item| item.id.clone()).collect();
        let heat = match self.repository.heat_for_topics(team_id, &ids).await {
            Ok(heat) => heat,
            Err(error) => {
                tracing::warn!(%error, team_id, "topic heat unavailable; preserving readable content");
                Default::default()
            }
        };
        for item in items {
            item.heat = heat.get(&item.id).copied().unwrap_or_default().total();
        }
    }

    pub async fn record_visit(
        &self,
        visit: TopicVisit,
    ) -> Result<Option<u64>, DailyTopicsRepositoryError> {
        let Some(item) = self
            .repository
            .topic_for_team(visit.topic.team_id(), visit.topic.id())
            .await?
        else {
            return Ok(None);
        };
        if item.team_id != visit.topic.team_id() || item.id != visit.topic.id() {
            return Err(DailyTopicsRepositoryError::InvalidEdition);
        }
        Ok(self
            .repository
            .record_visit(&visit)
            .await?
            .map(|heat| heat.total()))
    }
}

fn normalize_images(item: &mut DailyTopic) {
    let mut seen = std::collections::HashSet::new();
    item.images.retain(|image| {
        let valid_url = url::Url::parse(&image.url).ok().is_some_and(|url| {
            url.scheme() == "https"
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        });
        valid_url
            && image.width > 0
            && image.height > 0
            && u64::from(image.width) * u64::from(image.height) <= 40_000_000
            && image.after_paragraph > 0
            && image.after_paragraph <= item.paragraphs.len()
            && item
                .sources
                .iter()
                .any(|source| source.url == image.source_url && source.publisher == image.publisher)
            && seen.insert(image.url.clone())
    });
    item.images.truncate(4);
}

fn normalize_tags(item: &mut DailyTopic) {
    let mut seen = std::collections::HashSet::new();
    item.team_tags.retain(|tag| {
        tag.team_id > 0
            && !tag.name.trim().is_empty()
            && tag.name.chars().count() <= 80
            && !tag.name.contains(['<', '>'])
            && seen.insert(tag.team_id)
    });
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
