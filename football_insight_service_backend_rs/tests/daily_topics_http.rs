use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{DateTime, NaiveDate};
use football_insight_service_backend_rs::daily_topics::{
    adapters::web::routes::daily_topics_routes,
    application::get_daily_topics::GetDailyTopics,
    domain::edition::{
        AudioStatus, DailyTopic, DailyTopicAudio, DailyTopicEdition, DailyTopicSource,
    },
    ports::daily_topics_repository::{DailyTopicsRepository, DailyTopicsRepositoryError},
};
use serde_json::{Value, json};
use tower::ServiceExt;

struct Repository {
    edition: Option<DailyTopicEdition>,
    fail: bool,
    heat_fail: bool,
}

#[async_trait]
impl DailyTopicsRepository for Repository {
    async fn heat_for_topics(
        &self,
        _team_id: i64,
        _ids: &[String],
    ) -> Result<
        std::collections::HashMap<
            String,
            football_insight_service_backend_rs::daily_topics::domain::heat::TopicHeat,
        >,
        DailyTopicsRepositoryError,
    > {
        if self.heat_fail {
            Err(DailyTopicsRepositoryError::Unavailable)
        } else {
            Ok(std::collections::HashMap::new())
        }
    }

    async fn latest_for_team(
        &self,
        team_id: i64,
    ) -> Result<Option<DailyTopicEdition>, DailyTopicsRepositoryError> {
        if self.fail {
            return Err(DailyTopicsRepositoryError::Unavailable);
        }
        Ok(self
            .edition
            .clone()
            .filter(|edition| edition.team_id == team_id))
    }
}

fn app(edition: Option<DailyTopicEdition>, fail: bool) -> axum::Router {
    daily_topics_routes(Arc::new(GetDailyTopics::new(Arc::new(Repository {
        edition,
        fail,
        heat_fail: false,
    }))))
}

async fn get(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn edition() -> DailyTopicEdition {
    let published_at = DateTime::parse_from_rfc3339("2026-10-13T08:30:00+08:00").unwrap();
    DailyTopicEdition {
        team_id: 77680,
        edition_date: NaiveDate::from_ymd_opt(2026, 10, 13).unwrap(),
        published_at,
        items: (0..7)
            .map(|index| DailyTopic {
                id: format!("{index:064x}"),
                team_id: 77680,
                heat: 0,
                title: format!("隔离测试文章 {index}"),
                summary: "测试摘要".into(),
                paragraphs: vec!["第一段".into(), "第二段".into()],
                published_at,
                sources: vec![DailyTopicSource {
                    title: "测试来源".into(),
                    url: "https://example.com/original".into(),
                    publisher: "测试媒体".into(),
                    published_at: Some(
                        DateTime::parse_from_rfc3339("2026-10-12T20:00:00+08:00").unwrap(),
                    ),
                }],
                audio: DailyTopicAudio {
                    status: AudioStatus::Ready,
                    url: Some("https://example.com/audio.m4a".into()),
                    dialect: "sichuan".into(),
                    duration_seconds: Some(60.0),
                },
            })
            .collect(),
    }
}

#[tokio::test]
async fn history_is_public_paginated_and_detail_uses_stable_topic_identity() {
    let app = app(Some(edition()), false);
    let (status, first) = get(app.clone(), "/api/v1/daily-topics/history?limit=2").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["items"].as_array().unwrap().len(), 2);
    assert_eq!(first["has_more"], true);
    let cursor = first["next_cursor"].as_str().unwrap();
    let (status, second) = get(
        app.clone(),
        &format!("/api/v1/daily-topics/history?limit=2&cursor={cursor}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_ne!(first["items"][1]["id"], second["items"][0]["id"]);
    let id = first["items"][0]["id"].as_str().unwrap();
    let (status, detail) = get(app.clone(), &format!("/api/v1/daily-topics/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail, first["items"][0]);
    assert_eq!(
        get(app.clone(), "/api/v1/daily-topics/missing").await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(app.clone(), "/api/v1/daily-topics/history?cursor=invalid")
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        get(app.clone(), "/api/v1/daily-topics/history?limit=51")
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        get(
            app.clone(),
            &format!("/api/v1/daily-topics/history?team_id=999&cursor={cursor}")
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        get(app, &format!("/api/v1/daily-topics/{id}?team_id=999"))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn archive_and_detail_storage_failures_never_become_successful_empty_content() {
    assert_eq!(
        get(app(None, true), "/api/v1/daily-topics/history").await.0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        get(app(None, true), "/api/v1/daily-topics/known").await.0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn defaults_and_no_published_edition_return_public_empty_result() {
    let (status, body) = get(app(None, false), "/api/v1/daily-topics").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({"team_id":77680,"edition_date":null,"updated_at":null,"items":[]})
    );
}

#[tokio::test]
async fn invalid_parameters_are_bad_requests_before_any_database_read() {
    for query in [
        "team_id=0",
        "team_id=-1",
        "team_id=x",
        "team_id=",
        "team_id=9223372036854775808",
        "limit=0",
        "limit=6",
        "limit=-1",
        "limit=x",
        "limit=",
    ] {
        let (status, _) = get(app(None, true), &format!("/api/v1/daily-topics?{query}")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}");
    }
}

#[tokio::test]
async fn default_limit_is_five_and_requested_limit_is_respected() {
    for (query, count) in [("", 5), ("?limit=1", 1), ("?limit=3", 3), ("?limit=5", 5)] {
        let (status, body) = get(
            app(Some(edition()), false),
            &format!("/api/v1/daily-topics{query}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["items"].as_array().unwrap().len(), count);
    }
}

#[tokio::test]
async fn a_different_team_never_receives_other_teams_articles() {
    let (status, body) = get(
        app(Some(edition()), false),
        "/api/v1/daily-topics?team_id=999",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({"team_id":999,"edition_date":null,"updated_at":null,"items":[]})
    );
}

#[tokio::test]
async fn public_contract_preserves_content_sources_and_audio_without_data_wrapper() {
    let (status, body) = get(app(Some(edition()), false), "/api/v1/daily-topics?limit=1").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.get("data").is_none());
    assert_eq!(body["edition_date"], "2026-10-13");
    assert_eq!(body["updated_at"], "2026-10-13T08:30:00+08:00");
    assert_eq!(
        body["items"][0],
        json!({
            "id":"0000000000000000000000000000000000000000000000000000000000000000",
            "team_id":77680,"heat":0,"title":"隔离测试文章 0","summary":"测试摘要",
            "paragraphs":["第一段","第二段"],"published_at":"2026-10-13T08:30:00+08:00",
            "sources":[{"title":"测试来源","url":"https://example.com/original","publisher":"测试媒体","published_at":"2026-10-12T20:00:00+08:00"}],
            "audio":{"status":"ready","url":"https://example.com/audio.m4a","dialect":"sichuan","duration_seconds":60.0}
        })
    );
}

#[tokio::test]
async fn pending_and_failed_audio_never_publish_a_playback_url() {
    for status in [AudioStatus::Pending, AudioStatus::Failed] {
        let mut value = edition();
        value.items[0].audio.status = status;
        value.items[0].audio.duration_seconds = None;
        let (response_status, body) =
            get(app(Some(value), false), "/api/v1/daily-topics?limit=1").await;
        assert_eq!(response_status, StatusCode::OK);
        assert_eq!(
            body["items"][0]["audio"]["status"],
            match status {
                AudioStatus::Pending => "pending",
                _ => "failed",
            }
        );
        assert!(body["items"][0]["audio"]["url"].is_null());
        assert!(body["items"][0]["audio"]["duration_seconds"].is_null());
    }
}

#[tokio::test]
async fn unsafe_ready_audio_is_not_exposed_as_playable() {
    for url in [
        "http://example.com/audio.m4a",
        "javascript:alert(1)",
        "https://",
        "https://user:secret@example.com/audio.m4a",
    ] {
        let mut value = edition();
        value.items[0].audio.url = Some(url.into());
        let (status, body) = get(app(Some(value), false), "/api/v1/daily-topics?limit=1").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["items"][0]["audio"]["status"], "failed");
        assert!(body["items"][0]["audio"]["url"].is_null());
    }
}

#[tokio::test]
async fn repository_failure_is_not_disguised_as_an_empty_success() {
    let (status, body) = get(app(None, true), "/api/v1/daily-topics").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(body.get("items").is_none());
}

#[tokio::test]
async fn corrupt_edition_does_not_leak_another_teams_item() {
    let mut value = edition();
    value.items[0].team_id = 999;
    let (status, _) = get(app(Some(value), false), "/api/v1/daily-topics").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn public_cache_does_not_hide_a_newly_published_edition() {
    use axum::middleware::from_fn_with_state;
    use football_insight_service_backend_rs::http_cache::{HttpResponseCache, cache_get_responses};
    use std::{sync::Mutex, time::Duration};

    struct PublishedEdition(Mutex<Option<DailyTopicEdition>>);
    #[async_trait]
    impl DailyTopicsRepository for PublishedEdition {
        async fn latest_for_team(
            &self,
            _: i64,
        ) -> Result<Option<DailyTopicEdition>, DailyTopicsRepositoryError> {
            Ok(self.0.lock().unwrap().clone())
        }
    }
    let repository = Arc::new(PublishedEdition(Mutex::new(None)));
    let app = daily_topics_routes(Arc::new(GetDailyTopics::new(repository.clone()))).layer(
        from_fn_with_state(
            HttpResponseCache::new(Duration::from_secs(600)),
            cache_get_responses,
        ),
    );
    let (_, initial) = get(app.clone(), "/api/v1/daily-topics").await;
    assert_eq!(initial["items"], json!([]));
    *repository.0.lock().unwrap() = Some(edition());
    let (status, published) = get(app, "/api/v1/daily-topics").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(published["items"].as_array().unwrap().len(), 5);
    assert_eq!(published["items"][0]["audio"]["status"], "ready");
}

#[tokio::test]
async fn topics_expose_heat_without_counting_passive_reads() {
    let app = app(Some(edition()), false);
    let (_, home) = get(app.clone(), "/api/v1/daily-topics").await;
    let (_, history) = get(app.clone(), "/api/v1/daily-topics/history").await;
    let id = home["items"][0]["id"].as_str().unwrap();
    let (_, detail) = get(app, &format!("/api/v1/daily-topics/{id}")).await;
    assert_eq!(home["items"][0]["heat"], 0);
    assert_eq!(history["items"][0]["heat"], 0);
    assert_eq!(detail["heat"], 0);
}

#[tokio::test]
async fn visits_reject_invalid_visitors_and_unpublished_topics() {
    for (id, visitor, expected) in [
        ("known", "invalid", StatusCode::BAD_REQUEST),
        (
            "missing",
            "550e8400-e29b-41d4-a716-446655440000",
            StatusCode::NOT_FOUND,
        ),
    ] {
        let response = app(Some(edition()), false)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/daily-topics/{id}/visits"))
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"visitor_id":visitor}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
}

#[tokio::test]
async fn heat_failure_does_not_block_home_history_or_detail_content() {
    let app = daily_topics_routes(Arc::new(GetDailyTopics::new(Arc::new(Repository {
        edition: Some(edition()),
        fail: false,
        heat_fail: true,
    }))));
    for url in [
        "/api/v1/daily-topics",
        "/api/v1/daily-topics/history",
        "/api/v1/daily-topics/0000000000000000000000000000000000000000000000000000000000000000",
    ] {
        let (status, body) = get(app.clone(), url).await;
        assert_eq!(status, StatusCode::OK);
        let topic = if body.get("items").is_some() {
            &body["items"][0]
        } else {
            &body
        };
        assert_eq!(topic["heat"], 0);
        assert_eq!(topic["paragraphs"][0], "第一段");
    }
}
