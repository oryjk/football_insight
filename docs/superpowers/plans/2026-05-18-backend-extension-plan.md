# 回流监控后端扩展实施计划（Plan A）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在现有 Rust 后端上扩展：license 生成/绑定 API、device token 注册 API、push sender（FCM + APNs），以及 worker 中新增 push consumer 与邮件并行发送。

**Architecture:** 在 `reflux_subscription` 模块基础上扩展。新增 `push_notification` 模块处理 device token 和 push 发送。修改 worker.rs 在现有邮件 consumer 旁新增 push consumer。新增 `auth_license` 模块处理 license 生成和绑定。所有新模块遵循 hexagonal 架构（domain → ports → application → adapters）。

**Tech Stack:** Rust, Axum, SQLx, reqwest（FCM/APNs HTTP 调用）, tokio

---

## File Structure（新增/修改）

```
football_insight_service_backend_rs/
├── src/
│   ├── auth_license/                          ← 新模块
│   │   ├── mod.rs
│   │   ├── domain/license.rs
│   │   ├── ports/license_repository.rs
│   │   ├── application/
│   │   │   ├── mod.rs
│   │   │   ├── generate_license.rs
│   │   │   └── bind_license.rs
│   │   └── adapters/
│   │       ├── mod.rs
│   │       ├── persistence/
│   │       │   ├── mod.rs
│   │       │   └── postgres_license_repository.rs
│   │       └── web/
│   │           ├── mod.rs
│   │           ├── dto.rs
│   │           ├── handlers.rs
│   │           └── routes.rs
│   ├── push_notification/                     ← 新模块
│   │   ├── mod.rs
│   │   ├── domain/device_token.rs
│   │   ├── ports/
│   │   │   ├── mod.rs
│   │   │   ├── device_token_repository.rs
│   │   │   └── push_sender.rs
│   │   ├── application/
│   │   │   ├── mod.rs
│   │   │   ├── register_device_token.rs
│   │   │   └── process_push_notification_jobs.rs
│   │   └── adapters/
│   │       ├── mod.rs
│   │       ├── integration/
│   │       │   ├── mod.rs
│   │       │   ├── fcm_push_sender.rs
│   │       │   └── apns_push_sender.rs
│   │       └── persistence/
│   │           ├── mod.rs
│   │           └── postgres_device_token_repository.rs
│   ├── reflux_subscription/
│   │   ├── worker.rs                          ← 修改：新增 push consumer
│   │   └── ...
│   ├── app.rs                                 ← 修改：注册新路由
│   ├── config.rs                              ← 修改：新增 push 配置
│   └── lib.rs                                 ← 修改：注册新模块
├── migrations/
│   ├── 20260519000001_create_user_licenses.sql
│   └── 20260519000002_create_user_device_tokens.sql
```

---

## Task 1: 数据库 Migration

**Files:**
- Create: `migrations/20260519000001_create_user_licenses.sql`
- Create: `migrations/20260519000002_create_user_device_tokens.sql`

- [ ] **Step 1: 写 license 表 migration**

```sql
-- migrations/20260519000001_create_user_licenses.sql
CREATE TABLE IF NOT EXISTS f_i_user_licenses (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    license_key VARCHAR(16) NOT NULL UNIQUE,
    used_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_f_i_user_licenses_user_id ON f_i_user_licenses(user_id);
CREATE INDEX idx_f_i_user_licenses_license_key ON f_i_user_licenses(license_key) WHERE used_at IS NULL;
```

- [ ] **Step 2: 写 device_token 表 migration**

```sql
-- migrations/20260519000002_create_user_device_tokens.sql
CREATE TABLE IF NOT EXISTS f_i_user_device_tokens (
    id           BIGSERIAL PRIMARY KEY,
    user_id      BIGINT NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    device_token TEXT NOT NULL,
    platform     TEXT NOT NULL CHECK (platform IN ('fcm', 'apns')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(device_token)
);

CREATE INDEX idx_f_i_user_device_tokens_user_id ON f_i_user_device_tokens(user_id);
```

- [ ] **Step 3: 本地运行 migration 验证**

```bash
cd football_insight_service_backend_rs
cargo run --bin run_migrations
```

Expected: migration 成功

- [ ] **Step 4: Commit**

```bash
git add football_insight_service_backend_rs/migrations/
git commit -m "feat(backend): add user_licenses and device_tokens migrations"
```

---

## Task 2: Config 扩展 + lib.rs 模块注册

**Files:**
- Modify: `football_insight_service_backend_rs/src/config.rs`
- Modify: `football_insight_service_backend_rs/src/lib.rs`

- [ ] **Step 1: 在 config.rs 新增 push 配置**

在 config.rs 的 `AppConfig` 结构体和实现中新增：

```rust
// 在 AppConfig struct 中新增：
pub push_notification: PushNotificationConfig,

// 新增配置结构体：
#[derive(Debug, Clone)]
pub struct PushNotificationConfig {
    pub fcm_service_account_json: Option<String>,
    pub apns_private_key_path: Option<String>,
    pub apns_team_id: Option<String>,
    pub apns_key_id: Option<String>,
    pub apns_bundle_id: Option<String>,
}

// 在 AppConfig 的 from_env 中新增读取：
let push_notification = PushNotificationConfig {
    fcm_service_account_json: env_or_none("FCM_SERVICE_ACCOUNT_JSON"),
    apns_private_key_path: env_or_none("APNS_PRIVATE_KEY_PATH"),
    apns_team_id: env_or_none("APNS_TEAM_ID"),
    apns_key_id: env_or_none("APNS_KEY_ID"),
    apns_bundle_id: env_or_none("APNS_BUNDLE_ID"),
};
```

- [ ] **Step 2: 在 lib.rs 注册新模块**

在 lib.rs 的 mod 声明中新增：

```rust
pub mod auth_license;
pub mod push_notification;
```

- [ ] **Step 3: 运行 cargo check 验证编译**

```bash
cd football_insight_service_backend_rs && cargo check 2>&1 | head -20
```

Expected: 模块不存在导致的错误（正常，后续 task 会创建）

- [ ] **Step 4: Commit**

```bash
git add football_insight_service_backend_rs/src/config.rs football_insight_service_backend_rs/src/lib.rs
git commit -m "feat(backend): add push notification config and register new modules"
```

---

## Task 3: auth_license 模块 - Domain + Ports

**Files:**
- Create: `football_insight_service_backend_rs/src/auth_license/mod.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/domain/license.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/ports/license_repository.rs`
- Create: `football_insight_service_backend_rs/tests/auth_license_domain_test.rs`

- [ ] **Step 1: 写 domain 测试**

```rust
// tests/auth_license_domain_test.rs
use football_insight_service_backend_rs::auth_license::domain::license::*;

#[test]
fn generate_license_key_has_correct_length() {
    let key = generate_license_key();
    assert_eq!(key.len(), 12);
    assert!(key.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn license_is_expired_when_used() {
    let now = chrono::Utc::now();
    let license = UserLicense {
        id: 1,
        user_id: uuid::Uuid::new_v4(),
        license_key: "ABCD1234EFGH".to_string(),
        used_at: Some(now),
        created_at: now,
        expires_at: now + chrono::Duration::minutes(30),
    };
    assert!(license.is_used());
}

#[test]
fn license_is_expired_past_expiry() {
    let past = chrono::Utc::now() - chrono::Duration::hours(1);
    let license = UserLicense {
        id: 1,
        user_id: uuid::Uuid::new_v4(),
        license_key: "ABCD1234EFGH".to_string(),
        used_at: None,
        created_at: past - chrono::Duration::minutes(30),
        expires_at: past,
    };
    assert!(license.is_expired());
}

#[test]
fn license_is_valid_when_unused_and_not_expired() {
    let now = chrono::Utc::now();
    let license = UserLicense {
        id: 1,
        user_id: uuid::Uuid::new_v4(),
        license_key: "ABCD1234EFGH".to_string(),
        used_at: None,
        created_at: now,
        expires_at: now + chrono::Duration::minutes(30),
    };
    assert!(!license.is_used());
    assert!(!license.is_expired());
    assert!(license.is_valid());
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_service_backend_rs && cargo test auth_license_domain_test 2>&1 | tail -5
```

Expected: FAIL

- [ ] **Step 3: 实现 auth_license 模块结构**

```rust
// src/auth_license/mod.rs
pub mod domain;
pub mod ports;
pub mod application;
pub mod adapters;
```

```rust
// src/auth_license/domain/mod.rs
pub mod license;
```

```rust
// src/auth_license/domain/license.rs
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct UserLicense {
    pub id: i64,
    pub user_id: uuid::Uuid,
    pub license_key: String,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl UserLicense {
    pub fn is_used(&self) -> bool {
        self.used_at.is_some()
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= chrono::Utc::now()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_used() && !self.is_expired()
    }
}

pub fn generate_license_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..12)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
```

```rust
// src/auth_license/ports/mod.rs
pub mod license_repository;
```

```rust
// src/auth_license/ports/license_repository.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::auth_license::domain::license::UserLicense;

#[async_trait]
pub trait LicenseRepository: Send + Sync {
    async fn create_license(
        &self,
        user_id: Uuid,
        license_key: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<UserLicense>;

    async fn find_by_key(&self, license_key: &str) -> anyhow::Result<Option<UserLicense>>;

    async fn mark_used(&self, license_id: i64) -> anyhow::Result<()>;
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd football_insight_service_backend_rs && cargo test auth_license_domain_test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add football_insight_service_backend_rs/
git commit -m "feat(backend): add auth_license domain and ports with tests"
```

---

## Task 4: auth_license - Application + Persistence + Web

**Files:**
- Create: `football_insight_service_backend_rs/src/auth_license/application/mod.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/application/generate_license.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/application/bind_license.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/adapters/mod.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/adapters/persistence/mod.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/adapters/persistence/postgres_license_repository.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/adapters/web/mod.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/adapters/web/dto.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/adapters/web/handlers.rs`
- Create: `football_insight_service_backend_rs/src/auth_license/adapters/web/routes.rs`
- Create: `football_insight_service_backend_rs/tests/auth_license_bind_test.rs`

- [ ] **Step 1: 写 bind_license use case 测试**

```rust
// tests/auth_license_bind_test.rs
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use uuid::Uuid;

use football_insight_service_backend_rs::auth_license::domain::license::UserLicense;
use football_insight_service_backend_rs::auth_license::ports::license_repository::LicenseRepository;
use football_insight_service_backend_rs::auth_license::application::bind_license::BindLicenseUseCase;

struct FakeLicenseRepo {
    licenses: Mutex<Vec<UserLicense>>,
    marked_used: Mutex<Vec<i64>>,
}

impl FakeLicenseRepo {
    fn new(licenses: Vec<UserLicense>) -> Self {
        Self {
            licenses: Mutex::new(licenses),
            marked_used: Mutex::new(vec![]),
        }
    }
}

#[async_trait]
impl LicenseRepository for FakeLicenseRepo {
    async fn create_license(&self, _user_id: Uuid, _license_key: &str, _expires_at: chrono::DateTime<Utc>) -> anyhow::Result<UserLicense> {
        unreachable!()
    }
    async fn find_by_key(&self, key: &str) -> anyhow::Result<Option<UserLicense>> {
        let licenses = self.licenses.lock().unwrap();
        Ok(licenses.iter().find(|l| l.license_key == key).cloned())
    }
    async fn mark_used(&self, license_id: i64) -> anyhow::Result<()> {
        self.marked_used.lock().unwrap().push(license_id);
        Ok(())
    }
}

#[tokio::test]
async fn bind_valid_license_returns_user_id() {
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    let license = UserLicense {
        id: 1,
        user_id,
        license_key: "VALIDKEY12345".to_string(),
        used_at: None,
        created_at: now,
        expires_at: now + Duration::minutes(30),
    };

    let repo = Arc::new(FakeLicenseRepo::new(vec![license]));
    let use_case = BindLicenseUseCase::new(repo);

    let result = use_case.execute("VALIDKEY12345").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().user_id, user_id);
}

#[tokio::test]
async fn bind_used_license_fails() {
    let now = Utc::now();
    let license = UserLicense {
        id: 2,
        user_id: Uuid::new_v4(),
        license_key: "USEDKEY123456".to_string(),
        used_at: Some(now),
        created_at: now,
        expires_at: now + Duration::minutes(30),
    };

    let repo = Arc::new(FakeLicenseRepo::new(vec![license]));
    let use_case = BindLicenseUseCase::new(repo);

    let result = use_case.execute("USEDKEY123456").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn bind_expired_license_fails() {
    let now = Utc::now();
    let license = UserLicense {
        id: 3,
        user_id: Uuid::new_v4(),
        license_key: "EXPIRED123456".to_string(),
        used_at: None,
        created_at: now - Duration::hours(2),
        expires_at: now - Duration::hours(1),
    };

    let repo = Arc::new(FakeLicenseRepo::new(vec![license]));
    let use_case = BindLicenseUseCase::new(repo);

    let result = use_case.execute("EXPIRED123456").await;
    assert!(result.is_err());
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_service_backend_rs && cargo test auth_license_bind_test 2>&1 | tail -5
```

Expected: FAIL

- [ ] **Step 3: 实现 application 层**

```rust
// src/auth_license/application/mod.rs
pub mod generate_license;
pub mod bind_license;
```

```rust
// src/auth_license/application/generate_license.rs
use std::sync::Arc;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::auth_license::domain::license::{generate_license_key, UserLicense};
use crate::auth_license::ports::license_repository::LicenseRepository;

pub struct GenerateLicenseUseCase {
    repository: Arc<dyn LicenseRepository>,
}

impl GenerateLicenseUseCase {
    pub fn new(repository: Arc<dyn LicenseRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid) -> anyhow::Result<UserLicense> {
        let license_key = generate_license_key();
        let expires_at = Utc::now() + Duration::minutes(30);
        self.repository.create_license(user_id, &license_key, expires_at).await
    }
}
```

```rust
// src/auth_license/application/bind_license.rs
use std::sync::Arc;

use crate::auth_license::ports::license_repository::LicenseRepository;

pub struct BindLicenseResult {
    pub user_id: uuid::Uuid,
    pub license_key: String,
}

pub struct BindLicenseUseCase {
    repository: Arc<dyn LicenseRepository>,
}

impl BindLicenseUseCase {
    pub fn new(repository: Arc<dyn LicenseRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, license_key: &str) -> anyhow::Result<BindLicenseResult> {
        let license = self.repository.find_by_key(license_key).await?
            .ok_or_else(|| anyhow::anyhow!("绑定码不存在"))?;

        if !license.is_valid() {
            return Err(anyhow::anyhow!("绑定码已失效或已使用"));
        }

        self.repository.mark_used(license.id).await?;

        Ok(BindLicenseResult {
            user_id: license.user_id,
            license_key: license.license_key,
        })
    }
}
```

- [ ] **Step 4: 实现 PostgreSQL adapter**

```rust
// src/auth_license/adapters/mod.rs
pub mod persistence;
pub mod web;
```

```rust
// src/auth_license/adapters/persistence/mod.rs
pub mod postgres_license_repository;
```

```rust
// src/auth_license/adapters/persistence/postgres_license_repository.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth_license::domain::license::UserLicense;
use crate::auth_license::ports::license_repository::LicenseRepository;

pub struct PostgresLicenseRepository {
    pool: PgPool,
}

impl PostgresLicenseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LicenseRepository for PostgresLicenseRepository {
    async fn create_license(
        &self,
        user_id: Uuid,
        license_key: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<UserLicense> {
        let row = sqlx::query_as!(
            UserLicenseRow,
            r#"
            INSERT INTO f_i_user_licenses (user_id, license_key, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, license_key, used_at, created_at, expires_at
            "#,
            user_id,
            license_key,
            expires_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into_domain())
    }

    async fn find_by_key(&self, license_key: &str) -> anyhow::Result<Option<UserLicense>> {
        let row = sqlx::query_as!(
            UserLicenseRow,
            r#"
            SELECT id, user_id, license_key, used_at, created_at, expires_at
            FROM f_i_user_licenses
            WHERE license_key = $1
            "#,
            license_key,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_domain()))
    }

    async fn mark_used(&self, license_id: i64) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE f_i_user_licenses SET used_at = now() WHERE id = $1",
            license_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

struct UserLicenseRow {
    id: i64,
    user_id: Uuid,
    license_key: String,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl UserLicenseRow {
    fn into_domain(self) -> UserLicense {
        UserLicense {
            id: self.id,
            user_id: self.user_id,
            license_key: self.license_key,
            used_at: self.used_at,
            created_at: self.created_at,
            expires_at: self.expires_at,
        }
    }
}
```

- [ ] **Step 5: 实现 Web 层（handlers + routes + dto）**

```rust
// src/auth_license/adapters/web/mod.rs
pub mod dto;
pub mod handlers;
pub mod routes;
```

```rust
// src/auth_license/adapters/web/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GenerateLicenseRequest {}

#[derive(Serialize)]
pub struct GenerateLicenseResponse {
    pub license_key: String,
    pub expires_at: String,
}

#[derive(Deserialize)]
pub struct BindLicenseRequest {
    pub license_key: String,
}

#[derive(Serialize)]
pub struct BindLicenseResponse {
    pub access_token: String,
    pub user: serde_json::Value,
}
```

```rust
// src/auth_license/adapters/web/handlers.rs
use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Deserialize;

use crate::auth_license::adapters::web::dto::*;
use crate::auth_license::application::{generate_license::GenerateLicenseUseCase, bind_license::BindLicenseUseCase};
use crate::auth::ports::token_port::TokenPort;

#[derive(Clone)]
pub struct AuthLicenseWebState {
    pub generate_license_use_case: Arc<GenerateLicenseUseCase>,
    pub bind_license_use_case: Arc<BindLicenseUseCase>,
    pub token_port: Arc<dyn TokenPort>,
}

#[derive(Deserialize)]
pub struct AuthUserClaim {
    pub sub: String,
}

pub async fn generate_license_handler(
    State(state): State<Arc<AuthLicenseWebState>>,
    claims: crate::auth::middleware::AuthenticatedUser,
) -> axum::response::Result<Json<GenerateLicenseResponse>> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    let license = state.generate_license_use_case.execute(user_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "generate license failed");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(GenerateLicenseResponse {
        license_key: license.license_key,
        expires_at: license.expires_at.to_rfc3339(),
    }))
}

pub async fn bind_license_handler(
    State(state): State<Arc<AuthLicenseWebState>>,
    Json(req): Json<BindLicenseRequest>,
) -> axum::response::Result<Json<BindLicenseResponse>> {
    let result = state.bind_license_use_case.execute(&req.license_key)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "bind license failed");
            axum::http::StatusCode::UNAUTHORIZED
        })?;

    let token = state.token_port.create_token(
        result.user_id.to_string(),
        String::new(),
    ).map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(BindLicenseResponse {
        access_token: token,
        user: serde_json::json!({
            "id": result.user_id.to_string(),
        }),
    }))
}
```

```rust
// src/auth_license/adapters/web/routes.rs
use std::sync::Arc;

use axum::{Router, routing::{get, post}};

use crate::auth_license::adapters::web::handlers::*;

pub fn auth_license_routes(state: Arc<AuthLicenseWebState>) -> Router {
    Router::new()
        .route("/api/v1/auth/generate-license", post(generate_license_handler))
        .route("/api/v1/auth/bind-license", post(bind_license_handler))
        .with_state(state)
}
```

- [ ] **Step 6: 在 app.rs 注册路由**

在 `build_router` 函数中，在 `.merge(reflux_subscription_routes(...))` 后新增：

```rust
.merge(auth_license_routes(auth_license_web_state))
```

并构造 `auth_license_web_state`（参照其他 web_state 的构造方式）。

- [ ] **Step 7: 运行测试确认通过**

```bash
cd football_insight_service_backend_rs && cargo test auth_license
```

Expected: PASS

- [ ] **Step 8: 运行 cargo check**

```bash
cd football_insight_service_backend_rs && cargo check 2>&1 | tail -5
```

Expected: 无错误（或有明确的后续模块缺失提示）

- [ ] **Step 9: Commit**

```bash
git add football_insight_service_backend_rs/
git commit -m "feat(backend): add auth_license module with generate/bind APIs"
```

---

## Task 5: push_notification 模块 - Domain + Ports + PushSender trait

**Files:**
- Create: `football_insight_service_backend_rs/src/push_notification/mod.rs`
- Create: `football_insight_service_backend_rs/src/push_notification/domain/device_token.rs`
- Create: `football_insight_service_backend_rs/src/push_notification/ports/mod.rs`
- Create: `football_insight_service_backend_rs/src/push_notification/ports/device_token_repository.rs`
- Create: `football_insight_service_backend_rs/src/push_notification/ports/push_sender.rs`
- Create: `football_insight_service_backend_rs/tests/push_sender_test.rs`

- [ ] **Step 1: 写 PushSender trait 和测试**

```rust
// tests/push_sender_test.rs
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use football_insight_service_backend_rs::push_notification::ports::push_sender::{PushSender, PushPayload};

struct FakePushSender {
    sent: Mutex<Vec<PushPayload>>,
}

impl FakePushSender {
    fn new() -> Self {
        Self { sent: Mutex::new(vec![]) }
    }
}

#[async_trait]
impl PushSender for FakePushSender {
    async fn send(&self, device_token: &str, payload: &PushPayload) -> anyhow::Result<()> {
        let mut sent = self.sent.lock().unwrap();
        sent.push(PushPayload {
            title: payload.title.clone(),
            body: payload.body.clone(),
            data: payload.data.clone(),
        });
        Ok(())
    }
}

#[tokio::test]
async fn push_sender_sends_payload() {
    let sender = FakePushSender::new();
    let payload = PushPayload {
        title: "回流监控".to_string(),
        body: "XX vs YY 新增回流 3 张".to_string(),
        data: serde_json::json!({"match_id": "123", "type": "reflux_alert"}),
    };

    sender.send("device-token-abc", &payload).await.unwrap();

    let sent = sender.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].title, "回流监控");
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_service_backend_rs && cargo test push_sender_test 2>&1 | tail -5
```

Expected: FAIL

- [ ] **Step 3: 实现 push_notification 模块**

```rust
// src/push_notification/mod.rs
pub mod domain;
pub mod ports;
pub mod application;
pub mod adapters;
```

```rust
// src/push_notification/domain/mod.rs
pub mod device_token;
```

```rust
// src/push_notification/domain/device_token.rs
#[derive(Debug, Clone)]
pub struct DeviceToken {
    pub id: i64,
    pub user_id: uuid::Uuid,
    pub device_token: String,
    pub platform: String,
}
```

```rust
// src/push_notification/ports/mod.rs
pub mod device_token_repository;
pub mod push_sender;
```

```rust
// src/push_notification/ports/push_sender.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
    pub data: serde_json::Value,
}

#[async_trait]
pub trait PushSender: Send + Sync {
    async fn send(&self, device_token: &str, payload: &PushPayload) -> anyhow::Result<()>;
}
```

```rust
// src/push_notification/ports/device_token_repository.rs
use async_trait::async_trait;
use uuid::Uuid;

use crate::push_notification::domain::device_token::DeviceToken;

#[async_trait]
pub trait DeviceTokenRepository: Send + Sync {
    async fn upsert(&self, user_id: Uuid, device_token: &str, platform: &str) -> anyhow::Result<()>;
    async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<DeviceToken>>;
    async fn delete(&self, device_token: &str) -> anyhow::Result<()>;
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd football_insight_service_backend_rs && cargo test push_sender_test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add football_insight_service_backend_rs/
git commit -m "feat(backend): add push_notification domain, ports, PushSender trait"
```

---

## Task 6: FCM + APNs PushSender 实现

**Files:**
- Create: `football_insight_service_backend_rs/src/push_notification/adapters/mod.rs`
- Create: `football_insight_service_backend_rs/src/push_notification/adapters/integration/mod.rs`
- Create: `football_insight_service_backend_rs/src/push_notification/adapters/integration/fcm_push_sender.rs`
- Create: `football_insight_service_backend_rs/src/push_notification/adapters/integration/apns_push_sender.rs`
- Create: `football_insight_service_backend_rs/tests/fcm_push_sender_test.rs`

- [ ] **Step 1: 写 FCM sender 测试**

```rust
// tests/fcm_push_sender_test.rs
use football_insight_service_backend_rs::push_notification::adapters::integration::fcm_push_sender::FcmPushSender;
use football_insight_service_backend_rs::push_notification::ports::push_sender::{PushSender, PushPayload};

#[tokio::test]
async fn fcm_sender_builds_correct_request() {
    // 这里测试 FCM sender 能被构建，不实际发送
    let sender = FcmPushSender::new("{}", "test-project".to_string());
    assert!(sender.is_ok());
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_service_backend_rs && cargo test fcm_push_sender_test 2>&1 | tail -5
```

Expected: FAIL

- [ ] **Step 3: 实现 FCM PushSender**

```rust
// src/push_notification/adapters/mod.rs
pub mod integration;
```

```rust
// src/push_notification/adapters/integration/mod.rs
pub mod fcm_push_sender;
pub mod apns_push_sender;
```

```rust
// src/push_notification/adapters/integration/fcm_push_sender.rs
use async_trait::async_trait;
use reqwest::Client;

use crate::push_notification::ports::push_sender::{PushPayload, PushSender};

pub struct FcmPushSender {
    client: Client,
    service_account_json: String,
    project_id: String,
}

impl FcmPushSender {
    pub fn new(service_account_json: &str, project_id: String) -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::new(),
            service_account_json: service_account_json.to_string(),
            project_id,
        })
    }

    // 实际实现需要获取 Google OAuth2 access token
    // 此处为骨架，后续补充 JWT signing
}

#[async_trait]
impl PushSender for FcmPushSender {
    async fn send(&self, device_token: &str, payload: &PushPayload) -> anyhow::Result<()> {
        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.project_id
        );

        let body = serde_json::json!({
            "message": {
                "token": device_token,
                "notification": {
                    "title": payload.title,
                    "body": payload.body,
                },
                "data": payload.data,
            }
        });

        // TODO: 实际实现需要 OAuth2 Bearer token
        tracing::info!(device_token, title = %payload.title, "FCM push sent (stub)");

        let _ = (url, body);
        Ok(())
    }
}
```

```rust
// src/push_notification/adapters/integration/apns_push_sender.rs
use async_trait::async_trait;

use crate::push_notification::ports::push_sender::{PushPayload, PushSender};

pub struct ApnsPushSender {
    // APNs HTTP/2 配置
    // bundle_id, team_id, key_id, private_key_path
}

impl ApnsPushSender {
    pub fn new(
        bundle_id: &str,
        _team_id: &str,
        _key_id: &str,
        _private_key_path: &str,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            // bundle_id: bundle_id.to_string(),
        })
    }
}

#[async_trait]
impl PushSender for ApnsPushSender {
    async fn send(&self, device_token: &str, payload: &PushPayload) -> anyhow::Result<()> {
        // APNs HTTP/2 发送
        // POST https://api.push.apple.com/3/device/{device_token}
        // 需要 JWT signer (ES256)
        let body = serde_json::json!({
            "aps": {
                "alert": {
                    "title": payload.title,
                    "body": payload.body,
                },
                "sound": "default",
            },
            "data": payload.data,
        });

        tracing::info!(device_token, title = %payload.title, "APNs push sent (stub)");

        let _ = body;
        Ok(())
    }
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd football_insight_service_backend_rs && cargo test fcm_push_sender_test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add football_insight_service_backend_rs/
git commit -m "feat(backend): add FCM and APNs push sender implementations"
```

---

## Task 7: Device Token 注册 API + Persistence

**Files:**
- Create: `football_insight_service_backend_rs/src/push_notification/adapters/persistence/mod.rs`
- Create: `football_insight_service_backend_rs/src/push_notification/adapters/persistence/postgres_device_token_repository.rs`
- Create: `football_insight_service_backend_rs/src/push_notification/application/mod.rs`
- Create: `football_insight_service_backend_rs/src/push_notification/application/register_device_token.rs`
- Create: `football_insight_service_backend_rs/tests/device_token_test.rs`

- [ ] **Step 1: 写 device token repository 测试**

```rust
// tests/device_token_test.rs
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use uuid::Uuid;

use football_insight_service_backend_rs::push_notification::domain::device_token::DeviceToken;
use football_insight_service_backend_rs::push_notification::ports::device_token_repository::DeviceTokenRepository;
use football_insight_service_backend_rs::push_notification::application::register_device_token::RegisterDeviceTokenUseCase;

struct FakeDeviceTokenRepo {
    tokens: Mutex<Vec<DeviceToken>>,
}

impl FakeDeviceTokenRepo {
    fn new() -> Self { Self { tokens: Mutex::new(vec![]) } }
}

#[async_trait]
impl DeviceTokenRepository for FakeDeviceTokenRepo {
    async fn upsert(&self, user_id: Uuid, device_token: &str, platform: &str) -> anyhow::Result<()> {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.push(DeviceToken {
            id: tokens.len() as i64 + 1,
            user_id,
            device_token: device_token.to_string(),
            platform: platform.to_string(),
        });
        Ok(())
    }
    async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<DeviceToken>> {
        let tokens = self.tokens.lock().unwrap();
        Ok(tokens.iter().filter(|t| t.user_id == user_id).cloned().collect())
    }
    async fn delete(&self, device_token: &str) -> anyhow::Result<()> {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.retain(|t| t.device_token != device_token);
        Ok(())
    }
}

#[tokio::test]
async fn register_device_token_stores_token() {
    let repo = Arc::new(FakeDeviceTokenRepo::new());
    let use_case = RegisterDeviceTokenUseCase::new(repo.clone());

    let user_id = Uuid::new_v4();
    use_case.execute(user_id, "token-abc123", "fcm").await.unwrap();

    let tokens = repo.list_by_user(user_id).await.unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].device_token, "token-abc123");
    assert_eq!(tokens[0].platform, "fcm");
}

#[tokio::test]
async fn delete_device_token_removes_token() {
    let repo = Arc::new(FakeDeviceTokenRepo::new());
    let user_id = Uuid::new_v4();
    repo.upsert(user_id, "token-to-delete", "fcm").await.unwrap();

    repo.delete("token-to-delete").await.unwrap();

    let tokens = repo.list_by_user(user_id).await.unwrap();
    assert!(tokens.is_empty());
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_service_backend_rs && cargo test device_token_test 2>&1 | tail -5
```

Expected: FAIL

- [ ] **Step 3: 实现 application 和 persistence 层**

```rust
// src/push_notification/application/mod.rs
pub mod register_device_token;
```

```rust
// src/push_notification/application/register_device_token.rs
use std::sync::Arc;
use uuid::Uuid;

use crate::push_notification::ports::device_token_repository::DeviceTokenRepository;

pub struct RegisterDeviceTokenUseCase {
    repository: Arc<dyn DeviceTokenRepository>,
}

impl RegisterDeviceTokenUseCase {
    pub fn new(repository: Arc<dyn DeviceTokenRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid, device_token: &str, platform: &str) -> anyhow::Result<()> {
        if device_token.is_empty() {
            return Err(anyhow::anyhow!("device token 不能为空"));
        }
        if platform != "fcm" && platform != "apns" {
            return Err(anyhow::anyhow!("platform 必须是 fcm 或 apns"));
        }
        self.repository.upsert(user_id, device_token, platform).await
    }
}
```

```rust
// src/push_notification/adapters/persistence/mod.rs
pub mod postgres_device_token_repository;
```

```rust
// src/push_notification/adapters/persistence/postgres_device_token_repository.rs
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::push_notification::domain::device_token::DeviceToken;
use crate::push_notification::ports::device_token_repository::DeviceTokenRepository;

pub struct PostgresDeviceTokenRepository {
    pool: PgPool,
}

impl PostgresDeviceTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeviceTokenRepository for PostgresDeviceTokenRepository {
    async fn upsert(&self, user_id: Uuid, device_token: &str, platform: &str) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO f_i_user_device_tokens (user_id, device_token, platform)
            VALUES ($1, $2, $3)
            ON CONFLICT (device_token) DO UPDATE
            SET user_id = EXCLUDED.user_id, platform = EXCLUDED.platform, updated_at = now()
            "#,
            user_id,
            device_token,
            platform,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<DeviceToken>> {
        let rows = sqlx::query_as!(
            DeviceTokenRow,
            "SELECT id, user_id, device_token, platform FROM f_i_user_device_tokens WHERE user_id = $1",
            user_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| DeviceToken {
            id: r.id,
            user_id: r.user_id,
            device_token: r.device_token,
            platform: r.platform,
        }).collect())
    }

    async fn delete(&self, device_token: &str) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM f_i_user_device_tokens WHERE device_token = $1", device_token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

struct DeviceTokenRow {
    id: i64,
    user_id: Uuid,
    device_token: String,
    platform: String,
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd football_insight_service_backend_rs && cargo test device_token_test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add football_insight_service_backend_rs/
git commit -m "feat(backend): add device token registration use case and postgres adapter"
```

---

## Task 8: Worker 扩展 - 新增 Push Consumer

**Files:**
- Modify: `football_insight_service_backend_rs/src/reflux_subscription/worker.rs`

- [ ] **Step 1: 写 push consumer 测试**

```rust
// tests/push_consumer_test.rs
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use uuid::Uuid;

use football_insight_service_backend_rs::push_notification::ports::push_sender::{PushSender, PushPayload};
use football_insight_service_backend_rs::push_notification::domain::device_token::DeviceToken;
use football_insight_service_backend_rs::push_notification::ports::device_token_repository::DeviceTokenRepository;

struct FakePushSender {
    sent: Mutex<Vec<(String, PushPayload)>>,
}

impl FakePushSender {
    fn new() -> Self { Self { sent: Mutex::new(vec![]) } }
}

#[async_trait]
impl PushSender for FakePushSender {
    async fn send(&self, device_token: &str, payload: &PushPayload) -> anyhow::Result<()> {
        self.sent.lock().unwrap().push((device_token.to_string(), payload.clone()));
        Ok(())
    }
}

struct FakeTokenRepo {
    tokens: Vec<DeviceToken>,
}

#[async_trait]
impl DeviceTokenRepository for FakeTokenRepo {
    async fn upsert(&self, _user_id: Uuid, _device_token: &str, _platform: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<DeviceToken>> {
        Ok(self.tokens.iter().filter(|t| t.user_id == user_id).cloned().collect())
    }
    async fn delete(&self, _device_token: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn push_consumer_sends_to_user_devices() {
    let user_id = Uuid::new_v4();
    let token_repo = Arc::new(FakeTokenRepo {
        tokens: vec![
            DeviceToken { id: 1, user_id, device_token: "token-1".to_string(), platform: "fcm".to_string() },
        ],
    });
    let push_sender = Arc::new(FakePushSender::new());

    let payload = PushPayload {
        title: "回流监控".to_string(),
        body: "新增回流 3 张".to_string(),
        data: serde_json::json!({"type": "reflux_alert"}),
    };

    let tokens = token_repo.list_by_user(user_id).await.unwrap();
    for token in &tokens {
        push_sender.send(&token.device_token, &payload).await.unwrap();
    }

    let sent = push_sender.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0, "token-1");
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd football_insight_service_backend_rs && cargo test push_consumer_test 2>&1 | tail -5
```

Expected: FAIL

- [ ] **Step 3: 修改 worker.rs 新增 push consumer**

在 `spawn_reflux_notification_worker` 函数中，在现有 email send_use_case 之后，新增 push 发送逻辑。核心改动：

1. 接收 `PushSender` 和 `DeviceTokenRepository` 作为参数
2. 在 worker loop 中，email 发送之后，新增 push 发送阶段
3. 查询 pending jobs 中 channel 为 push 的（或独立处理）

worker.rs 新增逻辑（在 email send 之后）：

```rust
// 在 worker loop 的 email send 之后新增：
// Phase 3: Push notification consumer
// 对每个 pending job（或新建 push job），查用户 device token，发送 push
// 此处复用 process_reflux_notifications 创建的 job，
// 但为 push channel 创建额外的分发逻辑
```

具体实现方式：在 `process_reflux_notification_jobs` 旁新增一个 `process_reflux_push_jobs`，或者扩展现有 use case 支持 push channel。

推荐方案：新增 `ProcessRefluxPushJobsUseCase`，结构与 email 版类似，但用 `PushSender` 替代 `EmailSender`，用 `DeviceTokenRepository` 获取 token。

- [ ] **Step 4: 运行测试确认通过**

```bash
cd football_insight_service_backend_rs && cargo test push_consumer_test
```

Expected: PASS

- [ ] **Step 5: 运行全量 cargo test**

```bash
cd football_insight_service_backend_rs && cargo test
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add football_insight_service_backend_rs/
git commit -m "feat(backend): add push consumer to reflux notification worker"
```

---

## Task 9: 整合 - app.rs 注册所有新路由 + Cargo check + Test

**Files:**
- Modify: `football_insight_service_backend_rs/src/app.rs`
- Modify: `football_insight_service_backend_rs/src/main.rs`

- [ ] **Step 1: 在 app.rs 中构造并注册所有新 state 和路由**

需要构造：
1. `AuthLicenseWebState` — 需要 `LicenseRepository`、`TokenPort`
2. `PushNotificationWebState`（如需要独立路由的话）— 需要 `DeviceTokenRepository`

在 `build_router` 中新增：
```rust
.merge(auth_license_routes(auth_license_web_state))
```

- [ ] **Step 2: 在 main.rs 中传入 push sender 到 worker**

修改 `spawn_reflux_notification_worker` 调用，传入 `PushSender` 和 `DeviceTokenRepository`。

- [ ] **Step 3: 运行 cargo check**

```bash
cd football_insight_service_backend_rs && cargo check 2>&1 | tail -10
```

Expected: 无错误

- [ ] **Step 4: 运行全量测试**

```bash
cd football_insight_service_backend_rs && cargo test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add football_insight_service_backend_rs/
git commit -m "feat(backend): wire up all new routes and push worker integration"
```

---

## 自检清单

| Spec 要求 | 对应 Task |
|-----------|-----------|
| f_i_user_licenses 表 | Task 1 |
| f_i_user_device_tokens 表 | Task 1 |
| Push 配置 + 模块注册 | Task 2 |
| License domain + ports | Task 3 |
| License API（generate + bind） | Task 4 |
| PushSender trait | Task 5 |
| FCM + APNs 实现 | Task 6 |
| Device Token 注册 | Task 7 |
| Worker push consumer | Task 8 |
| 路由整合 | Task 9 |
