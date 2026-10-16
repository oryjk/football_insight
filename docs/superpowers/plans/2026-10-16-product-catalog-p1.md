# Product Catalog P1 Implementation Plan

> Agent workers use TDD and the approved product-catalog-data-driven-design. Work directly on main; root creates two commits after verification.

**Goal:** Enable V4/V5 passes by inserting product rows, without altering legacy annual purchases.

**Architecture:** Pure membership kind validation/eligibility/expiry; existing legacy annual catalog plus data-backed optional offers; immutable grant snapshots through existing payment ports. Keep fulfillment SQL in persistence, observer spawned and isolated. No P2/P3, web admin UI, match unlock or reflux changes.

**Spec:** ../specs/2026-10-16-product-catalog-data-driven-design.md

## Tasks

- [x] Root: domain ProductSpec, GrantSnapshot, expiry, singleton tier validator; failing pure tests, then implementation. Unknown qualified membership tiers error; historical unqualified membership retains V9 semantics.
- [x] Root: failing fake catalog/gating/order/snapshot tests; implement optional response metadata and capability header, data row lookup plus audience checks at listing and ordering. Annual calculation remains legacy.
- [x] Settlement worker: test snapshot fulfillment and unchanged legacy, day boundaries/non-downgrade/non-shortening/referral interactions; add nullable snapshot INSERT and additive product columns/backfill/relaxed tier CHECK migration >20261016120000.
- [x] Mini worker: tests first, optional DTOs, capability+attempt headers, grouping and day agreement; old response rendering and annual wording unchanged.
- [x] SQL worker: prepare scratch-only script extracting actual SQL; production PG version first; BEGIN + lock_timeout2s, migration, own simulated rows, execute every new/changed SQL, unconditional ROLLBACK and schema/data restoration assertions. Never change existing user data; shadow user updates with temporary table.
- [x] Root: restart local backend before final validation, cargo test/check, mini four required checks and offline build; independent review and fixes via TDD.
- [x] Root: update FEATURES/CHANGELOG/API docs/verification report with review checklist, deviations, actual SQL evidence and ordered operator actions; backend and mini commits with Co-Authored-By, push main. No persistent migration, production deploy or upload.

## Review focus

- Annual JSON bytes/prices/credit/duration remain unchanged; catalog_source stays legacy.
- Missing capabilities and stale invitation/paid tier reject pass before order creation; no bypass at virtual parameters.
- Snapshot reads at settlement do not consult mutable catalog; NULL snapshots retain old path.
- All new create/params/settlement events remain nonblocking, including missing observability table.
- P2 shadow comparison/table switch is not claimed as P1; rollout INSERTs start disabled and activation remains an explicit operator step.
