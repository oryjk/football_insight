import { readonly, ref, type Ref } from 'vue'
import * as authApi from '../api/auth'
import type {
  AuthResponse,
  CurrentUser,
  LoginPayload,
  MiniWechatBindPayload,
  MiniWechatLoginResponse,
  RegisterPayload,
} from '../types/auth'
import { extractApiErrorMessage } from '../utils/apiError'
import { getAccessToken, setAccessToken } from '../utils/authStorage'

export type SessionStatus = 'guest' | 'loading' | 'authenticated' | 'error'

/** 会话被更新的操作替代时抛出；页面识别后静默放弃该次的成功提示/跳转。 */
export class SessionSupersededError extends Error {
  constructor() {
    super('会话已被更新的操作替代')
    this.name = 'SessionSupersededError'
  }
}

export function isSessionSupersededError(error: unknown): boolean {
  return error instanceof SessionSupersededError
}

export interface AppSession {
  readonly currentUser: Readonly<Ref<CurrentUser | null>>
  readonly hasAccessToken: Readonly<Ref<boolean>>
  readonly status: Readonly<Ref<SessionStatus>>
  readonly errorMessage: Readonly<Ref<string>>
  readonly revision: Readonly<Ref<number>>
  ensureSessionReady(options?: { force?: boolean }): Promise<CurrentUser | null>
  loginWithPassword(payload: LoginPayload): Promise<AuthResponse>
  registerAccount(payload: RegisterPayload): Promise<AuthResponse>
  loginAsTestUser(userId: string): Promise<AuthResponse>
  loginWithWechatCode(code: string): Promise<MiniWechatLoginResponse>
  bindWechatAccount(payload: MiniWechatBindPayload): Promise<AuthResponse>
  logoutSession(): Promise<void>
  invalidateSession(expectedRevision: number): void
}

const ENSURE_SUCCESS_CACHE_MS = 60_000

/**
 * 认证写入的唯一入口：api/auth 只做原子请求，token（fi_access_token）与当前用户
 * 只由本模块在核对 revision / token 快照后发布或清除。
 *
 * revision 在新登录意图、成功替换 token、退出及认证失效时递增；
 * 所有异步结果与 finally 都核对捕获的 revision 与 token 快照，旧账号的请求
 * （成功、401、错误、finally）不能改新账号状态。
 */
export function createAppSession(source: typeof authApi = authApi): AppSession {
  const currentUser = ref<CurrentUser | null>(null)
  const hasAccessToken = ref(Boolean(getAccessToken()))
  const status = ref<SessionStatus>('guest')
  const errorMessage = ref('')
  const revision = ref(0)

  let lastEnsureSuccessAt = 0
  let inFlightEnsure: {
    revision: number
    token: string
    promise: Promise<CurrentUser | null>
  } | null = null

  function clearLocalSession(): void {
    setAccessToken(null)
    currentUser.value = null
    hasAccessToken.value = false
    status.value = 'guest'
    errorMessage.value = ''
    lastEnsureSuccessAt = 0
  }

  function assertRevisionCurrent(expectedRevision: number): void {
    if (revision.value !== expectedRevision) {
      throw new SessionSupersededError()
    }
  }

  /** 新登录意图：递增 revision 使旧 token 发出的在途请求全部失效。 */
  function beginAuthIntent(): number {
    revision.value += 1
    status.value = 'loading'
    errorMessage.value = ''
    return revision.value
  }

  /** 发布新 token 时再次递增 revision，使登录等待期间用旧 token 发出的刷新也失效。 */
  function publishAuthResponse(response: AuthResponse, baseRevision: number): void {
    setAccessToken(response.access_token)
    currentUser.value = response.user
    hasAccessToken.value = true
    status.value = 'authenticated'
    errorMessage.value = ''
    lastEnsureSuccessAt = Date.now()
    revision.value = baseRevision + 1
  }

  /** 登录类意图失败：保留已有 token/user，error 可观察，下次可重试。 */
  function settleAuthIntentFailure(expectedRevision: number, error: unknown, fallback: string): void {
    if (revision.value !== expectedRevision) {
      throw new SessionSupersededError()
    }
    status.value = currentUser.value ? 'authenticated' : 'guest'
    errorMessage.value = extractApiErrorMessage(error, fallback)
  }

  async function runAuthIntent<TResponse extends AuthResponse>(
    intentRevision: number,
    action: () => Promise<TResponse>,
    fallback: string,
  ): Promise<TResponse> {
    try {
      const response = await action()
      assertRevisionCurrent(intentRevision)
      publishAuthResponse(response, intentRevision)
      return response
    } catch (error) {
      if (isSessionSupersededError(error)) {
        throw error
      }
      settleAuthIntentFailure(intentRevision, error, fallback)
      throw error
    }
  }

  async function ensureSessionReady(options?: { force?: boolean }): Promise<CurrentUser | null> {
    const force = options?.force === true
    const token = getAccessToken()

    // 无 token 直接游客态，不请求 /auth/me。
    if (!token) {
      clearLocalSession()
      return null
    }

    hasAccessToken.value = true

    if (!force && status.value === 'authenticated' && currentUser.value
      && Date.now() - lastEnsureSuccessAt < ENSURE_SUCCESS_CACHE_MS) {
      return currentUser.value
    }

    // 同一 revision + token 的并发 ensure 合并在途请求（force 也只绕过成功缓存）。
    if (inFlightEnsure
      && inFlightEnsure.revision === revision.value
      && inFlightEnsure.token === token) {
      return inFlightEnsure.promise
    }

    const ensureRevision = revision.value
    const tokenSnapshot = token
    if (!currentUser.value) {
      status.value = 'loading'
    }

    const entry = {
      revision: ensureRevision,
      token: tokenSnapshot,
      promise: null as unknown as Promise<CurrentUser | null>,
    }

    entry.promise = (async (): Promise<CurrentUser | null> => {
      const isStale = () =>
        revision.value !== ensureRevision || getAccessToken() !== tokenSnapshot

      try {
        const user = await source.getCurrentUser()
        if (isStale()) {
          throw new SessionSupersededError()
        }

        if (user === null) {
          // 401：仅失效当前 revision 的会话。
          revision.value += 1
          clearLocalSession()
          return null
        }

        currentUser.value = user
        status.value = 'authenticated'
        errorMessage.value = ''
        lastEnsureSuccessAt = Date.now()
        return user
      } catch (error) {
        if (isSessionSupersededError(error)) {
          throw error
        }
        if (isStale()) {
          throw new SessionSupersededError()
        }
        // 断网/超时/403/5xx：保留当前 token/user，记录可重试错误。
        lastEnsureSuccessAt = 0
        status.value = 'error'
        errorMessage.value = extractApiErrorMessage(error, '会话刷新失败')
        throw error
      } finally {
        if (inFlightEnsure === entry) {
          inFlightEnsure = null
        }
      }
    })()

    inFlightEnsure = entry
    return entry.promise
  }

  function loginWithPassword(payload: LoginPayload): Promise<AuthResponse> {
    const intentRevision = beginAuthIntent()
    return runAuthIntent(intentRevision, () => source.login(payload), '登录失败')
  }

  function registerAccount(payload: RegisterPayload): Promise<AuthResponse> {
    const intentRevision = beginAuthIntent()
    return runAuthIntent(intentRevision, () => source.register(payload), '注册失败')
  }

  function loginAsTestUser(userId: string): Promise<AuthResponse> {
    const intentRevision = beginAuthIntent()
    return runAuthIntent(intentRevision, () => source.loginAsH5TestUser(userId), '测试登录失败')
  }

  async function loginWithWechatCode(code: string): Promise<MiniWechatLoginResponse> {
    const intentRevision = beginAuthIntent()
    try {
      const response = await source.loginWithMiniWechat(code)
      assertRevisionCurrent(intentRevision)
      if (response.status === 'authenticated') {
        publishAuthResponse(response, intentRevision)
      } else {
        // binding_required：不发布 token，绑定表单状态留在页面内。
        status.value = currentUser.value ? 'authenticated' : 'guest'
      }
      return response
    } catch (error) {
      if (isSessionSupersededError(error)) {
        throw error
      }
      settleAuthIntentFailure(intentRevision, error, '微信登录失败')
      throw error
    }
  }

  function bindWechatAccount(payload: MiniWechatBindPayload): Promise<AuthResponse> {
    const intentRevision = beginAuthIntent()
    return runAuthIntent(intentRevision, () => source.bindMiniWechatAccount(payload), '绑定失败')
  }

  async function logoutSession(): Promise<void> {
    revision.value += 1
    // 先发起携带旧 token 的注销请求（request 层同步读取 token），再同步清本地会话；
    // 远端失败不恢复本地退出，旧 logout 完成也不会清掉期间登录的新账号。
    const remoteLogout = hasAccessToken.value
      ? source.logout()
      : Promise.resolve()
    clearLocalSession()
    try {
      await remoteLogout
    } catch {
      // 远端注销失败：本地退出保持不变。
    }
  }

  function invalidateSession(expectedRevision: number): void {
    if (revision.value !== expectedRevision) {
      return
    }
    revision.value += 1
    clearLocalSession()
  }

  return {
    currentUser: readonly(currentUser),
    hasAccessToken: readonly(hasAccessToken),
    status: readonly(status),
    errorMessage: readonly(errorMessage),
    revision: readonly(revision),
    ensureSessionReady,
    loginWithPassword,
    registerAccount,
    loginAsTestUser,
    loginWithWechatCode,
    bindWechatAccount,
    logoutSession,
    invalidateSession,
  }
}

let singleton: AppSession | null = null

export function useAppSession(): AppSession {
  if (!singleton) {
    singleton = createAppSession()
  }
  return singleton
}
