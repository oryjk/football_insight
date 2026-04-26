# Mini WeChat Login Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `football_insight_mini` 增加微信小程序登录能力：已绑定微信的用户可直接登录，未绑定时填写邀请码并完成注册/绑定，同时拉取并保存微信头像和昵称。

**Architecture:** 复用后端现有 `wechat bind token` 思路，但将入口从 H5 公众号 OAuth 改为小程序 `wx.login(code)`。后端新增“小程序微信登录 / 绑定”接口，首次登录只要求邀请码，不再要求用户名、手机号、密码。前端小程序在 `我的` 页接入微信登录按钮，未绑定时展示邀请码补全弹层，再用微信头像昵称完成注册绑定。

**Tech Stack:** Rust + Axum + SQLx + PostgreSQL, uni-app + Vue 3 + TypeScript, WeChat Mini Program login APIs.

---

### Task 1: Define mini-program auth contract

**Files:**
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/domain/wechat.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/ports/wechat_oauth_port.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_mini/src/types/auth.ts`

**Step 1: Write the failing test**

Add a backend test that expects a mini-program login result to carry either:
- authenticated token bundle, or
- bind token + invite-required metadata

**Step 2: Run test to verify it fails**

Run:
```bash
cd /Users/carlwang/football_insight/football_insight_service_backend_rs
cargo test wechat_oauth_flow -- --nocapture
```

Expected: compile or assertion failure because mini-program result types do not exist yet.

**Step 3: Write minimal implementation**

Introduce:
- `MiniWechatProfile` / expanded `WechatOauthProfile` fields for nickname + avatar
- `WechatOauthPort::fetch_mini_program_profile(code: &str)`
- mini login response DTOs in mini frontend types

**Step 4: Run test to verify it passes**

Run the same test command and confirm the new types compile and the test moves green.

**Step 5: Commit**

```bash
git add /Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/domain/wechat.rs /Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/ports/wechat_oauth_port.rs /Users/carlwang/football_insight/football_insight_mini/src/types/auth.ts
git commit -m "feat: define mini wechat auth contract"
```

### Task 2: Add backend mini-program login use case

**Files:**
- Create: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/application/login_with_mini_wechat.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/tests/wechat_oauth_flow.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/application/mod.rs` if present

**Step 1: Write the failing tests**

Add tests for:
- bound `openid` -> returns authenticated user
- unbound `openid` -> returns bind token with invite-required state

Use the existing fake repository pattern from `tests/wechat_oauth_flow.rs`.

**Step 2: Run test to verify it fails**

Run:
```bash
cd /Users/carlwang/football_insight/football_insight_service_backend_rs
cargo test wechat_oauth_flow -- --nocapture
```

Expected: missing use case / enum / behavior.

**Step 3: Write minimal implementation**

Implement a new use case that:
- validates non-empty code
- calls `fetch_mini_program_profile`
- if `wx_open_id` exists -> issues normal session token
- else -> issues bind token containing `open_id`, `union_id`, `display_name`, `avatar_url`

**Step 4: Run test to verify it passes**

Run the same test target until both tests are green.

**Step 5: Commit**

```bash
git add /Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/application/login_with_mini_wechat.rs /Users/carlwang/football_insight/football_insight_service_backend_rs/tests/wechat_oauth_flow.rs
git commit -m "feat: add mini program wechat login use case"
```

### Task 3: Replace bind flow with invite-only mini registration

**Files:**
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/application/bind_wechat_account.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/ports/auth_repository.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/adapters/persistence/postgres_auth_repository.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/tests/wechat_oauth_flow.rs`

**Step 1: Write the failing tests**

Cover:
- unbound mini-program user + valid invite code -> creates bound account without password/account_identifier input
- created user persists `display_name` and `avatar_url`

**Step 2: Run test to verify it fails**

Run:
```bash
cd /Users/carlwang/football_insight/football_insight_service_backend_rs
cargo test wechat_oauth_flow -- --nocapture
```

Expected: failing because repository methods still require phone/password.

**Step 3: Write minimal implementation**

Refactor bind/register path so mini-program registration:
- accepts only `bind_token + invite_code`
- creates a user record with generated fallback `account_identifier`
- uses WeChat nickname as `display_name`
- uses WeChat avatar as `avatar_url`
- binds `wx_open_id/union_id`

Repository methods should be renamed/reworked away from `phone_number/password_hash` assumptions for this branch.

**Step 4: Run test to verify it passes**

Run backend tests until both old and new WeChat auth scenarios are green.

**Step 5: Commit**

```bash
git add /Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/application/bind_wechat_account.rs /Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/ports/auth_repository.rs /Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/adapters/persistence/postgres_auth_repository.rs /Users/carlwang/football_insight/football_insight_service_backend_rs/tests/wechat_oauth_flow.rs
git commit -m "feat: bind mini wechat accounts with invite only"
```

### Task 4: Add Axum handlers and routes for mini-program login

**Files:**
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/adapters/web/handlers.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/adapters/web/routes.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/app.rs`
- Test: `/Users/carlwang/football_insight/football_insight_service_backend_rs/tests/wechat_webhook.rs` only if route wiring tests are needed

**Step 1: Write the failing test**

Add handler-level or app integration tests for:
- `POST /api/v1/auth/mini-wechat/login`
- `POST /api/v1/auth/mini-wechat/bind`

**Step 2: Run test to verify it fails**

Run:
```bash
cd /Users/carlwang/football_insight/football_insight_service_backend_rs
cargo test mini_wechat -- --nocapture
```

Expected: route missing or DTO mismatch.

**Step 3: Write minimal implementation**

Add:
- `MiniWechatLoginRequest { code }`
- `MiniWechatBindRequest { bind_token, invite_code }`
- response DTO for binding-required state
- route wiring in `routes.rs` and `app.rs`

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test
```

Expected: full backend green.

**Step 5: Commit**

```bash
git add /Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/adapters/web/handlers.rs /Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/adapters/web/routes.rs /Users/carlwang/football_insight/football_insight_service_backend_rs/src/app.rs
git commit -m "feat: expose mini wechat auth endpoints"
```

### Task 5: Implement WeChat mini-program adapter

**Files:**
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/adapters/integration/wechat_oauth_port.rs`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/ports/wechat_oauth_port.rs`
- Optional docs: `/Users/carlwang/football_insight/football_insight_service_backend_rs/README.md`

**Step 1: Write the failing test**

Add a focused unit test for parsing mini-program login responses and propagating nickname/avatar values.

**Step 2: Run test to verify it fails**

Run:
```bash
cd /Users/carlwang/football_insight/football_insight_service_backend_rs
cargo test wechat_oauth_port -- --nocapture
```

**Step 3: Write minimal implementation**

Use WeChat mini-program endpoints:
- `jscode2session` for `openid/unionid/session_key`

For nickname/avatar, accept them from the client-side authorized profile payload when binding; do not hardcode secrets into git-tracked files.

**Step 4: Run test to verify it passes**

Run the focused test plus full `cargo test`.

**Step 5: Commit**

```bash
git add /Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/adapters/integration/wechat_oauth_port.rs /Users/carlwang/football_insight/football_insight_service_backend_rs/src/auth/ports/wechat_oauth_port.rs
git commit -m "feat: support mini program wechat code exchange"
```

### Task 6: Implement mini-program frontend login flow

**Files:**
- Modify: `/Users/carlwang/football_insight/football_insight_mini/src/types/auth.ts`
- Modify: `/Users/carlwang/football_insight/football_insight_mini/src/api/auth.ts`
- Modify: `/Users/carlwang/football_insight/football_insight_mini/src/pages/user/index.vue`
- Modify: `/Users/carlwang/football_insight/football_insight_mini/src/utils/authStorage.ts` if extra temporary storage is needed

**Step 1: Write the failing test**

If there are no component tests, add focused utility tests around response normalization or login state helpers. At minimum create a manual verification checklist in `progress.md` before implementation.

**Step 2: Run test to verify it fails**

Run:
```bash
cd /Users/carlwang/football_insight/football_insight_mini
bun run type-check
```

Expected: type errors once new mini login DTOs are referenced.

**Step 3: Write minimal implementation**

On `我的` 页:
- add `微信登录` button
- call `uni.login({ provider: 'weixin' })`
- send `code` to `/auth/mini-wechat/login`
- if authenticated -> store token and refresh user
- if binding required -> show invite-only completion sheet
- request user profile nickname/avatar (through mini-program profile API) before final bind
- send `bind_token + invite_code + display_name + avatar_url` to `/auth/mini-wechat/bind`

Remove password/account_identifier requirement from the mini WeChat bind branch.

**Step 4: Run test to verify it passes**

Run:
```bash
bun run type-check
bun run build:mp-weixin
```

**Step 5: Commit**

```bash
git add /Users/carlwang/football_insight/football_insight_mini/src/types/auth.ts /Users/carlwang/football_insight/football_insight_mini/src/api/auth.ts /Users/carlwang/football_insight/football_insight_mini/src/pages/user/index.vue
git commit -m "feat: add mini program wechat login flow"
```

### Task 7: Document secrets and rollout constraints

**Files:**
- Modify: `/Users/carlwang/football_insight/football_insight_mini/README.md`
- Modify: `/Users/carlwang/football_insight/football_insight_service_backend_rs/README.md`
- Modify: `/Users/carlwang/football_insight/README.md`

**Step 1: Write the failing doc checklist**

Add a checklist in `progress.md` for:
- appid source
- appsecret storage
- local env setup
- backend restart requirement
- production deployment notes

**Step 2: Verify checklist reveals gaps**

Read docs and confirm mini-program WeChat login is not yet documented.

**Step 3: Write minimal documentation**

Document:
- appid usage
- appsecret must live in env only
- local testing requirements in WeChat DevTools
- backend routes involved

**Step 4: Run verification**

No build step required; read docs for completeness.

**Step 5: Commit**

```bash
git add /Users/carlwang/football_insight/football_insight_mini/README.md /Users/carlwang/football_insight/football_insight_service_backend_rs/README.md /Users/carlwang/football_insight/README.md
git commit -m "docs: add mini wechat login setup notes"
```
