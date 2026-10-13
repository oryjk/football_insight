use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

use crate::daily_topics::domain::edition::{
    AudioStatus, DailyTopic, DailyTopicAudio, DailyTopicImage, DailyTopicSource,
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
    #[serde(default)]
    images: serde_json::Value,
    audio: StoredAudio,
}

#[derive(Deserialize)]
struct StoredImage {
    url: String,
    width: u32,
    height: u32,
    source_url: String,
    publisher: String,
    after_paragraph: usize,
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
            images: value
                .images
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|image| serde_json::from_value::<StoredImage>(image.clone()).ok())
                .map(|image| DailyTopicImage {
                    url: image.url,
                    width: image.width,
                    height: image.height,
                    source_url: image.source_url,
                    publisher: image.publisher,
                    after_paragraph: image.after_paragraph,
                })
                .collect(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn malformed_optional_photos_preserve_article_and_voice() {
        let mut payload = json!({"id":"topic", "team_id":77680, "title":"title", "summary":"summary",
            "paragraphs":["body"], "published_at":"2026-10-13T08:00:00+08:00", "sources":[],
            "audio":{"status":"ready", "url":"https://example.com/a.m4a", "dialect":"sichuan", "duration_seconds":5}});
        for photos in [
            serde_json::Value::Null,
            json!("invalid"),
            json!([null, {"width": "not-a-number"}]),
        ] {
            payload["images"] = photos;
            let stored: StoredTopic = serde_json::from_value(payload.clone()).unwrap();
            let topic: DailyTopic = stored.into();
            assert!(topic.images.is_empty());
            assert_eq!(topic.paragraphs, vec!["body"]);
            assert_eq!(topic.audio.status, AudioStatus::Ready);
        }
    }
}
