//! Only run against an explicit disposable localhost database ending in _test.
use football_insight_service_backend_rs::{
    auth::{
        domain::official_account_match_id_access::OfficialAccountMatchIdAccess,
        ports::official_account_match_id_access_port::OfficialAccountMatchIdAccessPort,
    },
    match_id_unlock::adapters::persistence::postgres_official_account_match_id_access::PostgresOfficialAccountMatchIdAccess,
};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::sync::Arc;
use uuid::Uuid;

struct Database {
    admin: PgPool,
    pool: PgPool,
    schema: String,
}

impl Database {
    async fn new() -> Self {
        let url = std::env::var("MATCH_ID_ACCESS_TEST_DATABASE_URL")
            .expect("explicit disposable test database URL required");
        let parsed = url::Url::parse(&url).unwrap();
        assert!(matches!(
            parsed.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]")
        ));
        assert!(parsed.path().ends_with("_test"));
        let admin = PgPool::connect(&url).await.unwrap();
        let schema = format!("official_match_id_{}", Uuid::new_v4().simple());
        sqlx::raw_sql(&format!("CREATE SCHEMA {schema}"))
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
        sqlx::raw_sql(
            "CREATE TABLE f_i_users (
                id UUID PRIMARY KEY, official_wechat_open_id VARCHAR(128),
                membership_tier TEXT NOT NULL DEFAULT 'V1', membership_expires_at TIMESTAMPTZ,
                status TEXT NOT NULL DEFAULT 'active'
             );
             CREATE TABLE f_i_wechat_followers (
                id UUID NOT NULL UNIQUE, open_id VARCHAR(128) PRIMARY KEY,
                latest_invite_code_id UUID, subscribe_count INT NOT NULL DEFAULT 1,
                subscribed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), unsubscribed_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
             );
             CREATE TABLE f_i_invite_codes (issued_for_wechat_open_id VARCHAR(128));
             CREATE TABLE f_i_user_match_id_unlocks (user_id UUID, match_id BIGINT, UNIQUE(user_id, match_id));"
        ).execute(&pool).await.unwrap();
        Self {
            admin,
            pool,
            schema,
        }
    }

    async fn migrate(&self) {
        sqlx::raw_sql(include_str!(
            "../migrations/20261020160000_add_official_match_id_acquisitions.sql"
        ))
        .execute(&self.pool)
        .await
        .unwrap();
    }

    async fn user(&self, openid: Option<&str>, tier: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO f_i_users (id, official_wechat_open_id, membership_tier) VALUES ($1, $2, $3)")
            .bind(id).bind(openid).bind(tier).execute(&self.pool).await.unwrap();
        id
    }

    fn access(&self) -> PostgresOfficialAccountMatchIdAccess {
        PostgresOfficialAccountMatchIdAccess::new(self.pool.clone())
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

#[tokio::test]
#[ignore = "requires explicit disposable localhost PostgreSQL database"]
async fn migration_marks_every_existing_account_and_known_official_identity_as_received() {
    let db = Database::new().await;
    db.user(Some("linked-old"), "V1").await;
    db.user(Some("linked-old"), "V6").await;
    let unbound = db.user(None, "V1").await;
    sqlx::raw_sql(
        "INSERT INTO f_i_wechat_followers (id, open_id, unsubscribed_at) VALUES
            (gen_random_uuid(), 'old-follower', NULL), (gen_random_uuid(), 'old-unfollowed', NOW()),
            (gen_random_uuid(), 'linked-old', NULL);
         INSERT INTO f_i_invite_codes VALUES ('old-invite'), (NULL), ('');",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    db.migrate().await;
    let sources: Vec<(String, String)> = sqlx::query_as(
        "SELECT official_open_id, source FROM f_i_official_match_id_acquisitions ORDER BY official_open_id"
    ).fetch_all(&db.pool).await.unwrap();
    assert_eq!(
        sources,
        vec![
            ("linked-old".into(), "historical".into()),
            ("old-follower".into(), "historical".into()),
            ("old-invite".into(), "historical".into()),
            ("old-unfollowed".into(), "historical".into()),
        ]
    );
    let marked: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM f_i_users WHERE official_match_id_received")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(marked, 3);
    for openid in ["linked-old", "old-follower", "old-invite", "old-unfollowed"] {
        assert!(!db.access().record_acquisition(openid, true).await.unwrap());
    }
    // An existing account that had no official binding must still be excluded after binding.
    sqlx::query("UPDATE f_i_users SET official_wechat_open_id='new-binding' WHERE id=$1")
        .bind(unbound)
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(
        !db.access()
            .record_acquisition("new-binding", true)
            .await
            .unwrap()
    );
    let new_user = db.user(Some("new-account"), "V1").await;
    let received: bool =
        sqlx::query_scalar("SELECT official_match_id_received FROM f_i_users WHERE id=$1")
            .bind(new_user)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert!(!received);
    assert!(
        db.access()
            .record_acquisition("new-account", true)
            .await
            .unwrap()
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable localhost PostgreSQL database"]
async fn first_acquisition_is_lifetime_and_records_all_linked_accounts_even_for_members() {
    use OfficialAccountMatchIdAccess::{Granted, NotMember, Unrecognized};
    let db = Database::new().await;
    db.migrate().await;
    let access = db.access();
    assert_eq!(
        access.check("unbound-new", Some(101)).await.unwrap(),
        Unrecognized
    );
    assert!(
        access
            .record_acquisition("unbound-new", true)
            .await
            .unwrap()
    );
    assert!(
        !access
            .record_acquisition("unbound-new", true)
            .await
            .unwrap()
    );

    let first = db.user(Some("bound-new"), "V1").await;
    let second = db.user(Some("bound-new"), "V3").await;
    assert_eq!(
        access.check("bound-new", Some(101)).await.unwrap(),
        NotMember
    );
    assert!(access.record_acquisition("bound-new", true).await.unwrap());
    assert_eq!(
        access.check("bound-new", Some(102)).await.unwrap(),
        NotMember
    );
    assert!(!access.record_acquisition("bound-new", true).await.unwrap());
    let marked: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM f_i_users WHERE official_match_id_received")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(marked, 2);
    sqlx::query("UPDATE f_i_users SET official_wechat_open_id='changed-binding' WHERE id=$1")
        .bind(first)
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(
        !access
            .record_acquisition("changed-binding", true)
            .await
            .unwrap()
    );
    sqlx::query("UPDATE f_i_users SET official_wechat_open_id=NULL WHERE id=$1")
        .bind(second)
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(!access.record_acquisition("bound-new", true).await.unwrap());

    let member = db.user(Some("member-new"), "V6").await;
    assert_eq!(
        access.check("member-new", Some(101)).await.unwrap(),
        Granted
    );
    assert!(
        access
            .record_acquisition("member-new", false)
            .await
            .unwrap()
    );
    assert!(
        access
            .record_acquisition("member-new", false)
            .await
            .unwrap()
    );
    sqlx::query(
        "UPDATE f_i_users SET membership_expires_at=NOW() - INTERVAL '1 second' WHERE id=$1",
    )
    .bind(member)
    .execute(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        access.check("member-new", Some(102)).await.unwrap(),
        NotMember
    );
    assert!(!access.record_acquisition("member-new", true).await.unwrap());

    let purchaser = db.user(Some("purchase-new"), "V1").await;
    sqlx::query("INSERT INTO f_i_user_match_id_unlocks VALUES($1,101)")
        .bind(purchaser)
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(
        access.check("purchase-new", Some(101)).await.unwrap(),
        Granted
    );
    assert!(
        access
            .record_acquisition("purchase-new", false)
            .await
            .unwrap()
    );
    assert_eq!(
        access.check("purchase-new", Some(102)).await.unwrap(),
        NotMember
    );
    assert!(
        !access
            .record_acquisition("purchase-new", true)
            .await
            .unwrap()
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable localhost PostgreSQL database"]
async fn concurrent_queries_only_admit_one_first_acquisition_for_bound_or_unbound_users() {
    let db = Database::new().await;
    db.migrate().await;
    db.user(Some("bound-race"), "V1").await;
    for openid in ["bound-race", "unbound-race"] {
        let access = Arc::new(db.access());
        let barrier = Arc::new(tokio::sync::Barrier::new(16));
        let mut tasks = Vec::new();
        for _ in 0..16 {
            let access = access.clone();
            let barrier = barrier.clone();
            tasks.push(tokio::spawn(async move {
                barrier.wait().await;
                access.record_acquisition(openid, true).await.unwrap()
            }));
        }
        let mut granted = 0;
        for task in tasks {
            granted += usize::from(task.await.unwrap());
        }
        assert_eq!(
            granted, 1,
            "only one query may acquire the first ID: {openid}"
        );
    }
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable localhost PostgreSQL database"]
async fn a_failed_history_write_rolls_back_and_preserves_first_acquisition() {
    let db = Database::new().await;
    db.migrate().await;
    db.user(Some("new-failure"), "V1").await;
    sqlx::raw_sql(
        "CREATE FUNCTION reject_received_update() RETURNS trigger LANGUAGE plpgsql AS
         'BEGIN RAISE EXCEPTION ''test storage failure''; END';
         CREATE TRIGGER reject_received BEFORE UPDATE ON f_i_users
         FOR EACH ROW EXECUTE FUNCTION reject_received_update();",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    assert!(
        db.access()
            .record_acquisition("new-failure", true)
            .await
            .is_err()
    );
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_official_match_id_acquisitions")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    sqlx::raw_sql("DROP TRIGGER reject_received ON f_i_users")
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(
        db.access()
            .record_acquisition("new-failure", true)
            .await
            .unwrap()
    );
    assert!(
        !db.access()
            .record_acquisition("new-failure", true)
            .await
            .unwrap()
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable localhost PostgreSQL database"]
async fn binding_preserves_history_from_both_old_and_new_official_identities() {
    use chrono::{Duration, Utc};
    use football_insight_service_backend_rs::auth::{
        adapters::persistence::postgres_official_bind_repository::PostgresOfficialBindRepository,
        domain::official_bind::ConsumeBindCodeOutcome,
        ports::official_bind_repository::OfficialBindRepository,
    };
    let db = Database::new().await;
    db.migrate().await;
    sqlx::raw_sql(include_str!(
        "../migrations/20261016150000_add_wechat_official_bind_codes.sql"
    ))
    .execute(&db.pool)
    .await
    .unwrap();
    let access = db.access();
    let binding = PostgresOfficialBindRepository::new(db.pool.clone());
    assert!(
        access
            .record_acquisition("previously-anonymous", true)
            .await
            .unwrap()
    );
    for (old, new, code) in [
        (None, "previously-anonymous", "first-code"),
        (Some("previously-anonymous"), "fresh-binding", "second-code"),
    ] {
        let user = db.user(old, "V1").await;
        let now = Utc::now();
        binding
            .issue(user, code, now, now + Duration::minutes(10))
            .await
            .unwrap();
        assert_eq!(
            binding.consume(code, new).await.unwrap(),
            ConsumeBindCodeOutcome::Success
        );
        let received: bool =
            sqlx::query_scalar("SELECT official_match_id_received FROM f_i_users WHERE id=$1")
                .bind(user)
                .fetch_one(&db.pool)
                .await
                .unwrap();
        assert!(
            received,
            "binding must inherit acquisition history from old or new identity"
        );
        assert!(!access.record_acquisition(new, true).await.unwrap());
    }
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable localhost PostgreSQL database"]
async fn binding_and_acquisition_wait_for_the_same_history_transaction_before_locking_accounts() {
    use chrono::{Duration, Utc};
    use football_insight_service_backend_rs::auth::{
        adapters::persistence::postgres_official_bind_repository::PostgresOfficialBindRepository,
        domain::official_bind::ConsumeBindCodeOutcome,
        ports::official_bind_repository::OfficialBindRepository,
    };
    let db = Database::new().await;
    db.migrate().await;
    sqlx::raw_sql(include_str!(
        "../migrations/20261016150000_add_wechat_official_bind_codes.sql"
    ))
    .execute(&db.pool)
    .await
    .unwrap();
    let user = db.user(Some("before-binding"), "V1").await;
    let now = Utc::now();
    let binding = PostgresOfficialBindRepository::new(db.pool.clone());
    binding
        .issue(user, "race-code", now, now + Duration::minutes(10))
        .await
        .unwrap();
    let mut blocker = db.pool.begin().await.unwrap();
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('football_insight:official-match-id-history', 0))")
        .execute(&mut *blocker).await.unwrap();
    let access = db.access();
    let mut acquisition =
        tokio::spawn(async move { access.record_acquisition("before-binding", true).await });
    let mut rebinding =
        tokio::spawn(async move { binding.consume("race-code", "after-binding").await });
    let acquisition_waited =
        tokio::time::timeout(std::time::Duration::from_millis(200), &mut acquisition)
            .await
            .is_err();
    let binding_waited =
        tokio::time::timeout(std::time::Duration::from_millis(200), &mut rebinding)
            .await
            .is_err();
    blocker.rollback().await.unwrap();
    // Drain pending tasks before assertions so a failing test does not leave blocked transactions.
    if acquisition_waited {
        acquisition.await.unwrap().unwrap();
    }
    if binding_waited {
        assert_eq!(
            rebinding.await.unwrap().unwrap(),
            ConsumeBindCodeOutcome::Success
        );
    }
    db.close().await;
    assert!(
        acquisition_waited,
        "acquisition must wait for an in-flight history/binding transaction"
    );
    assert!(
        binding_waited,
        "binding must wait for an in-flight history/acquisition transaction"
    );
}

#[tokio::test]
#[ignore = "requires explicit disposable localhost PostgreSQL database"]
async fn registration_and_reactivation_inherit_history_in_all_invited_identity_flows() {
    use football_insight_service_backend_rs::auth::{
        adapters::persistence::postgres_auth_repository::PostgresAuthRepository,
        domain::{
            registration::{RegistrationIdentity, UserRegistration},
            wechat::WechatOauthProfile,
        },
        ports::auth_repository::AuthRepository,
    };
    let db = Database::new().await;
    db.migrate().await;
    sqlx::raw_sql(
        "ALTER TABLE f_i_users ADD account_identifier TEXT, ADD display_name TEXT NOT NULL DEFAULT 'Test',
             ADD password_hash TEXT, ADD wx_open_id TEXT, ADD union_id TEXT, ADD avatar_url TEXT,
             ADD created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), ADD updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
         ALTER TABLE f_i_invite_codes ADD code TEXT, ADD is_active BOOLEAN NOT NULL DEFAULT TRUE,
             ADD used_by_user_id UUID, ADD used_at TIMESTAMPTZ;
         CREATE TABLE f_i_system_configs (config_key TEXT PRIMARY KEY, config_value TEXT);"
    ).execute(&db.pool).await.unwrap();
    let repo = PostgresAuthRepository::new(db.pool.clone());
    for kind in 0..3 {
        let old = format!("register-old-{kind}");
        let new = format!("register-new-{kind}");
        assert!(db.access().record_acquisition(&old, true).await.unwrap());
        let account = format!("account-{kind}");
        let profile = WechatOauthProfile {
            open_id: format!("mini-{kind}"),
            union_id: None,
            display_name: None,
            avatar_url: None,
        };
        let mut user_id = None;
        for (index, openid) in [&old, &new].into_iter().enumerate() {
            let code = format!("invite-{kind}-{index}");
            sqlx::query(
                "INSERT INTO f_i_invite_codes (code, issued_for_wechat_open_id) VALUES ($1,$2)",
            )
            .bind(&code)
            .bind(openid)
            .execute(&db.pool)
            .await
            .unwrap();
            let identity = match kind {
                0 => RegistrationIdentity::Password {
                    account_identifier: &account,
                    password_hash: "unused",
                },
                1 => RegistrationIdentity::Wechat {
                    phone_number: &account,
                    password_hash: "unused",
                    profile: &profile,
                },
                _ => RegistrationIdentity::MiniWechat { profile: &profile },
            };
            let registered = repo
                .register_user(UserRegistration {
                    invite_code: Some(&code),
                    referral_code: None,
                    identity,
                })
                .await
                .unwrap();
            assert_eq!(registered.newly_created, index == 0);
            if let Some(previous_id) = user_id {
                assert_eq!(registered.user.id, previous_id);
            }
            user_id = Some(registered.user.id);
            let received: bool =
                sqlx::query_scalar("SELECT official_match_id_received FROM f_i_users WHERE id=$1")
                    .bind(registered.user.id)
                    .fetch_one(&db.pool)
                    .await
                    .unwrap();
            assert!(
                received,
                "registration/reactivation must inherit previous acquisition: {kind}/{index}"
            );
            assert!(!db.access().record_acquisition(openid, true).await.unwrap());
            if index == 0 {
                // Also exercise reactivation inheritance independently of the INSERT path.
                sqlx::query("UPDATE f_i_users SET status='disabled', official_match_id_received=FALSE WHERE id=$1")
                    .bind(registered.user.id).execute(&db.pool).await.unwrap();
            }
        }
    }
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable localhost PostgreSQL database"]
async fn verified_binding_records_missing_followers_including_already_bound_accounts() {
    use chrono::{Duration, Utc};
    use football_insight_service_backend_rs::auth::{
        adapters::persistence::postgres_official_bind_repository::PostgresOfficialBindRepository,
        domain::official_bind::ConsumeBindCodeOutcome,
        ports::official_bind_repository::OfficialBindRepository,
    };
    let db = Database::new().await;
    db.migrate().await;
    sqlx::raw_sql(include_str!(
        "../migrations/20261016150000_add_wechat_official_bind_codes.sql"
    ))
    .execute(&db.pool)
    .await
    .unwrap();
    let binding = PostgresOfficialBindRepository::new(db.pool.clone());
    for (prior, openid, code, expected) in [
        (
            None,
            "new-follower",
            "new-code",
            ConsumeBindCodeOutcome::Success,
        ),
        (
            Some("already-bound"),
            "already-bound",
            "same-code",
            ConsumeBindCodeOutcome::AlreadyBound,
        ),
    ] {
        let user = db.user(prior, "V9").await;
        let now = Utc::now();
        binding
            .issue(user, code, now, now + Duration::minutes(10))
            .await
            .unwrap();
        assert_eq!(binding.consume(code, openid).await.unwrap(), expected);
        let follower: Option<(bool, i32, Option<Uuid>)> = sqlx::query_as(
            "SELECT unsubscribed_at IS NULL, subscribe_count, latest_invite_code_id FROM f_i_wechat_followers WHERE open_id=$1"
        ).bind(openid).fetch_optional(&db.pool).await.unwrap();
        assert_eq!(
            follower,
            Some((true, 1, None)),
            "verified binding must supply the missing active follow without issuing invites"
        );
    }
    assert_eq!(
        binding
            .consume("invalid-code", "invalid-follower")
            .await
            .unwrap(),
        ConsumeBindCodeOutcome::Invalid
    );
    let invalid_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM f_i_wechat_followers WHERE open_id='invalid-follower'",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(invalid_count, 0);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit disposable localhost PostgreSQL database"]
async fn binding_preserves_existing_follow_history_and_explicit_unsubscribe() {
    use chrono::{Duration, Utc};
    use football_insight_service_backend_rs::auth::{
        adapters::persistence::postgres_official_bind_repository::PostgresOfficialBindRepository,
        ports::official_bind_repository::OfficialBindRepository,
    };
    let db = Database::new().await;
    db.migrate().await;
    sqlx::raw_sql(include_str!(
        "../migrations/20261016150000_add_wechat_official_bind_codes.sql"
    ))
    .execute(&db.pool)
    .await
    .unwrap();
    let binding = PostgresOfficialBindRepository::new(db.pool.clone());
    for (openid, unsubscribed) in [("active-follow", false), ("unsubscribed-follow", true)] {
        let follower_id = Uuid::new_v4();
        sqlx::query("INSERT INTO f_i_wechat_followers (id, open_id, subscribe_count, unsubscribed_at) VALUES ($1,$2,4,CASE WHEN $3 THEN NOW() ELSE NULL END)")
            .bind(follower_id).bind(openid).bind(unsubscribed).execute(&db.pool).await.unwrap();
        let before: serde_json::Value =
            sqlx::query_scalar("SELECT to_jsonb(f) FROM f_i_wechat_followers f WHERE open_id=$1")
                .bind(openid)
                .fetch_one(&db.pool)
                .await
                .unwrap();
        let user = db.user(None, "V9").await;
        let now = Utc::now();
        binding
            .issue(user, openid, now, now + Duration::minutes(10))
            .await
            .unwrap();
        binding.consume(openid, openid).await.unwrap();
        let after: serde_json::Value =
            sqlx::query_scalar("SELECT to_jsonb(f) FROM f_i_wechat_followers f WHERE open_id=$1")
                .bind(openid)
                .fetch_one(&db.pool)
                .await
                .unwrap();
        assert_eq!(before, after);
    }
    db.close().await;
}
