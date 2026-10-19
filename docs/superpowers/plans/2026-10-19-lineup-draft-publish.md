# Lineup draft and publish implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Separate private cloud drafts from published lineups and give all four editor actions icons.

**Architecture:** Keep published records and likes intact, add a private draft repository projection, and atomically synchronize both on publishing. The mini page owns UI intent; its composable owns request/revision coordination; local display components only emit actions.

**Tech Stack:** Rust/Axum/SQLx/PostgreSQL, uni-app/Vue/TypeScript/Bun.

**Spec:** `docs/superpowers/specs/2026-10-19-lineup-draft-publish-design.md`.

## Global Constraints

- Preserve one-screen editor, centered publish/reset confirmation, stable share codes and likes.
- Keep existing API compatible; new publish requires 11 players; cloud draft remains private.
- Rust hexagonal boundaries; mini pragmatic layering and token styles; Bun/Cargo.
- User subsequently authorized deployment and upload: verify/commit/push, migrate and deploy backend from local233-built image, then upload mini robot 2.

## Review Focus

- Draft saves after a publication must not expose edits or remove the published version.
- Old clients must still save using the old API and leave editor snapshots coherent.
- Guest/account drafts and delayed saves must never cross identity or team boundaries.
- Sharing must not create/update public content; login resumption must not publish silently.
- Four actions must fit 320px widths and remain registered native mini components.

### Task 1: Backend private draft and explicit publish

Files: `football_insight_service_backend_rs/src/lineup/{domain.rs,ports.rs,application/my_lineup.rs,adapters/*}`, new migration and lineup SQL/HTTP tests.

Produces: GET editor `{lineup,draft}`, PUT draft `{draft}`, PUT publish `{lineup,draft}`, private draft `{formation,slots,updated_at}`.

- [x] Add application/HTTP/SQL regressions for privacy, full-team validation, ownership, stable public identity and atomic writes; observe failures.
- [x] Add draft table and migration backfill; implement port-neutral state and adapter transaction, retaining legacy API behavior.
- [x] Run cargo fmt, relevant tests, full tests/check and isolated PostgreSQL tests.

### Task 2: Mini state and explicit write intent

Files: `api/lineup.ts`, `types/lineup.ts`, `pages/lineup/{useLineupPage,lineupDraft,lineupEditorShare}.*`, `utils/lineupLinks.*`.

Consumes Task 1 JSON contracts. Produces `saveDraft()`, `publish()`, separate busy states, `cloudDraft`, `savedLineup`, `hasUnpublishedChanges`; share result `needs-publish` with no write.

- [x] Add critical state tests for two snapshots, save races, account scope and explicit guest handoff.
- [x] Implement cloud/local precedence and scoped recovery; save draft and publish independently; old autosave only resumes private save.
- [x] Run state/link/share tests and type checking after page integration.

### Task 3: Action bar and page integration

Files: `pages/lineup/index.vue`, local `LineupActionBar.vue`, `src/static/lineup/*`, validation and project memory documents.

- [x] Add four icon/text actions with compact reset/share and secondary draft/primary publish; use SVG sources and PNG runtime assets.
- [x] Wire draft toast, centered publish confirmation, disabled incomplete publish, published-state labels and login action resumption.
- [x] Validate 320/375/390/430 browser layouts with mock APIs; cancel has no writes, draft stays private, share has no writes and update publication retains identity.
- [x] Run type, import boundaries, frontend tests and H5/WeChat builds; restart local Cargo backend before local API checks.
- [x] Review combined diff, update validation/memory, commit and push main; continue with user-authorized production migration/deployment and mini upload.

### Authorized release

- [ ] Verify sole pending migration and preserve current published records/likes.
- [ ] Build image on local233, transfer with docker save/load, apply production migration with run_migrations, retain old container and restart new backend.
- [ ] Verify production health, private endpoint authorization and public compatibility; upload robot 2 from clean synced main and record release evidence.
