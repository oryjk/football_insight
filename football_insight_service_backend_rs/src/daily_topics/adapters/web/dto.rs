use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::daily_topics::domain::{
    edition::{AudioStatus, DailyTopic, DailyTopicAudio, DailyTopicSource, DailyTopicsView},
    query::{DailyTopicsQuery, InvalidDailyTopicsQuery},
};

#[derive(Deserialize)]
pub struct DailyTopicsParameters {
    #[serde(default = "default_team")]
    team_id: i64,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_team() -> i64 {
    77680
}
fn default_limit() -> usize {
    5
}

impl TryFrom<DailyTopicsParameters> for DailyTopicsQuery {
    type Error = InvalidDailyTopicsQuery;

    fn try_from(value: DailyTopicsParameters) -> Result<Self, Self::Error> {
        DailyTopicsQuery::new(value.team_id, value.limit)
    }
}

#[derive(Serialize)]
pub struct DailyTopicsResponse {
    team_id: i64,
    edition_date: Option<NaiveDate>,
    updated_at: Option<DateTime<FixedOffset>>,
    items: Vec<DailyTopicDto>,
}

#[derive(Serialize)]
struct DailyTopicDto {
    id: String,
    team_id: i64,
    title: String,
    summary: String,
    paragraphs: Vec<String>,
    published_at: DateTime<FixedOffset>,
    sources: Vec<DailyTopicSourceDto>,
    audio: DailyTopicAudioDto,
}

#[derive(Serialize)]
struct DailyTopicSourceDto {
    title: String,
    url: String,
    publisher: String,
    published_at: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize)]
struct DailyTopicAudioDto {
    status: &'static str,
    url: Option<String>,
    dialect: String,
    duration_seconds: Option<f64>,
}

impl From<DailyTopicsView> for DailyTopicsResponse {
    fn from(value: DailyTopicsView) -> Self {
        Self {
            team_id: value.team_id,
            edition_date: value.edition_date,
            updated_at: value.updated_at,
            items: value.items.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DailyTopic> for DailyTopicDto {
    fn from(value: DailyTopic) -> Self {
        Self {
            id: value.id,
            team_id: value.team_id,
            title: value.title,
            summary: value.summary,
            paragraphs: value.paragraphs,
            published_at: value.published_at,
            sources: value.sources.into_iter().map(Into::into).collect(),
            audio: value.audio.into(),
        }
    }
}

impl From<DailyTopicSource> for DailyTopicSourceDto {
    fn from(value: DailyTopicSource) -> Self {
        Self {
            title: value.title,
            url: value.url,
            publisher: value.publisher,
            published_at: value.published_at,
        }
    }
}

impl From<DailyTopicAudio> for DailyTopicAudioDto {
    fn from(value: DailyTopicAudio) -> Self {
        Self {
            status: match value.status {
                AudioStatus::Ready => "ready",
                AudioStatus::Pending => "pending",
                AudioStatus::Failed => "failed",
            },
            url: value.url,
            dialect: value.dialect,
            duration_seconds: value.duration_seconds,
        }
    }
}
