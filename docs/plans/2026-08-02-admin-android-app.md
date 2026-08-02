# Football Insight Admin Android App Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a native Android owner app for managing Football Insight users through a separate, audited admin API.

**Architecture:** Extend the existing Rust `admin` bounded context with separate admin identities, revocable sessions, admin JWTs, audit logs, and richer user queries. Add a standalone Kotlin/Jetpack Compose app that only calls `/api/v1/admin/**`, stores its session securely, and never embeds backend secrets.

**Tech Stack:** Rust 2024, Axum, SQLx, PostgreSQL, Kotlin, Jetpack Compose, Material 3, Retrofit, OkHttp, Coroutines, StateFlow, Android Keystore, JUnit, MockWebServer.

---

### Task 1: Admin Identity Persistence

**Files:**
- Create: `football_insight_service_backend_rs/migrations/20260802120000_add_admin_identity_and_audit.sql`
- Create: `football_insight_service_backend_rs/src/admin/domain/admin_auth.rs`
- Create: `football_insight_service_backend_rs/src/admin/ports/admin_auth_repository.rs`
- Create: `football_insight_service_backend_rs/src/admin/ports/admin_token_port.rs`
- Create: `football_insight_service_backend_rs/src/admin/adapters/persistence/postgres_admin_auth_repository.rs`
- Create: `football_insight_service_backend_rs/src/admin/adapters/security/jwt_admin_token_port.rs`

1. Write failing domain/token tests for admin-only claims and expiration.
2. Add tables for admin users, sessions, and audit logs.
3. Implement repository ports and SQLx adapters.
4. Run focused Rust tests and keep the module dependency direction clean.

### Task 2: Admin Authentication API

**Files:**
- Create: `football_insight_service_backend_rs/src/admin/application/admin_auth_service.rs`
- Modify: `football_insight_service_backend_rs/src/admin/adapters/web/dto.rs`
- Modify: `football_insight_service_backend_rs/src/admin/adapters/web/handlers.rs`
- Modify: `football_insight_service_backend_rs/src/admin/adapters/web/routes.rs`
- Modify: `football_insight_service_backend_rs/src/admin/bootstrap.rs`
- Modify: `football_insight_service_backend_rs/src/config.rs`
- Modify: `football_insight_service_backend_rs/src/app.rs`
- Modify: `football_insight_service_backend_rs/.env.example`
- Test: `football_insight_service_backend_rs/tests/admin_auth_http.rs`

1. Write failing HTTP tests for login, me, logout, and rejected C-end/shared tokens.
2. Implement bootstrap-owner creation from environment variables.
3. Issue admin JWTs tied to live database sessions.
4. Replace `X-Admin-Token` enforcement with admin Bearer authentication.

### Task 3: Rich User Management API

**Files:**
- Modify: `football_insight_service_backend_rs/src/admin/domain/admin_user.rs`
- Modify: `football_insight_service_backend_rs/src/admin/ports/admin_user_repository.rs`
- Modify: `football_insight_service_backend_rs/src/admin/application/admin_user_service.rs`
- Modify: `football_insight_service_backend_rs/src/admin/adapters/persistence/postgres_admin_user_repository.rs`
- Modify: `football_insight_service_backend_rs/src/admin/adapters/web/dto.rs`
- Modify: `football_insight_service_backend_rs/src/admin/adapters/web/handlers.rs`
- Modify: `football_insight_service_backend_rs/src/admin/adapters/web/routes.rs`
- Modify: `football_insight_service_backend_rs/tests/admin_users_http.rs`

1. Write failing tests for list filters, detail aggregates, disable/restore, membership adjustment, and audit listing.
2. Implement account/nickname/status/tier filters and disabled-user visibility.
3. Implement detail queries for referrals, activity, orders/subscriptions, and devices.
4. Implement soft state transitions and membership changes with required reasons and audit records.

### Task 4: Native Android Project

**Files:**
- Create: `football_insight_admin_android/` Gradle project
- Create: `football_insight_admin_android/app/src/main/java/com/footballinsight/admin/data/**`
- Create: `football_insight_admin_android/app/src/main/java/com/footballinsight/admin/domain/**`
- Create: `football_insight_admin_android/app/src/main/java/com/footballinsight/admin/ui/**`
- Create: `football_insight_admin_android/app/src/test/**`

1. Generate a Gradle Wrapper and minimal Compose application.
2. Write failing tests for API serialization, authorization headers, session storage, filters, and membership request mapping.
3. Implement login, secure session persistence, and authenticated Retrofit client.
4. Implement Users, User Detail, Audit, and Settings screens.
5. Implement membership adjustment, disable/restore confirmations, loading/empty/error states, and logout.

### Task 5: Verification

1. Run `cargo fmt --check`, `cargo test`, and `cargo check`.
2. Start a freshly built local backend before runtime API checks.
3. Run Android `testDebugUnitTest`, `lintDebug`, and `assembleDebug` with the local Android SDK.
4. Confirm no `.env`, credentials, APKs, or build outputs are tracked.
5. Update root documentation to list the fourth monorepo project and its commands.
