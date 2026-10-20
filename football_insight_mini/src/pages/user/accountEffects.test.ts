import { describe, expect, test } from 'bun:test'
import { ref, type Ref } from 'vue'
import { createUserAccountEffects, type UserAccountEffectsDeps, type UserAccountEffectsState } from './accountEffects'
import type { AppSession } from '../../stores/appSession'
import type { CurrentUser, MiniWechatBindingRequiredResponse } from '../../types/auth'

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
  reject: (error: unknown) => void
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  let reject!: (error: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

async function flushMicrotasks(times = 20): Promise<void> {
  for (let index = 0; index < times; index += 1) {
    await Promise.resolve()
  }
}

const userA: CurrentUser = {
  id: 'a',
  display_name: '用户a',
  account_identifier: 'account-a',
  avatar_url: null,
  has_wechat_binding: false,
  membership_tier: 'FREE',
  created_at: '2026-01-01T00:00:00Z',
}

interface SessionStub {
  session: AppSession
  revision: Ref<number>
  hasAccessToken: Ref<boolean>
  currentUser: Ref<CurrentUser | null>
  ensureCalls: number
  ensureHandler: () => Promise<CurrentUser | null>
}

function createSessionStub(options?: { hasAccessToken?: boolean }): SessionStub {
  const revision = ref(0)
  const hasAccessToken = ref(options?.hasAccessToken ?? true)
  const currentUser = ref<CurrentUser | null>(options?.hasAccessToken === false ? null : userA)
  const stub: SessionStub = {
    session: {} as AppSession,
    revision,
    hasAccessToken,
    currentUser,
    ensureCalls: 0,
    ensureHandler: () => Promise.resolve(currentUser.value),
  }
  stub.session = {
    currentUser,
    hasAccessToken,
    revision,
    status: ref('authenticated'),
    errorMessage: ref(''),
    ensureSessionReady: () => {
      stub.ensureCalls += 1
      return stub.ensureHandler()
    },
  } as unknown as AppSession
  return stub
}

interface StateFixture extends UserAccountEffectsState {
  deps: UserAccountEffectsDeps
}

function createState(overrides?: Partial<UserAccountEffectsDeps>): StateFixture {
  const deps: UserAccountEffectsDeps = {
    canControlMiniReview: () => Promise.resolve({ allowed: false }),
    ...overrides,
  }
  return {
    loading: ref(false),
    isReviewController: ref(false),
    miniWechatBindState: ref<MiniWechatBindingRequiredResponse | null>(null),
    passwordLoginSheetVisible: ref(false),
    passwordLoginSubmitting: ref(false),
    deps,
  }
}

describe('user 页账号读写 revision 防护', () => {
  test('正常路径：发布白名单并结束 loading', async () => {
    const stub = createSessionStub()
    const state = createState({
      canControlMiniReview: () => Promise.resolve({ allowed: true }),
    })
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    await effects.loadUser()

    expect(stub.ensureCalls).toBe(1)
    expect(state.isReviewController.value).toBe(true)
    expect(state.loading.value).toBe(false)
  })

  test('旧会话的账号恢复不会结束新会话的 loading', async () => {
    const stub = createSessionStub()
    const pending = deferred<CurrentUser | null>()
    stub.ensureHandler = () => pending.promise
    const state = createState()
    const effects = createUserAccountEffects(stub.session, state, state.deps)
    const load = effects.loadUser()
    await flushMicrotasks()
    stub.revision.value += 1
    pending.resolve(userA)
    await load
    expect(state.loading.value).toBe(true)
  })

  test('resetSessionScopedState 清空会话私有状态', () => {
    const stub = createSessionStub()
    const state = createState()
    state.miniWechatBindState.value = {
      status: 'binding_required',
      bind_token: 'bind-token',
      expires_at: '2026-02-01T00:00:00Z',
      display_name: '微信用户',
      avatar_url: null,
    }
    state.passwordLoginSheetVisible.value = true
    state.passwordLoginSubmitting.value = true
    state.isReviewController.value = true
    state.loading.value = true

    const effects = createUserAccountEffects(stub.session, state, state.deps)
    effects.resetSessionScopedState()

    expect(state.miniWechatBindState.value).toBeNull()
    expect(state.passwordLoginSheetVisible.value).toBe(false)
    expect(state.passwordLoginSubmitting.value).toBe(false)
    expect(state.isReviewController.value).toBe(false)
    expect(state.loading.value).toBe(false)
  })

  test('resetPrivateViewState 只清展示数据，保留正在提交的认证表单上下文', () => {
    const stub = createSessionStub()
    const state = createState()
    state.isReviewController.value = true
    state.loading.value = true
    state.miniWechatBindState.value = {
      status: 'binding_required',
      bind_token: 'bind-token',
      expires_at: '2026-02-01T00:00:00Z',
      display_name: '微信用户',
      avatar_url: null,
    }
    state.passwordLoginSheetVisible.value = true
    state.passwordLoginSubmitting.value = true

    const effects = createUserAccountEffects(stub.session, state, state.deps)
    effects.resetPrivateViewState()

    // 旧账号展示数据被清理。
    expect(state.isReviewController.value).toBe(false)
    expect(state.loading.value).toBe(false)
    // 认证表单上下文保留：绑定 token、密码弹层与提交标记不受影响。
    expect(state.miniWechatBindState.value?.bind_token).toBe('bind-token')
    expect(state.passwordLoginSheetVisible.value).toBe(true)
    expect(state.passwordLoginSubmitting.value).toBe(true)
  })

  test('同一会话（revision）的并发 loadUser 合并为一个请求', async () => {
    const stub = createSessionStub()
    const state = createState({
    })
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    // 页面 onShow 初始化与会话恢复完成通知同时触发：只执行一次真实加载。
    await Promise.all([effects.loadUser(), effects.loadUser(), effects.loadUser()])

    expect(stub.ensureCalls).toBe(1)
    expect(state.loading.value).toBe(false)
  })

  test('会话被替代（非本页提交）时 handleSessionRevisionChange 全量清理并重新加载私有区', async () => {
    const stub = createSessionStub()
    const state = createState({
    })
    state.miniWechatBindState.value = {
      status: 'binding_required',
      bind_token: 'bind-token',
      expires_at: '2026-02-01T00:00:00Z',
      display_name: '微信用户',
      avatar_url: null,
    }
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    // 后台静默恢复/其他登录发布用户：清掉旧展示并立即刷新私有区（无需切 Tab）。
    effects.handleSessionRevisionChange(false)
    expect(state.miniWechatBindState.value).toBeNull()
    await flushMicrotasks()

    expect(stub.ensureCalls).toBe(1)
    expect(state.loading.value).toBe(false)
  })

  test('本页自己在途认证提交的 revision 变化只清展示数据，不触发额外加载', async () => {
    const stub = createSessionStub()
    const state = createState()
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    effects.handleSessionRevisionChange(true)
    await flushMicrotasks()

    expect(stub.ensureCalls).toBe(0)
  })
})

describe('审核展示不能阻断账号认证', () => {
  test('审核中仍恢复当前账号', async () => {
    const stub = createSessionStub()
    const state = createState()
    const effects = createUserAccountEffects(stub.session, state, state.deps)
    await effects.loadUser()
    expect(stub.ensureCalls).toBe(1)
  })
  test('审核中刷新页面保留待完成的微信绑定表单', async () => {
    const stub = createSessionStub({ hasAccessToken: false })
    const state = createState()
    state.miniWechatBindState.value = {
      status: 'binding_required', bind_token: 'pending-bind', expires_at: '2026-10-14T00:00:00Z',
      display_name: '球迷', avatar_url: null,
    }
    const effects = createUserAccountEffects(stub.session, state, state.deps)
    await effects.loadUser()
    expect(state.miniWechatBindState.value?.bind_token).toBe('pending-bind')
  })
})
