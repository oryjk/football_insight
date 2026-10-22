//! Only run against an explicit disposable localhost database ending in _test.
use chrono::DateTime;
use football_insight_service_backend_rs::ticket_watch::{
    adapters::persistence::PostgresRestockProfileStore,
    ports::restock_profile_store::RestockProfileStore,
};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use uuid::Uuid;

async fn insert(pool: &PgPool, match_id: i64, since: &str, total: i64, version: i16) {
    // 与 ticket-monitor 批处理一致：北京墙钟时间按 Asia/Shanghai 转 timestamptz，幂等 upsert。
    sqlx::query(
        "INSERT INTO f_i_ticket_restock_profiles
            (match_id, schema_version, stat_unit, since_at, total_events, appear_events, up_events, seats_with_events, hourly)
         VALUES ($1, $2, 'seat_restock_event', ($3::timestamp AT TIME ZONE 'Asia/Shanghai'), $4, $4, 0, LEAST($4, 1),
            ARRAY[0,0,0,0,0,0,0,0,0,$4,0,0,0,0,0,0,0,0,0,0,0,0,0,0]::BIGINT[])
         ON CONFLICT (match_id) DO UPDATE SET total_events = EXCLUDED.total_events",
    )
    .bind(match_id).bind(version).bind(since).bind(total)
    .execute(pool).await.unwrap();
}

#[tokio::test]
#[ignore = "requires explicit disposable localhost PostgreSQL database"]
async fn restock_store_matches_since_schema_and_prefers_first_non_empty_id() {
    let url = std::env::var("RESTOCK_TEST_DATABASE_URL")
        .expect("explicit disposable test database URL required");
    let parsed = url::Url::parse(&url).unwrap();
    assert!(matches!(
        parsed.host_str(),
        Some("localhost" | "127.0.0.1" | "[::1]")
    ));
    assert!(parsed.path().ends_with("_test"));
    let admin = PgPool::connect(&url).await.unwrap();
    let schema = format!("restock_{}", Uuid::new_v4().simple());
    sqlx::raw_sql(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let options = url
        .parse::<PgConnectOptions>()
        .unwrap()
        .options([("search_path", schema.as_str())]);
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261022160000_add_ticket_restock_profiles.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    let store = PostgresRestockProfileStore::new(pool.clone());
    let since = DateTime::parse_from_rfc3339("2026-10-08T14:10:00+08:00").unwrap();
    assert!(store.load(&[583], since).await.unwrap().is_none());

    insert(&pool, 583, "2026-10-08 14:10:00", 0, 2).await;
    insert(&pool, 9583, "2026-10-08 14:10:00", 40, 2).await;
    let profile = store.load(&[583, 9583], since).await.unwrap().unwrap();
    assert_eq!((profile.total_events, profile.hourly[9]), (40, 40));
    // 只有空快照时返回空快照，而不是 None。
    assert_eq!(
        store
            .load(&[583], since)
            .await
            .unwrap()
            .unwrap()
            .total_events,
        0
    );
    // 统计起点不一致、schema 版本不匹配都视为没有快照。
    let other = DateTime::parse_from_rfc3339("2026-10-08T14:20:00+08:00").unwrap();
    assert!(store.load(&[9583], other).await.unwrap().is_none());
    insert(&pool, 7, "2026-10-08 14:10:00", 30, 1).await;
    assert!(store.load(&[7], since).await.unwrap().is_none());
    // 表约束：24 个小时桶、appear+up=total。
    assert!(sqlx::query("UPDATE f_i_ticket_restock_profiles SET hourly = ARRAY[1]::BIGINT[] WHERE match_id = 583")
        .execute(&pool).await.is_err());

    pool.close().await;
    sqlx::raw_sql(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
    admin.close().await;
}
