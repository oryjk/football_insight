# Pre-Match Support Mini MVP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `football_insight_mini` 落地“赛前助力”MVP：用户关注 1 支主队，在比赛前 24 小时内为下一场比赛助力 1 次，查看双方助力对比，并支持分享回流到该场助力页。

**Architecture:** 后端新增独立 `support` 模块和两张表：用户当前主队偏好、单场助力记录。比赛时间窗口统一由 Rust 后端根据 `f_i_matches` 判断；小程序只消费新的赛前助力接口并渲染首页卡片、主队选择器、助力详情页和分享/海报能力。MVP 只做中超、只支持单主队、只做“下一场比赛”助力，不引入评论、奖励、复杂排行或 scraper 改造。

**Tech Stack:** Rust + Axum + SQLx + PostgreSQL, uni-app + Vue 3 + TypeScript + Bun, WeChat Mini Program share/canvas APIs.

---

### Task 1: Define backend support domain and migration

**Files:**
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/migrations/20260410110000_add_pre_match_support_tables.sql`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/domain/mod.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/domain/support.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/ports/mod.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/ports/support_repository.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/lib.rs`

**Step 1: Write the failing tests**

Add unit tests for domain/window rules:
- only the followed team can be supported
- one user can only support once per match
- support opens 24 hours before kickoff and closes at kickoff

**Step 2: Run test to verify it fails**

Run:
```bash
cd /Users/carlwang/football_insight/football_insight_service_backend_rs
cargo test support::domain -- --nocapture
```

Expected: missing module/types.

**Step 3: Write minimal implementation**

Introduce:
- `favorite_team_id` on `f_i_users`
- `f_i_match_support_votes`
- domain models for favorite team, support match detail, support totals and viewer state
- pure rule helpers for window validation

**Step 4: Run test to verify it passes**

Run the same `cargo test` command until domain tests pass.

### Task 2: Add backend use cases, repository and HTTP routes

**Files:**
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/application/mod.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/application/get_support_profile.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/application/list_support_teams.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/application/set_favorite_team.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/application/get_match_support_detail.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/application/cast_match_support_vote.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/adapters/mod.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/adapters/persistence/mod.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/adapters/persistence/postgres_support_repository.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/adapters/web/mod.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/adapters/web/dto.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/adapters/web/handlers.rs`
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/support/adapters/web/routes.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/app.rs`

**Step 1: Write the failing tests**

Add application tests for:
- profile returns current favorite team and the next match support card
- setting favorite team rejects unknown team ids
- casting vote rejects no-favorite-team, wrong team and duplicate votes
- match detail returns public totals and viewer vote state when logged in

**Step 2: Run test to verify it fails**

Run:
```bash
cd /Users/carlwang/football_insight/football_insight_service_backend_rs
cargo test support::application -- --nocapture
```

Expected: missing repository methods/use cases.

**Step 3: Write minimal implementation**

Expose new endpoints:
- `GET /api/v1/support/teams`
- `GET /api/v1/support/profile`
- `PUT /api/v1/support/favorite-team`
- `GET /api/v1/support/matches/{match_id}`
- `POST /api/v1/support/matches/{match_id}/votes`

Rules:
- read endpoints may be public, but profile and vote require auth
- vote team must match viewer `favorite_team_id`
- supported team must be one side of that match
- duplicate support returns clear error

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test support -- --nocapture
```

Then run:
```bash
cargo test
```

### Task 3: Add mini-app API/types and support page

**Files:**
- Create: `/Users/carlwang/football_insight/football_insight_mini/src/api/support.ts`
- Create: `/Users/carlwang/football_insight/football_insight_mini/src/types/support.ts`
- Create: `/Users/carlwang/football_insight/football_insight_mini/src/pages/support/index.vue`
- Modify: `/Users/carlwang/football_insight/football_insight_mini/src/pages.json`
- Optional create: `/Users/carlwang/football_insight/football_insight_mini/src/utils/supportSharePoster.ts`

**Step 1: Write the failing test / type-check target**

Add the new page, types and imports first, then run type-check/build so it fails on missing API contracts and page references.

**Step 2: Run test to verify it fails**

Run:
```bash
cd /Users/carlwang/football_insight/football_insight_mini
bun run type-check
```

Expected: missing support API/types/page contracts.

**Step 3: Write minimal implementation**

Implement:
- support detail page with match info, dual support bars, countdown/status, vote button, poster button
- share path back to `/pages/support/index?matchId=...`
- login gating for vote action
- poster generation using local canvas, matching existing rankings poster approach

**Step 4: Run test to verify it passes**

Run:
```bash
bun run type-check
```

### Task 4: Wire home-page favorite team and entry flow

**Files:**
- Modify: `/Users/carlwang/football_insight/football_insight_mini/src/pages/home/index.vue`
- Modify: `/Users/carlwang/football_insight/football_insight_mini/src/pages/user/index.vue`
- Modify: `/Users/carlwang/football_insight/football_insight_mini/src/utils/postLoginRedirect.ts` only if needed
- Modify: `/Users/carlwang/football_insight/football_insight_mini/src/utils/authStorage.ts` only if needed

**Step 1: Write the failing test / type-check target**

Update the home page to reference support profile state before implementation, then run type-check.

**Step 2: Run test to verify it fails**

Run:
```bash
cd /Users/carlwang/football_insight/football_insight_mini
bun run type-check
```

Expected: missing support state/computed/actions.

**Step 3: Write minimal implementation**

On home:
- logged-out users see “登录后关注主队”
- logged-in users without favorite team see team selector
- logged-in users with favorite team see next-match support card and CTA into detail page

On user page:
- show current favorite team summary
- allow re-open team selector / jump to home to switch team

**Step 4: Run test to verify it passes**

Run:
```bash
bun run type-check
bun run build:mp-weixin
```

### Task 5: Verify end-to-end behavior and deployment impact

**Files:**
- Optional docs note: `/Users/carlwang/football_insight/README.md`
- Optional docs note: `/Users/carlwang/football_insight/DEPLOYMENT.md`

**Step 1: Run backend verification**

```bash
cd /Users/carlwang/football_insight/football_insight_service_backend_rs
cargo test
```

**Step 2: Run mini-app verification**

```bash
cd /Users/carlwang/football_insight/football_insight_mini
bun run type-check
bun run build:mp-weixin
```

**Step 3: Local runtime check**

If local env is available:
- restart local backend
- verify `GET /api/v1/support/teams`
- verify `GET /api/v1/support/matches/{match_id}`
- verify vote flow in mini app with a logged-in WeChat-bound test user

**Step 4: Summarize deploy impact**

Call out clearly:
- local backend whether restarted
- production backend not deployed / not restarted
- migration required before production rollout
