# Insights tactics first-screen implementation plan

> **For agentic workers:** Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Team-scoped data/tactics first-screen tabs with private overview and public lineup list together.

**Architecture:** Insights owns public team and tab selection. Shared community composables/cards serve insights and existing lineup routes; APIs and paid statistics stay unchanged.

**Tech Stack:** uni-app, Vue 3, TypeScript, Bun.

**Spec:** Approved “战术双区同屏” in `docs/design/insights-tactics-layout.html`.

## Constraints and review focus

- Tactics defaults first; tabs follow team selection; outlook/ticket entry move inside data.
- Guest public lists and authenticated private cards are independent of paid-stat/review restrictions.
- No cross-page imports or new exceptions; old standalone editor/list/detail routes stay compatible.
- Private/member/outlook results must not overwrite public team selection.
- Team/tab/account transitions and return refresh must prevent stale drafts and liked flags.
- Guest like returns via switchTab; likes must not open cards. Verify small screens and component registration.
- No backend changes/deployment. Verify/commit/push; mini upload follows user release instruction.

## Tasks

- [x] Extract community state to `src/composables/useLineupCommunity.ts`, card to `src/components/FiLineupCommunityCard.vue`, date helpers to utils; preserve existing routes and add compact layout plus insights login return tests.
- [x] Add `useInsightsTactics.ts` with parallel private/public reads, cache, refresh, pagination/likes, session/team guards and critical state tests.
- [x] Give insights one public team owner; inject selection into paid data and outlook without letting those reset it. Test slower metadata and identity changes.
- [x] Build main tabs, my-lineup thumbnail/status card and inline team list; move prior content into data and remove buried tactics entry.
- [x] Verify full mini tests/type/boundaries/H5/WeChat builds and browser guest/member/locked/race/small-screen behavior.
- [x] Independently review, record validation/memory, commit/push.

## Verification outcome

- 1241 tests, type-check, import boundaries, H5/WeChat builds and component registration passed.
- Browser 320/390/430: public guest, authenticated and benefits-paused views, sort/like, team/tab selection, child-editor return refresh, and visible-only paywall impressions passed.
- Final independent review found no remaining blocking issues.
- No backend deployment/restart or mini upload performed in this change.
