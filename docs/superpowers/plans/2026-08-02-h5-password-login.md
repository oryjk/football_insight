# H5 Password Login Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Give H5 users an account/password login sheet while preserving Mini Program WeChat login and the shared authenticated token flow.

**Architecture:** A focused pure helper module defines platform presentation and password-form validation/payload construction. The existing user page owns the H5-only sheet and calls the existing `login` API; Mini Program continues through the existing WeChat handler. Both success paths use the existing access-token storage and user state.

**Tech Stack:** uni-app, Vue 3, TypeScript, Bun test runner, Bun-based uni-app builds

## Global Constraints

- H5 uses account identifier and password login.
- WeChat Mini Program continues to use mini-program WeChat login.
- Both paths produce the existing access token and then share the same authenticated application flow.
- Registration, password reset, fake WeChat identities, JWT injection, and embedded test credentials are out of scope.
- The Rust backend API contract remains unchanged.
- Frontend commands must use Bun.

---

### Task 1: Platform And Form Helpers

**Files:**
- Create: `football_insight_mini/src/pages/user/loginHelpers.ts`
- Create: `football_insight_mini/src/pages/user/loginHelpers.test.ts`

**Interfaces:**
- Consumes: `LoginPayload` from `src/types/auth.ts`.
- Produces: `resolveUserLoginPresentation(isH5: boolean): UserLoginPresentation`, `validatePasswordLoginForm(form: PasswordLoginForm): string | null`, and `buildPasswordLoginPayload(form: PasswordLoginForm): LoginPayload`.

- [x] **Step 1: Write failing helper tests**

```ts
import { describe, expect, test } from 'bun:test'
import {
  buildPasswordLoginPayload,
  resolveUserLoginPresentation,
  validatePasswordLoginForm,
} from './loginHelpers'

test('uses password login presentation for H5', () => {
  expect(resolveUserLoginPresentation(true)).toEqual({
    method: 'password',
    actionText: '账号登录',
    switchAccountCaption: '如需切换账号，可退出后重新使用账号密码登录。',
  })
})

test('uses WeChat login presentation for Mini Program', () => {
  expect(resolveUserLoginPresentation(false)).toEqual({
    method: 'mini-wechat',
    actionText: '去登录',
    switchAccountCaption: '如需切换账号，可退出后重新使用微信登录。',
  })
})

test('validates required fields and builds the existing API payload', () => {
  expect(validatePasswordLoginForm({ accountIdentifier: '', password: 'secret123' })).toBe('请输入账号')
  expect(validatePasswordLoginForm({ accountIdentifier: 'tester', password: '' })).toBe('请输入密码')
  expect(buildPasswordLoginPayload({ accountIdentifier: ' tester ', password: ' secret123 ' })).toEqual({
    account_identifier: 'tester',
    password: ' secret123 ',
  })
})
```

- [x] **Step 2: Run the focused test and confirm it fails**

Run: `cd football_insight_mini && bun test src/pages/user/loginHelpers.test.ts`

Expected: FAIL because `loginHelpers.ts` does not exist.

- [x] **Step 3: Implement the pure helpers**

```ts
import type { LoginPayload } from '../../types/auth'

export interface PasswordLoginForm {
  accountIdentifier: string
  password: string
}

export interface UserLoginPresentation {
  method: 'password' | 'mini-wechat'
  actionText: string
  switchAccountCaption: string
}

export function resolveUserLoginPresentation(isH5: boolean): UserLoginPresentation {
  return isH5
    ? {
        method: 'password',
        actionText: '账号登录',
        switchAccountCaption: '如需切换账号，可退出后重新使用账号密码登录。',
      }
    : {
        method: 'mini-wechat',
        actionText: '去登录',
        switchAccountCaption: '如需切换账号，可退出后重新使用微信登录。',
      }
}

export function validatePasswordLoginForm(form: PasswordLoginForm): string | null {
  if (!form.accountIdentifier.trim()) return '请输入账号'
  if (!form.password.trim()) return '请输入密码'
  return null
}

export function buildPasswordLoginPayload(form: PasswordLoginForm): LoginPayload {
  return {
    account_identifier: form.accountIdentifier.trim(),
    password: form.password,
  }
}
```

- [x] **Step 4: Run the focused test and confirm it passes**

Run: `cd football_insight_mini && bun test src/pages/user/loginHelpers.test.ts`

Expected: all helper tests PASS.

### Task 2: H5 Login Sheet And Platform Dispatch

**Files:**
- Modify: `football_insight_mini/src/pages/user/index.vue`
- Modify: `football_insight_mini/src/components/FiLoginFloat.vue`
- Test: `football_insight_mini/src/pages/user/loginHelpers.test.ts`

**Interfaces:**
- Consumes: Task 1 helper exports and the existing `login(payload: LoginPayload): Promise<AuthResponse>` API.
- Produces: H5 `账号登录` sheet behavior and unchanged Mini Program WeChat login dispatch.

- [x] **Step 1: Wire platform presentation into the guest and account actions**

Replace the disabled H5 float with `:action-text="loginPresentation.actionText"` and `@action="handleLoginAction"`. Use `loginPresentation.switchAccountCaption` in the authenticated account section. `handleLoginAction` opens the password sheet when `method === 'password'`; otherwise it invokes `handleMiniWechatLogin`.

- [x] **Step 2: Add the H5 password sheet**

Add H5-guarded template markup with account and masked-password inputs, cancel and submit actions, keyboard submission on the password input, and a disabled/loading submit state. Add reactive state:

```ts
const passwordLoginSheetVisible = ref(false)
const passwordLoginSubmitting = ref(false)
const passwordLoginForm = reactive({ accountIdentifier: '', password: '' })
```

Include the password sheet in `hasOpenSheet` so the guest background and floating login control do not overlap it.

- [x] **Step 3: Implement password login submission**

Validate with `validatePasswordLoginForm`. On success call the existing `login(buildPasswordLoginPayload(passwordLoginForm))`, update the local token flag and current user, close the sheet, refresh authenticated page data through `loadUser`, show `登录成功`, and consume the existing post-login redirect. On error keep the sheet open and show `extractApiErrorMessage(error, '登录失败')`; always release the submitting state.

- [x] **Step 4: Add focused sheet styling**

Reuse `.sheet-mask`, `.sheet-card`, `.auth-input`, `.sheet-actions`, and `.primary-action`. Add only a `.password-login-form` layout rule and disabled-button styling required for a stable compact form; do not introduce a separate visual system. Position the login float and sheet mask above uni-app's `--window-bottom` so H5's TabBar cannot intercept the login action or cover the sheet.

- [x] **Step 5: Run focused and existing user tests**

Run: `cd football_insight_mini && bun test src/pages/user/loginHelpers.test.ts src/pages/user/helpers.test.ts`

Expected: all tests PASS.

- [x] **Step 6: Run static and cross-platform build verification**

Run:

```bash
cd football_insight_mini
bun run type-check
bun run build:h5
bun run build:mp-weixin
```

Expected: type-check, H5 build, and WeChat Mini Program build all exit successfully.

- [x] **Step 7: Browser smoke test**

Start `bun run dev:h5`, open the user page in a desktop browser and a mobile viewport, and verify that `账号登录` opens a non-overlapping password sheet. Confirm empty submission is rejected and that a real account can authenticate when valid test credentials are available. Do not create or commit credentials.

- [x] **Step 8: Commit and push**

```bash
git add docs/superpowers/plans/2026-08-02-h5-password-login.md \
  football_insight_mini/src/components/FiLoginFloat.vue \
  football_insight_mini/src/pages/user/loginHelpers.ts \
  football_insight_mini/src/pages/user/loginHelpers.test.ts \
  football_insight_mini/src/pages/user/index.vue
git commit -m "feat: support password login on H5"
git push origin main
```
