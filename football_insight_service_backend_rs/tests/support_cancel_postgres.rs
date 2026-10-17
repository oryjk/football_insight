//! Real cancellation SQL against an explicit disposable local test database.
use football_insight_service_backend_rs::support::{
    adapters::persistence::postgres_support_repository::PostgresSupportRepository,
    ports::support_repository::SupportRepository,
};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires SUPPORT_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn cancel_only_own_match_vote_and_allow_reselection() {
    let url = std::env::var("SUPPORT_TEST_DATABASE_URL").expect("explicit test database required");
    let parsed = url::Url::parse(&url).unwrap();
    assert!(matches!(
        parsed.host_str(),
        Some("localhost" | "127.0.0.1" | "[::1]")
    ));
    assert!(parsed.path().ends_with("_test"));
    let admin = PgPool::connect(&url).await.unwrap();
    let schema = format!("support_cancel_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let options = url
        .parse::<PgConnectOptions>()
        .unwrap()
        .options([("search_path", schema.as_str())]);
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::raw_sql("CREATE TABLE f_i_match_support_votes (id UUID PRIMARY KEY, match_id BIGINT NOT NULL, user_id UUID NOT NULL, supported_team_id BIGINT NOT NULL, UNIQUE(match_id,user_id))")
        .execute(&pool).await.unwrap();
    let repository = PostgresSupportRepository::new(pool.clone());
    let actor = Uuid::new_v4();
    let other = Uuid::new_v4();
    repository.create_vote(actor, 1, 11).await.unwrap();
    repository.create_vote(other, 1, 22).await.unwrap();
    repository.create_vote(actor, 2, 11).await.unwrap();
    repository.remove_vote(actor, 1).await.unwrap();
    repository.remove_vote(actor, 1).await.unwrap();
    let remaining: Vec<(i64, Uuid, i64)> = sqlx::query_as(
        "SELECT match_id,user_id,supported_team_id FROM f_i_match_support_votes ORDER BY match_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(remaining, vec![(1, other, 22), (2, actor, 11)]);
    repository.create_vote(actor, 1, 22).await.unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM f_i_match_support_votes WHERE match_id=1 AND supported_team_id=22",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 2);
    assert!(repository.create_vote(actor, 1, 11).await.is_err());
    pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
    admin.close().await;
}

#[tokio::test]
#[ignore = "requires SUPPORT_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn match_support_totals_add_operator_base_to_real_votes() {
    let url = std::env::var("SUPPORT_TEST_DATABASE_URL").expect("explicit test database required");
    let parsed = url::Url::parse(&url).unwrap();
    assert!(matches!(
        parsed.host_str(),
        Some("localhost" | "127.0.0.1" | "[::1]")
    ));
    assert!(parsed.path().ends_with("_test"));
    let admin = PgPool::connect(&url).await.unwrap();
    let schema = format!("support_base_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let options = url
        .parse::<PgConnectOptions>()
        .unwrap()
        .options([("search_path", schema.as_str())]);
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::raw_sql(
        r#"
        CREATE TABLE f_i_teams (team_id BIGINT PRIMARY KEY, avatar_storage_url TEXT NULL);
        CREATE TABLE f_i_users (id UUID PRIMARY KEY, favorite_team_id BIGINT NULL);
        CREATE TABLE f_i_matches (
            match_id BIGINT PRIMARY KEY, season INT NOT NULL, round_number INT NOT NULL,
            match_date DATE NOT NULL, match_time TEXT NOT NULL, status TEXT NOT NULL,
            source_active BOOLEAN NOT NULL DEFAULT TRUE,
            home_team_id BIGINT NOT NULL, home_team_name TEXT NOT NULL, home_score TEXT NOT NULL DEFAULT '',
            away_team_id BIGINT NOT NULL, away_team_name TEXT NOT NULL, away_score TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE f_i_match_support_votes (id UUID PRIMARY KEY, match_id BIGINT NOT NULL, user_id UUID NOT NULL, supported_team_id BIGINT NOT NULL, UNIQUE(match_id,user_id));
        INSERT INTO f_i_teams(team_id) VALUES (11), (22);
        INSERT INTO f_i_matches(match_id, season, round_number, match_date, match_time, status, home_team_id, home_team_name, away_team_id, away_team_name)
            VALUES (1, 2026, 28, '2026-10-18', '20:00', 'scheduled', 11, '主队', 22, '客队'),
                   (2, 2026, 29, '2026-10-25', '20:00', 'scheduled', 11, '主队', 22, '客队');
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261017130000_add_match_support_bases.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    let repository = PostgresSupportRepository::new(pool.clone());
    let voter = Uuid::new_v4();
    repository.create_vote(voter, 1, 22).await.unwrap();
    sqlx::query("INSERT INTO f_i_match_support_bases(match_id, team_id, base_count) VALUES (1, 11, 2), (1, 22, 20)")
        .execute(&pool)
        .await
        .unwrap();

    let detail = repository.find_match_detail(1, None).await.unwrap().unwrap();
    assert_eq!(detail.home_team.support_count, 2);
    assert_eq!(detail.away_team.support_count, 21);
    assert_eq!(detail.total_support_count, 23);
    // A base on one match never leaks into another.
    let other = repository.find_match_detail(2, None).await.unwrap().unwrap();
    assert_eq!(other.total_support_count, 0);

    pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
    admin.close().await;
}
