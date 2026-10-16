//! Explicit disposable localhost PostgreSQL database only; every test owns its schema.
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use football_insight_service_backend_rs::{
    admin::{
        adapters::{
            persistence::{
                postgres_admin_auth_repository::PostgresAdminAuthRepository,
                postgres_admin_credentials_repository::PostgresAdminCredentialsRepository,
            },
            security::{
                jwt_admin_token_port::JwtAdminTokenPort,
                role_based_admin_authorization::RoleBasedAdminAuthorization,
            },
            web::{credentials_routes::admin_credentials_routes, routes::admin_auth_routes},
        },
        application::{
            admin_auth_service::{AdminAuthService, AdminLoginInput},
            admin_credentials_service::AdminCredentialsService,
        },
        domain::{
            admin_auth::{AdminBootstrapOwner, AdminPrincipal, AdminSession},
            admin_credentials::{AdminCredentialsError, AdminCredentialsInput},
        },
        ports::{admin_auth_repository::AdminAuthRepository, admin_token_port::AdminTokenPort},
    },
    auth::{
        adapters::security::argon2_password_port::Argon2PasswordPort,
        ports::password_port::PasswordPort,
    },
};
use serde_json::{Value, json};
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;
const CURRENT: &str = "test-current-password";
const NEW: &str = "  test-new-password  ";
struct Database {
    admin: PgPool,
    pool: PgPool,
    schema: String,
    actor: Uuid,
}
impl Database {
    async fn new() -> Self {
        let url = std::env::var("ADMIN_CREDENTIALS_TEST_DATABASE_URL")
            .expect("explicit disposable test URL required");
        let parsed = url::Url::parse(&url).unwrap();
        assert!(matches!(
            parsed.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]")
        ));
        assert!(parsed.path().ends_with("_test"));
        let admin = PgPool::connect(&url).await.unwrap();
        let schema = format!("admin_credentials_{}", Uuid::new_v4().simple());
        sqlx::raw_sql(&format!("CREATE SCHEMA {schema}"))
            .execute(&admin)
            .await
            .unwrap();
        let options = url
            .parse::<PgConnectOptions>()
            .unwrap()
            .options([("search_path", schema.as_str())])
            .application_name(&schema);
        let pool = PgPoolOptions::new()
            .max_connections(6)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::raw_sql(include_str!(
            "../migrations/20260802120000_add_admin_identity_and_audit.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        let actor = Uuid::new_v4();
        let hash = Argon2PasswordPort.hash_password(CURRENT).unwrap();
        sqlx::query("INSERT INTO f_i_admin_users(id,username,password_hash,display_name,role) VALUES($1,'test_owner',$2,'Test Owner','owner')").bind(actor).bind(hash).execute(&pool).await.unwrap();
        Self {
            admin,
            pool,
            schema,
            actor,
        }
    }
    fn repository(&self) -> PostgresAdminAuthRepository {
        PostgresAdminAuthRepository::new(self.pool.clone())
    }
    fn auth(&self) -> Arc<AdminAuthService> {
        Arc::new(AdminAuthService::new(
            Arc::new(self.repository()),
            Arc::new(Argon2PasswordPort),
            Arc::new(JwtAdminTokenPort::new(
                "isolated-credentials-test-secret".into(),
            )),
            Arc::new(RoleBasedAdminAuthorization),
            Duration::hours(12),
            Arc::new(Utc::now),
        ))
    }
    fn credentials(&self) -> Arc<AdminCredentialsService> {
        Arc::new(AdminCredentialsService::new(
            Arc::new(PostgresAdminCredentialsRepository::new(self.pool.clone())),
            Arc::new(Argon2PasswordPort),
        ))
    }
    fn app(&self) -> axum::Router {
        let auth = self.auth();
        admin_auth_routes(auth.clone()).merge(admin_credentials_routes(auth, self.credentials()))
    }
    async fn login(&self, username: &str, password: &str) -> String {
        self.auth()
            .login(AdminLoginInput {
                username: username.into(),
                password: password.into(),
            })
            .await
            .unwrap()
            .access_token
    }
    async fn principal(&self, token: &str) -> AdminPrincipal {
        self.auth().authenticate(token).await.unwrap()
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
fn input(username: &str, current: &str, new: Option<&str>) -> AdminCredentialsInput {
    AdminCredentialsInput {
        username: username.into(),
        current_password: current.into(),
        new_password: new.map(Into::into),
    }
}
async fn request(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    payload: Option<Value>,
) -> axum::response::Response {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let body = match payload {
        Some(value) => {
            builder = builder.header("Content-Type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    app.clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap()
}
async fn text(response: axum::response::Response) -> String {
    String::from_utf8(
        to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}
fn bootstrap(username: &str) -> AdminBootstrapOwner {
    AdminBootstrapOwner {
        username: username.into(),
        password_hash: "unused-test-bootstrap-hash".into(),
        display_name: "Bootstrap Owner".into(),
    }
}

#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn username_only_password_only_and_both_preserve_identity_revoke_all_and_audit_without_secrets()
 {
    for (username, password) in [
        (" Renamed.Owner ", None),
        ("test_owner", Some(NEW)),
        (" Renamed.Owner ", Some(NEW)),
    ] {
        let db = Database::new().await;
        let auth = db.auth();
        let first = db.login("test_owner", CURRENT).await;
        let second = db.login("test_owner", CURRENT).await;
        let before = db
            .repository()
            .find_by_username("test_owner")
            .await
            .unwrap()
            .unwrap();
        db.credentials()
            .change(
                db.principal(&first).await,
                input(username, CURRENT, password),
            )
            .await
            .unwrap();
        let canonical = username.trim().to_ascii_lowercase();
        let account = db
            .repository()
            .find_by_username(&canonical)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(account.id, before.id);
        assert_eq!(account.role, before.role);
        assert_eq!(account.display_name, before.display_name);
        assert!(
            Argon2PasswordPort
                .verify_password(password.unwrap_or(CURRENT), &account.password_hash)
                .unwrap()
        );
        assert!(auth.authenticate(&first).await.is_err());
        assert!(auth.authenticate(&second).await.is_err());
        let active: i64 =
            sqlx::query_scalar("SELECT count(*) FROM f_i_admin_sessions WHERE revoked_at IS NULL")
                .fetch_one(&db.pool)
                .await
                .unwrap();
        assert_eq!(active, 0);
        assert!(
            auth.login(AdminLoginInput {
                username: canonical.clone(),
                password: password.unwrap_or(CURRENT).into()
            })
            .await
            .is_ok()
        );
        if password.is_some() {
            assert!(
                auth.login(AdminLoginInput {
                    username: canonical.clone(),
                    password: CURRENT.into()
                })
                .await
                .is_err()
            );
        }
        if canonical != "test_owner" {
            assert!(
                auth.login(AdminLoginInput {
                    username: "test_owner".into(),
                    password: CURRENT.into()
                })
                .await
                .is_err()
            );
        }
        let audit = sqlx::query(
            "SELECT action,target_type,target_id,before_json,after_json FROM f_i_admin_audit_logs",
        )
        .fetch_one(&db.pool)
        .await
        .unwrap();
        assert_eq!(audit.get::<String, _>("action"), "admin.credentials.update");
        assert_eq!(audit.get::<String, _>("target_id"), db.actor.to_string());
        assert_eq!(
            audit.get::<Value, _>("before_json")["username"],
            "test_owner"
        );
        assert_eq!(audit.get::<Value, _>("after_json")["username"], canonical);
        assert_eq!(
            audit.get::<Value, _>("after_json")["password_changed"],
            password.is_some()
        );
        let audit_text = format!(
            "{}{}",
            audit.get::<Value, _>("before_json"),
            audit.get::<Value, _>("after_json")
        );
        for secret in [
            CURRENT,
            NEW,
            before.password_hash.as_str(),
            account.password_hash.as_str(),
        ] {
            assert!(!audit_text.contains(secret));
        }
        assert!(!audit_text.contains("password_hash"));
        assert!(!audit_text.contains("current_password"));
        db.close().await;
    }
}

#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn wrong_current_conflict_noop_and_audit_failure_never_mutate_or_revoke() {
    let db = Database::new().await;
    let token = db.login("test_owner", CURRENT).await;
    let service = db.credentials();
    let before = db
        .repository()
        .find_by_username("test_owner")
        .await
        .unwrap()
        .unwrap();
    let principal = db.principal(&token).await;
    assert_eq!(
        service
            .change(
                principal.clone(),
                input("renamed", "wrong-test-password", Some(NEW))
            )
            .await
            .unwrap_err(),
        AdminCredentialsError::WrongPassword
    );
    for new in [None, Some(CURRENT)] {
        assert!(matches!(
            service
                .change(principal.clone(), input(" TEST_OWNER ", CURRENT, new))
                .await,
            Err(AdminCredentialsError::InvalidInput(_))
        ));
    }
    sqlx::query("INSERT INTO f_i_admin_users(id,username,password_hash,display_name,role) VALUES($1,'occupied','unused','Other','admin')").bind(Uuid::new_v4()).execute(&db.pool).await.unwrap();
    assert_eq!(
        service
            .change(principal.clone(), input("occupied", CURRENT, Some(NEW)))
            .await
            .unwrap_err(),
        AdminCredentialsError::Conflict
    );
    sqlx::raw_sql("CREATE FUNCTION reject_credentials_audit() RETURNS TRIGGER AS $$ BEGIN RAISE EXCEPTION 'forced audit failure'; END $$ LANGUAGE plpgsql; CREATE TRIGGER fail_audit BEFORE INSERT ON f_i_admin_audit_logs FOR EACH ROW EXECUTE FUNCTION reject_credentials_audit();").execute(&db.pool).await.unwrap();
    assert_eq!(
        service
            .change(principal, input("renamed", CURRENT, Some(NEW)))
            .await
            .unwrap_err(),
        AdminCredentialsError::Internal
    );
    assert!(
        db.repository()
            .find_by_username("test_owner")
            .await
            .unwrap()
            .unwrap()
            == before
    );
    assert!(db.auth().authenticate(&token).await.is_ok());
    let audits: i64 = sqlx::query_scalar("SELECT count(*) FROM f_i_admin_audit_logs")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(audits, 0);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn http_credentials_contract_statuses_and_secret_free_errors() {
    let db = Database::new().await;
    let app = db.app();
    let token = db.login("test_owner", CURRENT).await;
    let path = "/api/v1/admin/auth/credentials";
    let payload = json!({"username":"renamed","current_password":CURRENT,"new_password":NEW});
    assert_eq!(
        request(&app, "PUT", path, None, Some(payload.clone()))
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &app,
            "PUT",
            path,
            Some("invalid-token"),
            Some(payload.clone())
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    let wrong =
        json!({"username":"renamed","current_password":"wrong-test-password","new_password":NEW});
    let response = request(&app, "PUT", path, Some(&token), Some(wrong)).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let error = text(response).await;
    assert!(!error.contains(CURRENT));
    assert!(!error.contains(NEW));
    for invalid in [
        json!({"username":"ab","current_password":CURRENT}),
        json!({"username":"valid"}),
        json!({"username":"valid","current_password":""}),
        json!({"username":"valid","current_password":CURRENT,"new_password":"short"}),
        json!({"username":"test_owner","current_password":CURRENT,"new_password":null}),
    ] {
        assert_eq!(
            request(&app, "PUT", path, Some(&token), Some(invalid))
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
    sqlx::query("INSERT INTO f_i_admin_users(id,username,password_hash,display_name,role) VALUES($1,'occupied','unused','Other','admin')").bind(Uuid::new_v4()).execute(&db.pool).await.unwrap();
    assert_eq!(
        request(
            &app,
            "PUT",
            path,
            Some(&token),
            Some(json!({"username":"occupied","current_password":CURRENT,"new_password":NEW}))
        )
        .await
        .status(),
        StatusCode::CONFLICT
    );
    let principal = db.principal(&token).await;
    let expired = JwtAdminTokenPort::new("isolated-credentials-test-secret".into())
        .issue_token(
            db.actor,
            principal.session_id,
            "test_owner",
            "owner",
            Utc::now() - Duration::hours(1),
        )
        .unwrap();
    assert_eq!(
        request(&app, "PUT", path, Some(&expired), Some(payload.clone()))
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    sqlx::raw_sql("CREATE FUNCTION reject_http_credentials_audit() RETURNS TRIGGER AS $$ BEGIN RAISE EXCEPTION 'forced audit failure'; END $$ LANGUAGE plpgsql; CREATE TRIGGER fail_audit BEFORE INSERT ON f_i_admin_audit_logs FOR EACH ROW EXECUTE FUNCTION reject_http_credentials_audit();").execute(&db.pool).await.unwrap();
    let response = request(&app, "PUT", path, Some(&token), Some(payload.clone())).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let message = text(response).await;
    assert!(!message.contains("forced audit"));
    assert!(!message.contains("f_i_"));
    assert!(!message.contains(CURRENT));
    assert!(!message.contains(NEW));
    assert!(db.auth().authenticate(&token).await.is_ok());
    sqlx::query("DROP TRIGGER fail_audit ON f_i_admin_audit_logs")
        .execute(&db.pool)
        .await
        .unwrap();
    let response = request(&app, "PUT", path, Some(&token), Some(payload)).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(text(response).await.is_empty());
    assert_eq!(
        request(
            &app,
            "PUT",
            path,
            Some(&token),
            Some(json!({"username":"again","current_password":NEW}))
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    let token = db.login("renamed", NEW).await;
    let response = request(
        &app,
        "PUT",
        path,
        Some(&token),
        Some(json!({"username":"renamed_again","current_password":NEW,"new_password":null})),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let token = db.login("renamed_again", NEW).await;
    sqlx::query("UPDATE f_i_admin_sessions SET expires_at=NOW()-INTERVAL '1 second'")
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(
        request(
            &app,
            "PUT",
            path,
            Some(&token),
            Some(json!({"username":"again","current_password":NEW}))
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn bootstrap_reuses_renamed_owner_without_restoring_original_username_or_password() {
    let db = Database::new().await;
    let repository = db.repository();
    sqlx::query("UPDATE f_i_admin_users SET username='renamed_owner' WHERE id=$1")
        .bind(db.actor)
        .execute(&db.pool)
        .await
        .unwrap();
    let before = repository
        .find_by_username("renamed_owner")
        .await
        .unwrap()
        .unwrap();
    let result = repository
        .ensure_owner(bootstrap("test_owner"))
        .await
        .unwrap();
    assert!(result == before);
    assert!(
        repository
            .find_by_username("test_owner")
            .await
            .unwrap()
            .is_none()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM f_i_admin_users")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn concurrent_bootstraps_create_one_owner_even_with_different_defaults() {
    let db = Database::new().await;
    sqlx::query("DELETE FROM f_i_admin_users")
        .execute(&db.pool)
        .await
        .unwrap();
    let repo = db.repository();
    let (first, second) = tokio::join!(
        repo.ensure_owner(bootstrap("first_owner")),
        repo.ensure_owner(bootstrap("second_owner"))
    );
    assert_eq!(first.unwrap().id, second.unwrap().id);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM f_i_admin_users WHERE role='owner'")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn guarded_session_creation_rechecks_changed_credentials_under_account_lock() {
    let db = Database::new().await;
    let repository = db.repository();
    let snapshot = repository
        .find_by_username("test_owner")
        .await
        .unwrap()
        .unwrap();
    let mut tx = db.pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM f_i_admin_users WHERE id=$1 FOR UPDATE")
        .bind(db.actor)
        .execute(&mut *tx)
        .await
        .unwrap();
    let session = AdminSession {
        id: Uuid::new_v4(),
        admin_user_id: db.actor,
        expires_at: Utc::now() + Duration::hours(1),
        revoked_at: None,
        created_at: Utc::now(),
    };
    let future = tokio::spawn(async move {
        repository
            .create_session_if_current(session, &snapshot.username, &snapshot.password_hash)
            .await
            .unwrap()
    });
    tokio::time::timeout(std::time::Duration::from_secs(5),async{loop{let waiting:i64=sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE application_name=$1 AND wait_event_type='Lock'").bind(&db.schema).fetch_one(&db.admin).await.unwrap();if waiting>0{break;}tokio::task::yield_now().await;}}).await.unwrap();
    sqlx::query("UPDATE f_i_admin_users SET username='changed_during_login' WHERE id=$1")
        .bind(db.actor)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(!future.await.unwrap());
    let sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM f_i_admin_sessions")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(sessions, 0);
    db.close().await;
}

struct PausedPasswordPort {
    reached: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    release: Arc<std::sync::Barrier>,
}
impl PasswordPort for PausedPasswordPort {
    fn hash_password(&self, password: &str) -> anyhow::Result<String> {
        Argon2PasswordPort.hash_password(password)
    }
    fn verify_password(&self, password: &str, hash: &str) -> anyhow::Result<bool> {
        let verified = Argon2PasswordPort.verify_password(password, hash)?;
        if let Some(reached) = self.reached.lock().unwrap().take() {
            reached.send(()).unwrap();
            self.release.wait();
        }
        Ok(verified)
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn login_cannot_issue_a_live_session_from_verified_but_replaced_credentials() {
    for (username, password) in [("renamed_owner", None), ("test_owner", Some(NEW))] {
        let db = Database::new().await;
        let token = db.login("test_owner", CURRENT).await;
        let principal = db.principal(&token).await;
        let (reached, wait) = tokio::sync::oneshot::channel();
        let release = Arc::new(std::sync::Barrier::new(2));
        let login_service = AdminAuthService::new(
            Arc::new(db.repository()),
            Arc::new(PausedPasswordPort {
                reached: std::sync::Mutex::new(Some(reached)),
                release: release.clone(),
            }),
            Arc::new(JwtAdminTokenPort::new(
                "isolated-credentials-test-secret".into(),
            )),
            Arc::new(RoleBasedAdminAuthorization),
            Duration::hours(12),
            Arc::new(Utc::now),
        );
        let pending = tokio::spawn(async move {
            login_service
                .login(AdminLoginInput {
                    username: "test_owner".into(),
                    password: CURRENT.into(),
                })
                .await
        });
        tokio::time::timeout(std::time::Duration::from_secs(5), wait)
            .await
            .unwrap()
            .unwrap();
        let change = db
            .credentials()
            .change(principal, input(username, CURRENT, password))
            .await;
        release.wait();
        change.unwrap();
        assert!(pending.await.unwrap().is_err());
        let active: i64 =
            sqlx::query_scalar("SELECT count(*) FROM f_i_admin_sessions WHERE revoked_at IS NULL")
                .fetch_one(&db.pool)
                .await
                .unwrap();
        assert_eq!(active, 0);
        db.close().await;
    }
}

use football_insight_service_backend_rs::{
    admin::{
        adapters::web::setup_routes::admin_setup_routes,
        application::admin_setup_service::AdminSetupService,
        bootstrap::bootstrap_admin_owner,
        domain::admin_setup::{AdminSetupError, AdminSetupInput},
    },
    config::{AppConfig, DatabasePoolConfig, JPushConfig, RefluxNotificationWorkerConfig},
    http_cache::{HttpResponseCache, cache_get_responses},
};
fn setup_service(db: &Database) -> Arc<AdminSetupService> {
    Arc::new(AdminSetupService::new(
        Arc::new(db.repository()),
        Arc::new(Argon2PasswordPort),
    ))
}
fn setup_app(db: &Database) -> axum::Router {
    admin_setup_routes(setup_service(db)).layer(axum::middleware::from_fn_with_state(
        HttpResponseCache::new(std::time::Duration::from_secs(600)),
        cache_get_responses,
    ))
}
#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn setup_initializes_empty_database_once_without_autologin_or_cache_or_secret_response() {
    let db = Database::new().await;
    sqlx::query("DELETE FROM f_i_admin_users")
        .execute(&db.pool)
        .await
        .unwrap();
    let app = setup_app(&db);
    let path = "/api/v1/admin/auth/setup";
    let response = request(&app, "GET", path, None, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(
        serde_json::from_str::<Value>(&text(response).await).unwrap(),
        json!({"needs_setup":true,"username":"superadmin"})
    );
    for password in ["".to_string(), "short".into(), "x".repeat(129)] {
        assert_eq!(
            request(&app, "POST", path, None, Some(json!({"password":password})))
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
    let response = request(&app, "POST", path, None, Some(json!({"password":NEW}))).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(text(response).await.is_empty());
    let sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM f_i_admin_sessions")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(sessions, 0);
    let response = request(&app, "GET", path, None, None).await;
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(
        serde_json::from_str::<Value>(&text(response).await).unwrap(),
        json!({"needs_setup":false,"username":"superadmin"})
    );
    assert_eq!(
        request(&app, "POST", path, None, Some(json!({"password":CURRENT})))
            .await
            .status(),
        StatusCode::CONFLICT
    );
    let account = db
        .repository()
        .find_by_username("superadmin")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(account.role, "owner");
    assert!(
        Argon2PasswordPort
            .verify_password(NEW, &account.password_hash)
            .unwrap()
    );
    assert!(
        db.auth()
            .login(AdminLoginInput {
                username: "superadmin".into(),
                password: NEW.into()
            })
            .await
            .is_ok()
    );
    db.close().await;
}
#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn any_existing_administrator_including_disabled_prevents_setup_and_additional_bootstrap() {
    for (role, status) in [
        ("owner", "disabled"),
        ("admin", "active"),
        ("admin", "disabled"),
    ] {
        let db = Database::new().await;
        sqlx::query("UPDATE f_i_admin_users SET role=$1,status=$2")
            .bind(role)
            .bind(status)
            .execute(&db.pool)
            .await
            .unwrap();
        let before = db
            .repository()
            .find_by_username("test_owner")
            .await
            .unwrap()
            .unwrap();
        let service = setup_service(&db);
        assert!(!service.needs_setup().await.unwrap());
        assert_eq!(
            service
                .initialize(AdminSetupInput {
                    password: NEW.into()
                })
                .await
                .unwrap_err(),
            AdminSetupError::AlreadyInitialized
        );
        let owner = db
            .repository()
            .ensure_owner(bootstrap("another_default"))
            .await
            .unwrap();
        assert!(owner == before);
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM f_i_admin_users")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
        db.close().await;
    }
}
#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn concurrent_setup_only_accepts_one_password() {
    let db = Database::new().await;
    sqlx::query("DELETE FROM f_i_admin_users")
        .execute(&db.pool)
        .await
        .unwrap();
    let service = setup_service(&db);
    let (first, second) = tokio::join!(
        service.initialize(AdminSetupInput {
            password: CURRENT.into()
        }),
        service.initialize(AdminSetupInput {
            password: NEW.into()
        })
    );
    assert!(matches!(
        (&first, &second),
        (Ok(()), Err(AdminSetupError::AlreadyInitialized))
            | (Err(AdminSetupError::AlreadyInitialized), Ok(()))
    ));
    let account = db
        .repository()
        .find_by_username("superadmin")
        .await
        .unwrap()
        .unwrap();
    let winning_password = if first.is_ok() { CURRENT } else { NEW };
    assert!(
        Argon2PasswordPort
            .verify_password(winning_password, &account.password_hash)
            .unwrap()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM f_i_admin_users")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    db.close().await;
}
#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn setup_and_legacy_bootstrap_share_one_initialization_lock() {
    let db = Database::new().await;
    sqlx::query("DELETE FROM f_i_admin_users")
        .execute(&db.pool)
        .await
        .unwrap();
    let service = setup_service(&db);
    let repo = db.repository();
    let (setup, bootstrap) = tokio::join!(
        service.initialize(AdminSetupInput {
            password: NEW.into()
        }),
        repo.ensure_owner(bootstrap("legacy_owner"))
    );
    let account = bootstrap.unwrap();
    assert_eq!(
        account.username,
        if setup.is_ok() {
            "superadmin"
        } else {
            assert_eq!(setup.unwrap_err(), AdminSetupError::AlreadyInitialized);
            "legacy_owner"
        }
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM f_i_admin_users")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    assert!(!service.needs_setup().await.unwrap());
    db.close().await;
}

fn empty_bootstrap_config() -> AppConfig {
    AppConfig {
        port: 8080,
        database_url: "postgresql://example:example@127.0.0.1:5432/example".to_string(),
        database_pool: DatabasePoolConfig {
            max_connections: 10,
            min_connections: 3,
            acquire_timeout_secs: 5,
            idle_timeout_secs: 90,
            max_lifetime_secs: 600,
        },
        jwt_secret: "dev-secret".to_string(),
        openai_api_key: None,
        openai_base_url: None,
        ai_chat_model: "gpt-4o-mini".to_string(),
        wechat_app_id: "wx_test_app".to_string(),
        wechat_app_secret: "wx_test_secret".to_string(),
        wechat_mini_app_id: "wx_test_mini_app".to_string(),
        wechat_mini_app_secret: "wx_test_mini_secret".to_string(),
        wechat_webhook_token: String::new(),
        wechat_encoding_aes_key: String::new(),
        seat_swap_mini_subscribe_template_id: String::new(),
        seat_swap_mini_subscribe_page: String::new(),
        ticket_monitor_base_url: None,
        ticket_restock_profile_base_url: None,
        redis_url: "redis://127.0.0.1:6379".to_string(),
        wechat_pay_mch_id: String::new(),
        virtual_payment: football_insight_service_backend_rs::config::VirtualPaymentConfig {
            enabled: false,
            offer_id: String::new(),
            app_key: String::new(),
            webhook_token: String::new(),
            encoding_aes_key: String::new(),
        },
        wechat_pay_api_key: String::new(),
        public_base_url: "http://127.0.0.1:8080".to_string(),
        reflux_notification_worker: RefluxNotificationWorkerConfig {
            enabled: false,
            poll_seconds: 60,
        },
        smtp_email: None,
        minio: None,
        jpush: Some(JPushConfig {
            app_key: String::new(),
            master_secret: String::new(),
        }),
        admin_jwt_secret: "test-admin-jwt-secret-at-least-32-characters".to_string(),
        admin_owner_username: "superadmin".to_string(),
        admin_owner_password: String::new(),
        admin_owner_display_name: "Test Owner".to_string(),
        mini_review_api_key: None,
        mini_review_control_user_ids: Vec::new(),
        h5_test_login_user_ids: Vec::new(),
    }
}

#[tokio::test]
#[ignore = "requires explicit isolated localhost PostgreSQL database"]
async fn absent_initial_password_allows_startup_without_seeded_administrator() {
    let db = Database::new().await;
    sqlx::query("DELETE FROM f_i_admin_users")
        .execute(&db.pool)
        .await
        .unwrap();
    bootstrap_admin_owner(db.pool.clone(), &empty_bootstrap_config())
        .await
        .unwrap();
    assert!(setup_service(&db).needs_setup().await.unwrap());
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM f_i_admin_users")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    db.close().await;
}
