# Season recap mileage implementation

Spec: `docs/superpowers/specs/2026-10-18-season-recap-mileage-design.md`.

Scope: free Chengdu first-team attendance, cloud persistence, backend mileage calculation, mini poster with real WeChat code. Main branch authorized by workspace convention; no production changes or upload.

## Task 1: backend calculation and persistence

1. Verify home cities; use a constant city table, including Yuxi for Yunnan (spec example needs correction).
2. Write failing tests for spherical distance, duplicates, city deduplication, home distance bounds, unfinished/inactive/other-team rejection and stale stored selections.
3. Implement season_recap domain / ports / application / adapters, additive migration, authenticated GET/PUT and cached real mini-program code.
4. Run isolated PostgreSQL migration/query tests using only synthetic data; never apply production DDL.
5. Restart local backend, cargo test/check/build/clippy. Expected: all pass; production unchanged.

## Task 2: mini page and poster

1. Write failing tests for numeric validation, share scene attribution, saving/permission failure and stale-session responses.
2. Add typed atomic API, page state, match selection and poster component; entry in My account. Reuse shared canvas primitives; no page-to-page imports.
3. Run bun test/type-check/check:boundaries/verify:mp-components and H5/offline WeChat builds. Expected: all pass; no version allocation/upload.

## Task 3: review and delivery

Independent whole-change review with focus on SQL atomicity, user isolation, stale responses, real scan attribution and canvas cross-platform behavior. Fix important findings with regression tests. Update FEATURES/CHANGELOG and validation report, separate backend and mini commits, pull --rebase and push main.

## Interfaces and decisions

- GET/PUT `/api/v1/season-recap/{season}` uses JWT identity; no client user_id.
- POST `/api/v1/season-recap/{season}/code` returns cached base64 PNG; fixed page and `s=share&c=recap2026`-style scene, no personal data. POST avoids global GET cache retention.
- GET returns eligible matches, saved selected IDs, home one-way km and calculated summary. PUT replaces only that season's selections, updates per-user distance in one transaction and returns the same view.
- Each away round trip rounds once to whole km, then sums; home subtotal rounds once. City count includes Chengdu for any home match; identical cities count once.
- Inactive/uncompleted stored selections are omitted on reads; invalid new selections rejected. Unknown away city fails explicitly rather than silently producing zero.
- New page defaults to 2026, so season-end return and poster links stay stable.

Review focus: hidden API assumptions, unknown/variant team names, races between selection and schedule reconciliation, code failure caching, permission denial recovery, H5 canvas export, switch-account leakage.
