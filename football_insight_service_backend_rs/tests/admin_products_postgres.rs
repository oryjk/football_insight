//! Run explicitly against a disposable localhost database ending in _test.
use football_insight_service_backend_rs::admin::{
    adapters::persistence::postgres_admin_product_repository::PostgresAdminProductRepository,
    application::admin_product_service::AdminProductService,
    domain::admin_product::{AdminProductError, AdminProductInput, AdminProductQuery},
};
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::sync::Arc;
use uuid::Uuid;
struct Database {
    admin: PgPool,
    pool: PgPool,
    schema: String,
    actor: Uuid,
}
impl Database {
    async fn new() -> Self {
        let url = std::env::var("ADMIN_PRODUCTS_TEST_DATABASE_URL")
            .expect("explicit disposable test URL required");
        let parsed = url::Url::parse(&url).expect("valid test URL");
        assert!(
            matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")),
            "only localhost test databases permitted"
        );
        assert!(
            parsed.path().ends_with("_test"),
            "test database must end in _test"
        );
        let admin = PgPool::connect(&url).await.unwrap();
        let schema = format!("admin_products_{}", Uuid::new_v4().simple());
        sqlx::raw_sql(&format!("CREATE SCHEMA {schema}"))
            .execute(&admin)
            .await
            .unwrap();
        let options = url
            .parse::<PgConnectOptions>()
            .unwrap()
            .options([("search_path", schema.as_str())]);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .unwrap();
        for migration in [
            include_str!("../migrations/20260802120000_add_admin_identity_and_audit.sql"),
            include_str!("../migrations/20261013220000_add_virtual_payment_products.sql"),
            include_str!("../migrations/20261013223000_add_restricted_virtual_products.sql"),
        ] {
            sqlx::raw_sql(migration).execute(&pool).await.unwrap();
        }
        // The duration migration also targets order snapshots; these tests only exercise catalog CRUD.
        sqlx::raw_sql("CREATE TABLE f_i_virtual_payment_orders (order_no TEXT PRIMARY KEY,product_id TEXT,amount_cents INTEGER,grant_before JSONB,grant_after JSONB)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::raw_sql(include_str!(
            "../migrations/20261013230000_add_membership_duration.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        sqlx::raw_sql("CREATE TABLE f_i_users(id UUID PRIMARY KEY,display_name TEXT,avatar_url TEXT,membership_tier TEXT NOT NULL DEFAULT 'V1',membership_expires_at TIMESTAMPTZ,status TEXT NOT NULL DEFAULT 'active',created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()); CREATE TABLE f_i_payment_orders(order_no TEXT PRIMARY KEY,user_id UUID NOT NULL,product_type TEXT NOT NULL,amount_cents INTEGER NOT NULL,status TEXT NOT NULL,paid_at TIMESTAMPTZ);").execute(&pool).await.unwrap();
        sqlx::raw_sql(include_str!(
            "../migrations/20261014093000_add_virtual_product_purchase_identity.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        sqlx::raw_sql(include_str!(
            "../migrations/20261014110000_add_membership_origin.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        sqlx::raw_sql(include_str!(
            "../migrations/20261016130000_add_product_catalog_specs.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        let actor = Uuid::new_v4();
        sqlx::query("INSERT INTO f_i_admin_users(id,username,password_hash,display_name,role) VALUES($1,'test_owner','unused','Test Owner','owner')").bind(actor).execute(&pool).await.unwrap();
        Self {
            admin,
            pool,
            schema,
            actor,
        }
    }
    fn service(&self) -> AdminProductService {
        AdminProductService::new(Arc::new(PostgresAdminProductRepository::new(
            self.pool.clone(),
        )))
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
fn input(tier: &str, price: i32) -> AdminProductInput {
    AdminProductInput {
        membership_tier: tier.into(),
        name: "QA Annual".into(),
        product_id: "qa-wechat-id".into(),
        price_cents: price,
        membership_duration_months: 12,
        enabled: false,
        remark: "QA only".into(),
        allowed_user_ids: vec![],
    }
}
#[tokio::test]
#[ignore = "requires an explicit disposable localhost PostgreSQL database"]
async fn crud_change_key_conflict_filter_scope_and_audit_snapshots() {
    let db = Database::new().await;
    let service = db.service();
    let created = service.create(input("V7", 3500), db.actor).await.unwrap();
    assert_eq!(created.product_key, "membership:V7@3500");
    assert_eq!(service.get(&created.product_key).await.unwrap(), created);
    assert_eq!(
        service
            .create(input("V7", 3500), db.actor)
            .await
            .unwrap_err(),
        AdminProductError::Conflict
    );
    sqlx::query("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents) VALUES('match_id_unlock@100','QA Annual','other',100)").execute(&db.pool).await.unwrap();
    let mut changed = input("V8", 4500);
    changed.enabled = true;
    let updated = service
        .update(&created.product_key, changed, db.actor)
        .await
        .unwrap();
    assert_eq!(updated.product_key, "membership:V8@4500");
    assert_eq!(updated.created_at, created.created_at);
    assert!(updated.enabled);
    assert_eq!(
        service.get(&created.product_key).await.unwrap_err(),
        AdminProductError::NotFound
    );
    assert_eq!(
        service
            .update(&updated.product_key, input("V6", 3900), db.actor)
            .await
            .unwrap_err(),
        AdminProductError::Conflict
    );
    assert_eq!(service.get(&updated.product_key).await.unwrap(), updated);
    let page = service
        .list(AdminProductQuery {
            query: Some("QA".into()),
            membership_tier: Some("V8".into()),
            enabled: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items, vec![updated.clone()]);
    assert_eq!(
        service
            .list(AdminProductQuery {
                query: Some("%".into()),
                ..Default::default()
            })
            .await
            .unwrap()
            .total,
        0
    );
    let page = service
        .list(AdminProductQuery {
            page_size: 1,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.items.len(), 1);
    assert!(service.get("match_id_unlock@100").await.is_err());
    assert!(
        service
            .delete("match_id_unlock@100", db.actor)
            .await
            .is_err()
    );
    service
        .delete(&updated.product_key, db.actor)
        .await
        .unwrap();
    assert_eq!(
        service
            .delete(&updated.product_key, db.actor)
            .await
            .unwrap_err(),
        AdminProductError::NotFound
    );
    let rows=sqlx::query("SELECT action,admin_user_id,target_id,before_json,after_json FROM f_i_admin_audit_logs ORDER BY created_at,id").fetch_all(&db.pool).await.unwrap();
    assert_eq!(rows.len(), 3);
    let create = &rows[0];
    assert_eq!(create.get::<String, _>("action"), "product.create");
    assert_eq!(create.get::<Uuid, _>("admin_user_id"), db.actor);
    assert!(
        create
            .get::<Option<serde_json::Value>, _>("before_json")
            .is_none()
    );
    assert_eq!(
        create.get::<serde_json::Value, _>("after_json")["product_key"],
        "membership:V7@3500"
    );
    assert_eq!(
        rows[1].get::<serde_json::Value, _>("before_json")["product_key"],
        "membership:V7@3500"
    );
    assert_eq!(
        rows[1].get::<serde_json::Value, _>("after_json")["price_cents"],
        4500
    );
    assert_eq!(rows[2].get::<String, _>("action"), "product.delete");
    assert_eq!(
        rows[2].get::<serde_json::Value, _>("before_json")["product_key"],
        "membership:V8@4500"
    );
    assert!(
        rows[2]
            .get::<Option<serde_json::Value>, _>("after_json")
            .is_none()
    );
    db.close().await;
}
#[tokio::test]
#[ignore = "requires an explicit disposable localhost PostgreSQL database"]
async fn audit_failure_rolls_back_create_update_and_delete() {
    let db = Database::new().await;
    let service = db.service();
    let missing_actor = Uuid::new_v4();
    assert_eq!(
        service
            .create(input("V9", 9999), missing_actor)
            .await
            .unwrap_err(),
        AdminProductError::Internal
    );
    assert_eq!(
        service.get("membership:V9@9999").await.unwrap_err(),
        AdminProductError::NotFound
    );
    let created = service.create(input("V7", 3500), db.actor).await.unwrap();
    assert_eq!(
        service
            .update(&created.product_key, input("V8", 4500), missing_actor)
            .await
            .unwrap_err(),
        AdminProductError::Internal
    );
    assert_eq!(service.get(&created.product_key).await.unwrap(), created);
    assert_eq!(
        service.get("membership:V8@4500").await.unwrap_err(),
        AdminProductError::NotFound
    );
    assert_eq!(
        service
            .delete(&created.product_key, missing_actor)
            .await
            .unwrap_err(),
        AdminProductError::Internal
    );
    assert_eq!(service.get(&created.product_key).await.unwrap(), created);
    let audits: i64 = sqlx::query_scalar("SELECT count(*) FROM f_i_admin_audit_logs")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(audits, 1);
    let missing = "membership:V8@5000";
    assert_eq!(
        service
            .update(missing, input("V8", 5000), db.actor)
            .await
            .unwrap_err(),
        AdminProductError::NotFound
    );
    db.close().await;
}

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use football_insight_service_backend_rs::{
    admin::{
        adapters::{
            persistence::postgres_admin_auth_repository::PostgresAdminAuthRepository,
            security::{
                jwt_admin_token_port::JwtAdminTokenPort,
                role_based_admin_authorization::RoleBasedAdminAuthorization,
            },
            web::product_routes::admin_product_routes,
        },
        application::admin_auth_service::AdminAuthService,
        ports::admin_token_port::AdminTokenPort,
    },
    auth::{
        adapters::security::{
            argon2_password_port::Argon2PasswordPort, jwt_token_port::JwtTokenPort,
        },
        ports::token_port::TokenPort,
    },
};
use serde_json::{Value, json};
use tower::ServiceExt;
fn http_app(db: &Database) -> axum::Router {
    let auth = Arc::new(AdminAuthService::new(
        Arc::new(PostgresAdminAuthRepository::new(db.pool.clone())),
        Arc::new(Argon2PasswordPort),
        Arc::new(JwtAdminTokenPort::new(
            "isolated-admin-products-test-secret".into(),
        )),
        Arc::new(RoleBasedAdminAuthorization),
        Duration::hours(12),
        Arc::new(Utc::now),
    ));
    admin_product_routes(Arc::new(db.service()), auth)
}
async fn token(db: &Database) -> (String, Uuid) {
    let sid = Uuid::new_v4();
    let expires = Utc::now() + Duration::hours(1);
    sqlx::query("INSERT INTO f_i_admin_sessions(id,admin_user_id,expires_at) VALUES($1,$2,$3)")
        .bind(sid)
        .bind(db.actor)
        .bind(expires)
        .execute(&db.pool)
        .await
        .unwrap();
    let token = JwtAdminTokenPort::new("isolated-admin-products-test-secret".into())
        .issue_token(db.actor, sid, "test_owner", "owner", expires)
        .unwrap();
    (token, sid)
}
async fn request(
    app: &axum::Router,
    method: &str,
    path: &str,
    bearer: Option<&str>,
    body: Option<Value>,
) -> axum::response::Response {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = bearer {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let body = if let Some(body) = body {
        builder = builder.header("Content-Type", "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    app.clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap()
}
async fn body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap()
}
fn payload() -> Value {
    json!({"membership_tier":"V7","name":"API Annual","product_id":"qa-api-id","price_cents":3500,"enabled":false,"remark":"","allowed_user_ids":[]})
}
#[tokio::test]
#[ignore = "requires an explicit disposable localhost PostgreSQL database"]
async fn http_crud_matches_contract_and_returns_safe_errors() {
    let db = Database::new().await;
    let app = http_app(&db);
    let (token, _) = token(&db).await;
    let r = request(
        &app,
        "POST",
        "/api/v1/admin/products",
        Some(&token),
        Some(payload()),
    )
    .await;
    assert_eq!(r.status(), StatusCode::OK);
    let created = body(r).await;
    assert_eq!(created["product_key"], "membership:V7@3500");
    assert_eq!(created["membership_tier"], "V7");
    assert!(created["created_at"].is_string());
    assert_eq!(created.as_object().unwrap().len(), 12);
    assert!(
        created["catalog_product_id"]
            .as_str()
            .and_then(|value| Uuid::parse_str(value).ok())
            .is_some()
    );
    assert_eq!(created["membership_duration_months"], 12);
    let r = request(
        &app,
        "GET",
        "/api/v1/admin/products?query=&membership_tier=&enabled=&page=1&page_size=20",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(r.status(), StatusCode::OK);
    let page = body(r).await;
    assert_eq!(page["total"], 2);
    assert_eq!(page["page_size"], 20);
    let key = "/api/v1/admin/products/membership%3AV7%403500";
    let r = request(&app, "GET", key, Some(&token), None).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(body(r).await, created);
    let mut changed = payload();
    changed["membership_tier"] = json!("V8");
    changed["price_cents"] = json!(4500);
    let r = request(&app, "PUT", key, Some(&token), Some(changed.clone())).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(body(r).await["product_key"], "membership:V8@4500");
    assert_eq!(
        request(&app, "GET", key, Some(&token), None).await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/v1/admin/products",
            Some(&token),
            Some(changed)
        )
        .await
        .status(),
        StatusCode::CONFLICT
    );
    let mut invalid = payload();
    invalid["membership_tier"] = json!("V3");
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/v1/admin/products",
            Some(&token),
            Some(invalid)
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );
    for path in [
        "/api/v1/admin/products?page=0",
        "/api/v1/admin/products?enabled=wat",
        "/api/v1/admin/products/match_id_unlock%40100",
    ] {
        assert_eq!(
            request(&app, "GET", path, Some(&token), None)
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
    let key = "/api/v1/admin/products/membership%3AV8%404500";
    assert_eq!(
        request(&app, "DELETE", key, Some(&token), None)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&app, "DELETE", key, Some(&token), None)
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    sqlx::query("DROP TABLE f_i_virtual_payment_products")
        .execute(&db.pool)
        .await
        .unwrap();
    let r = request(&app, "GET", "/api/v1/admin/products", Some(&token), None).await;
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let text = String::from_utf8(to_bytes(r.into_body(), 1024).await.unwrap().to_vec()).unwrap();
    assert!(!text.contains("f_i_"));
    assert!(!text.contains("SELECT"));
    db.close().await;
}
#[tokio::test]
#[ignore = "requires an explicit disposable localhost PostgreSQL database"]
async fn http_enforces_admin_jwt_sessions_live_roles_and_permission_on_all_routes() {
    let db = Database::new().await;
    let app = http_app(&db);
    let (token, sid) = token(&db).await;
    let routes = [
        ("GET", "/api/v1/admin/products"),
        ("POST", "/api/v1/admin/products"),
        ("GET", "/api/v1/admin/products/membership%3AV6%403900"),
        ("PUT", "/api/v1/admin/products/membership%3AV6%403900"),
        ("DELETE", "/api/v1/admin/products/membership%3AV6%403900"),
    ];
    for (method, path) in routes {
        assert_eq!(
            request(&app, method, path, None, Some(payload()))
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    let user_token = JwtTokenPort::new("isolated-admin-products-test-secret".into())
        .issue_token(db.actor, "user", Utc::now() + Duration::hours(1))
        .unwrap();
    let expired = JwtAdminTokenPort::new("isolated-admin-products-test-secret".into())
        .issue_token(
            db.actor,
            sid,
            "test_owner",
            "owner",
            Utc::now() - Duration::hours(1),
        )
        .unwrap();
    for invalid in ["invalid", user_token.as_str(), expired.as_str()] {
        assert_eq!(
            request(&app, "GET", "/api/v1/admin/products", Some(invalid), None)
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    sqlx::query("ALTER TABLE f_i_admin_users DROP CONSTRAINT chk_f_i_admin_users_role")
        .execute(&db.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE f_i_admin_users SET role='viewer' WHERE id=$1")
        .bind(db.actor)
        .execute(&db.pool)
        .await
        .unwrap();
    for (method, path) in routes {
        assert_eq!(
            request(&app, method, path, Some(&token), Some(payload()))
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    sqlx::query("UPDATE f_i_admin_users SET role='admin' WHERE id=$1")
        .bind(db.actor)
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(
        request(&app, "GET", "/api/v1/admin/products", Some(&token), None)
            .await
            .status(),
        StatusCode::OK
    );
    sqlx::query("UPDATE f_i_admin_users SET status='disabled' WHERE id=$1")
        .bind(db.actor)
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(
        request(&app, "GET", "/api/v1/admin/products", Some(&token), None)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    sqlx::query("UPDATE f_i_admin_users SET status='active' WHERE id=$1")
        .bind(db.actor)
        .execute(&db.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE f_i_admin_sessions SET expires_at=NOW()-INTERVAL '1 second' WHERE id=$1")
        .bind(sid)
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(
        request(&app, "GET", "/api/v1/admin/products", Some(&token), None)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    sqlx::query("UPDATE f_i_admin_sessions SET expires_at=NOW()+INTERVAL '1 hour' WHERE id=$1")
        .bind(sid)
        .execute(&db.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE f_i_admin_sessions SET revoked_at=NOW() WHERE id=$1")
        .bind(sid)
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(
        request(&app, "GET", "/api/v1/admin/products", Some(&token), None)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    sqlx::query("DROP TABLE f_i_admin_sessions")
        .execute(&db.pool)
        .await
        .unwrap();
    let r = request(&app, "GET", "/api/v1/admin/products", Some(&token), None).await;
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable localhost PostgreSQL database"]
async fn membership_duration_roundtrips_and_is_in_all_audit_snapshots() {
    let db = Database::new().await;
    let service = db.service();
    let mut new = input("V7", 3500);
    new.membership_duration_months = 1;
    let created = service.create(new, db.actor).await.unwrap();
    let stored: i32 = sqlx::query_scalar(
        "SELECT membership_duration_months FROM f_i_virtual_payment_products WHERE product_key=$1",
    )
    .bind(&created.product_key)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(stored, 1);
    assert_eq!(created.membership_duration_months, 1);
    let mut changed = input("V7", 3500);
    changed.membership_duration_months = 1200;
    let updated = service
        .update(&created.product_key, changed.clone(), db.actor)
        .await
        .unwrap();
    assert_eq!(updated.membership_duration_months, 1200);
    assert_eq!(
        service
            .get(&updated.product_key)
            .await
            .unwrap()
            .membership_duration_months,
        1200
    );
    let listed = service
        .list(AdminProductQuery {
            membership_tier: Some("V7".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(listed.items[0].membership_duration_months, 1200);
    changed.enabled = true;
    assert_eq!(
        service
            .update(&updated.product_key, changed, db.actor)
            .await
            .unwrap()
            .membership_duration_months,
        1200
    );
    for months in [0, 1201] {
        let mut invalid = input("V7", 3500);
        invalid.membership_duration_months = months;
        assert!(matches!(
            service
                .update(&updated.product_key, invalid, db.actor)
                .await,
            Err(AdminProductError::InvalidInput(_))
        ));
    }
    service
        .delete(&updated.product_key, db.actor)
        .await
        .unwrap();
    let rows = sqlx::query(
        "SELECT before_json,after_json FROM f_i_admin_audit_logs ORDER BY created_at,id",
    )
    .fetch_all(&db.pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(
        rows[0].get::<Value, _>("after_json")["membership_duration_months"],
        1
    );
    assert_eq!(
        rows[1].get::<Value, _>("before_json")["membership_duration_months"],
        1
    );
    assert_eq!(
        rows[1].get::<Value, _>("after_json")["membership_duration_months"],
        1200
    );
    assert_eq!(
        rows[3].get::<Value, _>("before_json")["membership_duration_months"],
        1200
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable localhost PostgreSQL database"]
async fn http_duration_defaults_for_old_clients_validates_and_survives_toggle() {
    let db = Database::new().await;
    let app = http_app(&db);
    let (token, _) = token(&db).await;
    let path = "/api/v1/admin/products";
    let mut product = payload();
    product["membership_duration_months"] = json!(1);
    let response = request(&app, "POST", path, Some(&token), Some(product.clone())).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body(response).await["membership_duration_months"], 1);
    let key = "/api/v1/admin/products/membership%3AV7%403500";
    product["membership_duration_months"] = json!(1200);
    let response = request(&app, "PUT", key, Some(&token), Some(product.clone())).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body(response).await["membership_duration_months"], 1200);
    product["enabled"] = json!(true);
    let response = request(&app, "PUT", key, Some(&token), Some(product.clone())).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body(response).await["membership_duration_months"], 1200);
    for months in [json!(0), json!(1201), json!(1.5)] {
        let mut invalid = product.clone();
        invalid["membership_duration_months"] = months;
        assert_eq!(
            request(&app, "PUT", key, Some(&token), Some(invalid))
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
    let response = request(&app, "GET", key, Some(&token), None).await;
    assert_eq!(body(response).await["membership_duration_months"], 1200);
    let response = request(&app, "PUT", key, Some(&token), Some(payload())).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body(response).await["membership_duration_months"], 12);
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable localhost PostgreSQL database"]
async fn product_purchases_summarize_history_and_preserve_identity_across_edits() {
    use chrono::{DateTime, Utc};
    use football_insight_service_backend_rs::admin::domain::admin_product_purchase::PurchaseExpirySource;
    let db = Database::new().await;
    let service = db.service();
    let id: Uuid = sqlx::query_scalar(
        "SELECT id FROM f_i_virtual_payment_products WHERE product_key='membership:V6@3900'",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    sqlx::query("INSERT INTO f_i_users(id,display_name,avatar_url) VALUES($1,'Buyer One','https://example.test/avatar.png'),($2,'Buyer Two',NULL)").bind(first).bind(second).execute(&db.pool).await.unwrap();
    for (no, user, status, paid, amount) in [
        (
            "snapshot",
            first,
            "paid",
            Some("2025-04-01T00:00:00Z"),
            3900,
        ),
        ("legacy", first, "paid", Some("2024-02-29T00:00:00Z"), 3900),
        (
            "refunded",
            second,
            "refunded",
            Some("2025-03-01T00:00:00Z"),
            3900,
        ),
        (
            "pending",
            second,
            "pending",
            Some("2025-05-01T00:00:00Z"),
            3900,
        ),
        (
            "failed",
            second,
            "failed",
            Some("2025-06-01T00:00:00Z"),
            3900,
        ),
        ("unpaid", second, "paid", None, 3900),
    ] {
        sqlx::query("INSERT INTO f_i_payment_orders(order_no,user_id,product_type,amount_cents,status,paid_at,catalog_product_id) VALUES($1,$2,'membership:V6',$3,$4,$5::text::timestamptz,$6)").bind(no).bind(user).bind(amount).bind(status).bind(paid).bind(id).execute(&db.pool).await.unwrap();
    }
    sqlx::query("INSERT INTO f_i_virtual_payment_orders(order_no,product_id,amount_cents,grant_after) VALUES('snapshot','fi_v6_year_39',3900,$1),('refunded','fi_v6_year_39',3900,NULL)").bind(json!({"expires_at":"2025-05-01T00:00:00Z"})).execute(&db.pool).await.unwrap();
    let page = service.purchases("membership:V6@3900", 1, 1).await.unwrap();
    assert_eq!(page.total, 3);
    assert_eq!(page.summary.purchased_user_count, 2);
    assert_eq!(page.summary.purchased_order_count, 3);
    assert_eq!(page.summary.refunded_order_count, 1);
    assert_eq!(page.summary.total_amount_cents, 11700);
    assert_eq!(page.items[0].order_no, "snapshot");
    assert_eq!(page.items[0].user_nickname, "Buyer One");
    assert_eq!(
        page.items[0].user_avatar_url.as_deref(),
        Some("https://example.test/avatar.png")
    );
    assert_eq!(
        page.items[0].expires_at.unwrap(),
        DateTime::parse_from_rfc3339("2025-05-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    );
    assert_eq!(page.items[0].expiry_source, PurchaseExpirySource::Snapshot);
    let page2 = service.purchases("membership:V6@3900", 2, 1).await.unwrap();
    assert_eq!(page2.summary, page.summary);
    assert_eq!(page2.items[0].order_no, "refunded");
    assert!(page2.items[0].user_avatar_url.is_none());
    assert_eq!(
        page2.items[0].expiry_source,
        PurchaseExpirySource::Unrecorded
    );
    assert!(page2.items[0].expires_at.is_none());
    let page3 = service.purchases("membership:V6@3900", 3, 1).await.unwrap();
    assert_eq!(page3.items[0].order_no, "legacy");
    assert_eq!(
        page3.items[0].expiry_source,
        PurchaseExpirySource::LegacyYear
    );
    assert_eq!(
        page3.items[0].expires_at.unwrap(),
        DateTime::parse_from_rfc3339("2025-02-28T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    );
    let mut changed = input("V7", 5900);
    changed.product_id = "different_wechat_id".into();
    service
        .update("membership:V6@3900", changed, db.actor)
        .await
        .unwrap();
    assert_eq!(
        service
            .purchases("membership:V7@5900", 1, 20)
            .await
            .unwrap()
            .summary,
        page.summary
    );
    assert_eq!(
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM f_i_virtual_payment_products WHERE product_key='membership:V7@5900'"
        )
        .fetch_one(&db.pool)
        .await
        .unwrap(),
        id
    );
    assert!(sqlx::query("UPDATE f_i_virtual_payment_products SET id=gen_random_uuid() WHERE product_key='membership:V7@5900'").execute(&db.pool).await.is_err());
    service
        .delete("membership:V7@5900", db.actor)
        .await
        .unwrap();
    service.create(input("V7", 5900), db.actor).await.unwrap();
    assert_eq!(
        service
            .purchases("membership:V7@5900", 1, 20)
            .await
            .unwrap()
            .summary
            .purchased_order_count,
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM f_i_payment_orders WHERE catalog_product_id=$1"
        )
        .bind(id)
        .fetch_one(&db.pool)
        .await
        .unwrap(),
        6
    );
    db.close().await;
}
#[tokio::test]
#[ignore = "requires an explicit disposable localhost PostgreSQL database"]
async fn product_purchases_http_requires_permission_validates_and_disables_cache() {
    let db = Database::new().await;
    let app = http_app(&db);
    let (token, _) = token(&db).await;
    let path = "/api/v1/admin/products/membership%3AV6%403900/purchases";
    assert_eq!(
        request(&app, "GET", path, None, None).await.status(),
        StatusCode::UNAUTHORIZED
    );
    let response = request(&app, "GET", path, Some(&token), None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let result = body(response).await;
    let id: Uuid = sqlx::query_scalar(
        "SELECT id FROM f_i_virtual_payment_products WHERE product_key='membership:V6@3900'",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(result["catalog_product_id"], json!(id));
    assert_eq!(result["summary"]["purchased_user_count"], 0);
    assert_eq!(result["total"], 0);
    assert_eq!(result["page"], 1);
    assert_eq!(result["page_size"], 20);
    for path in [
        "/api/v1/admin/products/membership%3AV6%403900/purchases?page=0",
        "/api/v1/admin/products/membership%3AV6%403900/purchases?page_size=101",
        "/api/v1/admin/products/malformed/purchases",
    ] {
        assert_eq!(
            request(&app, "GET", path, Some(&token), None)
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/v1/admin/products/membership%3AV7%405900/purchases",
            Some(&token),
            None
        )
        .await
        .status(),
        StatusCode::NOT_FOUND
    );
    sqlx::query("ALTER TABLE f_i_admin_users DROP CONSTRAINT chk_f_i_admin_users_role")
        .execute(&db.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE f_i_admin_users SET role='viewer'")
        .execute(&db.pool)
        .await
        .unwrap();
    assert_eq!(
        request(&app, "GET", path, Some(&token), None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable localhost PostgreSQL database"]
async fn same_tier_members_endpoint_includes_unknown_without_inventing_orders() {
    let db = Database::new().await;
    for (index, origin) in ["purchase", "referral", "unknown"].iter().enumerate() {
        sqlx::query("INSERT INTO f_i_users(id,display_name,membership_tier,membership_origin) VALUES($1,$2,'V6',$3)")
            .bind(Uuid::new_v4()).bind(format!("member{index}")).bind(origin).execute(&db.pool).await.unwrap();
    }
    sqlx::query("INSERT INTO f_i_users(id,membership_tier,status) VALUES($1,'V6','disabled'),($2,'V9','active')").bind(Uuid::new_v4()).bind(Uuid::new_v4()).execute(&db.pool).await.unwrap();
    let app = http_app(&db);
    let (token, _) = token(&db).await;
    let path = "/api/v1/admin/products/membership%3AV6%403900/members";
    assert_eq!(
        request(&app, "GET", path, None, None).await.status(),
        StatusCode::UNAUTHORIZED
    );
    let response = request(&app, "GET", path, Some(&token), None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let result = body(response).await;
    assert_eq!(result["total"], 3);
    assert_eq!(result["membership_tier"], "V6");
    for (origins, count) in [
        ("unknown", 1),
        ("purchase,referral", 2),
        ("referral,unknown", 2),
        ("purchase,referral,unknown", 3),
        ("", 0),
    ] {
        let filtered = body(
            request(
                &app,
                "GET",
                &format!("{path}?origins={origins}"),
                Some(&token),
                None,
            )
            .await,
        )
        .await;
        assert_eq!(filtered["total"], count);
        assert_eq!(filtered["items"].as_array().unwrap().len(), count as usize);
    }
    assert_eq!(
        request(
            &app,
            "GET",
            &format!("{path}?origins=bogus"),
            Some(&token),
            None
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );

    assert!(
        result["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["membership_origin"] == "unknown")
    );
    assert!(
        result["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item.get("order_no").is_none())
    );
    assert_eq!(
        body(
            request(
                &app,
                "GET",
                &format!("{path}?page_size=1&page=2"),
                Some(&token),
                None
            )
            .await
        )
        .await["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        request(&app, "GET", &format!("{path}?page=0"), Some(&token), None)
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        body(
            request(
                &app,
                "GET",
                "/api/v1/admin/products/membership%3AV6%403900/purchases",
                Some(&token),
                None
            )
            .await
        )
        .await["total"],
        0
    );
    db.close().await;
}

#[tokio::test]
#[ignore = "requires an explicit disposable localhost PostgreSQL database"]
async fn legacy_admin_crud_remains_compatible_with_catalog_kind_default_and_v4_v5_scope() {
    let db = Database::new().await;
    let service = db.service();
    for (tier, price) in [("V4", 900), ("V5", 1900)] {
        let created = service.create(input(tier, price), db.actor).await.unwrap();
        let persisted: (String, serde_json::Value) = sqlx::query_as(
            "SELECT kind,grant_spec FROM f_i_virtual_payment_products WHERE product_key=$1",
        )
        .bind(&created.product_key)
        .fetch_one(&db.pool)
        .await
        .unwrap();
        assert_eq!(persisted, ("membership".into(), serde_json::json!({})));
        assert_eq!(
            service
                .get(&created.product_key)
                .await
                .unwrap()
                .membership_tier,
            tier
        );
        let page = service
            .list(AdminProductQuery {
                membership_tier: Some(tier.into()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(
            service
                .purchases(&created.product_key, 1, 20)
                .await
                .unwrap()
                .total,
            0
        );
        let mut edited = input(tier, price);
        edited.membership_duration_months = 3;
        assert_eq!(
            service
                .update(&created.product_key, edited, db.actor)
                .await
                .unwrap()
                .membership_duration_months,
            3
        );
        service
            .delete(&created.product_key, db.actor)
            .await
            .unwrap();
        assert!(matches!(
            service.get(&created.product_key).await,
            Err(AdminProductError::NotFound)
        ));
    }
    db.close().await;
}
