# CFL Schedule Fallback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep Sina as the primary schedule source, fill incomplete rounds from the Chinese Football League official API, preserve postponed fixtures, and select the latest active incomplete round.

**Architecture:** A focused Python CFL adapter parses official match payloads, while a schedule reconciler fetches each Sina round and invokes CFL only when Sina fails or returns fewer than eight fixtures. A validated 240-match snapshot is persisted atomically by natural fixture identity; Rust filters inactive rows and exposes postponed/current-round semantics, while the frontend renders postponed explicitly and bounds kickoff inference.

**Tech Stack:** Python 3.13, requests, uv, pytest, psycopg, PostgreSQL, Rust 2024, Axum, SQLx, Vue 3, TypeScript, Bun.

## Global Constraints

- Keep Sina as the primary source and call CFL only for a failed or incomplete Sina round.
- The 2026 default CFL tournament calendar ID is `e6818x4pwankpph8awr91m1hw`; allow `FI_CFL_TOURNAMENT_CALENDAR_ID` to override it.
- A complete 2026 snapshot contains exactly 30 rounds, eight distinct matches per round, and 240 matches total.
- Never guess an unmapped CFL team identity and never write a partial schedule.
- Never change an existing public `match_id`; fixture reconciliation uses season, round, home team ID, and away team ID.
- Never physically delete historical fixtures.
- `Postponed` is a first-class status and must never be inferred as live.
- Do not run production migrations, scraper reconciliation, deployments, or production restarts without explicit authorization.
- Follow red-green-refactor for every behavior change.

---

### Task 1: Parse And Merge CFL Fallback Rounds

**Files:**
- Create: `sina_csl_scraper/src/sina_csl_scraper/cfl_client.py`
- Create: `sina_csl_scraper/src/sina_csl_scraper/schedule.py`
- Create: `sina_csl_scraper/tests/fixtures/cfl_round_18.json`
- Test: `sina_csl_scraper/tests/test_cfl_client.py`
- Test: `sina_csl_scraper/tests/test_schedule.py`

**Interfaces:**
- Produces: `CflCslClient.fetch_round_matches(season: int, round_number: int, team_index: TeamIdentityIndex) -> list[MatchResult]`.
- Produces: `fetch_reconciled_schedule(sina_client, cfl_client, season: int, max_round: int, expected_matches_per_round: int = 8) -> list[MatchResult]`.
- Produces stored codes: `Fixture -> "1"`, active statuses -> `"2"`, `Played -> "3"`, `Postponed -> "4"`.

- [ ] Add parser tests using the saved round-18 response. Assert eight matches, four `"4"` statuses, Beijing-local date/time fields, deterministic JavaScript-safe IDs, and known aliases such as `河南俱乐部彩陶坊 -> 河南队`.
- [ ] Run `uv run pytest tests/test_cfl_client.py -q` from `sina_csl_scraper`; confirm RED because `cfl_client.py` does not exist.
- [ ] Implement the CFL HTTP adapter, payload validation, explicit 2026 aliases, deterministic official-only match IDs, and strict team-index lookup.
- [ ] Re-run `uv run pytest tests/test_cfl_client.py -q`; confirm GREEN.
- [ ] Add schedule tests proving complete Sina rounds do not call CFL, incomplete/error Sina rounds call CFL, Sina wins duplicate fixture fields, official-only postponed fixtures are added, and a merged count other than eight raises `ScheduleSnapshotError`.
- [ ] Run `uv run pytest tests/test_schedule.py -q`; confirm RED because the reconciler is missing.
- [ ] Implement the per-round reconciler and verify `uv run pytest tests/test_cfl_client.py tests/test_schedule.py -q` is GREEN.

### Task 2: Connect Reconciled Schedule To Scrape Workflows

**Files:**
- Modify: `sina_csl_scraper/src/sina_csl_scraper/cli.py`
- Modify: `sina_csl_scraper/src/sina_csl_scraper/auto_sync.py`
- Test: `sina_csl_scraper/tests/test_cli_corners.py`
- Test: `sina_csl_scraper/tests/test_auto_sync_cli.py`

**Interfaces:**
- Consumes: `fetch_reconciled_schedule(...)` from Task 1.
- Produces: both `scrape` and `auto-sync` use the same validated merged schedule before catalog building, corner enrichment, JSON output, or database writes.

- [ ] Add CLI tests that inject fake Sina/CFL clients and prove both normal scrape and auto-sync receive all eight merged matches including a postponed fallback match.
- [ ] Run `uv run pytest tests/test_cli_corners.py tests/test_auto_sync_cli.py -q`; confirm RED because CLI still calls `SinaCslClient.fetch_all_matches` directly.
- [ ] Add CFL client construction/configuration and route both workflows through the reconciler without changing standings, ranking, avatar, or corner-enrichment ownership.
- [ ] Re-run the focused CLI tests and confirm GREEN.

### Task 3: Persist A Validated Snapshot Atomically

**Files:**
- Create: `football_insight_service_backend_rs/migrations/20260802130000_add_match_schedule_reconciliation.sql`
- Modify: `sina_csl_scraper/src/sina_csl_scraper/sync.py`
- Modify: `sina_csl_scraper/src/sina_csl_scraper/postgres_repository.py`
- Test: `sina_csl_scraper/tests/test_sync.py`
- Test: `sina_csl_scraper/tests/test_postgres_repository.py`

**Interfaces:**
- Adds columns: `source_active BOOLEAN NOT NULL DEFAULT TRUE`, `last_seen_run_id UUID NULL`, `schedule_source VARCHAR(16) NOT NULL DEFAULT 'sina'`, `source_match_id VARCHAR(64) NULL`.
- Adds natural unique index: `(season, round_number, home_team_id, away_team_id)`.
- Changes: `SyncPayload.max_round: int` and `InsightSyncRepository.upsert_matches(matches, run_id) -> int`.
- Adds: `InsightSyncRepository.deactivate_missing_matches(season: int, run_id: str) -> int`.

- [ ] Add repository SQL tests proving natural-key conflict updates retain stored `match_id`, seen rows set `source_active = TRUE` and `last_seen_run_id`, and missing rows are soft-deactivated by season/run ID.
- [ ] Run `uv run pytest tests/test_postgres_repository.py -q`; confirm RED against the existing match-ID-only UPSERT.
- [ ] Add sync-service tests proving 30 complete rounds trigger deactivation inside the transaction, while a missing/duplicate/overfull round raises before mutation and rolls back without deactivation.
- [ ] Run `uv run pytest tests/test_sync.py -q`; confirm RED because `SyncPayload.max_round` and reconciliation calls are absent.
- [ ] Add the migration, payload validation, natural-key UPSERT, run stamping, reactivation, and soft deactivation within the existing sync transaction.
- [ ] Re-run `uv run pytest tests/test_postgres_repository.py tests/test_sync.py -q`; confirm GREEN.

### Task 4: Expose Active, Postponed, And Latest-Current Semantics In Rust

**Files:**
- Modify: `football_insight_service_backend_rs/src/insight/adapters/persistence/postgres_insight_query_repository.rs`
- Modify: `football_insight_service_backend_rs/src/support/adapters/persistence/postgres_support_repository.rs`
- Modify other persistence adapters only where an existing normal product query reads `f_i_matches` without the active filter.

**Interfaces:**
- Maps stored `"4"` to API status `"postponed"`.
- Extends `RoundProgressRow` with `started_matches: i64`.
- Current round is the maximum incomplete round with `started_matches > 0`; fallback is the first incomplete round.

- [ ] Add a Rust unit regression where round 18 is incomplete, rounds 19/20 are complete, and round 21 is started/incomplete; assert only round 21 is `current`.
- [ ] Extend the existing status-mapping test to assert `"4" -> "postponed"` and unknown values do not silently become live.
- [ ] Run the focused repository test and confirm RED under the first-incomplete implementation.
- [ ] Add `source_active = TRUE` to normal match/progress/support SQL, compute `started_matches` from explicit live/finished status or elapsed non-postponed kickoff, and implement latest-started selection.
- [ ] Re-run focused Rust tests and confirm GREEN.

### Task 5: Render Postponed And Bound Kickoff Inference

**Files:**
- Modify: `football_insight_mini/src/types/insight.ts`
- Modify: `football_insight_mini/src/pages/matches/helpers.ts`
- Modify: `football_insight_mini/src/pages/matches/helpers.test.ts`
- Modify: `football_insight_mini/src/pages/matches/index.vue`
- Modify: `football_insight_mini/src/utils/teamSeasonMatches.ts`
- Modify: `football_insight_mini/src/utils/teamSeasonMatches.test.ts`

**Interfaces:**
- Adds `postponed` to match display status.
- `resolveMatchDisplayStatus` returns explicit `postponed` unchanged and infers `live` only when `kickoff <= now < kickoff + 3 hours`.
- Scoreboard text for postponed matches is `延期`.

- [ ] Change the existing “live after default window” test to assert `scheduled` after three hours; add tests for explicit postponed status, `延期` scoreboard text, hidden live tag, and team-season ordering.
- [ ] Run `bun test src/pages/matches/helpers.test.ts src/utils/teamSeasonMatches.test.ts`; confirm RED under the current unbounded inference and missing postponed handling.
- [ ] Implement the explicit status type, three-hour bound, postponed labels/card class, and deterministic postponed ordering without changing explicit live/finished behavior.
- [ ] Re-run the focused Bun tests and confirm GREEN.

### Task 6: Full Verification

**Files:**
- Verify every file changed by Tasks 1-5.

- [ ] Run `uv run pytest` in `sina_csl_scraper` and confirm zero failures.
- [ ] Run `cargo fmt --check`, `cargo test`, `cargo clippy -- -D warnings`, and `cargo check` in `football_insight_service_backend_rs` and confirm zero failures.
- [ ] Run `"${CODEX_HOME:-$HOME/.codex}/skills/rust-hexagonal/scripts/check-boundaries.sh" football_insight_service_backend_rs` from the monorepo root and inspect every hit.
- [ ] Run `bun test`, the repository's TypeScript check script, and `bun run build:h5` in `football_insight_mini` and confirm zero failures.
- [ ] Run `git diff --check`, inspect the complete diff, and verify that only the design, plan, migration, scraper, Rust, and frontend files required by this feature changed.
- [ ] Report local verification separately from local233 and production; do not claim either remote environment changed.
