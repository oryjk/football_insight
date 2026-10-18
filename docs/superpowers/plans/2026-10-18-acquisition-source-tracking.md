# Acquisition Source Tracking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Capture a new account's first channel/content without affecting registration, then report acquisition, return and payment counts.

**Architecture:** Mini utils capture and persist the first valid source before session bootstrap; auth API requests attach optional acquisition data. Auth application schedules bounded best-effort writes through a dedicated repository port/pool after successful account creation; existing account login/binding never attributes a source. A new table and read-only SQL provide reporting.

**Tech Stack:** Rust/Axum/SQLx/PostgreSQL; uni-app/Vue/TypeScript/Bun.

**Spec:** `docs/superpowers/specs/2026-10-18-acquisition-source-tracking-design.md`

## Global Constraints

- src permits only `[a-z0-9_-]`, maximum 32; cid maximum 64; invalid values discarded without request errors.
- Existing login, invitation, membership, payment, response fields/status/errors remain unchanged. No client-supplied user identity.
- Only add f_i_user_acquisitions; no modification to f_i_users, migration timestamp > 20261017140000. Never execute production migration or write real users.
- No Android access, backend production deployment, mini upload, manual manifest version change or boundary allowlist changes.
- Direct main commits/push remain authorized by session; backend and mini separately reviewable.

## Review Focus

- Malformed query, encoded scene, duplicate keys, non-string/oversized values must never block launch/login.
- First valid source persists through later launch, warm entry, login/binding and logout; storage denial never prevents authentication.
- Existing account login/binding/reactivation must not claim a new acquisition; distinguish INSERT versus reactivation explicitly.
- Missing table, repository error, timeout, panic or queue saturation must not fail or delay authentication.
- Reports must not multiply users by orders or mislabel latest-login snapshots as exact daily retention.

## Task 1: Shared parsing contracts

Files: mini `src/utils/acquisitionSource.ts` and tests; backend auth `domain/acquisition.rs` and tests; adapter DTO.

- [x] RED table-driven parser tests for exact limits, invalid/missing values, numeric scene, malformed nested JSON; GREEN sanitizer.
- [x] RED mini cold/warm direct query and encoded `s=...&c=...` scene parsing, first valid persistence, denied/corrupt storage; GREEN utils. Direct parameters take precedence as one complete source; no mixed content from conflicting scene.
- [x] Wire capture before session bootstrap; attach optional nested acquisition to auth requests via existing request only. Verify real API request payloads, old request payloads unchanged when no attribution.

## Task 2: Backend first-creation attribution

Files: auth ports, application registration/binding use cases, postgres auth repository, dedicated acquisition adapter, bootstrap, web request DTOs.

- [x] RED registration/binding tests: capture authenticated server-produced user ID only for new creation, no source for existing or reactivated user, no source on failed creation.
- [x] Preserve existing repository methods; add a registration outcome carrying explicit newly_created from INSERT/UPDATE branches, avoiding timestamp heuristics.
- [x] RED recorder tests: failure/timeout/panic/saturation remain nonblocking; GREEN semaphore-limited task with timeout and dedicated at-most-two-connection pool.
- [x] Wire optional metadata at auth request boundary; no changes to original authentication errors/responses.

## Task 3: Migration and reporting

Files: new migration, `docs/sql/user-acquisition-summary.sql`, operational/API documentation.

- [x] Add only new table, primary-key FK, sanitized columns and reporting index; INSERT ON CONFLICT(user_id) DO NOTHING.
- [x] Define D1+/D7+ as latest successful login at least 24/168 hours after registration, explicitly not exact natural-day retention. Report eligible denominators; paid users use EXISTS paid_at orders, including later refunded orders that did pay.
- [x] Validate migration and queries with isolated scratch/TEMP users and rollback only if a separate isolated DB can be established; otherwise no database writes, document lack of SQL runtime validation. Never run production migration.
- [x] Check existing privacy coverage; report missing coverage to owner without modifying privacy copy.

## Task 4: Verification, review and delivery

- [x] Restart local backend using deploy/local-test/up.sh before backend validation, cargo test/check/build/clippy and relevant formatting.
- [x] Mini bun test/type-check/check:boundaries/verify:mp-components; H5/offline MP builds without allocating a version.
- [x] Fresh whole-change review, address important findings with regression tests.
- [x] Update FEATURES/CHANGELOG and docs/acquisition-source-tracking-validation.md with six acceptance items, limitations and outputs.
- [x] Commit/push backend and mini separately on main, Co-Authored-By trailers. Production remains unchanged.
