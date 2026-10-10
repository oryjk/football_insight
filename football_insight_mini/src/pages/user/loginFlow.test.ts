import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import { createUserLoginFlow, type UserLoginFlowDeps, type UserLoginFlowState } from './loginFlow'
import { createAppSession, SessionSupersededError, type AppSession } from '../../stores/appSession'
import type * as authApi from '../../api/auth'
import type { AuthResponse, CurrentUser, MiniWechatLoginResponse } from '../../types/auth'
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
  handlers: {
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
    handlers: {
      login: unexpectedCall('login'),
      loginWithMiniWechat: unexpectedCall('loginWithMiniWechat'),
      bindMiniWechatAccount: unexpectedCall('bindMiniWechatAccount'),
      logout: unexpectedCall('logout'),
    },
    source: {} as AuthSource,
  }
  stub.source = {
    getCurrentUser: unexpectedCall('getCurrentUser'),
    login: () => stub.handlers.login(),
    register: unexpectedCall('register'),
    loginAsH5TestUser: unexpectedCall('loginAsH5TestUser'),
    loginWithMiniWechat: () => stub.handlers.loginWithMiniWechat(),
    bindMiniWechatAccount: () => stub.handlers.bindMiniWechatAccount(),
    logout: () => stub.handlers.logout(),
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
  deps: UserLoginFlowDeps
}

function createFlowFixture(): FlowFixture {
  const stub = createSourceStub()
  const session = createAppSession(stub.source)
  const state: UserLoginFlowState = {
    passwordLoginSheetVisible: ref(false),
    passwordLoginSubmitting: ref(false),
    miniWechatBindState: ref(null),
  }
  const toasts: string[] = []
  const loadUserRequest = deferred<void>()
  const fixture: FlowFixture = {
    session,
    state,
    toasts,
    redirects: 0,
    loadUserRequest,
    deps: {
      redirectAfterLogin: async () => {
        fixture.redirects += 1
      },
      showToast: (title) => {
        toasts.push(title)
      },
      getMiniWechatCode: () => Promise.resolve('wechat-code'),
    },
  }
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
