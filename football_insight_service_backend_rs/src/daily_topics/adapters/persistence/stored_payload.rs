use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

use crate::daily_topics::domain::edition::{
    AudioStatus, DailyTopic, DailyTopicAudio, DailyTopicSource,
};

#[derive(Deserialize)]
pub(super) struct StoredPayload {
    pub items: Vec<StoredTopic>,
}

#[derive(Deserialize)]
pub(super) struct StoredTopic {
    id: String,
    team_id: i64,
    title: String,
    summary: String,
    paragraphs: Vec<String>,
    published_at: DateTime<FixedOffset>,
    sources: Vec<StoredSource>,
    audio: StoredAudio,
}

#[derive(Deserialize)]
struct StoredSource {
    title: String,
    url: String,
    publisher: String,
    published_at: Option<DateTime<FixedOffset>>,
}

#[derive(Deserialize)]
struct StoredAudio {
    status: StoredAudioStatus,
    url: Option<String>,
    dialect: String,
    duration_seconds: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum StoredAudioStatus {
    Ready,
    Pending,
    Failed,
}

impl From<StoredTopic> for DailyTopic {
    fn from(value: StoredTopic) -> Self {
        Self {
            heat: 0,
            id: value.id,
            team_id: value.team_id,
            title: value.title,
            summary: value.summary,
            paragraphs: value.paragraphs,
            published_at: value.published_at,
            sources: value
                .sources
                .into_iter()
                .map(|source| DailyTopicSource {
                    title: source.title,
                    url: source.url,
                    publisher: source.publisher,
                    published_at: source.published_at,
                })
                .collect(),
            audio: DailyTopicAudio {
                status: match value.audio.status {
                    StoredAudioStatus::Ready => AudioStatus::Ready,
                    StoredAudioStatus::Pending => AudioStatus::Pending,
                    StoredAudioStatus::Failed => AudioStatus::Failed,
                },
                url: value.audio.url,
                dialect: value.audio.dialect,
                duration_seconds: value.audio.duration_seconds,
            },
        }
    }
}
