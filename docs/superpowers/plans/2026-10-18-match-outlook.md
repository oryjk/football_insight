# Match Outlook Implementation Plan

> Use executing-plans for native implementation, followed by one fresh whole-change reviewer.

**Goal:** Free pre-match data outlook, immutable published probabilities and honest scored record.
**Architecture:** Python precomputes from team-ID scores; Rust only reads and aggregates stored values; mini gates display by login/review mode, separately from member benefits.
**Spec:** `docs/superpowers/specs/2026-10-18-match-outlook-design.md`.

## Global Constraints

No production DDL/backfill/deployment/upload. Add two tables with a timestamp newer than20261018130000. No existing API changes, no personal data, no new data sources beyond existing Sina historical scores. Do not read Android. UI uses tokens/request and existing dependency boundaries. No live predictions after kickoff; no backdated public record. Core sync must commit despite outlook failure.

## Review Focus

- Delayed first generation and reschedules cannot backfill or unfreeze a prediction.
- Concurrent runs and score corrections cannot overwrite scored rows; use DB-time predicates.
- Research target-year leakage, team renames and neutral2020/21; reference vs production numeric parity.
- Low-tier/benefit-paused logged-in accounts see the free panel; review/guest and stale request transitions hide it.
- Missing history/times/tables/errors produce no fabricated record; old sync remains committed.

## Task 1 — model, history and independent sync hook

Files: new `sina_csl_scraper/src/sina_csl_scraper/match_outlook/{model,history,repository,service,cli}.py`; tests alongside existing scraper tests; cli.py and sync.py minimal wiring.
- [x] RED pure functions for probabilities/RPS/baseline, prior-only and neutral/rename/season rollover; production model vectorizes the full3888 grid, stable research order.
- [x] GREEN replay and rolling selection, use updated spec's newstart initialization and τ-applied expectation; parameter selection has τ=1, final replay is adaptive.
- [x] RED/GREEN history validator (counts, IDs, scores); command fetches rounds into resumable local JSON first, full validation before any DB write; dry run by default, --write-db explicit.
- [x] RED/GREEN sync post-commit callback failure leaves original commit/result successful.
- [x] Reproduce2026 selected params and 2019–2026 non-neutral RPS vs research using cached raw JSON; never write a public historical prediction.

## Task 2 — persistence, freezing and read API

Files: migration20261018140000, Python repository/service tests, Rust `src/match_outlook` layered module and app/lib assembly.
- [x] RED/GREEN actual isolated PostgreSQL statements: INSERT only future, conflict update only old+new future, writer SQL preserves frozen probability, once-only score, reschedule annotation; migrations rollback cleanly.
- [x] Service reads a repeatable snapshot, checks complete prior seasons, replays in Python, publishes future fixtures and scores only frozen stored forecasts. Independent connection/transaction after core sync.
- [x] Rust GET/live/match-outlook and record use domain/ports/application/adapters; normal/missing/invalid tests, stored aggregates only. Missing table gracefully unavailable, zero record explicit.
- [x] Restart local backend then cargo test/check/build/clippy.

## Task 3 — mini and delivery

Files: typed api/types, insights panel/helpers/loader, index wiring, home lightweight entry; no shared member loading rewrite.
- [x] RED/GREEN largest-remainder percentages sum100, τ wording thresholds, copy blacklist, load visibility/revision/team switch race.
- [x] Put free panel above membership branch; reuse selector with common selectedTeamId. Load public teams/matches and user's favorite independently so paused accounts still have options.
- [x] Home single neutral entry navigates insights; no added probability calls on home.
- [x] Bun test/type-check/boundaries/components/H5/offlineMP; scraper pytest; independent final review, validation report and API docs; FEATURES/CHANGELOG per commit.
- [x] Pull --rebase, separate data/backend and mini commits, push main. Production rollout left to owner.

## Owner follow-up — remove defensive DB trigger

The owner explicitly removed the additional unconditional-UPDATE trigger before production migration. Keep the original conditional publish/scoring SQL and normal constraints; add an isolated regression proving direct SQL is no longer intercepted. Production deployment, migration and history activation are now authorized.
