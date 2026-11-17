use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::daily_topics::domain::{
    archive::{
        InvalidTopicArchiveQuery, TopicDetailQuery, TopicHistoryPage, TopicHistoryQuery,
        TopicPosition,
    },
    edition::{
        AudioStatus, DailyTopic, DailyTopicAudio, DailyTopicEntry, DailyTopicImage,
        DailyTopicSource, DailyTopicsView,
    },
    interactions::{
        TelegraphComment, TelegraphCommentLikeSummary, TelegraphCommentsPage,
        TelegraphCommentsPosition, TelegraphInteractionError, TelegraphReactionSummary,
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
    team_tags: Vec<TopicTagDto>,
    title: String,
    summary: String,
    paragraphs: Vec<String>,
    published_at: DateTime<FixedOffset>,
    sources: Vec<DailyTopicSourceDto>,
    images: Vec<DailyTopicImageDto>,
    /// 每日话题快讯条目，新到旧；旧版多篇话题为空数组。
    entries: Vec<DailyTopicEntryDto>,
    audio: DailyTopicAudioDto,
    /// 每日电报分类语音：`{"domestic": {...}, "world": {...}}`；旧数据为空对象。
    category_audio: std::collections::BTreeMap<String, DailyTopicAudioDto>,
    heat: u64,
    comments_enabled: bool,
    comments_count: u64,
}

#[derive(Serialize)]
struct DailyTopicEntryDto {
    id: String,
    category: Option<String>,
    title: String,
    text: String,
    published_at: DateTime<FixedOffset>,
    sources: Vec<DailyTopicSourceDto>,
    team_tags: Vec<TopicTagDto>,
    likes_count: u64,
    dislikes_count: u64,
    comments_count: u64,
    liked_by_me: bool,
    disliked_by_me: bool,
}

impl From<DailyTopicEntry> for DailyTopicEntryDto {
    fn from(value: DailyTopicEntry) -> Self {
        Self {
            id: value.id,
            category: value.category,
            title: value.title,
            text: value.text,
            published_at: value.published_at,
            sources: value.sources.into_iter().map(Into::into).collect(),
            team_tags: value.team_tags.into_iter().map(Into::into).collect(),
            likes_count: value.likes_count,
            dislikes_count: value.dislikes_count,
            comments_count: value.comments_count,
            liked_by_me: value.liked_by_me,
            disliked_by_me: value.disliked_by_me,
        }
    }
}

#[derive(Serialize)]
pub(super) struct TopicTagDto {
    team_id: i64,
    name: String,
}

impl From<crate::daily_topics::domain::edition::DailyTopicTag> for TopicTagDto {
    fn from(tag: crate::daily_topics::domain::edition::DailyTopicTag) -> Self {
        Self {
            team_id: tag.team_id,
            name: tag.name,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct TopicTimelineParameters {
    #[serde(default = "default_team")]
    team_id: i64,
    tag_team_id: i64,
    #[serde(default = "history_limit")]
    limit: usize,
    cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct TimelineCursor {
    team_id: i64,
    tag_team_id: i64,
    anchor_id: String,
    published_at: DateTime<FixedOffset>,
    id: String,
}

impl TopicTimelineParameters {
    pub fn into_query(
        self,
        anchor_id: String,
    ) -> Result<crate::daily_topics::domain::timeline::TopicTimelineQuery, InvalidTopicArchiveQuery>
    {
        let after = if let Some(encoded) = self.cursor {
            if encoded.len() > 1024 {
                return Err(InvalidTopicArchiveQuery);
            }
            let cursor: TimelineCursor = serde_json::from_slice(
                &URL_SAFE_NO_PAD
                    .decode(encoded)
                    .map_err(|_| InvalidTopicArchiveQuery)?,
            )
            .map_err(|_| InvalidTopicArchiveQuery)?;
            if cursor.team_id != self.team_id
                || cursor.tag_team_id != self.tag_team_id
                || cursor.anchor_id != anchor_id
            {
                return Err(InvalidTopicArchiveQuery);
            }
            Some(TopicPosition {
                published_at: cursor.published_at,
                id: cursor.id,
            })
        } else {
            None
        };
        crate::daily_topics::domain::timeline::TopicTimelineQuery::new(
            TopicDetailQuery::new(self.team_id, anchor_id)?,
            self.tag_team_id,
            self.limit,
            after,
        )
    }
}

#[derive(Serialize)]
pub(super) struct TopicTimelineResponse {
    team_id: i64,
    anchor_id: String,
    tag: TopicTagDto,
    items: Vec<DailyTopicDto>,
    next_cursor: Option<String>,
    has_more: bool,
}

impl From<crate::daily_topics::domain::timeline::TopicTimelinePage> for TopicTimelineResponse {
    fn from(page: crate::daily_topics::domain::timeline::TopicTimelinePage) -> Self {
        let next_cursor = page.next.map(|p| {
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&TimelineCursor {
                    team_id: page.team_id,
                    tag_team_id: page.tag.team_id,
                    anchor_id: page.anchor_id.clone(),
                    published_at: p.published_at,
                    id: p.id,
                })
                .expect("timeline cursor is serializable"),
            )
        });
        Self {
            team_id: page.team_id,
            anchor_id: page.anchor_id,
            tag: page.tag.into(),
            items: page.items.into_iter().map(Into::into).collect(),
            has_more: next_cursor.is_some(),
            next_cursor,
        }
    }
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
    pub(super) team_id: i64,
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
            team_tags: value.team_tags.into_iter().map(Into::into).collect(),
            summary: value.summary,
            paragraphs: value.paragraphs,
            published_at: value.published_at,
            sources: value.sources.into_iter().map(Into::into).collect(),
            images: value.images.into_iter().map(Into::into).collect(),
            entries: value.entries.into_iter().map(Into::into).collect(),
            audio: value.audio.into(),
            category_audio: value
                .category_audio
                .into_iter()
                .map(|(category, audio)| (category, audio.into()))
                .collect(),
            heat: value.heat,
            comments_enabled: value.comments_enabled,
            comments_count: value.comments_count,
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

#[derive(Deserialize)]
pub(super) struct EntryInteractionParameters {
    #[serde(default = "default_team")]
    pub team_id: i64,
}

#[derive(Serialize)]
pub(super) struct TelegraphReactionResponseDto {
    pub entry_id: String,
    pub likes_count: u64,
    pub dislikes_count: u64,
    pub liked_by_me: bool,
    pub disliked_by_me: bool,
}

impl From<TelegraphReactionSummary> for TelegraphReactionResponseDto {
    fn from(value: TelegraphReactionSummary) -> Self {
        Self {
            entry_id: value.entry_id,
            likes_count: value.likes_count,
            dislikes_count: value.dislikes_count,
            liked_by_me: value.liked_by_me,
            disliked_by_me: value.disliked_by_me,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct TelegraphCommentsQueryParameters {
    #[serde(default = "default_team")]
    pub team_id: i64,
    #[serde(default = "history_limit")]
    pub limit: usize,
    pub cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct TelegraphCommentsCursorDto {
    team_id: i64,
    topic_id: String,
    entry_id: String,
    created_at: DateTime<Utc>,
    id: Uuid,
}

impl TelegraphCommentsQueryParameters {
    pub fn decode_cursor(
        &self,
        topic_id: &str,
        entry_id: &str,
    ) -> Result<Option<TelegraphCommentsPosition>, TelegraphInteractionError> {
        let Some(encoded) = self.cursor.as_deref().map(str::trim).filter(|v| !v.is_empty()) else {
            return Ok(None);
        };
        if encoded.len() > 1024 {
            return Err(TelegraphInteractionError::InvalidInput("分页游标无效"));
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| TelegraphInteractionError::InvalidInput("分页游标无效"))?;
        let cursor: TelegraphCommentsCursorDto = serde_json::from_slice(&bytes)
            .map_err(|_| TelegraphInteractionError::InvalidInput("分页游标无效"))?;
        if cursor.team_id != self.team_id
            || cursor.topic_id != topic_id
            || cursor.entry_id != entry_id
        {
            return Err(TelegraphInteractionError::InvalidInput("分页游标无效"));
        }
        Ok(Some(TelegraphCommentsPosition {
            created_at: cursor.created_at,
            id: cursor.id,
        }))
    }
}

#[derive(Serialize)]
pub(super) struct TelegraphCommentAuthorDto {
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Serialize)]
pub(super) struct TelegraphCommentReplyRefDto {
    pub comment_id: Uuid,
    pub author_display_name: String,
}

#[derive(Serialize)]
pub(super) struct TelegraphCommentLikeResponseDto {
    pub comment_id: Uuid,
    pub likes_count: u64,
    pub liked_by_me: bool,
}

impl From<TelegraphCommentLikeSummary> for TelegraphCommentLikeResponseDto {
    fn from(value: TelegraphCommentLikeSummary) -> Self {
        Self {
            comment_id: value.comment_id,
            likes_count: value.likes_count,
            liked_by_me: value.liked_by_me,
        }
    }
}

#[derive(Serialize)]
pub(super) struct TelegraphCommentDto {
    pub id: Uuid,
    pub topic_id: String,
    pub entry_id: String,
    pub author: TelegraphCommentAuthorDto,
    pub reply_to: Option<TelegraphCommentReplyRefDto>,
    pub content: String,
    pub likes_count: u64,
    pub liked_by_me: bool,
    pub replies_count: u64,
    pub is_mine: bool,
    pub created_at: DateTime<Utc>,
}

impl From<TelegraphComment> for TelegraphCommentDto {
    fn from(value: TelegraphComment) -> Self {
        Self {
            id: value.id,
            topic_id: value.topic_id,
            entry_id: value.entry_id,
            author: TelegraphCommentAuthorDto {
                display_name: value.author.display_name,
                avatar_url: value.author.avatar_url,
            },
            reply_to: value.reply_to.map(|reply| TelegraphCommentReplyRefDto {
                comment_id: reply.comment_id,
                author_display_name: reply.author_display_name,
            }),
            content: value.content,
            likes_count: value.likes_count,
            liked_by_me: value.liked_by_me,
            replies_count: value.replies_count,
            is_mine: value.is_mine,
            created_at: value.created_at,
        }
    }
}

#[derive(Serialize)]
pub(super) struct TelegraphCommentsPageDto {
    pub comments_enabled: bool,
    pub total_count: u64,
    pub items: Vec<TelegraphCommentDto>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

impl TelegraphCommentsPageDto {
    pub fn from_page(
        page: TelegraphCommentsPage,
        team_id: i64,
        topic_id: &str,
        entry_id: &str,
    ) -> Self {
        let next_cursor = page.next_cursor.map(|pos| {
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&TelegraphCommentsCursorDto {
                    team_id,
                    topic_id: topic_id.to_string(),
                    entry_id: entry_id.to_string(),
                    created_at: pos.created_at,
                    id: pos.id,
                })
                .expect("comment cursor contains serializable values"),
            )
        });
        Self {
            comments_enabled: page.comments_enabled,
            total_count: page.total_count,
            items: page.items.into_iter().map(Into::into).collect(),
            has_more: page.has_more && next_cursor.is_some(),
            next_cursor,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct CreateTelegraphCommentBody {
    pub content: String,
    pub reply_to_comment_id: Option<Uuid>,
}

#[derive(Serialize)]
pub(super) struct CreateTelegraphCommentResponseDto {
    pub comment: TelegraphCommentDto,
    pub comments_count: u64,
}

#[derive(Serialize)]
pub(super) struct DeleteTelegraphCommentResponseDto {
    pub comments_count: u64,
}
