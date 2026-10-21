//! 只在显式配置的一次性本地数据库上运行：TRANSFER_TEST_DATABASE_URL（localhost，库名以 _test 结尾）。
use chrono::{TimeZone, Utc};
use football_insight_service_backend_rs::transfer::{
    adapters::persistence::PostgresTransferRepository,
    domain::{TransferError, TransferIntent, VoteValue},
    ports::{NominateCommand, TransferCandidateDirectory, TransferRepository, TransferUserAccess},
};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use uuid::Uuid;

struct Db {
    admin: PgPool,
    pool: PgPool,
    schema: String,
}

async fn setup() -> Db {
    let url =
        std::env::var("TRANSFER_TEST_DATABASE_URL").expect("explicit disposable database required");
    let parsed = url::Url::parse(&url).unwrap();
    assert!(matches!(
        parsed.host_str(),
        Some("localhost" | "127.0.0.1" | "[::1]")
    ));
    assert!(parsed.path().ends_with("_test"));
    let admin = PgPool::connect(&url).await.unwrap();
    let schema = format!("transfer_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let options = url
        .parse::<PgConnectOptions>()
        .unwrap()
        .options([("search_path", schema.as_str())]);
    let pool = PgPoolOptions::new()
        .max_connections(12)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::raw_sql("CREATE TABLE f_i_users (id UUID PRIMARY KEY, status TEXT NOT NULL DEFAULT 'active', favorite_team_id BIGINT);
        CREATE TABLE f_i_teams (team_id BIGINT PRIMARY KEY, team_name TEXT NOT NULL, avatar_storage_url TEXT);
        INSERT INTO f_i_teams (team_id, team_name) VALUES (1, 'Alpha'), (2, 'Bravo'), (3, 'Charlie');
        CREATE TABLE f_i_squad_players (team_id BIGINT NOT NULL, player_id BIGINT NOT NULL, player_name TEXT NOT NULL,
            shirt_number INT, position TEXT, avatar_storage_url TEXT, last_season INT NOT NULL, active BOOLEAN NOT NULL, PRIMARY KEY (team_id, player_id));
        -- 10/11/12 本队现役；20 本队旧将现于 Bravo；30 Bravo 现役；31 Charlie 现役
        INSERT INTO f_i_squad_players VALUES
            (1, 10, 'Ten', 10, 'FW', NULL, 2026, true), (1, 11, 'Eleven', 11, 'MF', NULL, 2026, true),
            (1, 12, 'Twelve', 12, 'DF', NULL, 2026, true), (1, 20, 'Twenty', 20, 'GK', NULL, 2025, false),
            (2, 20, 'Twenty', 1, 'GK', NULL, 2026, true), (2, 30, 'Thirty', 9, 'FW', NULL, 2026, true),
            (3, 31, 'ThirtyOne', 7, 'MF', NULL, 2026, true);")
        .execute(&pool).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261020120000_add_transfer_votes.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261021120000_add_team_coaches.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
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

async fn fan(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO f_i_users (id, favorite_team_id) VALUES ($1, 1)")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    id
}

fn command(user: Uuid, player: i64, intent: TransferIntent, code: &str) -> NominateCommand {
    NominateCommand {
        window_id: 1,
        team_id: 1,
        player_id: player,
        intent,
        user_id: user,
        share_code: code.into(),
        limit: 3,
    }
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn windows_resolve_open_then_latest_closed() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let inside = Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap();
    let after = Utc.with_ymd_and_hms(2027, 4, 1, 0, 0, 0).unwrap();
    let before = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(
        repo.current_window(inside).await.unwrap().unwrap().code,
        "2026-winter"
    );
    assert_eq!(
        repo.current_window(after).await.unwrap().unwrap().code,
        "2026-winter"
    );
    assert!(repo.current_window(before).await.unwrap().is_none());
    assert!(repo.window_by_code("nope").await.unwrap().is_none());
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn nominating_creates_once_votes_up_and_ranks_by_net_support() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let (a, b, c) = (
        fan(&db.pool).await,
        fan(&db.pool).await,
        fan(&db.pool).await,
    );
    let first = repo
        .nominate(command(a, 10, TransferIntent::Out, "OUT00010"))
        .await
        .unwrap();
    assert!(first.created);
    let again = repo
        .nominate(command(b, 10, TransferIntent::Out, "OUT99999"))
        .await
        .unwrap();
    assert_eq!(
        (again.created, again.share_code.as_str()),
        (false, "OUT00010")
    );
    repo.nominate(command(c, 11, TransferIntent::Out, "OUT00011"))
        .await
        .unwrap();
    repo.set_vote("OUT00011", a, Some(VoteValue::Up))
        .await
        .unwrap();
    repo.set_vote("OUT00010", c, Some(VoteValue::Down))
        .await
        .unwrap();
    // OUT00010: up 2 down 1 → net 1；OUT00011: up 2 down 0 → net 2
    let (items, total) = repo
        .board(1, 1, TransferIntent::Out, 0, 20, Some(c))
        .await
        .unwrap();
    assert_eq!(total, 2);
    assert_eq!(
        items
            .iter()
            .map(|i| (i.share_code.as_str(), i.up, i.down, i.rank, i.my_vote))
            .collect::<Vec<_>>(),
        vec![
            ("OUT00011", 2, 0, Some(1), 1),
            ("OUT00010", 2, 1, Some(2), -1)
        ]
    );
    let guest = repo
        .board(1, 1, TransferIntent::Out, 0, 20, None)
        .await
        .unwrap()
        .0;
    assert!(guest.iter().all(|i| i.my_vote == 0));
    let tally = repo.set_vote("OUT00010", c, None).await.unwrap();
    assert_eq!((tally.up, tally.down, tally.my_vote), (2, 0, 0));
    let context = repo.card("OUT00010", Some(a)).await.unwrap().unwrap();
    assert_eq!(
        (
            context.team.team_name.as_str(),
            context.window.code.as_str(),
            context.card.my_vote
        ),
        ("Alpha", "2026-winter", 1)
    );
    assert_eq!(context.card.player.name, "Ten");
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn nomination_limit_counts_only_new_cards_per_intent() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let (a, b) = (fan(&db.pool).await, fan(&db.pool).await);
    repo.nominate(command(b, 12, TransferIntent::Out, "BBBBBB12"))
        .await
        .unwrap();
    for (player, code) in [(10, "AAAAAA10"), (11, "AAAAAA11")] {
        repo.nominate(command(a, player, TransferIntent::Out, code))
            .await
            .unwrap();
    }
    // 为别人建的卡加票不占名额
    assert!(
        !repo
            .nominate(command(a, 12, TransferIntent::Out, "AAAAAA12"))
            .await
            .unwrap()
            .created
    );
    let mut limited = command(a, 10, TransferIntent::Stay, "STAY0010");
    limited.limit = 1;
    repo.nominate(limited.clone()).await.unwrap();
    limited.player_id = 11;
    limited.share_code = "STAY0011".into();
    assert_eq!(
        repo.nominate(limited).await.unwrap_err(),
        TransferError::NominationLimit
    );
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn concurrent_nominations_never_exceed_limit_or_duplicate_cards() {
    let db = setup().await;
    let a = fan(&db.pool).await;
    let mut tasks = Vec::new();
    for (index, player) in [10, 11, 12, 10, 11, 12, 10, 11].into_iter().enumerate() {
        let repo = PostgresTransferRepository::new(db.pool.clone());
        let mut cmd = command(a, player, TransferIntent::Out, &format!("CONC{index:04}"));
        cmd.limit = 2;
        tasks.push(tokio::spawn(async move { repo.nominate(cmd).await }));
    }
    for task in tasks {
        let _ = task.await.unwrap();
    }
    let cards: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_transfer_nominations")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    let votes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_transfer_votes")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(cards, 2);
    assert_eq!(votes, 2);
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn disabled_voters_are_not_counted_and_empty_cards_hide() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let (a, b) = (fan(&db.pool).await, fan(&db.pool).await);
    repo.nominate(command(a, 10, TransferIntent::Out, "HIDE0010"))
        .await
        .unwrap();
    repo.set_vote("HIDE0010", b, Some(VoteValue::Up))
        .await
        .unwrap();
    sqlx::query("UPDATE f_i_users SET status = 'disabled' WHERE id = $1")
        .bind(b)
        .execute(&db.pool)
        .await
        .unwrap();
    let card = repo.card("HIDE0010", None).await.unwrap().unwrap().card;
    assert_eq!((card.up, card.down), (1, 0));
    assert_eq!(
        repo.set_vote("HIDE0010", b, None).await.unwrap_err(),
        TransferError::Unauthorized
    );
    repo.set_vote("HIDE0010", a, None).await.unwrap();
    assert_eq!(
        repo.board(1, 1, TransferIntent::Out, 0, 20, None)
            .await
            .unwrap(),
        (Vec::new(), 0)
    );
    let hidden = repo.card("HIDE0010", None).await.unwrap().unwrap().card;
    assert_eq!((hidden.up, hidden.down, hidden.rank), (0, 0, None));
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn returning_departed_and_current_team_flags() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let a = fan(&db.pool).await;
    repo.nominate(command(a, 20, TransferIntent::In, "BACK0020"))
        .await
        .unwrap();
    repo.nominate(command(a, 30, TransferIntent::In, "BUY00030"))
        .await
        .unwrap();
    repo.nominate(command(a, 12, TransferIntent::Stay, "STAY0012"))
        .await
        .unwrap();
    let back = repo.card("BACK0020", None).await.unwrap().unwrap().card;
    assert!(back.returning);
    assert_eq!(
        back.player
            .current_team
            .as_ref()
            .map(|t| t.team_name.as_str()),
        Some("Bravo")
    );
    assert!(
        !repo
            .card("BUY00030", None)
            .await
            .unwrap()
            .unwrap()
            .card
            .returning
    );
    sqlx::query("UPDATE f_i_squad_players SET active = false WHERE team_id = 1 AND player_id = 12")
        .execute(&db.pool)
        .await
        .unwrap();
    let stay = repo.card("STAY0012", None).await.unwrap().unwrap().card;
    assert!(stay.player.departed);
    assert!(stay.player.current_team.is_none());
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn coach_cards_use_the_coach_table_and_mark_departure() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let coach: i64 = sqlx::query_scalar(
        "INSERT INTO f_i_team_coaches (coach_key, coach_name, team_id, avatar_storage_url)
         VALUES ('Coach A', '教练甲', 1, 'https://cdn/coach.jpg') RETURNING coach_id",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(coach, 9_000_000_001);
    // 同队同时只能有一位现任主教练。
    assert!(
        sqlx::query("INSERT INTO f_i_team_coaches (coach_key, coach_name, team_id) VALUES ('Coach B', '教练乙', 1)")
            .execute(&db.pool)
            .await
            .is_err()
    );
    assert_eq!(repo.coach_rows(coach).await.unwrap(), vec![(1, true)]);
    assert!(repo.coach_rows(9_000_000_999).await.unwrap().is_empty());
    let candidate = repo.team_coach(1).await.unwrap().unwrap();
    assert_eq!(
        (
            candidate.player_id,
            candidate.name.as_str(),
            candidate.position.clone(),
            candidate.shirt_number,
            candidate.avatar_url.as_deref(),
            candidate.current_team.as_ref().map(|t| t.team_id)
        ),
        (
            coach,
            "教练甲",
            None,
            None,
            Some("https://cdn/coach.jpg"),
            Some(1)
        )
    );
    assert!(repo.team_coach(2).await.unwrap().is_none());

    let a = fan(&db.pool).await;
    repo.nominate(command(a, coach, TransferIntent::Out, "COACHOUT"))
        .await
        .unwrap();
    let card = repo.card("COACHOUT", None).await.unwrap().unwrap().card;
    assert_eq!(card.player.name, "教练甲");
    assert_eq!(
        card.player.avatar_url.as_deref(),
        Some("https://cdn/coach.jpg")
    );
    assert_eq!(card.player.position, None);
    assert!(!card.player.departed);
    assert!(!card.returning);
    assert_eq!(
        card.player
            .current_team
            .as_ref()
            .map(|t| t.team_name.as_str()),
        Some("Alpha")
    );
    let (items, total) = repo
        .board(1, 1, TransferIntent::Out, 0, 10, None)
        .await
        .unwrap();
    assert_eq!((total, items[0].player.name.as_str()), (1, "教练甲"));

    // 主教练转去 Bravo：旧卡标记离队，现所在队跟着变。
    sqlx::query("UPDATE f_i_team_coaches SET team_id = 2 WHERE coach_id = $1")
        .bind(coach)
        .execute(&db.pool)
        .await
        .unwrap();
    let moved = repo.card("COACHOUT", None).await.unwrap().unwrap().card;
    assert!(moved.player.departed);
    assert_eq!(
        moved
            .player
            .current_team
            .as_ref()
            .map(|t| t.team_name.as_str()),
        Some("Bravo")
    );
    // 下课后不在任何队：仍保留姓名与头像，没有现所在队。
    sqlx::query("UPDATE f_i_team_coaches SET active = false WHERE coach_id = $1")
        .bind(coach)
        .execute(&db.pool)
        .await
        .unwrap();
    let gone = repo.card("COACHOUT", None).await.unwrap().unwrap().card;
    assert!(gone.player.departed);
    assert_eq!(gone.player.name, "教练甲");
    assert!(gone.player.current_team.is_none());
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn card_survives_player_removed_from_all_squads() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let a = fan(&db.pool).await;
    repo.nominate(command(a, 31, TransferIntent::In, "GONE0031"))
        .await
        .unwrap();
    sqlx::query("DELETE FROM f_i_squad_players WHERE player_id = 31")
        .execute(&db.pool)
        .await
        .unwrap();
    let card = repo.card("GONE0031", None).await.unwrap().unwrap().card;
    assert_eq!(
        (
            card.player.name.as_str(),
            card.player.avatar_url.clone(),
            card.player.current_team.clone()
        ),
        ("未知球员", None, None)
    );
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn candidate_directory_and_user_profile() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    assert_eq!(repo.find_team(2).await.unwrap().unwrap().team_name, "Bravo");
    assert!(repo.find_team(9).await.unwrap().is_none());
    let mut rows = repo.player_rows(20).await.unwrap();
    rows.sort();
    assert_eq!(rows, vec![(1, false), (2, true)]);
    assert_eq!(
        repo.team_players(1)
            .await
            .unwrap()
            .iter()
            .map(|p| p.player_id)
            .collect::<Vec<_>>(),
        vec![12, 11, 10]
    );
    let former = repo.former_players(1).await.unwrap();
    assert_eq!(
        (
            former[0].player_id,
            former[0].current_team.as_ref().unwrap().team_id
        ),
        (20, 2)
    );
    assert_eq!(
        repo.other_active_players(1)
            .await
            .unwrap()
            .iter()
            .map(|p| p.player_id)
            .collect::<Vec<_>>(),
        vec![30, 31]
    );
    let a = fan(&db.pool).await;
    let profile = repo.profile(a).await.unwrap().unwrap();
    assert!(profile.active);
    assert_eq!(profile.favorite_team_id, Some(1));
    assert!(repo.profile(Uuid::new_v4()).await.unwrap().is_none());
    teardown(db).await;
}
