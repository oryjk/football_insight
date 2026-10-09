# Ticket Watch Refactor Implementation Plan

> **For agentic workers:** Use `superpowers:executing-plans` to implement this plan in the current session. Project AGENTS.md takes precedence: frontend refactoring uses existing business tests, type checks and builds rather than mechanical TDD.

**Goal:** Replace the 4,279-line ticket-watch page with focused presentation components and page modules, preserving inventory statistics, membership gates, subscriptions and payment behavior.

**Architecture:** The page wires lifecycle, navigation and component events. Local components receive typed props and emit intent; page composables own data loading, request guards, polling and payment actions. Pure presentation calculations stay in helpers. Styles live with their components and use shared design tokens.

**Tech Stack:** uni-app, Vue 3, TypeScript, Bun.

**Design basis:** The refactor proposed in this chat and authorized by the user's request to implement it; existing `football_insight_mini/AGENTS.md`.

## Constraints

- Keep both H5 and MP-WEIXIN builds working; import runtime components directly from `.vue`.
- Preserve `sale_start_at + 10 minutes`, missing-start rejection, block keys, historical caches and loading strategies.
- Keep API calls in page modules; presentation components never import API/auth storage.
- Preserve membership and review gates, payment confirmation and duplicate-request guards.
- Preserve visual values when introducing semantic tokens. Do not change API contracts, routes, release versions or backend code.
- Keep work reviewable on `codex/ticket-watch-component-refactor`; no production deployment.

## Tasks

- [x] Establish existing ticket-watch helper/API test baseline and inspect lifecycle, template branches and stylesheet dependencies.
- [x] Extract typed local presentation components for match summary, recent reflux, focus rankings, tracking, inventory, insights/decisions, history selection/statistics, loading and dialogs. Remove duplicated current/history presentation where their behavior is identical.
- [x] Separate state, board loading, interest actions, payments and polling into focused page composables. Keep lifecycle and routing in the page. Move pure formatting/calculation functions out of Vue script.
- [x] Migrate extracted structural styling to semantic/component design tokens, retaining existing values and component-scoped styles. Check CSS variable resolution and dynamic classes.
- [x] Run import boundary checks, frontend tests, type checking, offline MP-WEIXIN build including registration check, and H5 build. Review refactor for behavioral and cross-component layout regressions.

## Review focus

- Team/tab switching, in-flight requests and historical selections must preserve existing cache/request-token semantics.
- Polling and freshness clocks must stop when the page is hidden/unloaded and use the membership interval.
- Missing sale start must never fall back to a full inventory query.
- Payment/subscription/dialog events must reach their page handlers and remain reachable behind their intended visibility gates.
- Component host nodes and scoped CSS must not lose layout, conditional gating or dynamically selected heat/price styles in MP-WEIXIN.

## Progress

- Baseline prior to edits: import boundary check passed, its two regression tests passed, type check passed. Dependencies installed with frozen Bun lockfile, no tracked changes.

- Baseline ticket-watch helper/API tests: 67 passed. Added 6 behavioral tests for out-of-order history requests, missing sale start, silent poll failures, payment guards, timer disposal and hide-during-request behavior; ticket-watch total: 73 passed.
- `index.vue`: 4,279 → 500 lines. Presentation: 16 local components (15 added); page logic: state, boards, interests, match-ID payment, subscription and polling composables, plus shared payment actions and pure presentation functions. Every non-declarative extracted file is below 600 lines.
- Removed obsolete page-owned history chip scrolling state; selector component owns its DOM query and scroll offset. Current/history share inventory, focus and insight presentation.
- Fixed match-ID sheet being nested inside the interest-confirmation condition; payment guards now remain held until native payment and order confirmation finish. Polling cannot restart after an initial request resolves while the page is hidden.
- Introduced primitive/semantic/component tokens without changing the retained CSS values. Verified 1,284 original declarations after resolving token aliases; only unused original selectors were omitted. Dynamic shadows are composed locally so their RGB dependencies resolve in the component scope.
- Validation: `bun run type-check`, `bun run check:boundaries`, 2 boundary-check regression tests, `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` with component registrations, and `bun run build:h5` all passed. No release version synchronization or upload was performed.
- Full suite: 264 passed, 3 failed. The failures are existing stadium geometry expectations in `src/utils/stadiumRegions.test.ts` and `src/pages/seat-swap/helpers.test.ts`; these sources and tests are unchanged and the same failures reproduce when those tests run independently. They are outside this refactor.
- Independent code review completed. Findings about header eyebrow styles, scoped locked-upsell descendant styles and dynamic shadow scope were fixed and rechecked. No additional introduced regressions found; existing team-switch request races remain unchanged.
- Limits: build artifacts and scoped component registrations were checked; WeChat DevTools/real-device interaction and actual payments were not exercised. No backend changes or production deployment.


## Follow-up: stadium layout repair

The user separately requested fixing the three existing stadium layout failures. The latest region enlargement had made horizontal rectangles wider than their center spacing, overlapped the inner-ring corners, and left only a 0.5% gap above VIP. Both frontend copies now retain separate rectangles and the tested VIP gaps. VIP height accommodates the label after removing the CSS minimum-height override; highlighting preserves rectangle dimensions instead of scaling into neighboring regions.

- Changes are synchronized in mini/H5 `stadiumRegions.ts` and `StadiumMap.vue`; the two layout utilities remain identical.
- The 41 existing stadium/helper tests now pass without modifying their assertions. Mini full suite: **267 passed, 0 failed**; type checking and import-boundary checks passed.
- Offline MP-WEIXIN build and component registrations, mini H5 build, and standalone H5 build all passed.
- Chromium checks against the actual standalone H5 component covered all 71 regions at widths 320, 375 and 430, plus 639 filter/published/review highlighting cases. No overlapping DOM rectangles, blocked center hit targets or browser errors remained. Temporary test fixture files and the dev server were removed/stopped afterwards.
- No backend changes or production deployment. WeChat device interaction remains unverified.
