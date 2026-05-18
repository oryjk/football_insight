# Reflux Email Subscription Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add paid email subscriptions for ticket reflux alerts, with database-configured plans, WeChat Pay checkout, SMTP email delivery, and a one-minute backend poller.

**Architecture:** Add a new `reflux_subscription` backend module that owns plans, user notification targets, subscriptions, notification cursors, and email jobs. Reuse the existing payment order and WeChat Pay notify flow by dispatching settlement based on `product_type`. The mini app adds a subscription CTA beside the existing monitor button and renders plans from backend data.

**Tech Stack:** Rust/Axum/SQLx/PostgreSQL/Tokio/lettre on the backend; uni-app/Vue 3/TypeScript/Bun on the frontend.

---

## File Map

- Create `football_insight_service_backend_rs/migrations/20260518170000_add_reflux_email_subscriptions.sql`: tables, indexes, seed plans.
- Create backend module `football_insight_service_backend_rs/src/reflux_subscription/`: domain models, ports, use cases, persistence adapter, SMTP adapter, web adapter, worker.
- Modify `football_insight_service_backend_rs/src/lib.rs`: export module.
- Modify `football_insight_service_backend_rs/src/config.rs`: SMTP and worker config.
- Modify `football_insight_service_backend_rs/src/app.rs`: wire routes and use cases.
- Modify `football_insight_service_backend_rs/src/main.rs`: start worker after DB connection.
- Modify payment files to dispatch `reflux_subscription:*` settlement.
- Modify auth user model/DTO/repository to expose `notification_email`.
- Modify mini files `src/api/ticketWatch.ts`, `src/types/ticketWatch.ts`, `src/pages/ticket-watch/index.vue`, `src/pages/ticket-watch/helpers.ts`.
- Modify mini user page/types to show and edit notification email.

## Tasks

### Task 1: Backend Domain And Migration

**Files:**
- Create: `football_insight_service_backend_rs/migrations/20260518170000_add_reflux_email_subscriptions.sql`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/domain/mod.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/domain/subscription.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/mod.rs`

- [ ] Add tables and seed global plans for `single_match` and `season_2026`.
- [ ] Add domain types for plan scope, subscription status, notification target, and notification job status.
- [ ] Add domain tests for email validation, plan fallback ordering, and subscription matching.
- [ ] Run: `cd football_insight_service_backend_rs && cargo test reflux_subscription::domain -- --nocapture`.

### Task 2: Backend Persistence And Use Cases

**Files:**
- Create: `football_insight_service_backend_rs/src/reflux_subscription/ports/mod.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/ports/reflux_subscription_repository.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/application/mod.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/application/get_reflux_subscription_plans.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/application/create_reflux_subscription_order.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/application/settle_reflux_subscription_order.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/adapters/persistence/postgres_reflux_subscription_repository.rs`
- Modify: `football_insight_service_backend_rs/src/reflux_subscription/adapters/mod.rs`

- [ ] Write application tests with fake repositories for plan fallback, order creation, and settlement.
- [ ] Implement repository trait and Postgres adapter.
- [ ] Reuse existing `WechatPayPort` for order creation.
- [ ] Run targeted tests: `cd football_insight_service_backend_rs && cargo test reflux_subscription::application -- --nocapture`.

### Task 3: Payment Dispatch

**Files:**
- Modify: `football_insight_service_backend_rs/src/payment/domain/order.rs`
- Modify: `football_insight_service_backend_rs/src/payment/ports/payment_settlement_port.rs`
- Modify: `football_insight_service_backend_rs/src/payment/adapters/persistence/postgres_payment_settlement_port.rs`
- Modify: `football_insight_service_backend_rs/src/payment/application/handle_wechat_notify.rs`

- [ ] Add product type helpers for `reflux_subscription`.
- [ ] Extend notify settlement to route membership and reflux subscription orders.
- [ ] Keep existing membership behavior unchanged.
- [ ] Run: `cd football_insight_service_backend_rs && cargo test payment::application::handle_wechat_notify -- --nocapture`.

### Task 4: Web API

**Files:**
- Create: `football_insight_service_backend_rs/src/reflux_subscription/adapters/web/dto.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/adapters/web/handlers.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/adapters/web/routes.rs`
- Modify: `football_insight_service_backend_rs/src/app.rs`

- [ ] Add authenticated endpoints under `/api/v1/ticket-watch/reflux-subscriptions`.
- [ ] Implement `GET /plans`, `GET /status`, `POST /order`.
- [ ] Add route tests for auth rejection and happy-path DTO serialization.
- [ ] Run: `cd football_insight_service_backend_rs && cargo test reflux_subscription::adapters::web -- --nocapture`.

### Task 5: Email Sender And Worker

**Files:**
- Create: `football_insight_service_backend_rs/src/reflux_subscription/ports/email_sender.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/adapters/integration/smtp_email_sender.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/application/process_reflux_notifications.rs`
- Create: `football_insight_service_backend_rs/src/reflux_subscription/worker.rs`
- Modify: `football_insight_service_backend_rs/src/config.rs`
- Modify: `football_insight_service_backend_rs/src/main.rs`

- [ ] Add SMTP config using `FI_SMTP_*`.
- [ ] Add `lettre` dependency with rustls or native TLS support compatible with the existing backend.
- [ ] Implement HTML welcome and reflux alert email bodies.
- [ ] Implement one-minute worker with DB cursor and retry handling.
- [ ] Run: `cd football_insight_service_backend_rs && cargo test reflux_subscription -- --nocapture`.

### Task 6: Auth Email Exposure

**Files:**
- Modify: `football_insight_service_backend_rs/src/auth/domain/user.rs`
- Modify: `football_insight_service_backend_rs/src/auth/adapters/persistence/postgres_auth_repository.rs`
- Modify: `football_insight_service_backend_rs/src/auth/adapters/web/dto.rs`
- Create or modify auth email update endpoint if needed.

- [ ] Expose `notification_email` in current user DTO.
- [ ] Add persistence method to upsert email target.
- [ ] Add API for updating email from “我的” page.
- [ ] Run auth tests and targeted web tests.

### Task 7: Mini API And UI

**Files:**
- Modify: `football_insight_mini/src/types/auth.ts`
- Modify: `football_insight_mini/src/types/ticketWatch.ts`
- Modify: `football_insight_mini/src/api/ticketWatch.ts`
- Modify: `football_insight_mini/src/api/auth.ts`
- Modify: `football_insight_mini/src/pages/ticket-watch/helpers.ts`
- Modify: `football_insight_mini/src/pages/ticket-watch/index.vue`
- Modify: `football_insight_mini/src/pages/user/helpers.ts`
- Modify: `football_insight_mini/src/pages/user/index.vue`

- [ ] Add types for plans, status, and order creation.
- [ ] Add two-button action layout.
- [ ] Add subscription purchase modal with editable email.
- [ ] Reuse `uni.requestPayment` and poll order status.
- [ ] Add focused helper tests.
- [ ] Run: `cd football_insight_mini && bun run type-check`.

### Task 8: Verification

**Files:**
- Backend and frontend touched files.

- [ ] Run backend targeted tests.
- [ ] Run `cd football_insight_service_backend_rs && cargo test`.
- [ ] Run `cd football_insight_mini && bun run type-check`.
- [ ] Run `cd football_insight_mini && bun run build:mp-weixin`.
- [ ] Report local backend restart status separately from production deploy status.
