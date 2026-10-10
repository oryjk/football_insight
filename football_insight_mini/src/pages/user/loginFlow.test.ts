import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { effectScope, ref, watch } from 'vue'
import { createUserLoginFlow, type UserLoginFlowDeps, type UserLoginFlowState, type UserLoginFlow } from './loginFlow'
import { createUserAccountEffects, type UserAccountEffects, type UserAccountEffectsState } from './accountEffects'
import { createAppSession, SessionSupersededError, type AppSession } from '../../stores/appSession'
import { createMiniReviewStore } from '../../stores/miniReview'
import type * as authApi from '../../api/auth'
import type * as miniReviewApi from '../../api/miniReview'
import type { AuthResponse, CurrentUser, MiniWechatLoginResponse } from '../../types/auth'
import type { MiniReviewStatus } from '../../types/system'
import { ApiRequestError } from '../../utils/apiError'
import { AUTH_TOKEN_STORAGE_KEY } from '../../utils/authStorage'

type AuthSource = typeof authApi

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

function buildUser(id: string): CurrentUser {
  return {
    id,
    display_name: `用户${id}`,
    account_identifier: `account-${id}`,
    avatar_url: null,
    has_wechat_binding: false,
    membership_tier: 'FREE',
    created_at: '2026-01-01T00:00:00Z',
  }
}

function buildAuthResponse(user: CurrentUser, token: string): AuthResponse {
  return { access_token: token, expires_at: '2026-02-01T00:00:00Z', user }
}

const userA = buildUser('a')
const userB = buildUser('b')

const globalWithUni = globalThis as unknown as { uni?: unknown }
const previousUni = globalWithUni.uni

let storage: Record<string, unknown>

interface SourceStub {
  source: AuthSource
  calls: Record<'getCurrentUser' | 'login' | 'loginWithMiniWechat' | 'bindMiniWechatAccount' | 'logout', number>
  handlers: {
    getCurrentUser: () => Promise<CurrentUser | null>
    login: () => Promise<AuthResponse>
    loginWithMiniWechat: () => Promise<MiniWechatLoginResponse>
    bindMiniWechatAccount: () => Promise<AuthResponse>
    logout: () => Promise<void>
  }
}

function unexpectedCall(name: string): () => Promise<never> {
  return () => Promise.reject(new Error(`unexpected call: ${name}`))
}

function createSourceStub(): SourceStub {
  const stub: SourceStub = {
    calls: {
      getCurrentUser: 0,
      login: 0,
      loginWithMiniWechat: 0,
      bindMiniWechatAccount: 0,
      logout: 0,
    },
    handlers: {
      getCurrentUser: unexpectedCall('getCurrentUser'),
      login: unexpectedCall('login'),
      loginWithMiniWechat: unexpectedCall('loginWithMiniWechat'),
      bindMiniWechatAccount: unexpectedCall('bindMiniWechatAccount'),
      logout: unexpectedCall('logout'),
    },
    source: {} as AuthSource,
  }
  stub.source = {
    getCurrentUser: () => {
      stub.calls.getCurrentUser += 1
      return stub.handlers.getCurrentUser()
    },
    login: () => {
      stub.calls.login += 1
      return stub.handlers.login()
    },
    register: unexpectedCall('register'),
    loginAsH5TestUser: unexpectedCall('loginAsH5TestUser'),
    loginWithMiniWechat: () => {
      stub.calls.loginWithMiniWechat += 1
      return stub.handlers.loginWithMiniWechat()
    },
    bindMiniWechatAccount: () => {
      stub.calls.bindMiniWechatAccount += 1
      return stub.handlers.bindMiniWechatAccount()
    },
    logout: () => {
      stub.calls.logout += 1
      return stub.handlers.logout()
    },
  } as unknown as AuthSource
  return stub
}

beforeEach(() => {
  storage = {}
  globalWithUni.uni = {
    getStorageSync: (key: string) => storage[key] ?? '',
    setStorageSync: (key: string, value: unknown) => {
      storage[key] = value
    },
    removeStorageSync: (key: string) => {
      delete storage[key]
    },
  }
})

afterEach(() => {
  globalWithUni.uni = previousUni
})

interface FlowFixture {
  session: AppSession
  state: UserLoginFlowState
  toasts: string[]
  redirects: number
  loadUserRequest: Deferred<void>
  wechatCodeQueue: Array<Promise<string>>
  wechatCodeCalls: number
  deps: UserLoginFlowDeps
}

function createFlowFixture(): FlowFixture {
  const stub = createSourceStub()
  const state: UserLoginFlowState = {
    passwordLoginSheetVisible: ref(false),
    passwordLoginSubmitting: ref(false),
    miniWechatBindState: ref(null),
  }
  const toasts: string[] = []
  const loadUserRequest = deferred<void>()
  const fixture: FlowFixture = {
    session: {} as AppSession,
    state,
    toasts,
    redirects: 0,
    loadUserRequest,
    wechatCodeQueue: [],
    wechatCodeCalls: 0,
    deps: {
      redirectAfterLogin: async () => {
        fixture.redirects += 1
      },
      showToast: (title) => {
        toasts.push(title)
      },
    },
  }
  // 微信 code 由 session 的认证流程获取：测试通过队列注入可控延迟的 code。
  fixture.session = createAppSession(stub.source, {
    getMiniWechatCode: () => {
      fixture.wechatCodeCalls += 1
      return fixture.wechatCodeQueue.shift() ?? Promise.resolve('wechat-code')
    },
  })
  ;(fixture as { stub?: SourceStub }).stub = stub
  return fixture
}

function getStub(fixture: FlowFixture): SourceStub {
  return (fixture as unknown as { stub: SourceStub }).stub
}

function createFlow(fixture: FlowFixture) {
  return createUserLoginFlow(
    fixture.session,
    fixture.state,
    () => fixture.loadUserRequest.promise,
    fixture.deps,
  )
}

describe('user 登录编排 revision 防护', () => {
  test('密码登录成功正常路径：关弹层、toast、跳转、submitting 复位', async () => {
    const fixture = createFlowFixture()
    getStub(fixture).handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))
    fixture.state.passwordLoginSheetVisible.value = true
    const flow = createFlow(fixture)

    const pending = flow.submitPasswordLogin({ accountIdentifier: 'a', password: 'x' })
    await flushMicrotasks()
    fixture.loadUserRequest.resolve()
    await pending

    expect(fixture.state.passwordLoginSheetVisible.value).toBe(false)
    expect(fixture.state.passwordLoginSubmitting.value).toBe(false)
    expect(fixture.toasts).toEqual(['登录成功'])
    expect(fixture.redirects).toBe(1)
  })

  test('登录成功后 loadUser 期间被退出替代：不提示成功也不跳转', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))
    stub.handlers.logout = () => Promise.resolve()
    fixture.state.passwordLoginSheetVisible.value = true
    const flow = createFlow(fixture)

    const pending = flow.submitPasswordLogin({ accountIdentifier: 'a', password: 'x' })
    await flushMicrotasks()
    // 登录已发布，loadUser 等待期间用户退出。
    await fixture.session.logoutSession()
    fixture.loadUserRequest.resolve()
    await pending

    expect(fixture.toasts).toEqual([])
    expect(fixture.redirects).toBe(0)
    expect(fixture.session.status.value).toBe('guest')
  })

  test('A 登录在途时 B 登录成功：A 不提示不跳转，B 的会话不受影响', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    const loginA = deferred<AuthResponse>()
    stub.handlers.login = () => loginA.promise
    const flow = createFlow(fixture)
    const flowB = createFlow(fixture)

    const pendingA = flow.submitPasswordLogin({ accountIdentifier: 'a', password: 'x' })
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userB, 'token-b'))
    const pendingB = flowB.submitPasswordLogin({ accountIdentifier: 'b', password: 'y' })
    await flushMicrotasks()
    fixture.loadUserRequest.resolve()
    await pendingB

    loginA.resolve(buildAuthResponse(userA, 'token-a'))
    await pendingA
    await flushMicrotasks()

    expect(fixture.toasts).toEqual(['登录成功'])
    expect(fixture.redirects).toBe(1)
    expect(fixture.session.currentUser.value).toMatchObject({ id: 'b' })
  })

  test('密码登录失败：提示错误、保留弹层、submitting 复位可重试', async () => {
    const fixture = createFlowFixture()
    getStub(fixture).handlers.login = () =>
      Promise.reject(new ApiRequestError('账号或密码错误', 400))
    fixture.state.passwordLoginSheetVisible.value = true
    const flow = createFlow(fixture)

    await flow.submitPasswordLogin({ accountIdentifier: 'a', password: 'wrong' })

    expect(fixture.toasts).toEqual(['账号或密码错误'])
    expect(fixture.state.passwordLoginSheetVisible.value).toBe(true)
    expect(fixture.state.passwordLoginSubmitting.value).toBe(false)
  })

  test('微信登录 binding_required：发布绑定表单状态，不 toast', async () => {
    const fixture = createFlowFixture()
    getStub(fixture).handlers.loginWithMiniWechat = () =>
      Promise.resolve({
        status: 'binding_required',
        bind_token: 'bind-token',
        expires_at: '2026-02-01T00:00:00Z',
        display_name: '微信用户',
        avatar_url: null,
      })
    const flow = createFlow(fixture)

    await flow.loginWithMiniWechat()

    expect(fixture.state.miniWechatBindState.value?.bind_token).toBe('bind-token')
    expect(fixture.toasts).toEqual([])
  })

  test('绑定失败：保留绑定表单与 bind_token，提示错误可重试', async () => {
    const fixture = createFlowFixture()
    getStub(fixture).handlers.bindMiniWechatAccount = () =>
      Promise.reject(new ApiRequestError('邀请码无效', 400))
    fixture.state.miniWechatBindState.value = {
      status: 'binding_required',
      bind_token: 'bind-token',
      expires_at: '2026-02-01T00:00:00Z',
      display_name: '微信用户',
      avatar_url: null,
    }
    const flow = createFlow(fixture)

    await flow.bindMiniWechatAccount({
      bindToken: 'bind-token',
      inviteCode: 'bad-code',
      referralCode: '',
      displayName: '微信用户',
      avatarDataUrl: 'data:image/png;base64,x',
    })

    expect(fixture.toasts).toEqual(['邀请码无效'])
    expect(fixture.state.miniWechatBindState.value?.bind_token).toBe('bind-token')
  })

  test('退出等待期间新登录成功：不提示「已退出登录」', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))
    await fixture.session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    const logoutRequest = deferred<void>()
    stub.handlers.logout = () => logoutRequest.promise
    const flow = createFlow(fixture)

    const pending = flow.logout()
    // 远端注销等待期间，用户重新登录。
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userB, 'token-b'))
    await fixture.session.loginWithPassword({ account_identifier: 'b', password: 'y' })
    logoutRequest.resolve()
    await flushMicrotasks()
    fixture.loadUserRequest.resolve()
    await pending

    expect(fixture.toasts).toEqual([])
    expect(fixture.session.currentUser.value).toMatchObject({ id: 'b' })
  })

  test('isOwnAuthSubmit 只识别本 flow 在途提交的 revision', async () => {
    const fixture = createFlowFixture()
    const loginA = deferred<AuthResponse>()
    getStub(fixture).handlers.login = () => loginA.promise
    const flow = createFlow(fixture)

    expect(flow.isOwnAuthSubmit(fixture.session.revision.value + 1)).toBe(false)
    const pending = flow.submitPasswordLogin({ accountIdentifier: 'a', password: 'x' })
    const intentRevision = fixture.session.revision.value
    // 本 flow 发起的提交：revision 监听不应清认证表单。
    expect(flow.isOwnAuthSubmit(intentRevision)).toBe(true)
    // 其他 revision（退出/其他操作产生）：仍全量清理。
    expect(flow.isOwnAuthSubmit(intentRevision + 1)).toBe(false)

    loginA.resolve(buildAuthResponse(userA, 'token-a'))
    await flushMicrotasks()
    fixture.loadUserRequest.resolve()
    await pending
    expect(flow.isOwnAuthSubmit(intentRevision)).toBe(false)
  })

  test('A 绑定提交在途时被 B 登录替代：A 不提示，B 的 revision 不属于 A 的提交', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    const bindA = deferred<AuthResponse>()
    stub.handlers.bindMiniWechatAccount = () => bindA.promise
    fixture.state.miniWechatBindState.value = {
      status: 'binding_required',
      bind_token: 'bind-token',
      expires_at: '2026-02-01T00:00:00Z',
      display_name: '微信用户',
      avatar_url: null,
    }
    const flow = createFlow(fixture)

    const pendingA = flow.bindMiniWechatAccount({
      bindToken: 'bind-token',
      inviteCode: '',
      referralCode: '',
      displayName: '微信用户',
      avatarDataUrl: 'data:image/png;base64,x',
    })
    // A 提交在途时 B 登录成功（替代 A）。
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userB, 'token-b'))
    await fixture.session.loginWithPassword({ account_identifier: 'b', password: 'y' })
    expect(flow.isOwnAuthSubmit(fixture.session.revision.value)).toBe(false)

    bindA.resolve(buildAuthResponse(userA, 'token-a'))
    await pendingA

    expect(fixture.toasts).toEqual([])
    expect(fixture.session.currentUser.value).toMatchObject({ id: 'b' })
  })
})

// 保持 SessionSupersededError 引用（竞态语义文档化）：被替代的操作以该错误收尾。
void SessionSupersededError

describe('微信 code 竞态（认证意图在获取 code 前建立）', () => {
  test('第一次 code 慢、第二次微信登录成功、退出后第一次 code 才返回：认证 API 只调一次，最终游客，无旧成功提示/跳转', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    const slowCode = deferred<string>()
    fixture.wechatCodeQueue.push(slowCode.promise, Promise.resolve('code-fast'))
    stub.handlers.loginWithMiniWechat = () =>
      Promise.resolve({ status: 'authenticated', ...buildAuthResponse(userB, 'token-b') })
    stub.handlers.logout = () => Promise.resolve()
    const flow = createFlow(fixture)

    const pendingFirst = flow.loginWithMiniWechat()
    await flushMicrotasks()
    // 慢 code 还没返回：不能发出任何认证请求。
    expect(stub.calls.loginWithMiniWechat).toBe(0)

    const pendingSecond = flow.loginWithMiniWechat()
    await flushMicrotasks()
    fixture.loadUserRequest.resolve()
    await pendingSecond

    expect(stub.calls.loginWithMiniWechat).toBe(1)
    expect(fixture.toasts).toEqual(['微信登录成功'])
    expect(fixture.redirects).toBe(1)
    expect(fixture.session.currentUser.value).toMatchObject({ id: 'b' })

    await flow.logout()
    expect(fixture.toasts).toEqual(['微信登录成功', '已退出登录'])
    expect(fixture.session.status.value).toBe('guest')

    // 第一次的慢 code 此刻才返回：意图早已被替代，不能再发认证请求、不能复活会话。
    slowCode.resolve('code-slow')
    await pendingFirst
    await flushMicrotasks()

    expect(stub.calls.loginWithMiniWechat).toBe(1)
    expect(fixture.toasts).toEqual(['微信登录成功', '已退出登录'])
    expect(fixture.redirects).toBe(1)
    expect(fixture.session.currentUser.value).toBeNull()
    expect(fixture.session.status.value).toBe('guest')
    expect(fixture.state.miniWechatBindState.value).toBeNull()
  })

  test('code 在新密码登录完成后才返回：不发旧认证请求、不提示旧错误，密码登录流程不受影响', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    const slowCode = deferred<string>()
    fixture.wechatCodeQueue.push(slowCode.promise)
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))
    const flow = createFlow(fixture)

    const pendingWechat = flow.loginWithMiniWechat()
    await flushMicrotasks()

    fixture.state.passwordLoginSheetVisible.value = true
    const pendingPassword = flow.submitPasswordLogin({ accountIdentifier: 'a', password: 'x' })
    await flushMicrotasks()
    fixture.loadUserRequest.resolve()
    await pendingPassword
    expect(fixture.toasts).toEqual(['登录成功'])

    slowCode.resolve('code-late')
    await pendingWechat
    await flushMicrotasks()

    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(fixture.toasts).toEqual(['登录成功'])
    expect(fixture.redirects).toBe(1)
    expect(fixture.session.currentUser.value).toMatchObject({ id: 'a' })
    expect(fixture.state.passwordLoginSubmitting.value).toBe(false)
  })

  test('code 在新绑定登录完成后才失败：不提示旧错误，新会话不受影响', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    const slowCode = deferred<string>()
    fixture.wechatCodeQueue.push(slowCode.promise)
    const flow = createFlow(fixture)

    const pendingWechat = flow.loginWithMiniWechat()
    await flushMicrotasks()

    fixture.state.miniWechatBindState.value = {
      status: 'binding_required',
      bind_token: 'bind-token',
      expires_at: '2026-02-01T00:00:00Z',
      display_name: '微信用户',
      avatar_url: null,
    }
    stub.handlers.bindMiniWechatAccount = () => Promise.resolve(buildAuthResponse(userA, 'token-bind'))
    const pendingBind = flow.bindMiniWechatAccount({
      bindToken: 'bind-token',
      inviteCode: '',
      referralCode: '',
      displayName: '微信用户',
      avatarDataUrl: 'data:image/png;base64,x',
    })
    await flushMicrotasks()
    fixture.loadUserRequest.resolve()
    await pendingBind
    expect(fixture.toasts).toEqual(['微信绑定成功'])

    slowCode.reject(new Error('用户取消微信授权'))
    await pendingWechat
    await flushMicrotasks()

    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(fixture.toasts).toEqual(['微信绑定成功'])
    expect(fixture.session.currentUser.value).toMatchObject({ id: 'a' })
    expect(fixture.session.errorMessage.value).toBe('')
  })
})

describe('revision 监听与认证表单上下文（真实 Vue watcher）', () => {
  function createEffects(fixture: FlowFixture): UserAccountEffects {
    // watcher 触发的恢复刷新会走真实 session.ensureSessionReady：按当前会话用户应答。
    getStub(fixture).handlers.getCurrentUser = () => Promise.resolve(fixture.session.currentUser.value)
    const state: UserAccountEffectsState = {
      loading: ref(false),
      systemConfigUnderReview: ref(false),
      isReviewController: ref(false),
      notificationEmail: ref('a@example.com'),
      notificationEmailSheetVisible: ref(false),
      notificationEmailSaving: ref(false),
      miniWechatBindState: fixture.state.miniWechatBindState,
      passwordLoginSheetVisible: fixture.state.passwordLoginSheetVisible,
      passwordLoginSubmitting: fixture.state.passwordLoginSubmitting,
    }
    ;(fixture as unknown as { accountState: UserAccountEffectsState }).accountState = state
    return createUserAccountEffects(fixture.session, state, {
      canControlMiniReview: () => Promise.resolve({ allowed: false }),
      getNotificationEmail: () => Promise.resolve({ email: null }),
      updateNotificationEmail: () => Promise.resolve({ email: null }),
      loadSystemConfigUnderReview: () => Promise.resolve(false),
      showToast: () => {},
    })
  }

  /** 与页面相同的接线，但走真实 Vue watch：revision 变化由 session 真实发布触发。 */
  function wirePageRevisionWatch(fixture: FlowFixture, flow: UserLoginFlow, effects: UserAccountEffects): () => void {
    const scope = effectScope()
    scope.run(() => {
      watch(fixture.session.revision, (newRevision) => {
        effects.handleSessionRevisionChange(flow.isOwnAuthSubmit(newRevision))
      })
    })
    return () => scope.stop()
  }

  test('密码登录成功：真实 watcher 下弹层关闭、toast、跳转、会话发布', async () => {
    const fixture = createFlowFixture()
    getStub(fixture).handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))
    fixture.state.passwordLoginSheetVisible.value = true
    const flow = createFlow(fixture)
    const effects = createEffects(fixture)
    const stop = wirePageRevisionWatch(fixture, flow, effects)

    const pending = flow.submitPasswordLogin({ accountIdentifier: 'a', password: 'x' })
    await flushMicrotasks()
    fixture.loadUserRequest.resolve()
    await pending
    await flushMicrotasks()

    expect(fixture.state.passwordLoginSheetVisible.value).toBe(false)
    expect(fixture.state.passwordLoginSubmitting.value).toBe(false)
    expect(fixture.toasts).toEqual(['登录成功'])
    expect(fixture.redirects).toBe(1)
    expect(fixture.session.currentUser.value).toMatchObject({ id: 'a' })
    stop()
  })

  test('微信登录 authenticated：真实 watcher 下提示成功并跳转', async () => {
    const fixture = createFlowFixture()
    getStub(fixture).handlers.loginWithMiniWechat = () =>
      Promise.resolve({ status: 'authenticated', ...buildAuthResponse(userA, 'token-wechat') })
    const flow = createFlow(fixture)
    const effects = createEffects(fixture)
    const stop = wirePageRevisionWatch(fixture, flow, effects)

    const pending = flow.loginWithMiniWechat()
    await flushMicrotasks()
    fixture.loadUserRequest.resolve()
    await pending
    await flushMicrotasks()

    expect(fixture.toasts).toEqual(['微信登录成功'])
    expect(fixture.redirects).toBe(1)
    expect(fixture.session.currentUser.value).toMatchObject({ id: 'a' })
    expect(fixture.state.miniWechatBindState.value).toBeNull()
    stop()
  })

  test('微信登录 binding_required：真实 watcher 下展示绑定表单、不 toast', async () => {
    const fixture = createFlowFixture()
    getStub(fixture).handlers.loginWithMiniWechat = () =>
      Promise.resolve({
        status: 'binding_required',
        bind_token: 'bind-token',
        expires_at: '2026-02-01T00:00:00Z',
        display_name: '微信用户',
        avatar_url: null,
      })
    const flow = createFlow(fixture)
    const effects = createEffects(fixture)
    const stop = wirePageRevisionWatch(fixture, flow, effects)

    await flow.loginWithMiniWechat()
    await flushMicrotasks()

    expect(fixture.state.miniWechatBindState.value?.bind_token).toBe('bind-token')
    expect(fixture.toasts).toEqual([])
    expect(fixture.session.currentUser.value).toBeNull()
    stop()
  })

  test('绑定失败：真实 watcher 下保留绑定表单与 bind_token，可原地重试', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    stub.handlers.bindMiniWechatAccount = () =>
      Promise.reject(new ApiRequestError('邀请码无效', 400))
    fixture.state.miniWechatBindState.value = {
      status: 'binding_required',
      bind_token: 'bind-token',
      expires_at: '2026-02-01T00:00:00Z',
      display_name: '微信用户',
      avatar_url: null,
    }
    const flow = createFlow(fixture)
    const effects = createEffects(fixture)
    const stop = wirePageRevisionWatch(fixture, flow, effects)

    await flow.bindMiniWechatAccount({
      bindToken: 'bind-token',
      inviteCode: 'bad-code',
      referralCode: '',
      displayName: '微信用户',
      avatarDataUrl: 'data:image/png;base64,x',
    })
    await flushMicrotasks()

    expect(fixture.toasts).toEqual(['邀请码无效'])
    expect(fixture.state.miniWechatBindState.value?.bind_token).toBe('bind-token')
    stop()
  })

  test('真正退出：真实 watcher 下清空认证表单与绑定上下文', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))
    stub.handlers.logout = () => Promise.resolve()
    await fixture.session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    fixture.state.miniWechatBindState.value = {
      status: 'binding_required',
      bind_token: 'bind-token',
      expires_at: '2026-02-01T00:00:00Z',
      display_name: '微信用户',
      avatar_url: null,
    }
    fixture.state.passwordLoginSheetVisible.value = true
    const flow = createFlow(fixture)
    const effects = createEffects(fixture)
    const stop = wirePageRevisionWatch(fixture, flow, effects)

    const pending = flow.logout()
    await flushMicrotasks()
    fixture.loadUserRequest.resolve()
    await pending
    await flushMicrotasks()

    expect(fixture.state.miniWechatBindState.value).toBeNull()
    expect(fixture.state.passwordLoginSheetVisible.value).toBe(false)
    expect(fixture.session.currentUser.value).toBeNull()
    expect(fixture.toasts).toEqual(['已退出登录'])
    stop()
  })

  describe('慢审核结束后的迟到 bootstrap（真实 miniReview 等待 + Vue watcher）', () => {
    function buildReviewStatus(isReviewing: boolean): MiniReviewStatus {
      return {
        project_code: 'football_insight_mini',
        version: '1.0.0',
        is_reviewing: isReviewing,
        status_text: '',
      }
    }

    /**
     * 与 App.vue onLaunch 相同的后台恢复编排：先等审核查询，再按最新审核态 bootstrap。
     * 审核 GET 由 deferred 控制，模拟「慢审核」。
     */
    function createLaunchHarness(fixture: FlowFixture) {
      const reviewRequest = deferred<MiniReviewStatus>()
      const miniReview = createMiniReviewStore({
        getMiniReviewStatus: () => reviewRequest.promise,
      } as unknown as typeof miniReviewApi)
      const run = async () => {
        await miniReview.ensureMiniReviewReady()
        await fixture.session.bootstrapSession({
          allowSilentWechat: !miniReview.isUnderReview.value,
        })
      }
      return { reviewRequest, run }
    }

    test('显式 binding_required 完成后 bootstrap 才发起：不再获取 code/调用认证 API，绑定表单保留', async () => {
      const fixture = createFlowFixture()
      const stub = getStub(fixture)
      stub.handlers.loginWithMiniWechat = () =>
        Promise.resolve({
          status: 'binding_required',
          bind_token: 'bind-1',
          expires_at: '2026-02-01T00:00:00Z',
          display_name: '微信用户',
          avatar_url: null,
        })
      const flow = createFlow(fixture)
      const effects = createEffects(fixture)
      const stop = wirePageRevisionWatch(fixture, flow, effects)
      const launch = createLaunchHarness(fixture)
      // App 冷启动：审核 GET pending，bootstrap 尚未发起。
      const pendingLaunch = launch.run()

      // 审核等待期间：用户明确点击微信登录 → binding_required，真实 flow 发布绑定表单。
      await flow.loginWithMiniWechat()
      await flushMicrotasks()
      expect(fixture.state.miniWechatBindState.value?.bind_token).toBe('bind-1')
      expect(fixture.wechatCodeCalls).toBe(1)

      // 审核返回「非审核中」：App 此刻才 bootstrap。
      launch.reviewRequest.resolve(buildReviewStatus(false))
      await pendingLaunch
      await flushMicrotasks()

      // 显式操作已替代本次启动的自动恢复：不再获取 code、不再调用微信认证 API，
      // 真实 watcher 不得清掉用户正在填写的绑定表单（bind_token 保留可继续提交）。
      expect(fixture.wechatCodeCalls).toBe(1)
      expect(stub.calls.loginWithMiniWechat).toBe(1)
      expect(fixture.state.miniWechatBindState.value?.bind_token).toBe('bind-1')
      expect(fixture.session.currentUser.value).toBeNull()
      expect(fixture.session.status.value).toBe('guest')
      expect(fixture.toasts).toEqual([])
      stop()
    })

    test('显式微信登录失败后 bootstrap 才发起：不接管、不再探测，错误提示保留可原地重试', async () => {
      const fixture = createFlowFixture()
      const stub = getStub(fixture)
      stub.handlers.loginWithMiniWechat = () =>
        Promise.reject(new ApiRequestError('微信登录不可用', 500))
      const flow = createFlow(fixture)
      const effects = createEffects(fixture)
      const stop = wirePageRevisionWatch(fixture, flow, effects)
      const launch = createLaunchHarness(fixture)
      const pendingLaunch = launch.run()

      // 审核等待期间：显式微信登录失败（意图已结束），用户准备在页面上重试。
      await flow.loginWithMiniWechat()
      await flushMicrotasks()
      expect(fixture.toasts).toEqual(['微信登录不可用'])
      expect(fixture.wechatCodeCalls).toBe(1)

      launch.reviewRequest.resolve(buildReviewStatus(false))
      await pendingLaunch
      await flushMicrotasks()

      // 显式失败同样替代本次启动的自动恢复：不再次探测、不弹新的错误提示。
      expect(fixture.wechatCodeCalls).toBe(1)
      expect(stub.calls.loginWithMiniWechat).toBe(1)
      expect(fixture.toasts).toEqual(['微信登录不可用'])
      expect(fixture.session.currentUser.value).toBeNull()
      expect(fixture.session.status.value).toBe('guest')
      stop()
    })

    test('无显式操作时，慢审核结束后的 bootstrap 正常静默恢复已绑定账号', async () => {
      const fixture = createFlowFixture()
      const stub = getStub(fixture)
      stub.handlers.loginWithMiniWechat = () =>
        Promise.resolve({ status: 'authenticated', ...buildAuthResponse(userA, 'token-wechat') })
      const flow = createFlow(fixture)
      const effects = createEffects(fixture)
      const stop = wirePageRevisionWatch(fixture, flow, effects)
      const launch = createLaunchHarness(fixture)
      const pendingLaunch = launch.run()

      // 用户没有任何显式操作；审核结束后正常探测一次并完成恢复。
      launch.reviewRequest.resolve(buildReviewStatus(false))
      await pendingLaunch
      await flushMicrotasks()

      expect(fixture.wechatCodeCalls).toBe(1)
      expect(stub.calls.loginWithMiniWechat).toBe(1)
      expect(fixture.session.currentUser.value).toMatchObject({ id: 'a' })
      expect(fixture.session.status.value).toBe('authenticated')
      stop()
    })
  })
})

describe('revision 监听与认证表单上下文（watcher 联动）', () => {
  function createEffects(fixture: FlowFixture): UserAccountEffects {
    const state: UserAccountEffectsState = {
      loading: ref(false),
      systemConfigUnderReview: ref(false),
      isReviewController: ref(false),
      notificationEmail: ref('a@example.com'),
      notificationEmailSheetVisible: ref(false),
      notificationEmailSaving: ref(false),
      miniWechatBindState: fixture.state.miniWechatBindState,
      passwordLoginSheetVisible: fixture.state.passwordLoginSheetVisible,
      passwordLoginSubmitting: fixture.state.passwordLoginSubmitting,
    }
    ;(fixture as unknown as { accountState: UserAccountEffectsState }).accountState = state
    return createUserAccountEffects(fixture.session, state, {
      canControlMiniReview: () => Promise.resolve({ allowed: false }),
      getNotificationEmail: () => Promise.resolve({ email: null }),
      updateNotificationEmail: () => Promise.resolve({ email: null }),
      loadSystemConfigUnderReview: () => Promise.resolve(false),
      showToast: () => {},
    })
  }

  /** 与页面 watch(session.revision) 相同的分支逻辑。 */
  function watchRevision(
    flow: UserLoginFlow,
    effects: UserAccountEffects,
    revision: number,
  ): void {
    if (flow.isOwnAuthSubmit(revision)) {
      effects.resetPrivateViewState()
      return
    }
    effects.resetSessionScopedState()
  }

  test('绑定提交在途时 revision 变化：清展示数据，绑定表单与 token 保留，失败可原地重试', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    stub.handlers.loginWithMiniWechat = () =>
      Promise.resolve({
        status: 'binding_required',
        bind_token: 'bind-token',
        expires_at: '2026-02-01T00:00:00Z',
        display_name: '微信用户',
        avatar_url: null,
      })
    const flow = createFlow(fixture)
    const effects = createEffects(fixture)
    const accountState = (fixture as unknown as { accountState: UserAccountEffectsState }).accountState

    await flow.loginWithMiniWechat()
    expect(fixture.state.miniWechatBindState.value?.bind_token).toBe('bind-token')

    // 用户提交无效邀请码，请求在途：beginAuthIntent 同步递增 revision，watcher 触发。
    const bindRequest = deferred<AuthResponse>()
    stub.handlers.bindMiniWechatAccount = () => bindRequest.promise
    const pending = flow.bindMiniWechatAccount({
      bindToken: 'bind-token',
      inviteCode: 'bad-code',
      referralCode: '',
      displayName: '微信用户',
      avatarDataUrl: 'data:image/png;base64,x',
    })
    watchRevision(flow, effects, fixture.session.revision.value)

    // 旧账号展示数据被清理，但绑定表单与 token 保留。
    expect(accountState.notificationEmail.value).toBe('')
    expect(fixture.state.miniWechatBindState.value?.bind_token).toBe('bind-token')

    // 邀请码无效失败：提示错误，表单与 token 仍在，可修改后重试。
    bindRequest.reject(new ApiRequestError('邀请码无效', 400))
    await pending
    expect(fixture.toasts).toEqual(['邀请码无效'])
    expect(fixture.state.miniWechatBindState.value?.bind_token).toBe('bind-token')
  })

  test('退出或其他认证操作引起的 revision 变化：全量清理认证表单上下文', async () => {
    const fixture = createFlowFixture()
    const stub = getStub(fixture)
    const bindRequest = deferred<AuthResponse>()
    stub.handlers.bindMiniWechatAccount = () => bindRequest.promise
    fixture.state.miniWechatBindState.value = {
      status: 'binding_required',
      bind_token: 'bind-token',
      expires_at: '2026-02-01T00:00:00Z',
      display_name: '微信用户',
      avatar_url: null,
    }
    fixture.state.passwordLoginSheetVisible.value = true
    const flow = createFlow(fixture)
    const effects = createEffects(fixture)

    // A 绑定表单提交在途。
    const pendingA = flow.bindMiniWechatAccount({
      bindToken: 'bind-token',
      inviteCode: '',
      referralCode: '',
      displayName: '微信用户',
      avatarDataUrl: 'data:image/png;base64,x',
    })
    // B 登录成功（不经本 flow）：watcher 收到的 revision 不属于 A 的提交。
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userB, 'token-b'))
    await fixture.session.loginWithPassword({ account_identifier: 'b', password: 'y' })
    watchRevision(flow, effects, fixture.session.revision.value)

    expect(fixture.state.miniWechatBindState.value).toBeNull()
    expect(fixture.state.passwordLoginSheetVisible.value).toBe(false)

    bindRequest.resolve(buildAuthResponse(userA, 'token-a'))
    await pendingA
    expect(fixture.toasts).toEqual([])
  })
})
