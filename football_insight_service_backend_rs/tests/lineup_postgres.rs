//! Real lineup SQL against an explicit disposable local test database.
use football_insight_service_backend_rs::lineup::{
    adapters::persistence::PostgresLineupRepository,
    domain::LineupSlot,
    ports::{LineupRepository, StoreError},
};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::sync::Arc;
use uuid::Uuid;

struct Db {
    admin: PgPool,
    pool: PgPool,
    schema: String,
}

async fn setup() -> Db {
    let url = std::env::var("LINEUP_TEST_DATABASE_URL").expect("explicit test database required");
    let parsed = url::Url::parse(&url).unwrap();
    assert!(matches!(
        parsed.host_str(),
        Some("localhost" | "127.0.0.1" | "[::1]")
    ));
    assert!(parsed.path().ends_with("_test"));
    let admin = PgPool::connect(&url).await.unwrap();
    let schema = format!("lineup_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let options = url
        .parse::<PgConnectOptions>()
        .unwrap()
        .options([("search_path", schema.as_str())]);
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::raw_sql("CREATE TABLE f_i_users (id UUID PRIMARY KEY)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(
        "CREATE TABLE f_i_squad_players (team_id BIGINT NOT NULL, player_id BIGINT NOT NULL, active BOOLEAN NOT NULL DEFAULT FALSE, PRIMARY KEY (team_id, player_id))",
    )
    .execute(&pool)
    .await
    .unwrap();
    let migration = include_str!("../migrations/20261019130000_add_lineups.sql");
    sqlx::raw_sql(migration).execute(&pool).await.unwrap();
    Db {
        admin,
        pool,
        schema,
    }
}

async fn teardown(db: Db) {
    db.pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {} CASCADE", db.schema))
        .execute(&db.admin)
        .await
        .unwrap();
    db.admin.close().await;
}

async fn add_user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO f_i_users (id) VALUES ($1)")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    id
}

fn slots(first: Option<i64>) -> Vec<LineupSlot> {
    (0..11u8)
        .map(|i| LineupSlot {
            slot: i,
            player_id: if i == 0 { first } else { None },
            x: 0.5,
            y: 0.5,
        })
        .collect()
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn saving_twice_overwrites_one_row_and_keeps_share_code() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let user = add_user(&db.pool).await;
    let first = repo
        .upsert(user, 1, "4-3-3", &slots(Some(5)), "AAAAAAAA")
        .await
        .unwrap();
    let second = repo
        .upsert(user, 1, "4-4-2", &slots(Some(6)), "BBBBBBBB")
        .await
        .unwrap();
    assert_eq!(first.share_code, "AAAAAAAA");
    assert_eq!(second.share_code, "AAAAAAAA");
    assert_eq!(second.formation, "4-4-2");
    assert_eq!(second.slots[0].player_id, Some(6));
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_lineups")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(rows, 1);
    assert_eq!(
        repo.find(user, 1).await.unwrap().unwrap().formation,
        "4-4-2"
    );
    assert!(repo.find(user, 2).await.unwrap().is_none());
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn one_user_can_keep_one_lineup_per_team_and_users_do_not_collide() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let (a, b) = (add_user(&db.pool).await, add_user(&db.pool).await);
    repo.upsert(a, 1, "4-3-3", &slots(None), "CODE0001")
        .await
        .unwrap();
    repo.upsert(a, 2, "4-4-2", &slots(None), "CODE0002")
        .await
        .unwrap();
    repo.upsert(b, 1, "3-5-2", &slots(None), "CODE0003")
        .await
        .unwrap();
    assert_eq!(repo.find(a, 1).await.unwrap().unwrap().formation, "4-3-3");
    assert_eq!(repo.find(a, 2).await.unwrap().unwrap().formation, "4-4-2");
    assert_eq!(repo.find(b, 1).await.unwrap().unwrap().formation, "3-5-2");
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn share_code_collision_is_reported_not_swallowed() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let (a, b) = (add_user(&db.pool).await, add_user(&db.pool).await);
    repo.upsert(a, 1, "4-3-3", &slots(None), "SAMECODE")
        .await
        .unwrap();
    let err = repo
        .upsert(b, 1, "4-3-3", &slots(None), "SAMECODE")
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::ShareCodeTaken);
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn concurrent_first_saves_keep_one_row_and_one_share_code() {
    let db = setup().await;
    let repo = Arc::new(PostgresLineupRepository::new(db.pool.clone()));
    let user = add_user(&db.pool).await;
    let mut tasks = Vec::new();
    for i in 0..6 {
        let repo = repo.clone();
        tasks.push(tokio::spawn(async move {
            repo.upsert(user, 1, "4-3-3", &slots(Some(i)), &format!("CONC000{i}"))
                .await
        }));
    }
    let mut codes = std::collections::HashSet::new();
    for t in tasks {
        let lineup = t.await.unwrap().expect("no 500 on concurrent first save");
        codes.insert(lineup.share_code);
    }
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_lineups")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(rows, 1);
    assert_eq!(
        codes.len(),
        1,
        "every save returns the single stored share_code"
    );
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn squad_members_include_former_players_and_exclude_other_teams() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    sqlx::raw_sql(
        "INSERT INTO f_i_squad_players (team_id, player_id, active) VALUES (1, 10, TRUE), (1, 11, FALSE), (2, 12, TRUE)",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    let mut found = repo.squad_members(1, &[10, 11, 12, 99]).await.unwrap();
    found.sort();
    assert_eq!(found, vec![10, 11]);
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn deleting_the_user_cascades_to_the_lineup() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let user = add_user(&db.pool).await;
    repo.upsert(user, 1, "4-3-3", &slots(None), "CASCADE1")
        .await
        .unwrap();
    sqlx::query("DELETE FROM f_i_users WHERE id = $1")
        .bind(user)
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(repo.find(user, 1).await.unwrap().is_none());
    teardown(db).await;
}
