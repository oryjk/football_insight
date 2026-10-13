use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::daily_topics::domain::{
    archive::{
        InvalidTopicArchiveQuery, TopicDetailQuery, TopicHistoryPage, TopicHistoryQuery,
        TopicPosition,
    },
    edition::{
        AudioStatus, DailyTopic, DailyTopicAudio, DailyTopicImage, DailyTopicSource,
        DailyTopicsView,
    },
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
pub(super) struct DailyTopicDto {
    id: String,
    team_id: i64,
    title: String,
    summary: String,
    paragraphs: Vec<String>,
    published_at: DateTime<FixedOffset>,
    sources: Vec<DailyTopicSourceDto>,
    images: Vec<DailyTopicImageDto>,
    audio: DailyTopicAudioDto,
    heat: u64,
}

#[derive(Serialize)]
struct DailyTopicImageDto {
    url: String,
    width: u32,
    height: u32,
    source_url: String,
    publisher: String,
    after_paragraph: usize,
}

impl From<DailyTopicImage> for DailyTopicImageDto {
    fn from(value: DailyTopicImage) -> Self {
        Self {
            url: value.url,
            width: value.width,
            height: value.height,
            source_url: value.source_url,
            publisher: value.publisher,
            after_paragraph: value.after_paragraph,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct TopicHistoryParameters {
    #[serde(default = "default_team")]
    team_id: i64,
    #[serde(default = "history_limit")]
    limit: usize,
    cursor: Option<String>,
}

fn history_limit() -> usize {
    20
}

#[derive(Deserialize)]
pub(super) struct TopicDetailParameters {
    #[serde(default = "default_team")]
    team_id: i64,
}

impl TopicDetailParameters {
    pub fn into_query(self, id: String) -> Result<TopicDetailQuery, InvalidTopicArchiveQuery> {
        TopicDetailQuery::new(self.team_id, id)
    }
}

#[derive(Serialize, Deserialize)]
struct CursorDto {
    team_id: i64,
    published_at: DateTime<FixedOffset>,
    id: String,
}

impl TryFrom<TopicHistoryParameters> for TopicHistoryQuery {
    type Error = InvalidTopicArchiveQuery;

    fn try_from(value: TopicHistoryParameters) -> Result<Self, Self::Error> {
        let after = if let Some(encoded) = value.cursor {
            if encoded.len() > 512 {
                return Err(InvalidTopicArchiveQuery);
            }
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .map_err(|_| InvalidTopicArchiveQuery)?;
            let cursor: CursorDto =
                serde_json::from_slice(&bytes).map_err(|_| InvalidTopicArchiveQuery)?;
            if cursor.team_id != value.team_id {
                return Err(InvalidTopicArchiveQuery);
            }
            Some(TopicPosition {
                published_at: cursor.published_at,
                id: cursor.id,
            })
        } else {
            None
        };
        TopicHistoryQuery::new(value.team_id, value.limit, after)
    }
}

#[derive(Serialize)]
pub(super) struct TopicHistoryResponse {
    team_id: i64,
    items: Vec<DailyTopicDto>,
    next_cursor: Option<String>,
    has_more: bool,
}

impl From<TopicHistoryPage> for TopicHistoryResponse {
    fn from(value: TopicHistoryPage) -> Self {
        let next_cursor = value.next.map(|position| {
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&CursorDto {
                    team_id: value.team_id,
                    published_at: position.published_at,
                    id: position.id,
                })
                .expect("topic cursor contains serializable values"),
            )
        });
        Self {
            team_id: value.team_id,
            items: value.items.into_iter().map(Into::into).collect(),
            has_more: next_cursor.is_some(),
            next_cursor,
        }
    }
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
            images: value.images.into_iter().map(Into::into).collect(),
            audio: value.audio.into(),
            heat: value.heat,
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

#[derive(Deserialize)]
pub(super) struct TopicVisitBody {
    pub visitor_id: String,
}

#[derive(Serialize)]
pub(super) struct TopicVisitResponse {
    pub heat: u64,
}
