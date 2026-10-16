# Payment Observability Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans and TDD for each task. User authorizes direct work on main and two final commits.

**Goal:** Explain each purchase attempt across membership, match unlock and reflux subscriptions without changing payment behavior.

**Architecture:** Payment domain defines event vocabulary, sanitization and attribution; ports abstract recording. Application orchestrates best-effort background recording and ingestion. Admin provides read-only SQL aggregates through its own port. Mini batches telemetry separately from purchases.

**Tech Stack:** Rust/Axum/SQLx, Vue/TypeScript/Bun.

**Spec:** ../specs/2026-10-16-payment-observability-design.md

## Global Constraints

- Only add event table/indexes; timestamp > 20261014163000; do not execute migrations.
- Existing API contracts, error types/messages, prices, cancellation and payment state transitions stay unchanged.
- No production data writes or real payments in tests; use fakes and pure tests.
- Recorder must never block purchases/settlement/refunds. Client telemetry must never throw or show UI.
- Main only; backend and mini separate commits with English subjects and Co-Authored-By trailers; push after all checks.

## Review Focus

- Missing table, blocked recorder and timeout: original payment result returns promptly.
- Callback has no attempt ID: correlate to order's original server attempt without extra foreground IO.
- Out-of-order client/server events: delivered wins over late client events in attribution.
- Malformed/oversize ingestion: authenticated calls return 204; unauthenticated calls return 401.
- Old backend, logout and queued reports: silent failures and no cross-account reports.

## Tasks

1. Payment core (root):
   - [x] Add failing tests for vocabulary, attribution, sanitization, header fallback, migration constraints, background failures.
   - [x] Implement domain/payment_event.rs, ports/payment_event_recorder.rs, application/payment_events.rs and postgres recorder; only additive migration.
   - [x] Add ingestion tests for auth, size, ownership, whitelist/rate limits; implement new events route.
   - [x] Add failing fake-based tests at all three create use cases and settlement/refund; implement shared helper and injection. Preserve original errors.
2. Admin queries (independent):
   - [x] Read existing admin auth/query layers, then write failing validation/aggregation/auth tests.
   - [x] Implement domain, port, application, persistence and four GET routes; SQL aggregates counts, conversions, attribution and timelines. No audit writes.
3. Mini telemetry (independent):
   - [x] Read all three flows and tests; add failing batch/silence/auth/classification/optional-argument regression tests.
   - [x] Implement telemetry queue, optional attempt headers, requestDigitalPayment/waitForPaidOrder and all three purchase flows.
   - [x] Run Bun checks and offline mini build; no version allocation/upload.
4. Integration:
   - [x] Restart local backend with deploy/local-test/up.sh; cargo test and cargo check (no production writes).
   - [x] bun run type-check, bun test, bun run check:boundaries, bun run verify:mp-components.
   - [x] Review full diff against spec section 11; record deviations, omitted DB tests, real command outcomes and file list in delivery report.
   - [x] Update FEATURES.md and CHANGELOG.md; create backend then mini commits, push main. No production migration/deployment/upload.
