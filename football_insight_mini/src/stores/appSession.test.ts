import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import {
  createAppSession,
  isSessionSupersededError,
  SessionSupersededError,
  type AppSession,
} from './appSession'
import type * as authApi from '../api/auth'
import type { AuthResponse, CurrentUser, MiniWechatLoginResponse } from '../types/auth'
import { ApiRequestError } from '../utils/apiError'
import { AUTH_TOKEN_STORAGE_KEY, MANUAL_LOGOUT_STORAGE_KEY, hasManualLogout } from '../utils/authStorage'

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

/** 记录 logout 被调用瞬间存储里的 token，验证「先发起旧 token 请求再清本地」。 */
let tokenAtLogoutCall: string | null | undefined

interface SourceStub {
  source: AuthSource
  calls: Record<'getCurrentUser' | 'login' | 'register' | 'loginAsH5TestUser' | 'loginWithMiniWechat' | 'bindMiniWechatAccount' | 'logout', number>
  handlers: {
    getCurrentUser: () => Promise<CurrentUser | null>
    login: () => Promise<AuthResponse>
    register: () => Promise<AuthResponse>
    loginAsH5TestUser: () => Promise<AuthResponse>
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
      register: 0,
      loginAsH5TestUser: 0,
      loginWithMiniWechat: 0,
      bindMiniWechatAccount: 0,
      logout: 0,
    },
    handlers: {
      getCurrentUser: unexpectedCall('getCurrentUser'),
      login: unexpectedCall('login'),
      register: unexpectedCall('register'),
      loginAsH5TestUser: unexpectedCall('loginAsH5TestUser'),
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
    register: () => {
      stub.calls.register += 1
      return stub.handlers.register()
    },
    loginAsH5TestUser: () => {
      stub.calls.loginAsH5TestUser += 1
      return stub.handlers.loginAsH5TestUser()
    },
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
      tokenAtLogoutCall = (storage[AUTH_TOKEN_STORAGE_KEY] as string | undefined) ?? null
      return stub.handlers.logout()
    },
  } as unknown as AuthSource

  return stub
}

function getStoredToken(): string | null {
  const token = storage[AUTH_TOKEN_STORAGE_KEY]
  return typeof token === 'string' && token.length > 0 ? token : null
}

const realDateNow = Date.now
let nowOverride: number | null = null

function setNow(value: number): void {
  nowOverride = value
}

function advanceNow(ms: number): void {
  nowOverride = (nowOverride ?? realDateNow()) + ms
}

async function captureError(action: () => Promise<unknown>): Promise<unknown> {
  try {
    await action()
    return null
  } catch (error) {
    return error
  }
}

/** 建立已认证的 A 会话（token-a / userA），返回 source stub。 */
async function establishSessionA(session: AppSession, stub: SourceStub): Promise<void> {
  stub.handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))
  await session.loginWithPassword({ account_identifier: 'a', password: 'secret' })
}

beforeEach(() => {
  storage = {}
  tokenAtLogoutCall = undefined
  nowOverride = null
  Date.now = () => nowOverride ?? realDateNow()
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
  Date.now = realDateNow
})

describe('ensureSessionReady 加载与缓存', () => {
  test('无 token 时直接游客态，不请求 /auth/me', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)

    await expect(session.ensureSessionReady()).resolves.toBeNull()
    expect(stub.calls.getCurrentUser).toBe(0)
    expect(session.status.value).toBe('guest')
    expect(session.currentUser.value).toBeNull()
    expect(session.hasAccessToken.value).toBe(false)
  })

  test('同一 revision 并发 ensure 只发出一次请求', async () => {
    const stub = createSourceStub()
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    const pending = deferred<CurrentUser | null>()
    stub.handlers.getCurrentUser = () => pending.promise
    const session = createAppSession(stub.source)

    const first = session.ensureSessionReady()
    const second = session.ensureSessionReady()
    pending.resolve(userA)

    await expect(first).resolves.toMatchObject({ id: 'a' })
    await expect(second).resolves.toMatchObject({ id: 'a' })
    expect(stub.calls.getCurrentUser).toBe(1)
    expect(session.status.value).toBe('authenticated')
  })

  test('成功后 60 秒内复用缓存，超过后重新请求', async () => {
    const stub = createSourceStub()
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    stub.handlers.getCurrentUser = () => Promise.resolve(userA)
    const session = createAppSession(stub.source)
    setNow(1_000_000)

    await session.ensureSessionReady()
    await session.ensureSessionReady()
    expect(stub.calls.getCurrentUser).toBe(1)

    advanceNow(61_000)
    await session.ensureSessionReady()
    expect(stub.calls.getCurrentUser).toBe(2)
  })

  test('force 绕过成功缓存，但仍合并同一 revision 的在途请求', async () => {
    const stub = createSourceStub()
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    stub.handlers.getCurrentUser = () => Promise.resolve(userA)
    const session = createAppSession(stub.source)

    await session.ensureSessionReady()

    const pending = deferred<CurrentUser | null>()
    stub.handlers.getCurrentUser = () => pending.promise
    const first = session.ensureSessionReady({ force: true })
    const second = session.ensureSessionReady({ force: true })
    pending.resolve(userA)
    await first
    await second

    // 首次 1 次 + force 合并后 1 次
    expect(stub.calls.getCurrentUser).toBe(2)
  })

  test('force 在 60 秒缓存内也会重新请求', async () => {
    const stub = createSourceStub()
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    stub.handlers.getCurrentUser = () => Promise.resolve(userA)
    const session = createAppSession(stub.source)

    await session.ensureSessionReady()
    await session.ensureSessionReady({ force: true })
    expect(stub.calls.getCurrentUser).toBe(2)
  })
})

describe('ensureSessionReady 错误语义', () => {
  test('401（返回 null）清除当前会话', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    stub.handlers.getCurrentUser = () => Promise.resolve(null)
    await expect(session.ensureSessionReady({ force: true })).resolves.toBeNull()

    expect(getStoredToken()).toBeNull()
    expect(session.currentUser.value).toBeNull()
    expect(session.hasAccessToken.value).toBe(false)
    expect(session.status.value).toBe('guest')
  })

  test('5xx 保留当前 token 和用户，error 可观察，下次调用可重试', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)
    setNow(1_000_000)

    stub.handlers.getCurrentUser = () => Promise.reject(new ApiRequestError('请求失败（500）', 500))
    const error = await captureError(() => session.ensureSessionReady({ force: true }))
    expect(error).toBeInstanceOf(ApiRequestError)

    expect(getStoredToken()).toBe('token-a')
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
    expect(session.status.value).toBe('error')
    expect(session.errorMessage.value).toBe('请求失败（500）')

    stub.handlers.getCurrentUser = () => Promise.resolve(userA)
    await expect(session.ensureSessionReady()).resolves.toMatchObject({ id: 'a' })
    expect(session.status.value).toBe('authenticated')
    expect(session.errorMessage.value).toBe('')
  })

  test('断网错误保留当前会话并可重试', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    stub.handlers.getCurrentUser = () =>
      Promise.reject(new ApiRequestError('网络连接不可用，请检查网络后重试', 0, true))
    const error = await captureError(() => session.ensureSessionReady({ force: true }))
    expect(error).toBeInstanceOf(ApiRequestError)

    expect(getStoredToken()).toBe('token-a')
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
    expect(session.status.value).toBe('error')
  })
})

describe('登录意图竞态（deferred）', () => {
  test('登录 A pending 时开始 B，B 返回后 A 晚到：token/user 属于 B，A 抛 SessionSupersededError', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    const loginA = deferred<AuthResponse>()
    const loginB = deferred<AuthResponse>()
    stub.handlers.login = () => loginA.promise

    const promiseA = session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    const errorA = captureError(() => promiseA)

    stub.handlers.login = () => loginB.promise
    const promiseB = session.loginWithPassword({ account_identifier: 'b', password: 'y' })

    loginB.resolve(buildAuthResponse(userB, 'token-b'))
    await expect(promiseB).resolves.toMatchObject({ access_token: 'token-b' })

    loginA.resolve(buildAuthResponse(userA, 'token-a'))
    expect(await errorA).toBeInstanceOf(SessionSupersededError)

    expect(getStoredToken()).toBe('token-b')
    expect(session.currentUser.value).toMatchObject({ id: 'b' })
    expect(session.status.value).toBe('authenticated')
  })

  test('退出期间登录返回不能恢复会话', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    const loginA = deferred<AuthResponse>()
    stub.handlers.login = () => loginA.promise

    const promiseA = session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    const errorA = captureError(() => promiseA)

    stub.handlers.logout = () => Promise.resolve()
    await session.logoutSession()

    loginA.resolve(buildAuthResponse(userA, 'token-a'))
    expect(await errorA).toBeInstanceOf(SessionSupersededError)

    expect(getStoredToken()).toBeNull()
    expect(session.currentUser.value).toBeNull()
    expect(session.status.value).toBe('guest')
  })

  test('登录失败保留已有会话，错误可观察', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    stub.handlers.login = () => Promise.reject(new ApiRequestError('账号或密码错误', 400))
    const error = await captureError(() =>
      session.loginWithPassword({ account_identifier: 'b', password: 'wrong' }),
    )
    expect(error).toBeInstanceOf(ApiRequestError)

    expect(getStoredToken()).toBe('token-a')
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
    expect(session.status.value).toBe('authenticated')
    expect(session.errorMessage.value).toBe('账号或密码错误')
  })
})

describe('登录与刷新交错竞态', () => {
  test('登录 B 等待期间用 A token 发出的 me 刷新晚到，不能覆盖 B', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    const loginB = deferred<AuthResponse>()
    stub.handlers.login = () => loginB.promise
    const promiseB = session.loginWithPassword({ account_identifier: 'b', password: 'y' })

    // B 等待期间，旧 token A 发出的刷新在途。
    const meA = deferred<CurrentUser | null>()
    stub.handlers.getCurrentUser = () => meA.promise
    const refresh = session.ensureSessionReady({ force: true })
    const refreshError = captureError(() => refresh)

    loginB.resolve(buildAuthResponse(userB, 'token-b'))
    await promiseB

    meA.resolve(userA)
    expect(await refreshError).toBeInstanceOf(SessionSupersededError)

    expect(getStoredToken()).toBe('token-b')
    expect(session.currentUser.value).toMatchObject({ id: 'b' })
    expect(session.status.value).toBe('authenticated')
  })

  test('登录 B 等待期间用 A token 发出的 401 晚到，不能清掉 B', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    const loginB = deferred<AuthResponse>()
    stub.handlers.login = () => loginB.promise
    const promiseB = session.loginWithPassword({ account_identifier: 'b', password: 'y' })

    const meA = deferred<CurrentUser | null>()
    stub.handlers.getCurrentUser = () => meA.promise
    const refresh = session.ensureSessionReady({ force: true })
    const refreshError = captureError(() => refresh)

    loginB.resolve(buildAuthResponse(userB, 'token-b'))
    await promiseB

    meA.resolve(null)
    expect(await refreshError).toBeInstanceOf(SessionSupersededError)

    expect(getStoredToken()).toBe('token-b')
    expect(session.currentUser.value).toMatchObject({ id: 'b' })
    expect(session.status.value).toBe('authenticated')
  })

  test('登录 B 在途期间旧 token 的 401 先到，只清旧会话不取消 B', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    const loginB = deferred<AuthResponse>()
    stub.handlers.login = () => loginB.promise
    const promiseB = session.loginWithPassword({ account_identifier: 'b', password: 'y' })
    const errorB = captureError(() => promiseB)
    const revisionWithIntent = session.revision.value

    // B 等待期间，旧 token A 的刷新返回 401：只应清掉旧会话，不应取消在途的 B。
    const meA = deferred<CurrentUser | null>()
    stub.handlers.getCurrentUser = () => meA.promise
    const refresh = session.ensureSessionReady({ force: true })

    meA.resolve(null)
    await expect(refresh).resolves.toBeNull()

    // 旧 token 会话状态已清除，但 revision 不因 401 递增（不取消 B 的登录意图）。
    expect(getStoredToken()).toBeNull()
    expect(session.currentUser.value).toBeNull()
    expect(session.hasAccessToken.value).toBe(false)
    expect(session.revision.value).toBe(revisionWithIntent)

    loginB.resolve(buildAuthResponse(userB, 'token-b'))
    await expect(promiseB).resolves.toMatchObject({ access_token: 'token-b' })
    expect(await errorB).toBeNull()

    expect(getStoredToken()).toBe('token-b')
    expect(session.currentUser.value).toMatchObject({ id: 'b' })
    expect(session.status.value).toBe('authenticated')
  })

  test('被替代的旧登录仍在途，不阻止当前会话的 401 完全失效', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    const loginA = deferred<AuthResponse>()
    stub.handlers.login = () => loginA.promise
    const promiseA = session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    const errorA = captureError(() => promiseA)

    // B 登录成功，A 仍在途（已被替代的旧意图）。
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userB, 'token-b'))
    await session.loginWithPassword({ account_identifier: 'b', password: 'y' })
    expect(session.status.value).toBe('authenticated')
    const revisionAfterB = session.revision.value

    // 当前会话（B）的 /auth/me 返回 401：必须完全失效，不能被 A 的在途请求保护。
    stub.handlers.getCurrentUser = () => Promise.resolve(null)
    await expect(session.ensureSessionReady({ force: true })).resolves.toBeNull()

    expect(getStoredToken()).toBeNull()
    expect(session.currentUser.value).toBeNull()
    expect(session.hasAccessToken.value).toBe(false)
    expect(session.status.value).toBe('guest')
    expect(session.revision.value).toBe(revisionAfterB + 1)

    // A 晚到：superseded，不恢复任何状态。
    loginA.resolve(buildAuthResponse(userA, 'token-a'))
    expect(await errorA).toBeInstanceOf(SessionSupersededError)
    expect(getStoredToken()).toBeNull()
    expect(session.status.value).toBe('guest')
  })
})

describe('微信认证意图从获取 code 前建立', () => {
  test('第一次 code 慢、第二次登录成功、用户退出后第一次 code 才返回：不发旧认证请求，最终 guest', async () => {
    const stub = createSourceStub()
    const slowCode = deferred<string>()
    const codeQueue: Array<Promise<string>> = [slowCode.promise, Promise.resolve('code-fast')]
    const session = createAppSession(stub.source, {
      getMiniWechatCode: () => codeQueue.shift() ?? Promise.resolve('code-extra'),
    })

    const first = session.loginWithMiniWechat()
    const firstError = captureError(() => first)

    stub.handlers.loginWithMiniWechat = () =>
      Promise.resolve({ status: 'authenticated', ...buildAuthResponse(userB, 'token-b') })
    await session.loginWithMiniWechat()
    expect(stub.calls.loginWithMiniWechat).toBe(1)
    expect(session.currentUser.value).toMatchObject({ id: 'b' })

    stub.handlers.logout = () => Promise.resolve()
    await session.logoutSession()
    expect(session.status.value).toBe('guest')

    // 第一次的慢 code 此刻才返回：意图已被替代，不允许再发认证请求。
    slowCode.resolve('code-slow')
    expect(await firstError).toBeInstanceOf(SessionSupersededError)
    expect(stub.calls.loginWithMiniWechat).toBe(1)
    expect(getStoredToken()).toBeNull()
    expect(session.currentUser.value).toBeNull()
    expect(session.status.value).toBe('guest')
  })

  test('code 在新密码登录完成后才返回：旧 code 不换 token，会话与错误提示属于新登录', async () => {
    const stub = createSourceStub()
    const slowCode = deferred<string>()
    const session = createAppSession(stub.source, { getMiniWechatCode: () => slowCode.promise })

    const pending = session.loginWithMiniWechat()
    const pendingError = captureError(() => pending)

    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))
    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    expect(session.currentUser.value).toMatchObject({ id: 'a' })

    slowCode.resolve('code-late')
    expect(await pendingError).toBeInstanceOf(SessionSupersededError)
    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(getStoredToken()).toBe('token-a')
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
    expect(session.errorMessage.value).toBe('')
  })

  test('code 在被替代后失败：不覆盖新会话的状态与错误信息', async () => {
    const stub = createSourceStub()
    const slowCode = deferred<string>()
    const session = createAppSession(stub.source, { getMiniWechatCode: () => slowCode.promise })

    const pending = session.loginWithMiniWechat()
    const pendingError = captureError(() => pending)

    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))
    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })

    slowCode.reject(new Error('用户取消微信授权'))
    expect(await pendingError).toBeInstanceOf(SessionSupersededError)
    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(session.status.value).toBe('authenticated')
    expect(session.errorMessage.value).toBe('')
  })
})

describe('logoutSession', () => {
  test('先发起带旧 token 的后端注销请求，再同步清本地会话', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    const logoutRequest = deferred<void>()
    stub.handlers.logout = () => logoutRequest.promise
    const logoutPromise = session.logoutSession()

    // 本地立即退出，注销请求已携带旧 token 发出。
    expect(stub.calls.logout).toBe(1)
    expect(tokenAtLogoutCall).toBe('token-a')
    expect(getStoredToken()).toBeNull()
    expect(session.status.value).toBe('guest')

    logoutRequest.resolve()
    await logoutPromise
  })

  test('logout A pending 时 B 登录，A 完成后 B 保持登录', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    const logoutRequest = deferred<void>()
    stub.handlers.logout = () => logoutRequest.promise
    const logoutPromise = session.logoutSession()

    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userB, 'token-b'))
    await session.loginWithPassword({ account_identifier: 'b', password: 'y' })

    logoutRequest.resolve()
    await logoutPromise

    expect(getStoredToken()).toBe('token-b')
    expect(session.currentUser.value).toMatchObject({ id: 'b' })
    expect(session.status.value).toBe('authenticated')
  })

  test('logout 远端失败不恢复本地退出', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    stub.handlers.logout = () => Promise.reject(new ApiRequestError('请求失败（500）', 500))
    await session.logoutSession()

    expect(getStoredToken()).toBeNull()
    expect(session.currentUser.value).toBeNull()
    expect(session.status.value).toBe('guest')
  })
})

describe('invalidateSession', () => {
  test('只失效匹配 revision 的会话', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    const currentRevision = session.revision.value
    session.invalidateSession(currentRevision + 1)
    expect(getStoredToken()).toBe('token-a')
    expect(session.currentUser.value).toMatchObject({ id: 'a' })

    session.invalidateSession(currentRevision)
    expect(getStoredToken()).toBeNull()
    expect(session.currentUser.value).toBeNull()
    expect(session.status.value).toBe('guest')
    expect(session.revision.value).toBe(currentRevision + 1)
  })
})

describe('bootstrapSession 启动恢复（第一阶段静默登录）', () => {
  function buildWechatAuthenticated(user: CurrentUser, token: string): MiniWechatLoginResponse {
    return { status: 'authenticated', ...buildAuthResponse(user, token) }
  }

  function buildBindingRequired(): MiniWechatLoginResponse {
    return {
      status: 'binding_required',
      bind_token: 'bind-token',
      expires_at: '2026-02-01T00:00:00Z',
      display_name: '微信用户',
      avatar_url: null,
    }
  }

  test('MP 已有有效 token：复用 ensure 恢复同一用户，不获取微信 code', async () => {
    const stub = createSourceStub()
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    stub.handlers.getCurrentUser = () => Promise.resolve(userA)
    let codeCalls = 0
    const session = createAppSession(stub.source, {
      getMiniWechatCode: () => {
        codeCalls += 1
        return Promise.resolve('code')
      },
    })

    await expect(session.bootstrapSession({ allowSilentWechat: true })).resolves.toMatchObject({ id: 'a' })
    expect(stub.calls.getCurrentUser).toBe(1)
    expect(codeCalls).toBe(0)
    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(session.status.value).toBe('authenticated')
  })

  test('MP 无 token 且已绑定微信：后台探测恢复原 user id 与 token，无 toast/导航由页面决定', async () => {
    const stub = createSourceStub()
    stub.handlers.loginWithMiniWechat = () => Promise.resolve(buildWechatAuthenticated(userA, 'token-wechat'))
    const session = createAppSession(stub.source, { getMiniWechatCode: () => Promise.resolve('code-1') })

    await expect(session.bootstrapSession({ allowSilentWechat: true })).resolves.toMatchObject({ id: 'a' })
    expect(stub.calls.loginWithMiniWechat).toBe(1)
    expect(getStoredToken()).toBe('token-wechat')
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
    expect(session.status.value).toBe('authenticated')
  })

  test('MP 首次未绑定：binding_required 作为正常游客结果，不建号、不绑定、不写 token', async () => {
    const stub = createSourceStub()
    stub.handlers.loginWithMiniWechat = () => Promise.resolve(buildBindingRequired())
    const session = createAppSession(stub.source, { getMiniWechatCode: () => Promise.resolve('code-1') })

    await expect(session.bootstrapSession({ allowSilentWechat: true })).resolves.toBeNull()
    expect(getStoredToken()).toBeNull()
    expect(session.currentUser.value).toBeNull()
    expect(session.status.value).toBe('guest')
    // bind_token 不进入持久化：存储里没有任何 binding 相关内容。
    expect(Object.keys(storage)).toEqual([])
    expect(stub.calls.register).toBe(0)
    expect(stub.calls.bindMiniWechatAccount).toBe(0)
  })

  test('MP 旧 token 401：失效旧会话后允许本次恢复操作尝试微信登录', async () => {
    const stub = createSourceStub()
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-expired'
    stub.handlers.getCurrentUser = () => Promise.resolve(null)
    stub.handlers.loginWithMiniWechat = () => Promise.resolve(buildWechatAuthenticated(userB, 'token-b'))
    const session = createAppSession(stub.source, { getMiniWechatCode: () => Promise.resolve('code-1') })

    await expect(session.bootstrapSession({ allowSilentWechat: true })).resolves.toMatchObject({ id: 'b' })
    expect(getStoredToken()).toBe('token-b')
    expect(session.currentUser.value).toMatchObject({ id: 'b' })
  })

  test('me 断网/403/5xx：不调用微信接口、不清已有会话，错误可观察', async () => {
    const stub = createSourceStub()
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    stub.handlers.getCurrentUser = () =>
      Promise.reject(new ApiRequestError('网络连接不可用，请检查网络后重试', 0, true))
    let codeCalls = 0
    const session = createAppSession(stub.source, {
      getMiniWechatCode: () => {
        codeCalls += 1
        return Promise.resolve('code')
      },
    })

    await expect(session.bootstrapSession({ allowSilentWechat: true })).resolves.toBeNull()
    expect(codeCalls).toBe(0)
    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(getStoredToken()).toBe('token-a')
    expect(session.status.value).toBe('error')
    expect(session.errorMessage.value.length > 0).toBe(true)
  })

  test('并发 bootstrap 只获取一次 code、一次登录', async () => {
    const stub = createSourceStub()
    const codeRequest = deferred<string>()
    let codeCalls = 0
    stub.handlers.loginWithMiniWechat = () => Promise.resolve(buildWechatAuthenticated(userA, 'token-a'))
    const session = createAppSession(stub.source, {
      getMiniWechatCode: () => {
        codeCalls += 1
        return codeRequest.promise
      },
    })

    const first = session.bootstrapSession({ allowSilentWechat: true })
    const second = session.bootstrapSession({ allowSilentWechat: true })
    codeRequest.resolve('code-1')

    await expect(first).resolves.toMatchObject({ id: 'a' })
    await expect(second).resolves.toMatchObject({ id: 'a' })
    expect(codeCalls).toBe(1)
    expect(stub.calls.loginWithMiniWechat).toBe(1)
  })

  test('未知/失败结果本次启动不反复探测；新 store 冷启动可再次探测', async () => {
    const stub = createSourceStub()
    let codeCalls = 0
    stub.handlers.loginWithMiniWechat = () => Promise.resolve(buildBindingRequired())
    const deps = {
      getMiniWechatCode: () => {
        codeCalls += 1
        return Promise.resolve('code-1')
      },
    }
    const session = createAppSession(stub.source, deps)

    await expect(session.bootstrapSession({ allowSilentWechat: true })).resolves.toBeNull()
    // 同一次启动内再次 bootstrap：不再探测微信。
    await expect(session.bootstrapSession({ allowSilentWechat: true })).resolves.toBeNull()
    expect(codeCalls).toBe(1)
    expect(stub.calls.loginWithMiniWechat).toBe(1)

    // 冷启动（新 store 实例、同一本地存储）：允许重新探测。
    const coldStart = createAppSession(stub.source, deps)
    await expect(coldStart.bootstrapSession({ allowSilentWechat: true })).resolves.toBeNull()
    expect(codeCalls).toBe(2)
    expect(stub.calls.loginWithMiniWechat).toBe(2)
  })

  test('H5（allowSilentWechat=false）：无 token 从不获取 code', async () => {
    const stub = createSourceStub()
    let codeCalls = 0
    const session = createAppSession(stub.source, {
      getMiniWechatCode: () => {
        codeCalls += 1
        return Promise.resolve('code')
      },
    })

    await expect(session.bootstrapSession({ allowSilentWechat: false })).resolves.toBeNull()
    expect(codeCalls).toBe(0)
    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(session.status.value).toBe('guest')
  })

  test('H5（allowSilentWechat=false）：旧 token 401 也不探测微信', async () => {
    const stub = createSourceStub()
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-expired'
    stub.handlers.getCurrentUser = () => Promise.resolve(null)
    let codeCalls = 0
    const session = createAppSession(stub.source, {
      getMiniWechatCode: () => {
        codeCalls += 1
        return Promise.resolve('code')
      },
    })

    await expect(session.bootstrapSession({ allowSilentWechat: false })).resolves.toBeNull()
    expect(codeCalls).toBe(0)
    expect(getStoredToken()).toBeNull()
    expect(session.status.value).toBe('guest')
  })

  test('自动恢复等待 code 时用户退出：迟到的 code/响应/finally 全部失效', async () => {
    const stub = createSourceStub()
    const codeRequest = deferred<string>()
    const loginRequest = deferred<MiniWechatLoginResponse>()
    stub.handlers.loginWithMiniWechat = () => loginRequest.promise
    stub.handlers.logout = () => Promise.resolve()
    const session = createAppSession(stub.source, { getMiniWechatCode: () => codeRequest.promise })

    const bootstrap = session.bootstrapSession({ allowSilentWechat: true })
    await Promise.resolve()
    // 恢复还在等 code，用户显式退出（写标记 + 递增 revision）。
    await session.logoutSession()

    codeRequest.resolve('code-late')
    await expect(bootstrap).resolves.toBeNull()

    // 迟到的 code 不能发出认证请求；即使请求已发出，迟到的成功也不能复活会话。
    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(getStoredToken()).toBeNull()
    expect(session.currentUser.value).toBeNull()
    expect(session.status.value).toBe('guest')
    expect(hasManualLogout()).toBe(true)
  })

  test('自动恢复等待 code 时显式密码登录：迟到 code 不发认证请求，新会话不受影响', async () => {
    const stub = createSourceStub()
    const codeRequest = deferred<string>()
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))
    const session = createAppSession(stub.source, { getMiniWechatCode: () => codeRequest.promise })

    const bootstrap = session.bootstrapSession({ allowSilentWechat: true })
    await Promise.resolve()
    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })

    codeRequest.resolve('code-late')
    await expect(bootstrap).resolves.toBeNull()

    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(getStoredToken()).toBe('token-a')
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
    expect(session.status.value).toBe('authenticated')
  })

  test('me 在途期间外部退出导致 revision 变化：不等同于自身 401，不探测微信', async () => {
    const stub = createSourceStub()
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    const meRequest = deferred<CurrentUser | null>()
    stub.handlers.getCurrentUser = () => meRequest.promise
    stub.handlers.logout = () => Promise.resolve()
    let codeCalls = 0
    const session = createAppSession(stub.source, {
      getMiniWechatCode: () => {
        codeCalls += 1
        return Promise.resolve('code')
      },
    })

    const bootstrap = session.bootstrapSession({ allowSilentWechat: true })
    await Promise.resolve()
    // me 在途时用户显式退出：revision 变化来自外部退出，不是 bootstrap 自己的 401。
    await session.logoutSession()
    meRequest.resolve(null)

    await expect(bootstrap).resolves.toBeNull()
    expect(codeCalls).toBe(0)
    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(session.status.value).toBe('guest')
    expect(hasManualLogout()).toBe(true)
  })
})

describe('手动退出标记', () => {
  test('主动退出立即写标记；远端 logout 失败仍退出且标记保留', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)
    expect(hasManualLogout()).toBe(false)

    stub.handlers.logout = () => Promise.reject(new ApiRequestError('请求失败（500）', 500))
    await session.logoutSession()

    expect(hasManualLogout()).toBe(true)
    expect(storage[MANUAL_LOGOUT_STORAGE_KEY]).toBe('true')
    expect(session.status.value).toBe('guest')
  })

  test('手动退出后新 store（冷启动）读取同一存储仍阻止自动登录', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)
    stub.handlers.logout = () => Promise.resolve()
    await session.logoutSession()

    let codeCalls = 0
    stub.handlers.loginWithMiniWechat = () =>
      Promise.resolve({ status: 'authenticated', ...buildAuthResponse(userB, 'token-b') })
    const coldStart = createAppSession(stub.source, {
      getMiniWechatCode: () => {
        codeCalls += 1
        return Promise.resolve('code')
      },
    })

    await expect(coldStart.bootstrapSession({ allowSilentWechat: true })).resolves.toBeNull()
    expect(codeCalls).toBe(0)
    expect(stub.calls.loginWithMiniWechat).toBe(0)
    expect(coldStart.status.value).toBe('guest')
  })

  test('force ensure 不清手动退出标记', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)
    stub.handlers.logout = () => Promise.resolve()
    await session.logoutSession()
    expect(hasManualLogout()).toBe(true)

    // 退出后存储里塞回一个 token（例如其他入口写入），force ensure 不得清标记。
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    stub.handlers.getCurrentUser = () => Promise.resolve(userA)
    await session.ensureSessionReady({ force: true })
    expect(hasManualLogout()).toBe(true)
  })

  test('明确密码/微信/绑定/测试登录解除手动退出标记', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source, { getMiniWechatCode: () => Promise.resolve('code') })
    await establishSessionA(session, stub)
    stub.handlers.logout = () => Promise.resolve()

    // 密码登录
    await session.logoutSession()
    expect(hasManualLogout()).toBe(true)
    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    expect(hasManualLogout()).toBe(false)

    // 微信登录
    await session.logoutSession()
    stub.handlers.loginWithMiniWechat = () =>
      Promise.resolve({ status: 'authenticated', ...buildAuthResponse(userA, 'token-wechat') })
    await session.loginWithMiniWechat()
    expect(hasManualLogout()).toBe(false)

    // 绑定登录
    await session.logoutSession()
    stub.handlers.bindMiniWechatAccount = () => Promise.resolve(buildAuthResponse(userA, 'token-bind'))
    await session.bindWechatAccount({
      bind_token: 'bind-token',
      display_name: '用户a',
      avatar_data_url: 'data:image/png;base64,x',
    })
    expect(hasManualLogout()).toBe(false)

    // 测试登录
    await session.logoutSession()
    stub.handlers.loginAsH5TestUser = () => Promise.resolve(buildAuthResponse(userB, 'token-test'))
    await session.loginAsTestUser('user-b')
    expect(hasManualLogout()).toBe(false)
  })

  test('401 失效不产生手动退出标记，下次冷启动可自动恢复', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    await establishSessionA(session, stub)

    stub.handlers.getCurrentUser = () => Promise.resolve(null)
    await session.ensureSessionReady({ force: true })
    expect(hasManualLogout()).toBe(false)

    // 冷启动（新实例）：401 失效的账号允许静默恢复。
    stub.handlers.loginWithMiniWechat = () =>
      Promise.resolve({ status: 'authenticated', ...buildAuthResponse(userA, 'token-new') })
    const coldStart = createAppSession(stub.source, { getMiniWechatCode: () => Promise.resolve('code') })
    await expect(coldStart.bootstrapSession({ allowSilentWechat: true })).resolves.toMatchObject({ id: 'a' })
    expect(getStoredToken()).toBe('token-new')
  })
})

describe('会话发布与工具函数', () => {
  test('登录成功由 session 写入 token 并发布用户', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))

    const revisionBefore = session.revision.value
    const response = await session.loginWithPassword({ account_identifier: 'a', password: 'x' })

    expect(response.access_token).toBe('token-a')
    expect(getStoredToken()).toBe('token-a')
    expect(session.hasAccessToken.value).toBe(true)
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
    expect(session.status.value).toBe('authenticated')
    expect(session.revision.value > revisionBefore).toBe(true)
  })

  test('注册 / 测试登录 / 微信绑定成功同样由 session 发布 token', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source)
    stub.handlers.register = () => Promise.resolve(buildAuthResponse(userA, 'token-register'))
    stub.handlers.loginAsH5TestUser = () => Promise.resolve(buildAuthResponse(userB, 'token-test'))
    stub.handlers.bindMiniWechatAccount = () => Promise.resolve(buildAuthResponse(userA, 'token-bind'))

    await session.registerAccount({ account_identifier: 'a', password: 'x' })
    expect(getStoredToken()).toBe('token-register')

    await session.loginAsTestUser('user-b')
    expect(getStoredToken()).toBe('token-test')
    expect(session.currentUser.value).toMatchObject({ id: 'b' })

    await session.bindWechatAccount({
      bind_token: 'bind-token',
      display_name: '用户a',
      avatar_data_url: 'data:image/png;base64,x',
    })
    expect(getStoredToken()).toBe('token-bind')
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
  })

  test('微信 binding_required 不发布 token，保持游客态并返回响应', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source, { getMiniWechatCode: () => Promise.resolve('wechat-code') })
    stub.handlers.loginWithMiniWechat = () =>
      Promise.resolve({
        status: 'binding_required',
        bind_token: 'bind-token',
        expires_at: '2026-02-01T00:00:00Z',
        display_name: '微信用户',
        avatar_url: null,
      })

    const response = await session.loginWithMiniWechat()

    expect(response.status).toBe('binding_required')
    expect(getStoredToken()).toBeNull()
    expect(session.currentUser.value).toBeNull()
    expect(session.status.value).toBe('guest')
  })

  test('微信 code 登录成功发布 token', async () => {
    const stub = createSourceStub()
    const session = createAppSession(stub.source, { getMiniWechatCode: () => Promise.resolve('wechat-code') })
    stub.handlers.loginWithMiniWechat = () =>
      Promise.resolve({ status: 'authenticated', ...buildAuthResponse(userA, 'token-wechat') })

    const response = await session.loginWithMiniWechat()

    expect(response.status).toBe('authenticated')
    expect(stub.calls.loginWithMiniWechat).toBe(1)
    expect(getStoredToken()).toBe('token-wechat')
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
  })

  test('isSessionSupersededError 只识别 SessionSupersededError', () => {
    expect(isSessionSupersededError(new SessionSupersededError())).toBe(true)
    expect(isSessionSupersededError(new ApiRequestError('请求失败（500）', 500))).toBe(false)
    expect(isSessionSupersededError(new Error('其他'))).toBe(false)
    expect(isSessionSupersededError('SessionSupersededError')).toBe(false)
  })

  test('createAppSession 不同实例状态隔离', async () => {
    const stub = createSourceStub()
    const first = createAppSession(stub.source)
    const second = createAppSession(stub.source)
    stub.handlers.login = () => Promise.resolve(buildAuthResponse(userA, 'token-a'))

    await first.loginWithPassword({ account_identifier: 'a', password: 'x' })

    expect(first.currentUser.value).toMatchObject({ id: 'a' })
    expect(second.currentUser.value).toBeNull()
    expect(second.status.value).toBe('guest')
  })
})
