import { describe, expect, test } from 'bun:test'
import { ref, type Ref } from 'vue'
import { createUserAccountEffects, type UserAccountEffectsDeps, type UserAccountEffectsState } from './accountEffects'
import type { AppSession } from '../../stores/appSession'
import type { CurrentUser, MiniWechatBindingRequiredResponse, NotificationEmailResponse } from '../../types/auth'

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
  toasts: string[]
  deps: UserAccountEffectsDeps
  emailRequest: Deferred<NotificationEmailResponse>
}

function createState(overrides?: Partial<UserAccountEffectsDeps>): StateFixture {
  const toasts: string[] = []
  const emailRequest = deferred<NotificationEmailResponse>()
  const deps: UserAccountEffectsDeps = {
    canControlMiniReview: () => Promise.resolve({ allowed: false }),
    getNotificationEmail: () => emailRequest.promise,
    updateNotificationEmail: () => Promise.reject(new Error('unexpected update call')),
    loadSystemConfigUnderReview: () => Promise.resolve(false),
    showToast: (title) => {
      toasts.push(title)
    },
    ...overrides,
  }
  return {
    loading: ref(false),
    systemConfigUnderReview: ref(false),
    isReviewController: ref(false),
    notificationEmail: ref(''),
    notificationEmailSheetVisible: ref(false),
    notificationEmailSaving: ref(false),
    miniWechatBindState: ref<MiniWechatBindingRequiredResponse | null>(null),
    passwordLoginSheetVisible: ref(false),
    passwordLoginSubmitting: ref(false),
    toasts,
    deps,
    emailRequest,
  }
}

describe('user 页账号读写 revision 防护', () => {
  test('正常路径：发布白名单、邮箱并结束 loading', async () => {
    const stub = createSessionStub()
    const state = createState({
      canControlMiniReview: () => Promise.resolve({ allowed: true }),
      getNotificationEmail: () => Promise.resolve({ email: 'a@example.com' }),
    })
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    await effects.loadUser()

    expect(stub.ensureCalls).toBe(1)
    expect(state.isReviewController.value).toBe(true)
    expect(state.notificationEmail.value).toBe('a@example.com')
    expect(state.loading.value).toBe(false)
  })

  test('A 的邮箱 GET 在会话被替代后 resolve，不写入邮箱也不结束 loading', async () => {
    const stub = createSessionStub()
    const state = createState()
    state.notificationEmail.value = ''
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    const load = effects.loadUser()
    await flushMicrotasks()
    expect(stub.ensureCalls).toBe(1)

    // 会话被替代（新登录/退出）后，旧会话的邮箱响应才返回。
    stub.revision.value += 1
    state.emailRequest.resolve({ email: 'stale@example.com' })
    await load

    expect(state.notificationEmail.value).toBe('')
    expect(state.loading.value).toBe(true)
  })

  test('A 的邮箱 GET 在会话被替代后 reject，不覆盖既有邮箱', async () => {
    const stub = createSessionStub()
    const state = createState()
    state.notificationEmail.value = 'keep@example.com'
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    const load = effects.loadUser()
    await flushMicrotasks()

    stub.revision.value += 1
    state.emailRequest.reject(new Error('网络错误'))
    await load

    expect(state.notificationEmail.value).toBe('keep@example.com')
    expect(state.loading.value).toBe(true)
  })

  test('邮箱保存成功后会话被替代：不写邮箱、不关弹层、不弹 toast', async () => {
    const stub = createSessionStub()
    const updateRequest = deferred<NotificationEmailResponse>()
    const state = createState({
      updateNotificationEmail: () => updateRequest.promise,
    })
    state.notificationEmailSheetVisible.value = true
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    const save = effects.saveNotificationEmail('new@example.com')
    await flushMicrotasks()
    expect(state.notificationEmailSaving.value).toBe(true)

    stub.revision.value += 1
    updateRequest.resolve({ email: 'new@example.com' })
    await save

    expect(state.notificationEmail.value).toBe('')
    expect(state.notificationEmailSheetVisible.value).toBe(true)
    expect(state.notificationEmailSaving.value).toBe(true)
    expect(state.toasts.length).toBe(0)
  })

  test('邮箱保存在会话未变时正常发布并 toast', async () => {
    const stub = createSessionStub()
    const state = createState({
      updateNotificationEmail: () => Promise.resolve({ email: 'new@example.com' }),
    })
    state.notificationEmailSheetVisible.value = true
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    await effects.saveNotificationEmail('new@example.com')

    expect(state.notificationEmail.value).toBe('new@example.com')
    expect(state.notificationEmailSheetVisible.value).toBe(false)
    expect(state.notificationEmailSaving.value).toBe(false)
    expect(state.toasts).toEqual(['邮箱已保存'])
  })

  test('resetSessionScopedState 清空会话私有状态', () => {
    const stub = createSessionStub()
    const state = createState()
    state.notificationEmail.value = 'a@example.com'
    state.notificationEmailSheetVisible.value = true
    state.notificationEmailSaving.value = true
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

    expect(state.notificationEmail.value).toBe('')
    expect(state.notificationEmailSheetVisible.value).toBe(false)
    expect(state.notificationEmailSaving.value).toBe(false)
    expect(state.miniWechatBindState.value).toBeNull()
    expect(state.passwordLoginSheetVisible.value).toBe(false)
    expect(state.passwordLoginSubmitting.value).toBe(false)
    expect(state.isReviewController.value).toBe(false)
    expect(state.loading.value).toBe(false)
  })

  test('resetPrivateViewState 只清展示数据，保留正在提交的认证表单上下文', () => {
    const stub = createSessionStub()
    const state = createState()
    state.notificationEmail.value = 'a@example.com'
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
    expect(state.notificationEmail.value).toBe('')
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
      getNotificationEmail: () => Promise.resolve({ email: 'a@example.com' }),
    })
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    // 页面 onShow 初始化与会话恢复完成通知同时触发：只执行一次真实加载。
    await Promise.all([effects.loadUser(), effects.loadUser(), effects.loadUser()])

    expect(stub.ensureCalls).toBe(1)
    expect(state.notificationEmail.value).toBe('a@example.com')
    expect(state.loading.value).toBe(false)
  })

  test('会话被替代（非本页提交）时 handleSessionRevisionChange 全量清理并重新加载私有区', async () => {
    const stub = createSessionStub()
    const state = createState({
      getNotificationEmail: () => Promise.resolve({ email: 'restored@example.com' }),
    })
    state.notificationEmail.value = 'stale@example.com'
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
    expect(state.notificationEmail.value).toBe('restored@example.com')
    expect(state.loading.value).toBe(false)
  })

  test('本页自己在途认证提交的 revision 变化只清展示数据，不触发额外加载', async () => {
    const stub = createSessionStub()
    const state = createState()
    state.notificationEmail.value = 'a@example.com'
    const effects = createUserAccountEffects(stub.session, state, state.deps)

    effects.handleSessionRevisionChange(true)
    await flushMicrotasks()

    expect(state.notificationEmail.value).toBe('')
    expect(stub.ensureCalls).toBe(0)
  })
})
