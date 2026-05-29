# Admin User Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a React + shadcn/ui + TypeScript admin system whose first feature is user list search and user membership CRUD.

**Architecture:** Add a backend `admin` module that exposes `/api/v1/admin/users` endpoints protected by `X-Admin-Token`. Add a separate Vite React app in `football_insight_admin` so the admin UI does not mix with the existing uni-app mini program.

**Tech Stack:** Rust + Axum + SQLx + PostgreSQL, Vite + React + TypeScript + Tailwind CSS + shadcn-style components.

---

## File Structure

- `football_insight_service_backend_rs/src/admin/`: backend admin module.
- `football_insight_service_backend_rs/tests/admin_users_http.rs`: HTTP-level admin API tests.
- `football_insight_service_backend_rs/src/app.rs`: merge admin routes into the app router.
- `football_insight_service_backend_rs/src/lib.rs`: expose the admin module.
- `football_insight_admin/`: standalone React admin app.

## Tasks

### Task 1: Backend Admin User API

**Files:**
- Create: `football_insight_service_backend_rs/src/admin/mod.rs`
- Create: `football_insight_service_backend_rs/src/admin/bootstrap.rs`
- Create: `football_insight_service_backend_rs/src/admin/adapters/web/dto.rs`
- Create: `football_insight_service_backend_rs/src/admin/adapters/web/handlers.rs`
- Create: `football_insight_service_backend_rs/src/admin/adapters/web/routes.rs`
- Create: `football_insight_service_backend_rs/src/admin/adapters/persistence/postgres_admin_user_repository.rs`
- Create: `football_insight_service_backend_rs/src/admin/application/admin_user_service.rs`
- Create: `football_insight_service_backend_rs/src/admin/domain/admin_user.rs`
- Create: `football_insight_service_backend_rs/src/admin/ports/admin_user_repository.rs`
- Create: `football_insight_service_backend_rs/tests/admin_users_http.rs`
- Modify: `football_insight_service_backend_rs/src/app.rs`
- Modify: `football_insight_service_backend_rs/src/lib.rs`

- [ ] Write failing HTTP tests for missing token, user search, create, update membership, and delete.
- [ ] Run `cargo test --test admin_users_http` and confirm the tests fail because routes are missing.
- [ ] Implement admin domain, repository, service, DTOs, handlers, and routes.
- [ ] Run `cargo test --test admin_users_http` and confirm the tests pass.

### Task 2: React Admin App Scaffold

**Files:**
- Create: `football_insight_admin/package.json`
- Create: `football_insight_admin/index.html`
- Create: `football_insight_admin/src/main.tsx`
- Create: `football_insight_admin/src/App.tsx`
- Create: `football_insight_admin/src/lib/api.ts`
- Create: `football_insight_admin/src/lib/utils.ts`
- Create: `football_insight_admin/src/components/ui/*.tsx`
- Create: `football_insight_admin/src/styles.css`
- Create: `football_insight_admin/vite.config.ts`
- Create: `football_insight_admin/tsconfig.json`
- Create: `football_insight_admin/tsconfig.node.json`

- [ ] Scaffold Vite React TypeScript app with Tailwind and shadcn-style primitives.
- [ ] Build a token login screen storing `X-Admin-Token` in localStorage.
- [ ] Build user table, nickname search, create/edit dialogs, membership select, and delete confirmation.
- [ ] Run `bun install` then `bun run build` inside `football_insight_admin`.

### Task 3: Runtime Verification

**Files:**
- No new files unless a discovered issue requires a focused fix.

- [ ] Start backend after code changes before API runtime checks.
- [ ] Start the admin frontend dev server.
- [ ] Open the admin app in the in-app browser.
- [ ] Verify the table renders, search input is usable, dialogs open, and layout works at desktop width.
