# Topic History and Detail Pages Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Show the latest five topics across publication dates with the next carousel card visible, add a trailing more-topics entry, and open full articles on a second-level page.

**Architecture:** Reuse f_i_daily_topic_editions for read-only deduplicated history and topic lookup. Extend existing domain/ports/application/web/persistence boundaries. Mini-program uses shared topic mappers/audio UI, public paginated history and detail routes, and native page navigation instead of a home modal.

**Tech Stack:** Existing Rust/Axum/SQLx/PostgreSQL and uni-app/Vue/TypeScript/Bun.

**Spec:** User requirements in this conversation: latest five regardless of date; preview next card; more entry to the right of the final topic; newest-first full history; second-level detail page. Preserve daytime automatic publication.

## Constraints

- Work on main, preserve unrelated dirty documents, no new table/migration.
- Keep public root GET query and response compatible; limit remains1..5.
- New history endpoint uses bounded cursor pagination with timestamp/id ordering; duplicate ids choose the newest edition.
- New detail endpoint is scoped by team and returns404 for absent articles; malformed parameters400 and storage failures500.
- Keep sources, stable ids and audio fields, sanitize non-ready URLs consistently.
- Share pure helpers and audio components outside page directories; no cross-page imports or direct uni.request.
- Preserve actual news; do not seed fake history.

## Tasks

- [x] Add HTTP and disposable PostgreSQL regression cases for cross-edition recent topics, history order/duplicates/cursors, team isolation, detail fields and errors. Implement typed read queries, repository adapters and web mapping, then verify Rust boundaries/tests.
- [x] Add shared topic/history/detail mappers, native navigation and paginated list/detail pages. Restore partial next-card preview and trailing more card. Verify mapper tests, type-check, import boundaries and both platform builds.
- [x] Review, commit/push task changes, rebuild/restart local backend, deploy peiqian using the existing flow, verify public APIs/audio and upload robot2 mini-program for device testing.

## Review Focus

- Equal publication timestamps must paginate without repeats or gaps.
- Repeated article ids across editions must retain newest audio/version.
- Failed next-page requests must retain already displayed articles and cursor.
- Detail navigation/unload must preserve audio lifecycle and destroy stale contexts.
- Neither new pages nor history failures should alter unrelated home/account behavior.

## Verification and Release

- Rust: 358 unit/HTTP tests passed; 6 disposable PostgreSQL tests passed, including equal-time cursor pagination and newest-version deduplication. Test container removed.
- Mini: type-check, boundary check, 10 focused mapper tests, H5 and MP builds passed; all custom components registered.
- H5 browser check with real local APIs: next card visible at 390-wide viewport, home -> detail and More -> history -> detail worked with complete paragraphs.
- Local Docker rebuilt/restarted with original environment; production peiqian Docker deployed/restarted as 5827a95, identical environment and source digest. No migration executed.
- Production latest/history/detail returned200 and matched local article payloads; pagination had no duplicates, audio Range returned206; public-config and ticket-watch matches/regions returned200.
- Clean main checkout robot2 upload completed: version1.0.66, main package1,809,302bytes. No preview QR requested/generated.
