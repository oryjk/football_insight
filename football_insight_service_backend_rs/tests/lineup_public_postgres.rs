//! Public SQL contracts are exercised only in an explicitly configured disposable local database.
use async_trait::async_trait;
use football_insight_service_backend_rs::lineup::{
    adapters::{
        persistence::{PostgresLineupRepository, PostgresSquadRepository},
        public_persistence::PostgresPublicLineupRepository,
    },
    application::public_lineup::PublicLineupService,
    domain::{LineupCodeImage, LineupPublicError, LineupReaction, LineupSlot, PublicLineupQuery},
    ports::{LineupCodePort, LineupRepository, PublicLineupRepository},
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
    let url =
        std::env::var("LINEUP_TEST_DATABASE_URL").expect("explicit disposable database required");
    let parsed = url::Url::parse(&url).unwrap();
    assert!(matches!(
        parsed.host_str(),
        Some("localhost" | "127.0.0.1" | "[::1]")
    ));
    assert!(parsed.path().ends_with("_test"));
    let admin = PgPool::connect(&url).await.unwrap();
    let schema = format!("lineup_public_{}", Uuid::new_v4().simple());
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
    sqlx::raw_sql("CREATE TABLE f_i_users (id UUID PRIMARY KEY, status TEXT NOT NULL DEFAULT 'active', display_name TEXT, avatar_url TEXT);
        CREATE TABLE f_i_teams (team_id BIGINT PRIMARY KEY, team_name TEXT NOT NULL, avatar_storage_url TEXT);
        INSERT INTO f_i_teams (team_id,team_name) VALUES (1,'Team One'),(2,'Team Two');
        CREATE TABLE f_i_squad_players (team_id BIGINT NOT NULL, player_id BIGINT NOT NULL, player_name TEXT NOT NULL,
            shirt_number INT, position TEXT, avatar_storage_url TEXT, last_season INT NOT NULL, active BOOLEAN NOT NULL, PRIMARY KEY(team_id,player_id));
        INSERT INTO f_i_squad_players(team_id,player_id,player_name,shirt_number,position,last_season,active)
        SELECT 1, i, 'Player ' || i, i::INT, 'MF', 2026, i <> 11 FROM generate_series(1,12) i;")
        .execute(&pool).await.unwrap();
    sqlx::raw_sql(include_str!("../migrations/20261019130000_add_lineups.sql"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261019140000_add_lineup_likes.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261019150000_add_lineup_drafts.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261019160000_add_lineup_dislikes.sql"
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
async fn user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO f_i_users(id,display_name) VALUES($1,'Public Author')")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    id
}
fn slots(full: bool) -> Vec<LineupSlot> {
    (0..11)
        .map(|slot| LineupSlot {
            slot,
            player_id: (full || slot != 10).then_some(i64::from(slot) + 1),
            x: 0.5,
            y: 0.5,
        })
        .collect()
}
async fn save(pool: &PgPool, actor: Uuid, team: i64, code: &str, full: bool) {
    PostgresLineupRepository::new(pool.clone())
        .upsert(actor, team, "4-3-3", &slots(full), code)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn reactions_switch_cancel_and_remain_private_and_idempotent() {
    let db = setup().await;
    let author = user(&db.pool).await;
    let actor = user(&db.pool).await;
    let other = user(&db.pool).await;
    save(&db.pool, author, 1, "REACT001", true).await;
    let repo = PostgresPublicLineupRepository::new(db.pool.clone());
    for (reaction, count, liked, disliked) in [
        (Some(LineupReaction::Like), 1, true, false),
        (Some(LineupReaction::Like), 1, true, false),
        (Some(LineupReaction::Dislike), 0, false, true),
        (Some(LineupReaction::Dislike), 0, false, true),
        (None, 0, false, false),
        (Some(LineupReaction::Dislike), 0, false, true),
        (Some(LineupReaction::Like), 1, true, false),
    ] {
        let state = repo
            .set_reaction("REACT001", actor, reaction)
            .await
            .unwrap();
        assert_eq!(
            (state.likes_count, state.liked_by_me, state.disliked_by_me),
            (count, liked, disliked)
        );
    }
    repo.set_reaction("REACT001", other, Some(LineupReaction::Dislike))
        .await
        .unwrap();
    for (viewer, liked, disliked) in [
        (None, false, false),
        (Some(actor), true, false),
        (Some(other), false, true),
    ] {
        let detail = repo.shared("REACT001", viewer).await.unwrap().unwrap();
        assert_eq!(
            (
                detail.item.likes.likes_count,
                detail.item.likes.liked_by_me,
                detail.item.likes.disliked_by_me
            ),
            (1, liked, disliked)
        );
        let (items, _) = repo.list(1, query("likes", 1, 20), viewer).await.unwrap();
        assert_eq!(items[0].likes, detail.item.likes);
    }
    // A legacy production server inserting a like must also replace a newer dislike.
    sqlx::query("INSERT INTO f_i_lineup_likes(lineup_id,user_id) SELECT id,$1 FROM f_i_lineups WHERE share_code='REACT001' ON CONFLICT DO NOTHING")
        .bind(other).execute(&db.pool).await.unwrap();
    let detail = repo.shared("REACT001", Some(other)).await.unwrap().unwrap();
    assert_eq!(detail.item.likes.likes_count, 2);
    assert!(detail.item.likes.liked_by_me);
    assert!(!detail.item.likes.disliked_by_me);
    sqlx::query("UPDATE f_i_users SET status='disabled' WHERE id=$1")
        .bind(other)
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(
        repo.set_reaction("REACT001", other, Some(LineupReaction::Dislike))
            .await
            .unwrap_err(),
        LineupPublicError::Unauthorized
    );
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn concurrent_opposing_reactions_never_leave_both_votes() {
    let db = setup().await;
    let author = user(&db.pool).await;
    let actor = user(&db.pool).await;
    save(&db.pool, author, 1, "REACT002", true).await;
    let mut tasks = Vec::new();
    for index in 0..16 {
        let repo = PostgresPublicLineupRepository::new(db.pool.clone());
        tasks.push(tokio::spawn(async move {
            let reaction = if index % 2 == 0 {
                LineupReaction::Like
            } else {
                LineupReaction::Dislike
            };
            let state = repo
                .set_reaction("REACT002", actor, Some(reaction))
                .await
                .unwrap();
            assert_ne!(state.liked_by_me, state.disliked_by_me);
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    let rows: i64 = sqlx::query_scalar("SELECT (SELECT COUNT(*) FROM f_i_lineup_likes) + (SELECT COUNT(*) FROM f_i_lineup_dislikes)")
        .fetch_one(&db.pool).await.unwrap();
    assert_eq!(rows, 1);
    teardown(db).await;
}
fn query(sort: &str, page: u32, size: u32) -> PublicLineupQuery {
    PublicLineupQuery::parse(sort, page, size).unwrap()
}
struct Codes;
#[async_trait]
impl LineupCodePort for Codes {
    async fn image(&self, _: &str) -> Result<LineupCodeImage, LineupPublicError> {
        panic!("no upstream calls in DB tests")
    }
}
fn service(pool: &PgPool) -> PublicLineupService {
    PublicLineupService::new(
        Arc::new(PostgresSquadRepository::new(pool.clone())),
        Arc::new(PostgresPublicLineupRepository::new(pool.clone())),
        Arc::new(PostgresLineupRepository::new(pool.clone())),
        Arc::new(Codes),
    )
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn public_visibility_filters_full_eleven_active_authors_before_pagination() {
    let db = setup().await;
    let repo = PostgresPublicLineupRepository::new(db.pool.clone());
    for (code, full, disabled) in [
        ("FULL0001", true, false),
        ("PART0001", false, false),
        ("HIDE0001", true, true),
    ] {
        let author = user(&db.pool).await;
        save(&db.pool, author, 1, code, full).await;
        if disabled {
            sqlx::query("UPDATE f_i_users SET status='disabled' WHERE id=$1")
                .bind(author)
                .execute(&db.pool)
                .await
                .unwrap();
        }
    }
    let (items, total) = repo.list(1, query("latest", 1, 1), None).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].share_code, "FULL0001");
    for code in ["PART0001", "HIDE0001", "MISSING1"] {
        assert!(repo.shared(code, None).await.unwrap().is_none());
    }
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn concurrent_put_and_delete_likes_are_idempotent() {
    let db = setup().await;
    let author = user(&db.pool).await;
    save(&db.pool, author, 1, "LIKE0001", true).await;
    let actor = user(&db.pool).await;
    let repo = Arc::new(PostgresPublicLineupRepository::new(db.pool.clone()));
    for liked in [true, false] {
        let mut tasks = vec![];
        for _ in 0..10 {
            let repo = repo.clone();
            tasks.push(tokio::spawn(async move {
                repo.set_like("LIKE0001", actor, liked).await
            }));
        }
        for task in tasks {
            let result = task.await.unwrap().unwrap();
            assert_eq!(result.likes_count, i64::from(liked));
            assert_eq!(result.liked_by_me, liked);
        }
    }
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_lineup_likes")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn disabled_likers_are_excluded_and_old_tokens_cannot_write() {
    let db = setup().await;
    let repo = PostgresPublicLineupRepository::new(db.pool.clone());
    let author = user(&db.pool).await;
    save(&db.pool, author, 1, "LIKE0002", true).await;
    let actor = user(&db.pool).await;
    assert_eq!(
        repo.set_like("LIKE0002", actor, true)
            .await
            .unwrap()
            .likes_count,
        1
    );
    sqlx::query("UPDATE f_i_users SET status='disabled' WHERE id=$1")
        .bind(actor)
        .execute(&db.pool)
        .await
        .unwrap();
    let shared = repo.shared("LIKE0002", Some(actor)).await.unwrap().unwrap();
    assert_eq!(shared.item.likes.likes_count, 0);
    assert!(!shared.item.likes.liked_by_me);
    assert_eq!(
        repo.set_like("LIKE0002", actor, true).await.unwrap_err(),
        LineupPublicError::Unauthorized
    );
    assert_eq!(
        repo.set_like("LIKE0002", actor, false).await.unwrap_err(),
        LineupPublicError::Unauthorized
    );
    assert_eq!(
        service(&db.pool)
            .shared("LIKE0002", Some(actor))
            .await
            .unwrap_err(),
        LineupPublicError::Unauthorized
    );
    assert_eq!(
        repo.set_like("LIKE0002", author, true)
            .await
            .unwrap()
            .likes_count,
        1,
        "self-like is allowed"
    );
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn partial_resave_hides_all_public_surfaces_preserving_code_creation_and_likes() {
    let db = setup().await;
    let repo = PostgresPublicLineupRepository::new(db.pool.clone());
    let author = user(&db.pool).await;
    save(&db.pool, author, 1, "KEEP0001", true).await;
    repo.set_like("KEEP0001", author, true).await.unwrap();
    let before = repo.shared("KEEP0001", None).await.unwrap().unwrap();
    save(&db.pool, author, 1, "IGNORED1", false).await;
    assert!(repo.shared("KEEP0001", None).await.unwrap().is_none());
    assert_eq!(
        repo.list(1, query("latest", 1, 20), None).await.unwrap().1,
        0
    );
    assert_eq!(
        repo.set_like("KEEP0001", author, false).await.unwrap_err(),
        LineupPublicError::NotFound
    );
    assert_eq!(
        service(&db.pool)
            .image("KEEP0001", author)
            .await
            .unwrap_err(),
        LineupPublicError::NotFound
    );
    save(&db.pool, author, 1, "IGNORED2", true).await;
    let after = repo
        .shared("KEEP0001", Some(author))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(before.item.created_at, after.item.created_at);
    assert_eq!(after.item.likes.likes_count, 1);
    assert!(after.item.likes.liked_by_me);
    assert!(after.item.updated_at >= before.item.updated_at);
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn latest_sort_uses_creation_and_likes_sort_has_stable_creation_and_id_ties() {
    let db = setup().await;
    let repo = PostgresPublicLineupRepository::new(db.pool.clone());
    let mut authors = vec![];
    for (i, code, date) in [
        (1, "SORT0001", "2026-10-01"),
        (2, "SORT0002", "2026-10-02"),
        (3, "SORT0003", "2026-10-02"),
    ] {
        let author = user(&db.pool).await;
        authors.push(author);
        save(&db.pool, author, 1, code, true).await;
        sqlx::query(
            "UPDATE f_i_lineups SET id=$1, created_at=$2::TEXT::TIMESTAMPTZ WHERE share_code=$3",
        )
        .bind(Uuid::from_u128(i))
        .bind(date)
        .bind(code)
        .execute(&db.pool)
        .await
        .unwrap();
    }
    save(&db.pool, authors[0], 1, "IGNORED1", true).await;
    let codes = |items: Vec<
        football_insight_service_backend_rs::lineup::domain::PublicLineupItem,
    >| items.into_iter().map(|i| i.share_code).collect::<Vec<_>>();
    assert_eq!(
        codes(repo.list(1, query("latest", 1, 20), None).await.unwrap().0),
        vec!["SORT0003", "SORT0002", "SORT0001"]
    );
    repo.set_like("SORT0001", authors[0], true).await.unwrap();
    assert_eq!(
        codes(repo.list(1, query("likes", 1, 20), None).await.unwrap().0),
        vec!["SORT0001", "SORT0003", "SORT0002"]
    );
    repo.set_like("SORT0002", authors[0], true).await.unwrap();
    repo.set_like("SORT0003", authors[0], true).await.unwrap();
    assert_eq!(
        codes(repo.list(1, query("likes", 1, 20), None).await.unwrap().0),
        vec!["SORT0003", "SORT0002", "SORT0001"]
    );
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn pagination_team_and_viewer_isolation_use_real_database_counts() {
    let db = setup().await;
    let repo = PostgresPublicLineupRepository::new(db.pool.clone());
    let viewer = user(&db.pool).await;
    let other = user(&db.pool).await;
    for code in ["PAGE0001", "PAGE0002", "PAGE0003"] {
        let author = user(&db.pool).await;
        save(&db.pool, author, 1, code, true).await;
    }
    save(&db.pool, viewer, 2, "TEAM0002", true).await;
    repo.set_like("PAGE0001", viewer, true).await.unwrap();
    let mut seen = std::collections::HashSet::new();
    for page in 1..=3 {
        let (items, total) = repo
            .list(1, query("latest", page, 1), Some(viewer))
            .await
            .unwrap();
        assert_eq!(total, 3);
        assert_eq!(items.len(), 1);
        assert!(seen.insert(items[0].share_code.clone()));
    }
    assert!(
        repo.list(1, query("latest", 4, 1), None)
            .await
            .unwrap()
            .0
            .is_empty()
    );
    assert_eq!(
        repo.list(2, query("latest", 1, 20), None).await.unwrap().1,
        1
    );
    assert!(
        repo.shared("PAGE0001", Some(viewer))
            .await
            .unwrap()
            .unwrap()
            .item
            .likes
            .liked_by_me
    );
    assert!(
        !repo
            .shared("PAGE0001", Some(other))
            .await
            .unwrap()
            .unwrap()
            .item
            .likes
            .liked_by_me
    );
    assert!(
        !repo
            .shared("PAGE0001", None)
            .await
            .unwrap()
            .unwrap()
            .item
            .likes
            .liked_by_me
    );
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn details_resolve_selected_current_and_former_players_and_sanitize_author() {
    let db = setup().await;
    let author = user(&db.pool).await;
    save(&db.pool, author, 1, "READ0001", true).await;
    sqlx::query("UPDATE f_i_users SET display_name=NULL WHERE id=$1")
        .bind(author)
        .execute(&db.pool)
        .await
        .unwrap();
    let detail = service(&db.pool).shared("READ0001", None).await.unwrap();
    assert_eq!(detail.team.team_id, 1);
    assert_eq!(detail.slots.len(), 11);
    assert_eq!(detail.players.len(), 11);
    assert!(
        !detail
            .players
            .iter()
            .find(|p| p.player_id == 11)
            .unwrap()
            .active
    );
    assert!(detail.players.iter().all(|p| p.player_id != 12));
    assert_eq!(detail.item.author.display_name, "球迷");
    assert_eq!(
        service(&db.pool)
            .list(99, "latest", 1, 20, None)
            .await
            .unwrap_err(),
        LineupPublicError::NotFound
    );
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn author_disable_blocks_likes_and_deleting_author_cascades_likes() {
    let db = setup().await;
    let repo = PostgresPublicLineupRepository::new(db.pool.clone());
    let author = user(&db.pool).await;
    let viewer = user(&db.pool).await;
    save(&db.pool, author, 1, "DEL00001", true).await;
    repo.set_like("DEL00001", viewer, true).await.unwrap();
    sqlx::query("UPDATE f_i_users SET status='disabled' WHERE id=$1")
        .bind(author)
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(repo.shared("DEL00001", None).await.unwrap().is_none());
    assert_eq!(
        repo.set_like("DEL00001", viewer, true).await.unwrap_err(),
        LineupPublicError::NotFound
    );
    assert_eq!(
        repo.set_like("DEL00001", viewer, false).await.unwrap_err(),
        LineupPublicError::NotFound
    );
    sqlx::query("DELETE FROM f_i_users WHERE id=$1")
        .bind(author)
        .execute(&db.pool)
        .await
        .unwrap();
    let likes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_lineup_likes")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(likes, 0);
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn missing_team_blocks_shared_reads_and_like_writes_consistently() {
    let db = setup().await;
    let repo = PostgresPublicLineupRepository::new(db.pool.clone());
    let author = user(&db.pool).await;
    save(&db.pool, author, 1, "TEAMDEL1", true).await;
    sqlx::query("DELETE FROM f_i_teams WHERE team_id=1")
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(repo.shared("TEAMDEL1", None).await.unwrap().is_none());
    assert_eq!(
        repo.set_like("TEAMDEL1", author, true).await.unwrap_err(),
        LineupPublicError::NotFound
    );
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn private_drafts_do_not_change_public_snapshots_likes_identity_or_other_users() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let public = PostgresPublicLineupRepository::new(db.pool.clone());
    let author = user(&db.pool).await;
    let other = user(&db.pool).await;
    repo.save_draft(author, 1, "4-4-2", &slots(false))
        .await
        .unwrap();
    assert!(repo.find(author, 1).await.unwrap().is_none());
    assert!(repo.find_draft(other, 1).await.unwrap().is_none());
    assert!(repo.find_draft(author, 2).await.unwrap().is_none());
    assert_eq!(
        public
            .list(1, query("latest", 1, 20), None)
            .await
            .unwrap()
            .1,
        0
    );
    let first = repo
        .publish(author, 1, "4-3-3", &slots(true), "DRAFT001")
        .await
        .unwrap();
    assert_eq!(first.lineup.updated_at, first.draft.updated_at);
    public.set_like("DRAFT001", other, true).await.unwrap();
    let before = public
        .shared("DRAFT001", Some(other))
        .await
        .unwrap()
        .unwrap();
    repo.save_draft(author, 1, "3-5-2", &slots(false))
        .await
        .unwrap();
    assert_eq!(
        public
            .shared("DRAFT001", Some(other))
            .await
            .unwrap()
            .unwrap(),
        before
    );
    assert_eq!(
        repo.find_draft(author, 1).await.unwrap().unwrap().formation,
        "3-5-2"
    );
    assert_eq!(
        public
            .list(1, query("latest", 1, 20), None)
            .await
            .unwrap()
            .1,
        1
    );
    let mut modified = slots(true);
    modified[0].x = 0.25;
    let second = repo
        .publish(author, 1, "4-4-2", &modified, "IGNORED9")
        .await
        .unwrap();
    assert_eq!(second.lineup.share_code, "DRAFT001");
    assert_eq!(second.lineup.slots, second.draft.slots);
    let after = public
        .shared("DRAFT001", Some(other))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after.item.created_at, before.item.created_at);
    assert_eq!(after.item.likes, before.item.likes);
    assert_eq!(after.slots[0].x, 0.25);
    assert_eq!(after.item.formation, "4-4-2");
    assert!(repo.find(other, 1).await.unwrap().is_none());
    assert!(repo.find_draft(other, 1).await.unwrap().is_none());
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn failed_draft_write_rolls_back_first_publish_and_republish_in_full() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let author = user(&db.pool).await;
    repo.save_draft(author, 1, "4-3-3", &slots(false))
        .await
        .unwrap();
    let old_draft = repo.find_draft(author, 1).await.unwrap().unwrap();
    sqlx::raw_sql("CREATE FUNCTION reject_test_draft() RETURNS TRIGGER LANGUAGE plpgsql AS $$
        BEGIN IF NEW.formation = '3-5-2' THEN RAISE EXCEPTION 'test draft write failure'; END IF; RETURN NEW; END $$;
        CREATE TRIGGER reject_draft BEFORE INSERT OR UPDATE ON f_i_lineup_drafts FOR EACH ROW EXECUTE FUNCTION reject_test_draft();")
        .execute(&db.pool).await.unwrap();
    assert_eq!(
        repo.publish(author, 1, "3-5-2", &slots(true), "ATOMIC01")
            .await
            .unwrap_err(),
        football_insight_service_backend_rs::lineup::ports::StoreError::Unavailable
    );
    assert!(repo.find(author, 1).await.unwrap().is_none());
    assert_eq!(
        repo.find_draft(author, 1).await.unwrap().unwrap(),
        old_draft
    );
    let first = repo
        .publish(author, 1, "4-4-2", &slots(true), "ATOMIC01")
        .await
        .unwrap();
    assert!(
        repo.publish(author, 1, "3-5-2", &slots(true), "IGNORED1")
            .await
            .is_err()
    );
    assert_eq!(repo.find(author, 1).await.unwrap().unwrap(), first.lineup);
    assert_eq!(
        repo.find_draft(author, 1).await.unwrap().unwrap(),
        first.draft
    );
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn draft_migration_backfills_legacy_edits_without_overwriting_newer_private_edits() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let author = user(&db.pool).await;
    save(&db.pool, author, 1, "LEGACY01", false).await;
    sqlx::query("DELETE FROM f_i_lineup_drafts WHERE user_id = $1")
        .bind(author)
        .execute(&db.pool)
        .await
        .unwrap();
    let migration = include_str!("../migrations/20261019150000_add_lineup_drafts.sql");
    sqlx::raw_sql(migration).execute(&db.pool).await.unwrap();
    let legacy = repo.find(author, 1).await.unwrap().unwrap();
    let backfill = repo.find_draft(author, 1).await.unwrap().unwrap();
    assert_eq!(backfill.formation, legacy.formation);
    assert_eq!(backfill.slots, legacy.slots);
    assert_eq!(backfill.updated_at, legacy.updated_at);
    repo.save_draft(author, 1, "3-5-2", &slots(false))
        .await
        .unwrap();
    let private = repo.find_draft(author, 1).await.unwrap().unwrap();
    sqlx::raw_sql(migration).execute(&db.pool).await.unwrap();
    assert_eq!(repo.find_draft(author, 1).await.unwrap().unwrap(), private);
    // A legacy client save keeps the draft in sync with its successful public-table update.
    repo.upsert(author, 1, "4-4-2", &slots(false), "IGNORED1")
        .await
        .unwrap();
    assert_eq!(
        repo.find_draft(author, 1).await.unwrap().unwrap().formation,
        "4-4-2"
    );
    teardown(db).await;
}
