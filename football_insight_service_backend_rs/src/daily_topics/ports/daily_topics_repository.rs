use async_trait::async_trait;

use crate::daily_topics::domain::{
    archive::TopicPosition,
    edition::{DailyTopic, DailyTopicEdition},
    heat::{TopicHeat, TopicVisit},
};

#[derive(Debug, thiserror::Error)]
pub enum DailyTopicsRepositoryError {
    #[error("daily topics storage is unavailable")]
    Unavailable,
    #[error("published daily topic edition is invalid")]
    InvalidEdition,
}

#[async_trait]
pub trait DailyTopicsRepository: Send + Sync {
    async fn timeline_for_tag(
        &self,
        team_id: i64,
        tag_team_id: i64,
        before: &TopicPosition,
        after: Option<&TopicPosition>,
        limit: usize,
    ) -> Result<Vec<DailyTopic>, DailyTopicsRepositoryError> {
        let Some(mut edition) = self.latest_for_team(team_id).await? else {
            return Ok(Vec::new());
        };
        sort_topics(&mut edition.items);
        edition.items.retain(|item| {
            item.published_at < before.published_at
                && item.team_tags.iter().any(|tag| tag.team_id == tag_team_id)
                && after.is_none_or(|position| position.follows(item))
        });
        edition.items.truncate(limit + 1);
        Ok(edition.items)
    }

    async fn heat_for_topics(
        &self,
        _team_id: i64,
        _ids: &[String],
    ) -> Result<std::collections::HashMap<String, TopicHeat>, DailyTopicsRepositoryError> {
        Ok(std::collections::HashMap::new())
    }

    async fn record_visit(
        &self,
        _visit: &TopicVisit,
    ) -> Result<Option<TopicHeat>, DailyTopicsRepositoryError> {
        Err(DailyTopicsRepositoryError::Unavailable)
    }

    async fn latest_for_team(
        &self,
        team_id: i64,
    ) -> Result<Option<DailyTopicEdition>, DailyTopicsRepositoryError>;

    async fn recent_for_team(
        &self,
        team_id: i64,
        limit: usize,
    ) -> Result<Option<DailyTopicEdition>, DailyTopicsRepositoryError> {
        let mut edition = self.latest_for_team(team_id).await?;
        if let Some(edition) = &mut edition {
            sort_topics(&mut edition.items);
            edition.items.truncate(limit);
        }
        Ok(edition)
    }

    async fn history_for_team(
        &self,
        team_id: i64,
        after: Option<&TopicPosition>,
        limit: usize,
    ) -> Result<Vec<DailyTopic>, DailyTopicsRepositoryError> {
        let Some(mut edition) = self.latest_for_team(team_id).await? else {
            return Ok(Vec::new());
        };
        sort_topics(&mut edition.items);
        edition
            .items
            .retain(|item| after.is_none_or(|position| position.follows(item)));
        edition.items.truncate(limit + 1);
        Ok(edition.items)
    }

    async fn topic_for_team(
        &self,
        team_id: i64,
        id: &str,
    ) -> Result<Option<DailyTopic>, DailyTopicsRepositoryError> {
        Ok(self
            .latest_for_team(team_id)
            .await?
            .and_then(|edition| edition.items.into_iter().find(|item| item.id == id)))
    }
}

fn sort_topics(items: &mut Vec<DailyTopic>) {
    items.sort_by(|left, right| {
        right
            .published_at
            .cmp(&left.published_at)
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut seen = std::collections::HashSet::new();
    items.retain(|item| seen.insert(item.id.clone()));
}
