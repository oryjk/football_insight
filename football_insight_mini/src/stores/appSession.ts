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
import {
  clearManualLogout,
  getAccessToken,
  hasManualLogout,
  setAccessToken,
  setManualLogout,
} from '../utils/authStorage'

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
  /**
   * 显式微信登录完整入口：认证意图在调用瞬间、获取 code 前建立，
   * code 获取与 code 换 token 在同一个操作身份内完成；被替代后迟到的
   * code/响应不再发认证请求，也不复活会话。
   */
  loginWithMiniWechat(): Promise<MiniWechatLoginResponse>
  bindWechatAccount(payload: MiniWechatBindPayload): Promise<AuthResponse>
  /**
   * 启动恢复入口：已有 token 走 ensureSessionReady 恢复同一用户；
   * 无 token / 旧 token 被 401 失效时，仅当 allowSilentWechat（MP）且未手动退出，
   * 才允许一次后台微信探测；binding_required/失败一律静默回游客，不建号不写 token。
   * 同一次启动的并发调用合并为一个操作。
   * 显式认证/退出一旦发生（即使意图已结束，如审核等待期间完成的 binding_required
   * 或失败），即替代本次启动的自动恢复：迟到的 bootstrap 不再探测微信。
   * 业务 401（invalidateSession）导致的 revision 变化不算显式操作，不取消恢复。
   */
  bootstrapSession(options: { allowSilentWechat: boolean }): Promise<CurrentUser | null>
  logoutSession(): Promise<void>
  invalidateSession(expectedRevision: number): void
}

export interface AppSessionDeps {
  /** 微信 code 提供方（默认 uni.login）；测试注入可控延迟的 code promise。 */
  getMiniWechatCode?: () => Promise<string>
}

const ENSURE_SUCCESS_CACHE_MS = 60_000

function defaultGetMiniWechatCode(): Promise<string> {
  return new Promise((resolve, reject) => {
    uni.login({
      success: (result) => {
        if (!result.code) {
          reject(new Error('微信登录 code 获取失败'))
          return
        }

        resolve(result.code)
      },
      fail: (error) => {
        reject(new Error(error.errMsg || '微信登录失败'))
      },
    })
  })
}

/**
 * 认证写入的唯一入口：api/auth 只做原子请求，token（fi_access_token）与当前用户
 * 只由本模块在核对 revision / token 快照后发布或清除。
 *
 * revision 在新登录意图、成功替换 token、退出及认证失效时递增（旧 token 401 时
 * 若仍有在途的新登录意图，只清旧 token 会话状态、不递增 revision，以免取消新登录）；
 * 所有异步结果与 finally 都核对捕获的 revision 与 token 快照，旧账号的请求
 * （成功、401、错误、finally）不能改新账号状态。
 */
export function createAppSession(source: typeof authApi = authApi, deps: AppSessionDeps = {}): AppSession {
  const getMiniWechatCode = deps.getMiniWechatCode ?? defaultGetMiniWechatCode
  const currentUser = ref<CurrentUser | null>(null)
  const hasAccessToken = ref(Boolean(getAccessToken()))
  const status = ref<SessionStatus>('guest')
  const errorMessage = ref('')
  const revision = ref(0)

  let lastEnsureSuccessAt = 0
  // 最新一个认证意图的 intentRevision；意图结束（成功/失败/被替代）时清空。
  // 只有 intentRevision 仍等于当前 revision 的意图才算「当前有效」——
  // 被替代的意图即使还在途，也不能保护当前会话免于 401 失效。
  let activeAuthIntentRevision: number | null = null
  let inFlightEnsure: {
    revision: number
    token: string
    promise: Promise<CurrentUser | null>
  } | null = null
  // 同一次启动的 bootstrap 合并为一个操作；实例级标记保证每次冷启动最多探测一次微信。
  let inFlightBootstrap: Promise<CurrentUser | null> | null = null
  let silentWechatAttempted = false
  // 启动恢复的取消身份：显式认证操作（密码/注册/微信/绑定/测试登录）或退出一旦发生，
  // 即替代本次启动的自动恢复——即使其意图已结束（如 binding_required 或失败）。
  // 与业务 401 导致的 revision 变化无关；自动探测（silent 意图）不是显式操作，不置位。
  let startupRecoverySupersededByExplicit = false

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
  function beginAuthIntent(options?: { silent?: boolean }): number {
    // 自动探测（silent）不是显式操作：不解除手动退出标记，也不替代启动恢复自身；
    // 进入前已被 hasManualLogout 挡住，不会误清。
    if (options?.silent !== true) {
      // 只有明确的认证操作（密码/注册/微信/绑定/测试登录）才解除手动退出标记。
      clearManualLogout()
      // 显式操作替代本次启动的自动恢复：即使意图随后结束（binding_required/失败），
      // 迟到的 bootstrap（如慢审核结束后才发起）也不得再探测微信或覆盖表单。
      startupRecoverySupersededByExplicit = true
    }
    revision.value += 1
    activeAuthIntentRevision = revision.value
    status.value = 'loading'
    errorMessage.value = ''
    return revision.value
  }

  /** 登录意图结束（成功或失败）；ensure 的 401 据此判断是否还有当前有效的新登录在途。 */
  function endAuthIntent(intentRevision: number): void {
    if (activeAuthIntentRevision === intentRevision) {
      activeAuthIntentRevision = null
    }
  }

  /** 是否存在「当前有效」的认证意图：未结束且未被更新的操作替代。 */
  function hasActiveAuthIntent(): boolean {
    return activeAuthIntentRevision !== null
      && activeAuthIntentRevision === revision.value
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
    } finally {
      endAuthIntent(intentRevision)
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
          // 401：旧 token 已失效。若还有当前有效的新登录在途，只清旧 token 会话状态，
          // 不递增 revision，避免把在途新登录的 intentRevision 作废；
          // 已被替代的意图不算在内，当前会话的 401 必须完全失效。
          if (hasActiveAuthIntent()) {
            setAccessToken(null)
            currentUser.value = null
            hasAccessToken.value = false
            lastEnsureSuccessAt = 0
            return null
          }
          // 无在途登录意图：仅失效当前 revision 的会话。
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

  async function runBootstrap(options: { allowSilentWechat: boolean }): Promise<CurrentUser | null> {
    const hadToken = Boolean(getAccessToken())

    if (hadToken) {
      // 已有 token：复用共享 ensure 恢复同一用户，不调用微信 code 接口。
      let user: CurrentUser | null = null
      try {
        user = await ensureSessionReady()
      } catch {
        // 断网/超时/403/5xx：ensure 已保留 token/user 并记录可观察错误；
        // 被外部操作替代的 ensure 抛 superseded，也走这里——返回当前已确认用户，不探测微信。
        return currentUser.value
      }
      if (user) {
        return user
      }

      // ensure 返回 null：旧 token 被 401 失效，或等待期间被外部退出/新登录替代。
      // 外部退出（手动退出标记）、在途显式登录与已发生过的显式操作（即使意图已结束，
      // 如审核/me 等待期间完成的 binding_required 或失败）都必须让当前恢复操作立即
      // 停止；只有确认是「旧 token 401 失效」才允许继续微信探测。
      if (hasManualLogout() || hasActiveAuthIntent() || startupRecoverySupersededByExplicit) {
        return null
      }
      if (getAccessToken()) {
        return currentUser.value
      }
    }

    if (!options.allowSilentWechat) {
      return null
    }
    if (hasManualLogout()) {
      // 用户主动退出过：跨冷启动也不自动恢复，只有显式登录能解除。
      return null
    }
    if (startupRecoverySupersededByExplicit) {
      // 显式认证/退出一旦发生（无论成功/binding_required/失败），即替代本次启动的
      // 自动恢复：迟到的 bootstrap（如慢审核结束后才发起）不再探测微信。
      return null
    }
    if (silentWechatAttempted) {
      // 首次未知用户的自动探测每次启动最多一次；同次启动失败后由显式登录重试。
      return null
    }
    if (hasActiveAuthIntent()) {
      // 已有显式认证操作在途：不并行探测。
      return null
    }

    // 后台微信探测：只在 MP（allowSilentWechat）发生；binding_required/失败静默回游客。
    silentWechatAttempted = true
    const intentRevision = beginAuthIntent({ silent: true })
    try {
      const response = await runMiniWechatAuth(intentRevision)
      return response.status === 'authenticated' ? response.user : null
    } catch {
      // 失败（断网/5xx/接口错误）已记录可诊断 errorMessage；被替代则状态由新操作负责。
      return null
    }
  }

  function bootstrapSession(options: { allowSilentWechat: boolean }): Promise<CurrentUser | null> {
    // 同一次启动操作身份合并并发请求；迟到的 finally 不清掉新的 promise。
    if (inFlightBootstrap) {
      return inFlightBootstrap
    }

    const promise = runBootstrap(options)
    inFlightBootstrap = promise
    const clearIfCurrent = () => {
      if (inFlightBootstrap === promise) {
        inFlightBootstrap = null
      }
    }
    promise.then(clearIfCurrent, clearIfCurrent)
    return promise
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

  /**
   * 微信 code 换 token 的共享实现：调用方必须先建立认证意图（beginAuthIntent），
   * code 获取与兑换在同一操作身份内。等待 code 期间被退出/新登录替代后，
   * 迟到的 code 不再发起认证请求，迟到响应/错误/finally 全部失效。
   */
  async function runMiniWechatAuth(intentRevision: number): Promise<MiniWechatLoginResponse> {
    try {
      const code = await getMiniWechatCode()
      assertRevisionCurrent(intentRevision)
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
    } finally {
      endAuthIntent(intentRevision)
    }
  }

  function loginWithMiniWechat(): Promise<MiniWechatLoginResponse> {
    // 认证意图在调用瞬间、获取 code 前建立：旧 code 迟到不能复活已被替代的会话。
    const intentRevision = beginAuthIntent()
    return runMiniWechatAuth(intentRevision)
  }

  function bindWechatAccount(payload: MiniWechatBindPayload): Promise<AuthResponse> {
    const intentRevision = beginAuthIntent()
    return runAuthIntent(intentRevision, () => source.bindMiniWechatAccount(payload), '绑定失败')
  }

  async function logoutSession(): Promise<void> {
    revision.value += 1
    // 主动退出：立即持久化手动退出标记，跨冷启动禁止 MP 静默自动恢复；
    // 只有显式认证操作（密码/微信/绑定/测试登录）才解除。
    setManualLogout()
    // 退出同样是显式操作：替代本次启动的自动恢复（如在途/迟到的 bootstrap）。
    startupRecoverySupersededByExplicit = true
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
    loginWithMiniWechat,
    bindWechatAccount,
    bootstrapSession,
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
