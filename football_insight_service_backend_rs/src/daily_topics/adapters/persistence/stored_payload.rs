use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

use crate::daily_topics::domain::edition::{
    AudioStatus, DailyTopic, DailyTopicAudio, DailyTopicEntry, DailyTopicImage, DailyTopicSource,
    DailyTopicTag, ENTRY_CATEGORIES,
};

#[derive(Deserialize)]
pub(super) struct StoredPayload {
    pub items: Vec<StoredTopic>,
}

#[derive(Deserialize)]
pub(super) struct StoredTopic {
    id: String,
    team_id: i64,
    #[serde(default)]
    team_tags: serde_json::Value,
    title: String,
    summary: String,
    paragraphs: Vec<String>,
    published_at: DateTime<FixedOffset>,
    sources: Vec<StoredSource>,
    #[serde(default)]
    images: serde_json::Value,
    #[serde(default)]
    entries: serde_json::Value,
    audio: StoredAudio,
    #[serde(default)]
    category_audio: serde_json::Value,
}

#[derive(Deserialize)]
struct StoredEntry {
    id: String,
    #[serde(default)]
    category: Option<String>,
    title: String,
    text: String,
    published_at: DateTime<FixedOffset>,
    #[serde(default)]
    sources: Vec<StoredSource>,
    #[serde(default)]
    team_tags: serde_json::Value,
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
struct StoredTag {
    team_id: i64,
    name: String,
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

fn tags(value: &serde_json::Value) -> Vec<DailyTopicTag> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|tag| serde_json::from_value::<StoredTag>(tag.clone()).ok())
        .filter(|tag| {
            tag.team_id > 0
                && !tag.name.trim().is_empty()
                && tag.name.chars().count() <= 80
                && !tag.name.contains(['<', '>'])
        })
        .map(|tag| DailyTopicTag {
            team_id: tag.team_id,
            name: tag.name.trim().to_owned(),
        })
        .collect()
}

fn source(value: StoredSource) -> DailyTopicSource {
    DailyTopicSource {
        title: value.title,
        url: value.url,
        publisher: value.publisher,
        published_at: value.published_at,
    }
}

/// Malformed entries are skipped one by one; the topic itself stays readable.
fn entries(value: &serde_json::Value) -> Vec<DailyTopicEntry> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| serde_json::from_value::<StoredEntry>(entry.clone()).ok())
        .filter(|entry| !entry.title.trim().is_empty() && !entry.text.trim().is_empty())
        .map(|entry| DailyTopicEntry {
            team_tags: tags(&entry.team_tags),
            category: entry
                .category
                .filter(|category| ENTRY_CATEGORIES.contains(&category.as_str())),
            id: entry.id,
            title: entry.title.trim().to_owned(),
            text: entry.text.trim().to_owned(),
            published_at: entry.published_at,
            sources: entry.sources.into_iter().map(source).collect(),
        })
        .collect()
}

fn audio(value: StoredAudio) -> DailyTopicAudio {
    DailyTopicAudio {
        status: match value.status {
            StoredAudioStatus::Ready => AudioStatus::Ready,
            StoredAudioStatus::Pending => AudioStatus::Pending,
            StoredAudioStatus::Failed => AudioStatus::Failed,
        },
        url: value.url,
        dialect: value.dialect,
        duration_seconds: value.duration_seconds,
    }
}

/// Only the three known categories; anything malformed is dropped.
fn category_audio(
    value: &serde_json::Value,
) -> std::collections::BTreeMap<String, DailyTopicAudio> {
    value
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(category, _)| ENTRY_CATEGORIES.contains(&category.as_str()))
        .filter_map(|(category, value)| {
            serde_json::from_value::<StoredAudio>(value.clone())
                .ok()
                .map(|stored| (category.clone(), audio(stored)))
        })
        .collect()
}

impl From<StoredTopic> for DailyTopic {
    fn from(value: StoredTopic) -> Self {
        Self {
            heat: 0,
            team_tags: tags(&value.team_tags),
            entries: entries(&value.entries),
            category_audio: category_audio(&value.category_audio),
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
            sources: value.sources.into_iter().map(source).collect(),
            audio: audio(value.audio),
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
            assert!(topic.entries.is_empty());
        }
    }

    #[test]
    fn roundup_entries_are_read_and_malformed_ones_skipped() {
        let payload = json!({"id":"topic", "team_id":77680, "title":"title", "summary":"summary",
            "paragraphs":["【a】b"], "published_at":"2026-11-09T12:00:00+08:00", "sources":[],
            "category_audio":{
                "world":{"status":"ready", "url":"https://example.com/w.m4a", "dialect":"sichuan", "duration_seconds":30},
                "gossip":{"status":"ready", "url":"https://example.com/g.m4a", "dialect":"sichuan", "duration_seconds":30},
                "abroad":"broken"
            },
            "entries":[
                {"id":"e1", "category":"domestic", "title":"泰山3-2胜蓉城", "text":"山东泰山主场3-2击败成都蓉城。",
                 "published_at":"2026-11-09T10:00:00+08:00",
                 "sources":[{"title":"t", "url":"https://news.example/1", "publisher":"直播吧",
                             "published_at":"2026-11-09T10:00:00+08:00"}],
                 "team_tags":[{"team_id":136, "name":"山东泰山"}, {"team_id":-1, "name":"bad"}]},
                {"id":"e2", "title":"", "text":"空标题", "published_at":"2026-11-09T10:00:00+08:00"},
                {"id":"e3", "title":"缺时间", "text":"x"},
                "not-an-object"
            ],
            "audio":{"status":"ready", "url":"https://example.com/a.m4a", "dialect":"sichuan", "duration_seconds":5}});
        let stored: StoredTopic = serde_json::from_value(payload).unwrap();
        let topic: DailyTopic = stored.into();
        assert_eq!(topic.entries.len(), 1);
        let entry = &topic.entries[0];
        assert_eq!(entry.title, "泰山3-2胜蓉城");
        assert_eq!(entry.sources[0].publisher, "直播吧");
        assert_eq!(entry.team_tags.len(), 1);
        assert_eq!(entry.category.as_deref(), Some("domestic"));
        assert_eq!(
            topic.category_audio.keys().collect::<Vec<_>>(),
            vec!["world"]
        );
    }
}
