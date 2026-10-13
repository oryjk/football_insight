//! Run with an explicitly configured disposable database:
//! DAILY_TOPICS_TEST_DATABASE_URL=... cargo test --test daily_topics_postgres -- --ignored
use std::sync::Arc;

use chrono::{DateTime, NaiveDate};
use football_insight_service_backend_rs::daily_topics::{
    adapters::persistence::postgres_daily_topics_repository::PostgresDailyTopicsRepository,
    application::get_daily_topics::GetDailyTopics,
    domain::query::DailyTopicsQuery,
    ports::daily_topics_repository::{DailyTopicsRepository, DailyTopicsRepositoryError},
};
use serde_json::{Value, json};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use uuid::Uuid;

struct TestDatabase {
    admin: PgPool,
    pool: PgPool,
    schema: String,
}

impl TestDatabase {
    async fn new() -> Self {
        let database_url = std::env::var("DAILY_TOPICS_TEST_DATABASE_URL")
            .expect("explicit disposable test database is required");
        let url = url::Url::parse(&database_url).unwrap();
        assert!(url.path().ends_with("_test"), "refusing non-test database");
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .unwrap();
        let schema = format!("topics_test_{}", Uuid::new_v4().simple());
        sqlx::raw_sql(&format!("CREATE SCHEMA {schema}"))
            .execute(&admin)
            .await
            .unwrap();
        let options = database_url
            .parse::<PgConnectOptions>()
            .unwrap()
            .options([("search_path", schema.as_str())]);
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await
            .unwrap();
        let migration = include_str!("../migrations/20261013000100_add_daily_topic_editions.sql");
        sqlx::raw_sql(migration).execute(&pool).await.unwrap();
        Self {
            admin,
            pool,
            schema,
        }
    }

    async fn insert(&self, id: &str, team_id: i64, published_at: &str, title: &str) {
        sqlx::query("INSERT INTO f_i_daily_topic_editions (id, team_id, edition_date, published_at, payload, source_fingerprint, generator_version) VALUES ($1,$2,$3,$4,$5,'fixture-fingerprint','test-only')")
            .bind(id).bind(team_id).bind(NaiveDate::from_ymd_opt(2026, 10, 13).unwrap())
            .bind(DateTime::parse_from_rfc3339(published_at).unwrap())
            .bind(payload(team_id, title)).execute(&self.pool).await.unwrap();
    }

    async fn close(self) {
        self.pool.close().await;
        sqlx::raw_sql(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .execute(&self.admin)
            .await
            .unwrap();
        self.admin.close().await;
    }
}

fn payload(team_id: i64, title: &str) -> Value {
    json!({"team_id":team_id,"edition_date":"2026-10-13","updated_at":"2026-10-13T08:30:00+08:00","items":[{
        "id":"ad0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "team_id":team_id,"title":title,"summary":"隔离测试摘要","paragraphs":["第一段","第二段"],
        "published_at":"2026-10-13T08:30:00+08:00",
        "sources":[{"title":"原始测试来源","url":"https://example.com/original","publisher":"测试媒体","published_at":"2026-10-12T20:00:00+08:00"}],
        "audio":{"status":"pending","url":null,"dialect":"sichuan","duration_seconds":null}
    }]})
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL database"]
async fn migrated_empty_database_returns_no_edition() {
    let database = TestDatabase::new().await;
    let repository = PostgresDailyTopicsRepository::new(database.pool.clone());
    assert!(repository.latest_for_team(77680).await.unwrap().is_none());
    database.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL database"]
async fn query_isolates_teams_and_selects_by_publication_time() {
    let database = TestDatabase::new().await;
    database
        .insert("z-old", 77680, "2026-10-13T07:00:00+08:00", "旧批次")
        .await;
    database
        .insert("a-new", 77680, "2026-10-13T08:30:00+08:00", "最新批次")
        .await;
    database
        .insert("other-team", 999, "2026-10-13T12:30:00+08:00", "另一球队")
        .await;
    let repository = PostgresDailyTopicsRepository::new(database.pool.clone());
    let result = repository.latest_for_team(77680).await.unwrap().unwrap();
    assert_eq!(result.items[0].title, "最新批次");
    assert_eq!(result.team_id, 77680);
    assert_eq!(result.items[0].team_id, 77680);
    assert_eq!(
        result.published_at.to_rfc3339(),
        "2026-10-13T08:30:00+08:00"
    );
    assert_eq!(
        repository
            .latest_for_team(999)
            .await
            .unwrap()
            .unwrap()
            .items[0]
            .title,
        "另一球队"
    );
    assert!(repository.latest_for_team(123).await.unwrap().is_none());
    database.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL database"]
async fn jsonb_round_trip_and_atomic_upsert_preserve_sources_and_audio_identity() {
    let database = TestDatabase::new().await;
    database
        .insert(
            "same-edition",
            77680,
            "2026-10-13T08:30:00+08:00",
            "测试文章",
        )
        .await;
    let repository = PostgresDailyTopicsRepository::new(database.pool.clone());
    let before = repository.latest_for_team(77680).await.unwrap().unwrap();
    assert_eq!(before.items[0].paragraphs, ["第一段", "第二段"]);
    assert_eq!(before.items[0].sources[0].publisher, "测试媒体");
    assert_eq!(
        before.items[0].sources[0]
            .published_at
            .unwrap()
            .to_rfc3339(),
        "2026-10-12T20:00:00+08:00"
    );
    let mut updated = payload(77680, "测试文章");
    updated["items"][0]["audio"] = json!({"status":"ready","url":"https://example.com/immutable-article/audio-hash.m4a","dialect":"sichuan","duration_seconds":60});
    sqlx::query("INSERT INTO f_i_daily_topic_editions (id, team_id, edition_date, published_at, payload, source_fingerprint, generator_version) VALUES ('same-edition',77680,'2026-10-13','2026-10-13T00:30:00Z',$1,'fixture-fingerprint','test-only') ON CONFLICT(id) DO UPDATE SET payload=EXCLUDED.payload")
        .bind(updated).execute(&database.pool).await.unwrap();
    let after = repository.latest_for_team(77680).await.unwrap().unwrap();
    assert_eq!(after.items[0].id, before.items[0].id);
    assert_eq!(
        after.items[0].audio.url.as_deref(),
        Some("https://example.com/immutable-article/audio-hash.m4a")
    );
    assert_eq!(after.items[0].audio.duration_seconds, Some(60.0));
    database.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL database"]
async fn broken_jsonb_or_database_failure_never_becomes_an_empty_list() {
    let database = TestDatabase::new().await;
    database
        .insert("broken", 77680, "2026-10-13T08:30:00+08:00", "测试文章")
        .await;
    sqlx::query("UPDATE f_i_daily_topic_editions SET payload='{}'")
        .execute(&database.pool)
        .await
        .unwrap();
    let repository = PostgresDailyTopicsRepository::new(database.pool.clone());
    assert!(matches!(
        repository.latest_for_team(77680).await,
        Err(DailyTopicsRepositoryError::InvalidEdition)
    ));
    sqlx::raw_sql("DROP TABLE f_i_daily_topic_editions")
        .execute(&database.pool)
        .await
        .unwrap();
    assert!(matches!(
        repository.latest_for_team(77680).await,
        Err(DailyTopicsRepositoryError::Unavailable)
    ));
    database.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL database"]
async fn recent_articles_respect_limit_after_deduplication() {
    let database = TestDatabase::new().await;
    let mut value = payload(77680, "测试文章");
    let item = value["items"][0].clone();
    value["items"] = Value::Array(
        (0..7)
            .map(|index| {
                let mut value = item.clone();
                value["id"] = json!(format!("{index:064x}"));
                value
            })
            .collect(),
    );
    database
        .insert("batch", 77680, "2026-10-13T08:30:00+08:00", "测试文章")
        .await;
    sqlx::query("UPDATE f_i_daily_topic_editions SET payload=$1")
        .bind(value)
        .execute(&database.pool)
        .await
        .unwrap();
    let query = GetDailyTopics::new(Arc::new(PostgresDailyTopicsRepository::new(
        database.pool.clone(),
    )));
    assert_eq!(
        query
            .execute(DailyTopicsQuery::new(77680, 5).unwrap())
            .await
            .unwrap()
            .items
            .len(),
        5
    );
    assert_eq!(
        query
            .execute(DailyTopicsQuery::new(77680, 2).unwrap())
            .await
            .unwrap()
            .items
            .len(),
        2
    );
    database.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL database"]
async fn archive_pages_span_dates_and_choose_newest_version_without_repeats() {
    use football_insight_service_backend_rs::daily_topics::domain::archive::{
        TopicDetailQuery, TopicHistoryQuery,
    };
    let database = TestDatabase::new().await;
    let make_topic = |id: &str, time: &str, title: &str| {
        let mut item = payload(77680, title)["items"][0].clone();
        item["id"] = json!(id);
        item["published_at"] = json!(time);
        item
    };
    let a = "a".repeat(64);
    let b = "b".repeat(64);
    let c = "c".repeat(64);
    let d = "d".repeat(64);
    let old = make_topic(&a, "2026-10-12T09:00:00+08:00", "旧版本");
    let updated = make_topic(&a, "2026-10-12T09:00:00+08:00", "更新版本");
    database
        .insert("older", 77680, "2026-10-12T12:00:00+08:00", "旧批次")
        .await;
    database
        .insert("newer", 77680, "2026-10-13T08:00:00+08:00", "新批次")
        .await;
    database
        .insert("other", 999, "2026-10-13T12:00:00+08:00", "另一球队")
        .await;
    sqlx::query("UPDATE f_i_daily_topic_editions SET payload=$1 WHERE id='older'")
        .bind(json!({"items":[old,make_topic(&c,"2026-10-12T10:00:00+08:00","较新旧文"),make_topic(&d,"2026-10-12T09:00:00+08:00","同时间旧文")]}))
        .execute(&database.pool).await.unwrap();
    sqlx::query("UPDATE f_i_daily_topic_editions SET payload=$1 WHERE id='newer'")
        .bind(json!({"items":[updated,make_topic(&b,"2026-10-13T08:00:00+08:00","最新文章")]}))
        .execute(&database.pool)
        .await
        .unwrap();
    let use_case = GetDailyTopics::new(Arc::new(PostgresDailyTopicsRepository::new(
        database.pool.clone(),
    )));
    let recent = use_case
        .execute(DailyTopicsQuery::new(77680, 5).unwrap())
        .await
        .unwrap();
    assert_eq!(
        recent
            .items
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec![b.as_str(), c.as_str(), a.as_str(), d.as_str()]
    );
    assert_eq!(recent.items[2].title, "更新版本");
    let first = use_case
        .history(TopicHistoryQuery::new(77680, 2, None).unwrap())
        .await
        .unwrap();
    let second = use_case
        .history(TopicHistoryQuery::new(77680, 2, first.next.clone()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        first
            .items
            .iter()
            .chain(&second.items)
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec![b.as_str(), c.as_str(), a.as_str(), d.as_str()]
    );
    assert!(second.next.is_none());
    let mut cursor = None;
    let mut ids = Vec::new();
    for _ in 0..4 {
        let page = use_case
            .history(TopicHistoryQuery::new(77680, 1, cursor).unwrap())
            .await
            .unwrap();
        ids.push(page.items[0].id.clone());
        cursor = page.next;
    }
    assert_eq!(ids, vec![b.clone(), c.clone(), a.clone(), d.clone()]);
    assert!(cursor.is_none());

    assert_eq!(
        use_case
            .detail(TopicDetailQuery::new(77680, a.clone()).unwrap())
            .await
            .unwrap()
            .unwrap()
            .title,
        "更新版本"
    );
    assert!(
        use_case
            .detail(TopicDetailQuery::new(999, a).unwrap())
            .await
            .unwrap()
            .is_none()
    );
    database.close().await;
}
