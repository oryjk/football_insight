# Ticket Watch Page Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Show monitoring, refresh cadence, client retrieval age and retry status while unifying the ticket-watch presentation.

**Architecture:** Keep all existing API, polling and entitlement flows. Add passive presentation state at current-board success/error/reset points, and let the refresh-status component own its local clock. Style-only work on remaining cards must preserve templates/scripts.

**Tech Stack:** uni-app, Vue 3, TypeScript, Bun, existing design tokens and FiSegmented.

**Spec:** `docs/superpowers/specs/2026-10-16-ticket-watch-page-refactor-design.md` (read in full).

## Global Constraints

- Task A is already pushed as 232d9d4 then 74a2504; no Task A code in these commits.
- Do not read/search/modify Android. No backend/API changes, database work, deployment, upload, version changes or ALLOWLIST additions.
- Keep useTicketWatchPolling.ts and freshnessNowMs unchanged; preserve telemetry functions/watchers and upgrade bindings verbatim.
- Remove team selection UI only, retain selectedTeam's Chengdu initial state and all Yukun types, loaders and tests.
- All eight visual dimensions use existing tokens where possible; no gradient in ticket-watch, no UI hex literals. SVG icon strokes follow the explicit gray/white asset exception.
- Tests before new pure logic/state behavior. Each commit must pass the four required mini checks and follow pull --rebase; push main.

## Review Focus

- A failed retry after prior success must retain old board data and loaded time without displaying an error card.
- Reset/account switch while a request is pending must prevent stale success/failure from changing either new display field.
- Hidden status components must stop their timer; monitoring toggles must replace rather than accumulate intervals.
- Invalid cadence values must not display a broken interval; future loaded times read as 刚刚.
- New component host nodes must not break full-width layouts in WeChat; reduced-motion disables pulse.

### Task 1: Passive refresh data and formatting

**Files:** `presentation.ts`, new `presentation.test.ts`, `useTicketWatchState.ts`, `ticketWatchCurrentBoard.ts`, `effects.test.ts`.

**Interfaces:** Export formatRefreshInterval(number):string, formatFreshness(number,number|null):string, TicketRefreshState and resolveRefreshState({monitoring,refreshing,pollFailing}); state adds Ref<number|null> currentBoardLoadedAtMs, Ref<boolean> currentBoardPollFailing and isPageVisible.

- [x] Write/run failing table tests for 1/15/59/60/90/120/600, nonfinite/<1; null/future/4.999/5/59.999/60/3599.999/3600s; all eight state combinations.
- [x] Implement formatters/state resolver in presentation.ts; preserve existing exports.
- [x] Add/run failing effects tests for success timestamps, poll-fail retention/recovery, initial-error clearing, reset and stale account responses.
- [x] Set loaded time only alongside existing successful hasLoadedCurrentBoard assignment; set failure only on loaded poll failure; clear on success/initial failure/reset. Existing stale guards remain in place.
- [x] Run focused tests; retain this passive state with its consuming first-screen UI in Task 2’s functional commit so it can be reverted independently of the later style-only commit.

### Task 2: First-screen components and wiring

**Files:** header/current-card/skeleton, new TicketRefreshStatus.vue and focused tests, six static icons, index.vue.

**Interfaces:** Header consumes activeTab and emits tab; current card preserves monitor/subscribe/match-id and accepts the six refresh props in spec §3.1; status emits nothing.

- [x] Add a failing timer lifecycle regression proving 1s/15s/hidden/unmount scheduling before implementation.
- [x] Verify three SVG paths against official Lucide sources, preserve copyright and create inactive/active variants.
- [x] Replace header hero/team controls with FiSegmented; current card metadata+status+metrics+primary/secondary actions; skeleton uses global fi-skeleton classes and an optional showTabs flag. The page keeps real navigation actionable and disables the redundant skeleton navigation placeholder, including loading/locked states.
- [x] Wire six props and isPageVisible in existing lifecycle; remove switchTeam and membership prop. Move the existing tracked-interest block above recent reflux only when it has content, retaining its fold keys/events and empty position.
- [x] Preserve all telemetry and polling code verbatim; run timer tests, four checks, offline MP/H5 builds; document component structure/token sources and commit.

### Task 3: Remaining card style consistency

**Files:** nine cards in spec §3.6, ticket-watch-shared.css, fi-tokens.css only if essential, index background CSS.

- [x] Snapshot existing templates/scripts for comparison; inspect current gradients and token values.
- [x] Change styles only: solid colors, existing design scale, consistent icon sizes. No text/structure/event/fold changes in nine cards.
- [x] Verify templates/scripts are byte-identical, gradient/hex scans empty, telemetry and polling unchanged, all four checks pass; complete report and both memory entries.
- [x] Independent read-only review, fix findings, pull --rebase, commit, push main; record push order and hashes.

Commit grouping: Task 1 + Task 2 form the functional commit; Task 3 is a separate style-only commit. Each can be reverted without the other because no shared tokens or component contracts are introduced by Task 3.

The parent owns state/formatters/index and commits, a component agent owns first-screen components/icons, and a style agent owns remaining CSS/tokens. They do not edit shared files concurrently. Final validation/review occurs after all edits settle.
