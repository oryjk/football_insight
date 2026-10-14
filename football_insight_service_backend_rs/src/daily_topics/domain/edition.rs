use chrono::{DateTime, FixedOffset, NaiveDate};

#[derive(Debug, Clone)]
pub struct DailyTopicEdition {
    pub team_id: i64,
    pub edition_date: NaiveDate,
    pub published_at: DateTime<FixedOffset>,
    pub items: Vec<DailyTopic>,
}

#[derive(Debug, Clone)]
pub struct DailyTopicsView {
    pub team_id: i64,
    pub edition_date: Option<NaiveDate>,
    pub updated_at: Option<DateTime<FixedOffset>>,
    pub items: Vec<DailyTopic>,
}

#[derive(Debug, Clone)]
pub struct DailyTopic {
    pub id: String,
    pub team_id: i64,
    pub team_tags: Vec<DailyTopicTag>,
    pub title: String,
    pub summary: String,
    pub paragraphs: Vec<String>,
    pub published_at: DateTime<FixedOffset>,
    pub sources: Vec<DailyTopicSource>,
    pub images: Vec<DailyTopicImage>,
    pub audio: DailyTopicAudio,
    pub heat: u64,
}

#[derive(Debug, Clone)]
pub struct DailyTopicTag {
    pub team_id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DailyTopicImage {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub source_url: String,
    pub publisher: String,
    pub after_paragraph: usize,
}

#[derive(Debug, Clone)]
pub struct DailyTopicSource {
    pub title: String,
    pub url: String,
    pub publisher: String,
    pub published_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioStatus {
    Ready,
    Pending,
    Failed,
}

#[derive(Debug, Clone)]
pub struct DailyTopicAudio {
    pub status: AudioStatus,
    pub url: Option<String>,
    pub dialect: String,
    pub duration_seconds: Option<f64>,
}
